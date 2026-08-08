//! Adapter Qwen Code (tier `session_files`, D-4).
//!
//! Uma CLI (`qwen`), três backends por conta/billing distintos roteados via
//! `--model` (DeepSeek, Z.AI/GLM, Kimi) — cada um um `provider_id` próprio,
//! mesmo raciocínio já usado para o Grok (design.md: "um provider_id por
//! fonte de billing/conta distinta, não por CLI").
//!
//! Duas fontes locais, ambas com schema nomeado e versionado — a mais limpa
//! de todos os providers investigados nesta change:
//! - `~/.qwen/usage_record.jsonl`: um resumo por sessão, com `sessionId` e
//!   `project` (cwd) — usado só para filtrar por projeto.
//! - `~/.qwen/usage/token-usage-AAAA-MM.jsonl`: um registro por chamada, com
//!   `schemaVersion`, `id` (identificador estável), `model`, tokens e
//!   timestamp ISO-8601.
//!
//! Risco de ToS aceito conscientemente (design.md, R-4).

use super::tempo::parse_timestamp_iso8601;
use crate::domain::{BillingMode, Instante, Tokens, UsageSource};
use crate::importacao::{ColetorDeUso, ConsumoColetado, ErroColeta, TierIntegracao};
use crate::storage::Periodo;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub struct QwenAdapter {
    usage_record_path: PathBuf,
    usage_dir: PathBuf,
    cwd: PathBuf,
    provider_id: &'static str,
    /// Prefixo do nome do modelo que identifica este backend
    /// (`"deepseek"`, `"glm"`, `"kimi"`).
    model_prefix: &'static str,
}

impl QwenAdapter {
    pub fn deepseek(cwd: PathBuf) -> Self {
        Self::new(cwd, "qwen-deepseek", "deepseek")
    }

    pub fn zai(cwd: PathBuf) -> Self {
        Self::new(cwd, "qwen-zai", "glm")
    }

    pub fn kimi(cwd: PathBuf) -> Self {
        Self::new(cwd, "qwen-kimi", "kimi")
    }

    fn new(cwd: PathBuf, provider_id: &'static str, model_prefix: &'static str) -> Self {
        let home = dirs_home();
        Self {
            usage_record_path: home.join(".qwen").join("usage_record.jsonl"),
            usage_dir: home.join(".qwen").join("usage"),
            cwd,
            provider_id,
            model_prefix,
        }
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `sessionId -> project` a partir de `usage_record.jsonl`. Cada linha é um
/// resumo de sessão inteira, não um registro de uso individual.
fn indice_sessoes_do_projeto(
    caminho: &std::path::Path,
    cwd: &str,
) -> std::io::Result<HashSet<String>> {
    if !caminho.is_file() {
        return Ok(HashSet::new());
    }
    let conteudo = std::fs::read_to_string(caminho)?;
    let mut sessoes = HashSet::new();
    for linha in conteudo.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(linha) else {
            continue;
        };
        if v["project"].as_str() == Some(cwd)
            && let Some(sid) = v["sessionId"].as_str()
        {
            sessoes.insert(sid.to_string());
        }
    }
    Ok(sessoes)
}

struct RegistroExtraido {
    id: String,
    session_id: String,
    model: String,
    occurred_at: Instante,
    tokens: Tokens,
}

fn parse_linha(linha: &str) -> Option<RegistroExtraido> {
    let v: serde_json::Value = serde_json::from_str(linha).ok()?;
    let campo_u64 = |nome: &str| v.get(nome).and_then(|x| x.as_u64());

    Some(RegistroExtraido {
        id: v["id"].as_str()?.to_string(),
        session_id: v["sessionId"].as_str()?.to_string(),
        model: v["model"].as_str()?.to_string(),
        occurred_at: Instante(parse_timestamp_iso8601(v["timestamp"].as_str()?)?),
        tokens: Tokens {
            input: campo_u64("inputTokens"),
            cache: campo_u64("cachedTokens"),
            output: campo_u64("outputTokens"),
            reasoning: campo_u64("thoughtsTokens"),
        },
    })
}

fn listar_jsonl(dir: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut arquivos = Vec::new();
    for entrada in std::fs::read_dir(dir)? {
        let caminho = entrada?.path();
        if caminho.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            arquivos.push(caminho);
        }
    }
    Ok(arquivos)
}

impl ColetorDeUso for QwenAdapter {
    fn provider_id(&self) -> &str {
        self.provider_id
    }

    fn tier(&self) -> TierIntegracao {
        TierIntegracao::SessionFiles
    }

    fn campos_disponiveis(&self) -> &[&'static str] {
        &["tokens", "model", "occurred_at", "identificador_estavel"]
        // Sem "custo": sem campo de dólar exposto em nenhum dos dois
        // arquivos. billing_mode fica Unknown -- authType="openai" indica só
        // o protocolo de autenticação, não confirma modo de cobrança.
    }

    fn coletar(&self, periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta> {
        let cwd = self.cwd.to_string_lossy().to_string();
        let sessoes_do_projeto =
            indice_sessoes_do_projeto(&self.usage_record_path, &cwd).map_err(|e| ErroColeta {
                motivo: format!(
                    "não foi possível ler {}: {e}",
                    self.usage_record_path.display()
                ),
            })?;

        if sessoes_do_projeto.is_empty() {
            return Ok(Vec::new());
        }

        let arquivos = listar_jsonl(&self.usage_dir).map_err(|e| ErroColeta {
            motivo: format!("não foi possível listar {}: {e}", self.usage_dir.display()),
        })?;

        let mut resultado = Vec::new();
        let mut vistos: HashMap<String, ()> = HashMap::new();

        for arquivo in arquivos {
            let conteudo = std::fs::read_to_string(&arquivo).map_err(|e| ErroColeta {
                motivo: format!("não foi possível ler {}: {e}", arquivo.display()),
            })?;

            for linha in conteudo.lines() {
                let Some(reg) = parse_linha(linha) else {
                    continue;
                };
                if !sessoes_do_projeto.contains(&reg.session_id) {
                    continue;
                }
                if !reg.model.starts_with(self.model_prefix) {
                    continue;
                }
                if reg.occurred_at < periodo.desde {
                    continue;
                }
                if let Some(ate) = periodo.ate
                    && reg.occurred_at >= ate
                {
                    continue;
                }
                if vistos.insert(reg.id.clone(), ()).is_some() {
                    continue; // mesmo id visto em mais de um arquivo mensal
                }

                resultado.push(ConsumoColetado {
                    identificador_estavel: Some(format!("qwen:{}", reg.id)),
                    provider_id: self.provider_id.to_string(),
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

    const TOKEN_USAGE_FIXTURE: &str = include_str!("fixtures/qwen_token_usage.jsonl");
    const USAGE_RECORD_FIXTURE: &str = include_str!("fixtures/qwen_usage_record.jsonl");

    fn diretorio_temporario_unico() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "brian-teste-adapter-qwen-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        p
    }

    fn montar_fixture() -> (PathBuf, QwenAdapter, QwenAdapter) {
        let raiz = diretorio_temporario_unico();
        let usage_dir = raiz.join("usage");
        std::fs::create_dir_all(&usage_dir).unwrap();
        std::fs::write(raiz.join("usage_record.jsonl"), USAGE_RECORD_FIXTURE).unwrap();
        std::fs::write(
            usage_dir.join("token-usage-2026-08.jsonl"),
            TOKEN_USAGE_FIXTURE,
        )
        .unwrap();

        let cwd = PathBuf::from("/projeto/teste");
        let deepseek = QwenAdapter {
            usage_record_path: raiz.join("usage_record.jsonl"),
            usage_dir: usage_dir.clone(),
            cwd: cwd.clone(),
            provider_id: "qwen-deepseek",
            model_prefix: "deepseek",
        };
        let zai = QwenAdapter {
            usage_record_path: raiz.join("usage_record.jsonl"),
            usage_dir,
            cwd,
            provider_id: "qwen-zai",
            model_prefix: "glm",
        };
        (raiz, deepseek, zai)
    }

    #[test]
    fn separa_backends_por_prefixo_de_modelo() {
        let (raiz, deepseek, zai) = montar_fixture();

        let periodo = Periodo {
            desde: Instante(0),
            ate: None,
        };
        let itens_deepseek = deepseek.coletar(periodo).unwrap();
        let itens_zai = zai.coletar(periodo).unwrap();

        assert_eq!(itens_deepseek.len(), 1);
        assert_eq!(itens_deepseek[0].model, "deepseek-v4-pro");
        assert_eq!(itens_deepseek[0].provider_id, "qwen-deepseek");
        assert_eq!(itens_deepseek[0].tokens.input, Some(9145));
        assert_eq!(itens_deepseek[0].tokens.reasoning, Some(191));

        // fixture tem 2 registros glm-5.2, mas só 1 é da sessão do projeto
        // alvo -- o outro pertence a "sess-fixture-outro".
        assert_eq!(itens_zai.len(), 1);
        assert_eq!(itens_zai[0].model, "glm-5.2");
        assert_eq!(itens_zai[0].tokens.input, Some(32287));

        std::fs::remove_dir_all(&raiz).ok();
    }

    #[test]
    fn filtra_por_projeto_via_usage_record() {
        let (raiz, _deepseek, zai) = montar_fixture();

        let itens = zai
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();

        assert!(
            itens
                .iter()
                .all(|i| i.session_ref.as_deref() != Some("sess-fixture-outro")),
            "sessão de outro projeto não deve aparecer"
        );

        std::fs::remove_dir_all(&raiz).ok();
    }

    #[test]
    fn coletar_sem_usage_record_devolve_vazio() {
        let adapter = QwenAdapter {
            usage_record_path: PathBuf::from("/inexistente/de/proposito.jsonl"),
            usage_dir: PathBuf::from("/tambem/inexistente"),
            cwd: PathBuf::from("/qualquer"),
            provider_id: "qwen-kimi",
            model_prefix: "kimi",
        };
        let r = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn construtores_publicos_usam_provider_id_e_prefixo_certos() {
        let cwd = PathBuf::from("/x");
        assert_eq!(
            QwenAdapter::deepseek(cwd.clone()).provider_id(),
            "qwen-deepseek"
        );
        assert_eq!(QwenAdapter::zai(cwd.clone()).provider_id(), "qwen-zai");
        assert_eq!(QwenAdapter::kimi(cwd).provider_id(), "qwen-kimi");
    }
}
