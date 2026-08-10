//! Model Router (blueprint §12): provider e modelo são decisões separadas.
//! Ponteiro semântico ("coding") resolve provider+tier via `models/pointers.json`,
//! depois tier→nome concreto via `providers/<provider>/models.json` — camada
//! fina, chamada de `comandos::executar_run` antes de `execucao::iniciar_run`,
//! mesma estrutura de `router.rs` (Provider Router).

use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub enum ErroModelRouter {
    Json(String),
    PointerDesconhecido(String),
    /// Nem `primary` nem `fallback` do pointer têm provider disponível —
    /// spec: "Fallback também indisponível recusa o run explicitamente".
    Indisponivel(String),
}

impl std::fmt::Display for ErroModelRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(m) => write!(f, "erro lendo dados do model router: {m}"),
            Self::PointerDesconhecido(p) => write!(f, "pointer '{p}' não configurado"),
            Self::Indisponivel(p) => write!(
                f,
                "pointer '{p}': nem primary nem fallback têm provider com execução verificada"
            ),
        }
    }
}

impl std::error::Error for ErroModelRouter {}

#[derive(Debug, Clone)]
pub struct AlvoPointer {
    pub provider: String,
    pub tier: String,
}

#[derive(Debug, Clone)]
pub struct Pointer {
    pub primary: AlvoPointer,
    pub fallback: Option<AlvoPointer>,
}

/// tier → nome concreto de modelo, para um único provider.
pub type ModelosDoProvider = HashMap<String, String>;

const ORDEM_TIERS: &[&str] = &["strong", "balanced", "cheap"];

fn alvo_de(v: &serde_json::Value) -> Option<AlvoPointer> {
    Some(AlvoPointer {
        provider: v.get("provider")?.as_str()?.to_string(),
        tier: v.get("tier")?.as_str()?.to_string(),
    })
}

pub fn carregar_pointers(caminho: &Path) -> Result<HashMap<String, Pointer>, ErroModelRouter> {
    let texto =
        std::fs::read_to_string(caminho).map_err(|e| ErroModelRouter::Json(e.to_string()))?;
    let v: serde_json::Value =
        serde_json::from_str(&texto).map_err(|e| ErroModelRouter::Json(e.to_string()))?;

    let obj = v
        .get("pointers")
        .and_then(|p| p.as_object())
        .ok_or_else(|| ErroModelRouter::Json("campo 'pointers' ausente ou inválido".into()))?;

    let mut pointers = HashMap::new();
    for (nome, dado) in obj {
        let Some(primary) = dado.get("primary").and_then(alvo_de) else {
            continue;
        };
        let fallback = dado.get("fallback").and_then(alvo_de);
        pointers.insert(nome.clone(), Pointer { primary, fallback });
    }
    Ok(pointers)
}

/// Carrega `providers/<provider>/models.json` para todos os subdiretórios de
/// `providers_dir` — cada um é `{"tiers": {"strong": "...", ...}}`.
pub fn carregar_modelos(
    providers_dir: &Path,
) -> Result<HashMap<String, ModelosDoProvider>, ErroModelRouter> {
    let mut resultado = HashMap::new();
    let Ok(entradas) = std::fs::read_dir(providers_dir) else {
        return Ok(resultado);
    };
    for entrada in entradas {
        let entrada = entrada.map_err(|e| ErroModelRouter::Json(e.to_string()))?;
        if !entrada.path().is_dir() {
            continue;
        }
        let provider = entrada.file_name().to_string_lossy().to_string();
        let caminho = entrada.path().join("models.json");
        if !caminho.exists() {
            continue;
        }
        let texto =
            std::fs::read_to_string(&caminho).map_err(|e| ErroModelRouter::Json(e.to_string()))?;
        let v: serde_json::Value =
            serde_json::from_str(&texto).map_err(|e| ErroModelRouter::Json(e.to_string()))?;
        let tiers = v
            .get("tiers")
            .and_then(|t| t.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, val)| Some((k.clone(), val.as_str()?.to_string())))
                    .collect::<ModelosDoProvider>()
            })
            .unwrap_or_default();
        resultado.insert(provider, tiers);
    }
    Ok(resultado)
}

#[derive(Debug, Clone)]
pub struct Aviso {
    pub tier_pedido: String,
    pub tier_usado: String,
}

/// Resolve `tier` no `provider`, degradando para o tier mais próximo
/// disponível (`ORDEM_TIERS`) quando ausente — spec: "Tier ausente degrada
/// com aviso, nunca em silêncio". `None` = provider sem nenhum tier
/// configurado (equivalente a indisponível).
fn resolver_tier(
    modelos: &HashMap<String, ModelosDoProvider>,
    provider: &str,
    tier_pedido: &str,
) -> Option<(String, Option<Aviso>)> {
    let tiers_do_provider = modelos.get(provider)?;
    if tiers_do_provider.is_empty() {
        return None;
    }
    if let Some(modelo) = tiers_do_provider.get(tier_pedido) {
        return Some((modelo.clone(), None));
    }
    let tier_usado = ORDEM_TIERS
        .iter()
        .find(|t| tiers_do_provider.contains_key(**t))?;
    Some((
        tiers_do_provider[*tier_usado].clone(),
        Some(Aviso {
            tier_pedido: tier_pedido.to_string(),
            tier_usado: tier_usado.to_string(),
        }),
    ))
}

/// Resolve um pointer em `(provider, modelo_concreto, aviso_de_degradacao)`.
/// `disponiveis` é a lista de providers com execução verificada — injetada
/// para a função ficar pura e testável sem depender direto de
/// `execucao::PROVIDERS_EXECUCAO_VERIFICADA`.
pub fn resolver_pointer(
    pointers: &HashMap<String, Pointer>,
    modelos: &HashMap<String, ModelosDoProvider>,
    disponiveis: &[&str],
    nome: &str,
) -> Result<(String, String, Option<Aviso>), ErroModelRouter> {
    let pointer = pointers
        .get(nome)
        .ok_or_else(|| ErroModelRouter::PointerDesconhecido(nome.to_string()))?;

    let candidato = if disponiveis.contains(&pointer.primary.provider.as_str()) {
        Some(&pointer.primary)
    } else {
        pointer
            .fallback
            .as_ref()
            .filter(|f| disponiveis.contains(&f.provider.as_str()))
    };

    let alvo = candidato.ok_or_else(|| ErroModelRouter::Indisponivel(nome.to_string()))?;
    let (modelo, aviso) = resolver_tier(modelos, &alvo.provider, &alvo.tier)
        .ok_or_else(|| ErroModelRouter::Indisponivel(nome.to_string()))?;

    Ok((alvo.provider.clone(), modelo, aviso))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escrever(dir: &Path, nome: &str, conteudo: &str) -> std::path::PathBuf {
        let caminho = dir.join(nome);
        std::fs::write(&caminho, conteudo).unwrap();
        caminho
    }

    fn dir_temp(sufixo: &str) -> std::path::PathBuf {
        crate::testutil::dir_temporario_unico(&format!("model-router-{sufixo}"))
    }

    fn pointers_teste(dir: &std::path::Path) -> HashMap<String, Pointer> {
        std::fs::create_dir_all(dir).unwrap();
        let caminho = escrever(
            dir,
            "pointers.json",
            r#"{
                "pointers": {
                    "coding": {
                        "primary": {"provider": "codex", "tier": "strong"},
                        "fallback": {"provider": "claude", "tier": "strong"}
                    },
                    "reasoning": {
                        "primary": {"provider": "claude", "tier": "strong"},
                        "fallback": {"provider": "codex", "tier": "strong"}
                    },
                    "impossivel": {
                        "primary": {"provider": "grok", "tier": "strong"},
                        "fallback": {"provider": "gemini", "tier": "strong"}
                    }
                }
            }"#,
        );
        carregar_pointers(&caminho).unwrap()
    }

    fn modelos_teste(dir: &std::path::Path) -> HashMap<String, ModelosDoProvider> {
        let providers_dir = dir.join("providers");
        std::fs::create_dir_all(providers_dir.join("codex")).unwrap();
        std::fs::create_dir_all(providers_dir.join("claude")).unwrap();
        escrever(
            &providers_dir.join("codex"),
            "models.json",
            r#"{"tiers": {"strong": "gpt-5.4", "cheap": "gpt-5-mini"}}"#,
        );
        escrever(
            &providers_dir.join("claude"),
            "models.json",
            r#"{"tiers": {"balanced": "claude-sonnet-5"}}"#,
        );
        carregar_modelos(&providers_dir).unwrap()
    }

    #[test]
    fn primary_resolve_direto_quando_disponivel() {
        let dir = dir_temp("primary-direto");
        let pointers = pointers_teste(&dir);
        let modelos = modelos_teste(&dir);

        let (provider, modelo, aviso) =
            resolver_pointer(&pointers, &modelos, &["codex"], "coding").unwrap();
        assert_eq!(provider, "codex");
        assert_eq!(modelo, "gpt-5.4");
        assert!(aviso.is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn primary_indisponivel_cai_para_fallback() {
        let dir = dir_temp("fallback");
        let pointers = pointers_teste(&dir);
        let modelos = modelos_teste(&dir);

        // "reasoning" primary=claude/strong -- claude só tem "balanced"
        // configurado, então cai em degradação, não em fallback. Testamos
        // fallback de fato com "coding": primary=codex indisponível aqui.
        let (provider, _, _) =
            resolver_pointer(&pointers, &modelos, &["claude"], "coding").unwrap();
        assert_eq!(
            provider, "claude",
            "codex indisponível deveria cair no fallback"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn nenhum_dos_dois_disponivel_falha_nomeando_o_pointer() {
        let dir = dir_temp("indisponivel");
        let pointers = pointers_teste(&dir);
        let modelos = modelos_teste(&dir);

        let erro = resolver_pointer(&pointers, &modelos, &["codex"], "impossivel").unwrap_err();
        match erro {
            ErroModelRouter::Indisponivel(nome) => assert_eq!(nome, "impossivel"),
            outro => panic!("esperava Indisponivel, veio {outro:?}"),
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn tier_ausente_degrada_com_aviso() {
        let dir = dir_temp("degrada");
        let pointers = pointers_teste(&dir);
        let modelos = modelos_teste(&dir);

        // "reasoning" primary=claude/strong -- claude só tem "balanced".
        let (provider, modelo, aviso) =
            resolver_pointer(&pointers, &modelos, &["claude"], "reasoning").unwrap();
        assert_eq!(provider, "claude");
        assert_eq!(modelo, "claude-sonnet-5");
        let aviso = aviso.expect("deveria avisar degradação de tier");
        assert_eq!(aviso.tier_pedido, "strong");
        assert_eq!(aviso.tier_usado, "balanced");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn provider_sem_nenhum_tier_e_indisponivel() {
        let dir = dir_temp("sem-tier");
        let pointers = pointers_teste(&dir);
        let modelos = modelos_teste(&dir);
        // "gemini" não tem providers/gemini/models.json nenhum.
        let erro = resolver_pointer(&pointers, &modelos, &["gemini"], "impossivel").unwrap_err();
        assert!(matches!(erro, ErroModelRouter::Indisponivel(_)));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn pointer_desconhecido_falha_claro() {
        let dir = dir_temp("desconhecido");
        let pointers = pointers_teste(&dir);
        let modelos = modelos_teste(&dir);
        let erro = resolver_pointer(&pointers, &modelos, &["codex"], "nao-existe").unwrap_err();
        assert!(matches!(erro, ErroModelRouter::PointerDesconhecido(_)));
        std::fs::remove_dir_all(&dir).ok();
    }
}
