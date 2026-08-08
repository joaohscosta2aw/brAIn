//! Adapter Claude Code (tier `session_files`, D-4).
//!
//! Lê os arquivos de sessão que o próprio Claude Code grava em
//! `~/.claude/projects/<projeto-codificado>/*.jsonl`. **Somente campos de
//! uso são lidos** — nenhuma linha deste módulo acessa `message.content`.
//!
//! Risco de ToS aceito conscientemente (design.md, R-4) — ver ali antes de
//! estender este adapter para produção real.

use super::tempo::parse_timestamp_iso8601;
use crate::domain::{BillingMode, Instante, Tokens, UsageSource};
use crate::importacao::{ColetorDeUso, ConsumoColetado, ErroColeta, TierIntegracao};
use crate::storage::Periodo;
use std::io::BufRead;
use std::path::{Path, PathBuf};

/// Um registro de uso já extraído de uma linha — sem I/O, testável contra
/// fixture sem tocar no filesystem real (task 6.1).
#[derive(Debug, Clone, PartialEq)]
struct RegistroExtraido {
    message_id: String,
    session_id: String,
    model: String,
    occurred_at: Instante,
    tokens: Tokens,
}

/// Codifica um caminho de working directory no nome de diretório que o
/// próprio Claude Code usa em `~/.claude/projects/` — substitui cada `/` por
/// `-`. Convenção observada, não documentada publicamente; se o Claude Code
/// mudar isso, este adapter para de encontrar sessões e vira `NaoTentado`,
/// não silenciosamente vazio (a ausência de arquivos já é tratada como
/// "nada a importar", não como erro).
fn codificar_diretorio_projeto(cwd: &Path) -> String {
    cwd.to_string_lossy().replace('/', "-")
}

/// Extrai o que interessa de uma linha do arquivo de sessão. `None` para
/// linhas sem `usage` (ex.: linhas de resumo) — não é erro, é o formato
/// normal do arquivo.
///
/// Não lê `message.content` em nenhum caminho — só os campos nomeados aqui.
fn parse_linha(linha: &str) -> Option<RegistroExtraido> {
    let v: serde_json::Value = serde_json::from_str(linha).ok()?;

    let message = v.get("message")?;
    let usage = message.get("usage")?;

    let campo_u64 = |nome: &str| usage.get(nome).and_then(|x| x.as_u64());

    // cache_creation e cache_read são categorias distintas na API da
    // Anthropic; nosso domínio (Tokens::cache) não precisa dessa divisão
    // para o propósito do ledger de v0.0 — soma-se em uma única categoria.
    // Se a divisão vier a importar, é extensão aditiva, não retrabalho.
    let cache = match (
        campo_u64("cache_creation_input_tokens"),
        campo_u64("cache_read_input_tokens"),
    ) {
        (None, None) => None,
        (a, b) => Some(a.unwrap_or(0) + b.unwrap_or(0)),
    };

    Some(RegistroExtraido {
        message_id: message.get("id")?.as_str()?.to_string(),
        session_id: v.get("sessionId").and_then(|x| x.as_str())?.to_string(),
        model: message.get("model")?.as_str()?.to_string(),
        occurred_at: Instante(parse_timestamp_iso8601(v.get("timestamp")?.as_str()?)?),
        tokens: Tokens {
            input: campo_u64("input_tokens"),
            cache,
            output: campo_u64("output_tokens"),
            // A API da Anthropic não expõe tokens de reasoning separados no
            // formato de sessão do Claude Code — ausente, não zero.
            reasoning: None,
        },
    })
}

pub struct ClaudeAdapter {
    raiz_projects: PathBuf,
    cwd: PathBuf,
}

impl ClaudeAdapter {
    pub fn new(cwd: PathBuf) -> Self {
        let raiz_projects = dirs_home().join(".claude").join("projects");
        Self { raiz_projects, cwd }
    }

    fn diretorio_sessoes(&self) -> PathBuf {
        self.raiz_projects
            .join(codificar_diretorio_projeto(&self.cwd))
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

impl ColetorDeUso for ClaudeAdapter {
    fn provider_id(&self) -> &str {
        "claude"
    }

    fn tier(&self) -> TierIntegracao {
        TierIntegracao::SessionFiles
    }

    fn campos_disponiveis(&self) -> &[&'static str] {
        &["tokens", "model", "occurred_at", "identificador_estavel"]
        // Nota: sem "custo" — o arquivo de sessão não traz custo pago por
        // chamada. cost_source vem do catálogo (D-6), resolvido em
        // importacao::importar(), não aqui.
    }

    fn coletar(&self, periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta> {
        let dir = self.diretorio_sessoes();

        let entradas = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            // Diretório ausente = nenhuma sessão para este projeto ainda.
            // Não é erro do provider (task 5.6 reserva erro para fonte
            // indisponível, não para "nada a importar").
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(ErroColeta {
                    motivo: format!("não foi possível ler {}: {e}", dir.display()),
                });
            }
        };

        let mut resultado = Vec::new();

        for entrada in entradas {
            let entrada = entrada.map_err(|e| ErroColeta {
                motivo: format!("erro listando {}: {e}", dir.display()),
            })?;
            let caminho = entrada.path();
            if caminho.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let arquivo = std::fs::File::open(&caminho).map_err(|e| ErroColeta {
                motivo: format!("não foi possível abrir {}: {e}", caminho.display()),
            })?;

            for linha in std::io::BufReader::new(arquivo).lines() {
                let linha = linha.map_err(|e| ErroColeta {
                    motivo: format!("erro lendo {}: {e}", caminho.display()),
                })?;

                let Some(reg) = parse_linha(&linha) else {
                    continue;
                };

                if reg.occurred_at < periodo.desde {
                    continue;
                }
                if let Some(ate) = periodo.ate
                    && reg.occurred_at >= ate
                {
                    continue;
                }

                resultado.push(ConsumoColetado {
                    identificador_estavel: Some(reg.message_id),
                    provider_id: "claude".to_string(),
                    model: reg.model,
                    tokens: reg.tokens,
                    custo_pago: None,
                    billing_mode: BillingMode::Unknown,
                    usage_source: UsageSource::Provider,
                    session_ref: Some(reg.session_id),
                    occurred_at: reg.occurred_at,
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

    const FIXTURE: &str = include_str!("fixtures/claude_sessao.jsonl");

    #[test]
    fn parse_linha_extrai_campos_de_uso_reais() {
        let linhas: Vec<&str> = FIXTURE.lines().collect();
        let extraido = parse_linha(linhas[0]).expect("linha 1 tem usage, deve parsear");

        assert_eq!(extraido.message_id, "msg_FIXTURE0000000000000001");
        assert_eq!(extraido.model, "claude-sonnet-5");
        assert_eq!(extraido.tokens.input, Some(2));
        assert_eq!(extraido.tokens.cache, Some(17472 + 30692));
        assert_eq!(extraido.tokens.output, Some(19));
        assert_eq!(
            extraido.tokens.reasoning, None,
            "Claude não expõe reasoning aqui — ausente, não zero"
        );
    }

    #[test]
    fn parse_linha_ignora_linha_sem_usage() {
        let linhas: Vec<&str> = FIXTURE.lines().collect();
        assert!(
            parse_linha(linhas[2]).is_none(),
            "linha de resumo não tem usage"
        );
    }

    #[test]
    fn parse_linha_nunca_acessa_content() {
        // Prova estrutural: mesmo com um "content" gigante e sensível na
        // linha, o extraído não carrega esse campo em lugar nenhum — o tipo
        // RegistroExtraido não tem onde ele caberia.
        let linha = r#"{"requestId":"r1","sessionId":"s1","timestamp":"2026-08-08T00:00:00.000Z","message":{"id":"m1","model":"claude-sonnet-5","content":[{"type":"text","text":"SEGREDO_DO_CLIENTE_NUNCA_DEVE_APARECER"}],"usage":{"input_tokens":1,"output_tokens":1}}}"#;
        let extraido = parse_linha(linha).unwrap();
        let repr = format!("{extraido:?}");
        assert!(
            !repr.contains("SEGREDO"),
            "conteúdo de mensagem vazou para o registro extraído"
        );
    }

    #[test]
    fn timestamp_iso8601_bate_com_valor_conhecido() {
        // 2026-08-08T13:18:29Z -- conferido com Python datetime.timestamp().
        let t = parse_timestamp_iso8601("2026-08-08T13:18:29.657Z").unwrap();
        assert_eq!(t, 1_786_195_109);
    }

    #[test]
    fn timestamp_epoca_e_zero() {
        assert_eq!(
            parse_timestamp_iso8601("1970-01-01T00:00:00.000Z").unwrap(),
            0
        );
    }

    #[test]
    fn codificacao_de_diretorio_bate_com_a_convencao_observada() {
        assert_eq!(
            codificar_diretorio_projeto(Path::new("/Users/joaohscosta/Repos/BrIAn")),
            "-Users-joaohscosta-Repos-BrIAn"
        );
    }

    #[test]
    fn coletar_de_diretorio_inexistente_devolve_vazio_nao_erro() {
        let adapter = ClaudeAdapter {
            raiz_projects: PathBuf::from("/caminho/que/nao/existe/de/proposito"),
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

    fn diretorio_temporario_unico() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "brian-teste-adapter-claude-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        p
    }

    #[test]
    fn coletar_ponta_a_ponta_le_fixture_de_um_diretorio_real() {
        let raiz = diretorio_temporario_unico();
        let cwd = PathBuf::from("/projeto/teste");
        let dir_sessoes = raiz.join(codificar_diretorio_projeto(&cwd));
        std::fs::create_dir_all(&dir_sessoes).unwrap();
        std::fs::write(dir_sessoes.join("sessao.jsonl"), FIXTURE).unwrap();
        // Arquivo não-jsonl no mesmo diretório: deve ser ignorado, não erro.
        std::fs::write(dir_sessoes.join("nota.txt"), "não é sessão").unwrap();

        let adapter = ClaudeAdapter {
            raiz_projects: raiz.clone(),
            cwd,
        };

        let itens = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();

        assert_eq!(itens.len(), 3, "3 linhas com usage na fixture, 1 sem");
        assert!(itens.iter().all(|i| i.provider_id == "claude"));
        assert!(
            itens
                .iter()
                .any(|i| i.identificador_estavel.as_deref() == Some("msg_FIXTURE0000000000000001"))
        );

        std::fs::remove_dir_all(&raiz).ok();
    }

    #[test]
    fn coletar_respeita_recorte_de_periodo() {
        let raiz = diretorio_temporario_unico();
        let cwd = PathBuf::from("/projeto/teste2");
        let dir_sessoes = raiz.join(codificar_diretorio_projeto(&cwd));
        std::fs::create_dir_all(&dir_sessoes).unwrap();
        std::fs::write(dir_sessoes.join("sessao.jsonl"), FIXTURE).unwrap();

        let adapter = ClaudeAdapter {
            raiz_projects: raiz.clone(),
            cwd,
        };

        // As 3 linhas com usage vão de 13:18:29 a 13:20:15 UTC em 2026-08-08.
        // Recortando só até 13:19:30, a última (13:20:15) fica de fora.
        let ate = Instante(parse_timestamp_iso8601("2026-08-08T13:19:30.000Z").unwrap());
        let itens = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: Some(ate),
            })
            .unwrap();

        assert_eq!(itens.len(), 2, "recorte deve excluir a linha mais recente");

        std::fs::remove_dir_all(&raiz).ok();
    }
}
