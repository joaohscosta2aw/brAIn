//! Adapter Codex (tier `session_files`, D-4).
//!
//! Lê os arquivos de sessão em `~/.codex/sessions/AAAA/MM/DD/rollout-*.jsonl`.
//! Diferente do Claude e do Grok: as sessões do Codex são organizadas por
//! **data**, não por projeto — o filtro por `cwd` acontece lendo o próprio
//! conteúdo do arquivo (`session_meta.payload.cwd`), não o nome do diretório.
//!
//! Modelo e uso vêm em eventos separados dentro da mesma sessão:
//! `turn_context` traz o modelo (e carrega `developer_instructions`, um
//! campo de conteúdo que este adapter nunca lê); `token_count` traz os
//! tokens do turno em `info.last_token_usage`. Nenhum dos dois expõe custo em
//! dólar — Codex autenticado via ChatGPT é assinatura, não API por token.
//!
//! Risco de ToS aceito conscientemente (design.md, R-4).

use super::tempo::parse_timestamp_iso8601;
use crate::domain::{BillingMode, Instante, Tokens, UsageSource};
use crate::importacao::{ColetorDeUso, ConsumoColetado, ErroColeta, TierIntegracao};
use crate::storage::Periodo;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
struct UsoExtraido {
    session_id: String,
    turn_id: String,
    model: String,
    occurred_at: Instante,
    tokens: Tokens,
}

/// Estado acumulado ao varrer uma sessão linha a linha: o `turn_context` mais
/// recente é o que qualifica os `token_count` seguintes, até o próximo
/// `turn_context`.
#[derive(Default)]
struct EstadoSessao {
    session_id: Option<String>,
    turn_id: Option<String>,
    model: Option<String>,
}

/// Processa uma linha, atualizando o estado e devolvendo um uso quando a
/// linha for um `token_count` com contexto suficiente já visto.
///
/// Nunca lê `base_instructions` nem `developer_instructions` — o estado só
/// guarda `id`/`cwd`/`turn_id`/`model`, os únicos campos extraídos das linhas
/// `session_meta` e `turn_context`.
fn processar_linha(estado: &mut EstadoSessao, linha: &str) -> Option<UsoExtraido> {
    let v: serde_json::Value = serde_json::from_str(linha).ok()?;
    let tipo_evento = v["type"].as_str()?;
    let payload = &v["payload"];

    match tipo_evento {
        "session_meta" => {
            estado.session_id = payload["id"].as_str().map(String::from);
            None
        }
        "event_msg" => match payload["type"].as_str()? {
            "turn_context" => {
                estado.turn_id = payload["turn_id"].as_str().map(String::from);
                estado.model = payload["model"].as_str().map(String::from);
                None
            }
            "token_count" => {
                let session_id = estado.session_id.clone()?;
                let turn_id = estado.turn_id.clone()?;
                let model = estado.model.clone()?;
                let occurred_at = Instante(parse_timestamp_iso8601(v["timestamp"].as_str()?)?);

                let u = &payload["info"]["last_token_usage"];
                let campo_u64 = |nome: &str| u.get(nome).and_then(|x| x.as_u64());

                Some(UsoExtraido {
                    session_id,
                    turn_id,
                    model,
                    occurred_at,
                    tokens: Tokens {
                        input: campo_u64("input_tokens"),
                        cache: campo_u64("cached_input_tokens"),
                        output: campo_u64("output_tokens"),
                        reasoning: campo_u64("reasoning_output_tokens"),
                    },
                })
            }
            _ => None,
        },
        _ => None,
    }
}

/// `cwd` de uma sessão — lido só da linha `session_meta` (primeira do
/// arquivo), sem precisar ler o arquivo inteiro para decidir se ele é
/// relevante ao projeto pedido.
fn cwd_da_sessao(primeira_linha: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(primeira_linha).ok()?;
    if v["type"].as_str()? != "session_meta" {
        return None;
    }
    v["payload"]["cwd"].as_str().map(String::from)
}

pub struct CodexAdapter {
    raiz_sessions: PathBuf,
    cwd: PathBuf,
}

impl CodexAdapter {
    pub fn new(cwd: PathBuf) -> Self {
        let raiz_sessions = dirs_home().join(".codex").join("sessions");
        Self { raiz_sessions, cwd }
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Lista recursivamente os `.jsonl` sob `raiz` (estrutura AAAA/MM/DD/*.jsonl,
/// mas não presumimos a profundidade — só filtramos pela extensão).
fn listar_jsonl_recursivo(raiz: &Path, saida: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !raiz.is_dir() {
        return Ok(());
    }
    for entrada in std::fs::read_dir(raiz)? {
        let entrada = entrada?;
        let caminho = entrada.path();
        if caminho.is_dir() {
            listar_jsonl_recursivo(&caminho, saida)?;
        } else if caminho.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            saida.push(caminho);
        }
    }
    Ok(())
}

impl ColetorDeUso for CodexAdapter {
    fn provider_id(&self) -> &str {
        "codex"
    }

    fn tier(&self) -> TierIntegracao {
        TierIntegracao::SessionFiles
    }

    fn campos_disponiveis(&self) -> &[&'static str] {
        &["tokens", "model", "occurred_at", "identificador_estavel"]
        // Sem "custo": Codex via login ChatGPT é assinatura, sem custo por
        // chamada exposto no arquivo de sessão.
    }

    fn coletar(&self, periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta> {
        let mut arquivos = Vec::new();
        listar_jsonl_recursivo(&self.raiz_sessions, &mut arquivos).map_err(|e| ErroColeta {
            motivo: format!("erro listando {}: {e}", self.raiz_sessions.display()),
        })?;

        let cwd_alvo = self.cwd.to_string_lossy().to_string();
        let mut resultado = Vec::new();

        for caminho in arquivos {
            let conteudo = std::fs::read_to_string(&caminho).map_err(|e| ErroColeta {
                motivo: format!("não foi possível ler {}: {e}", caminho.display()),
            })?;

            let Some(primeira_linha) = conteudo.lines().next() else {
                continue;
            };
            if cwd_da_sessao(primeira_linha).as_deref() != Some(cwd_alvo.as_str()) {
                continue; // sessão de outro projeto — nem processamos o resto
            }

            let mut estado = EstadoSessao::default();
            for linha in conteudo.lines() {
                let Some(uso) = processar_linha(&mut estado, linha) else {
                    continue;
                };

                if uso.occurred_at < periodo.desde {
                    continue;
                }
                if let Some(ate) = periodo.ate
                    && uso.occurred_at >= ate
                {
                    continue;
                }

                resultado.push(ConsumoColetado {
                    identificador_estavel: Some(format!("{}:{}", uso.session_id, uso.turn_id)),
                    provider_id: "codex".to_string(),
                    model: uso.model,
                    tokens: uso.tokens,
                    custo_pago: None,
                    billing_mode: BillingMode::Subscription,
                    usage_source: UsageSource::Provider,
                    session_ref: Some(uso.session_id.clone()),
                    occurred_at: uso.occurred_at,
                    client_id: None,
                });
            }
        }

        Ok(resultado)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/codex_sessao.jsonl");

    #[test]
    fn processa_sessao_completa_e_extrai_dois_usos() {
        let mut estado = EstadoSessao::default();
        let usos: Vec<_> = FIXTURE
            .lines()
            .filter_map(|l| processar_linha(&mut estado, l))
            .collect();

        assert_eq!(usos.len(), 2);
        assert_eq!(usos[0].model, "gpt-5.4");
        assert_eq!(usos[0].turn_id, "turn-fixture-0001");
        assert_eq!(usos[0].tokens.input, Some(13375));
        assert_eq!(usos[0].tokens.reasoning, Some(34));

        // Segundo token_count usa last_token_usage (delta), não
        // total_token_usage (acumulado) -- são valores diferentes na fixture,
        // de propósito, para este teste distinguir os dois.
        assert_eq!(usos[1].turn_id, "turn-fixture-0002");
        assert_eq!(usos[1].tokens.input, Some(4625));
    }

    #[test]
    fn nunca_extrai_instructions() {
        let mut estado = EstadoSessao::default();
        for l in FIXTURE.lines() {
            processar_linha(&mut estado, l);
        }
        // Prova indireta: nenhum campo de UsoExtraido ou EstadoSessao tem
        // onde "SEGREDO" caberia -- os dois structs só carregam id/cwd
        // (descartado após uso)/turn_id/model/tokens/timestamp.
        let repr = format!(
            "{:?} {:?} {:?}",
            estado.session_id, estado.turn_id, estado.model
        );
        assert!(!repr.contains("SEGREDO"));
    }

    #[test]
    fn cwd_da_sessao_le_so_a_primeira_linha() {
        let primeira = FIXTURE.lines().next().unwrap();
        assert_eq!(cwd_da_sessao(primeira).as_deref(), Some("/projeto/teste"));
    }

    fn diretorio_temporario_unico() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "brian-teste-adapter-codex-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        p
    }

    #[test]
    fn coletar_ponta_a_ponta_filtra_por_cwd() {
        let raiz = diretorio_temporario_unico();
        let dia = raiz.join("2026").join("07").join("06");
        std::fs::create_dir_all(&dia).unwrap();
        std::fs::write(dia.join("rollout-fixture.jsonl"), FIXTURE).unwrap();

        // Sessão de outro projeto no mesmo diretório de data — deve ser
        // ignorada inteiramente.
        let outra = FIXTURE.replace("/projeto/teste", "/outro/projeto");
        std::fs::write(dia.join("rollout-outro.jsonl"), outra).unwrap();

        let adapter = CodexAdapter {
            raiz_sessions: raiz.clone(),
            cwd: PathBuf::from("/projeto/teste"),
        };

        let itens = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();

        assert_eq!(itens.len(), 2, "só a sessão do projeto certo, 2 usos nela");
        assert!(itens.iter().all(|i| i.provider_id == "codex"));

        std::fs::remove_dir_all(&raiz).ok();
    }

    #[test]
    fn coletar_de_raiz_inexistente_devolve_vazio() {
        let adapter = CodexAdapter {
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
