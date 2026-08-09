//! Adapter Grok (tier `session_files`, D-4).
//!
//! Lê os arquivos de sessão que o próprio CLI do Grok grava em
//! `~/.grok/sessions/<projeto-codificado>/<sessão>/updates.jsonl`.
//!
//! Estrutura diferente da do Claude, e mais favorável à restrição de "só
//! campos de uso": custo e tokens chegam em eventos `turn_completed`
//! totalmente separados dos eventos de conteúdo de mensagem
//! (`user_message_chunk`, `agent_message_chunk`). Filtrar pelo tipo de
//! evento já evita estruturalmente qualquer leitura de conteúdo — não é
//! preciso nem checar a ausência de um campo "content", ele simplesmente
//! não aparece nos eventos que este adapter olha.
//!
//! Risco de ToS aceito conscientemente (design.md, R-4).

use crate::domain::{BillingMode, Instante, Money, Tokens, UsageSource};
use crate::importacao::{ColetorDeUso, ConsumoColetado, ErroColeta, TierIntegracao};
use crate::storage::Periodo;
use std::io::BufRead;
use std::path::{Path, PathBuf};

/// Um `(model, uso)` já extraído de um evento `turn_completed` — um turno
/// pode usar mais de um modelo, e cada um vira um `ConsumoColetado` distinto.
#[derive(Debug, Clone, PartialEq)]
struct UsoPorModelo {
    model: String,
    prompt_id: String,
    session_id: String,
    occurred_at: Instante,
    tokens: Tokens,
    /// `costUsdTicks` já convertido. `None` se o evento não trouxer custo —
    /// nunca inventado como zero.
    custo_pago: Option<Money>,
}

/// Grok expressa custo em "ticks". Inferido cruzando duas amostras reais
/// desta sessão: a saída headless (`total_cost_usd` / `total_cost_usd_ticks`)
/// contra o evento `turn_completed` (`costUsdTicks`) do mesmo tipo de
/// chamada — a razão bate em `1 tick = 1e-10 USD` nas duas.
///
/// `Money` é micro-dólar (1e-6). Convertendo: `micros = ticks / 10_000`,
/// arredondado — ticks tem resolução mais fina que Money, então o
/// arredondamento é inerente, não erro de cálculo.
fn ticks_para_money(ticks: i64) -> Money {
    let micros = (ticks as i128 + 5_000) / 10_000;
    Money(micros as i64)
}

/// Mesma convenção observada do Claude (`/` vira algo), mas o Grok usa
/// percent-encoding de verdade: `/` vira `%2F`. Só esse caractere é tratado
/// porque é só o que a evidência real comprova — não generalizamos para
/// outros caracteres sem ter visto um caminho que os contenha.
fn codificar_diretorio_projeto(cwd: &Path) -> String {
    cwd.to_string_lossy().replace('/', "%2F")
}

/// Extrai os usos de um evento, um por entrada em `modelUsage`. `None` se a
/// linha não for um evento `turn_completed` — inclui, deliberadamente, todo
/// evento de conteúdo de mensagem, que nunca chega a ser examinado além do
/// campo `sessionUpdate`.
fn parse_linha(linha: &str) -> Vec<UsoPorModelo> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(linha) else {
        return Vec::new();
    };

    let update = &v["params"]["update"];
    if update["sessionUpdate"].as_str() != Some("turn_completed") {
        return Vec::new();
    }

    let Some(timestamp) = v["timestamp"].as_i64() else {
        return Vec::new();
    };
    let Some(session_id) = v["params"]["sessionId"].as_str() else {
        return Vec::new();
    };
    let Some(prompt_id) = update["prompt_id"].as_str() else {
        return Vec::new();
    };
    let Some(model_usage) = update["usage"]["modelUsage"].as_object() else {
        return Vec::new();
    };

    model_usage
        .iter()
        .map(|(model, u)| {
            let campo_u64 = |nome: &str| u.get(nome).and_then(|x| x.as_u64());
            let cache = match (
                campo_u64("cachedReadTokens"),
                campo_u64("cacheCreationTokens"),
            ) {
                (None, None) => None,
                (a, b) => Some(a.unwrap_or(0) + b.unwrap_or(0)),
            };

            UsoPorModelo {
                model: model.clone(),
                prompt_id: prompt_id.to_string(),
                session_id: session_id.to_string(),
                occurred_at: Instante(timestamp),
                tokens: Tokens {
                    input: campo_u64("inputTokens"),
                    cache,
                    output: campo_u64("outputTokens"),
                    reasoning: campo_u64("reasoningTokens"),
                },
                custo_pago: u
                    .get("costUsdTicks")
                    .and_then(|x| x.as_i64())
                    .map(ticks_para_money),
            }
        })
        .collect()
}

pub struct GrokAdapter {
    raiz_sessions: PathBuf,
    cwd: PathBuf,
}

impl GrokAdapter {
    pub fn new(cwd: PathBuf) -> Self {
        let raiz_sessions = dirs_home().join(".grok").join("sessions");
        Self { raiz_sessions, cwd }
    }

    fn diretorio_projeto(&self) -> PathBuf {
        self.raiz_sessions
            .join(codificar_diretorio_projeto(&self.cwd))
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

impl ColetorDeUso for GrokAdapter {
    fn provider_id(&self) -> &str {
        "grok"
    }

    fn tier(&self) -> TierIntegracao {
        TierIntegracao::SessionFiles
    }

    fn campos_disponiveis(&self) -> &[&'static str] {
        &[
            "tokens",
            "model",
            "occurred_at",
            "identificador_estavel",
            "custo_pago",
        ]
    }

    fn coletar(&self, periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta> {
        let dir_projeto = self.diretorio_projeto();

        let sessoes = match std::fs::read_dir(&dir_projeto) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(ErroColeta {
                    motivo: format!("não foi possível ler {}: {e}", dir_projeto.display()),
                });
            }
        };

        let mut resultado = Vec::new();

        for sessao in sessoes {
            let sessao = sessao.map_err(|e| ErroColeta {
                motivo: format!("erro listando {}: {e}", dir_projeto.display()),
            })?;
            let caminho_updates = sessao.path().join("updates.jsonl");
            if !caminho_updates.is_file() {
                continue;
            }

            let arquivo = std::fs::File::open(&caminho_updates).map_err(|e| ErroColeta {
                motivo: format!("não foi possível abrir {}: {e}", caminho_updates.display()),
            })?;

            for linha in std::io::BufReader::new(arquivo).lines() {
                let linha = linha.map_err(|e| ErroColeta {
                    motivo: format!("erro lendo {}: {e}", caminho_updates.display()),
                })?;

                for uso in parse_linha(&linha) {
                    if uso.occurred_at < periodo.desde {
                        continue;
                    }
                    if let Some(ate) = periodo.ate
                        && uso.occurred_at >= ate
                    {
                        continue;
                    }

                    resultado.push(ConsumoColetado {
                        identificador_estavel: Some(format!("{}:{}", uso.prompt_id, uso.model)),
                        provider_id: "grok".to_string(),
                        model: uso.model,
                        tokens: uso.tokens,
                        custo_pago: uso.custo_pago,
                        billing_mode: BillingMode::Api,
                        usage_source: UsageSource::Provider,
                        session_ref: Some(uso.session_id),
                        occurred_at: uso.occurred_at,
                        client_id: None,
                    });
                }
            }
        }

        Ok(resultado)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/grok_sessao.jsonl");

    #[test]
    fn ticks_para_money_bate_com_a_conversao_observada() {
        // 117_052_000 ticks == $0.0117052 nas duas amostras reais cruzadas.
        assert_eq!(ticks_para_money(117_052_000), Money(11_705));
    }

    #[test]
    fn parse_linha_ignora_eventos_de_conteudo_de_mensagem() {
        let linhas: Vec<&str> = FIXTURE.lines().collect();
        assert!(
            parse_linha(linhas[0]).is_empty(),
            "user_message_chunk não é turn_completed"
        );
    }

    #[test]
    fn parse_linha_extrai_uso_real_de_turn_completed() {
        let linhas: Vec<&str> = FIXTURE.lines().collect();
        let usos = parse_linha(linhas[1]);
        assert_eq!(usos.len(), 1);
        let u = &usos[0];
        assert_eq!(u.model, "grok-4.5-build");
        assert_eq!(u.tokens.input, Some(15277));
        assert_eq!(u.tokens.output, Some(50));
        assert_eq!(u.tokens.reasoning, Some(38));
        assert_eq!(u.tokens.cache, Some(11264));
        assert_eq!(u.custo_pago, Some(Money(11_705)));
    }

    #[test]
    fn parse_linha_nunca_extrai_content() {
        let linha = r#"{"timestamp":1,"params":{"sessionId":"s","update":{"sessionUpdate":"turn_completed","prompt_id":"p","usage":{"modelUsage":{"m":{"inputTokens":1,"outputTokens":1,"costUsdTicks":1,"content":"SEGREDO"}}}}}}"#;
        let usos = parse_linha(linha);
        let repr = format!("{usos:?}");
        assert!(!repr.contains("SEGREDO"));
    }

    fn diretorio_temporario_unico() -> PathBuf {
        crate::testutil::dir_temporario_unico("adapter-grok")
    }

    #[test]
    fn coletar_ponta_a_ponta_le_fixture_de_sessao_real() {
        let raiz = diretorio_temporario_unico();
        let cwd = PathBuf::from("/projeto/teste");
        let dir_sessao = raiz
            .join(codificar_diretorio_projeto(&cwd))
            .join("019fe187-fixture");
        std::fs::create_dir_all(&dir_sessao).unwrap();
        std::fs::write(dir_sessao.join("updates.jsonl"), FIXTURE).unwrap();

        let adapter = GrokAdapter {
            raiz_sessions: raiz.clone(),
            cwd,
        };

        let itens = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();

        assert_eq!(itens.len(), 2, "2 eventos turn_completed na fixture");
        assert!(itens.iter().all(|i| i.provider_id == "grok"));
        assert!(itens.iter().all(|i| i.custo_pago.is_some()));

        std::fs::remove_dir_all(&raiz).ok();
    }

    #[test]
    fn coletar_de_diretorio_inexistente_devolve_vazio() {
        let adapter = GrokAdapter {
            raiz_sessions: PathBuf::from("/caminho/inexistente/de/proposito"),
            cwd: PathBuf::from("/qualquer"),
        };
        let r = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();
        assert!(r.is_empty());
    }
}
