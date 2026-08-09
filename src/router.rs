//! Router Fase 1 (blueprint §11, D-8): regras explícitas + override manual,
//! sem termo histórico no scoring (`n≥30` não entra na conta nesta fase).
//! Camada fina acima de `execucao::iniciar_run` — não muda seu contrato, só
//! decide qual `provider_id` chega até ele quando o operador não informa um.

use std::path::Path;

#[derive(Debug)]
pub enum ErroRouter {
    Json(String),
    SemDefault,
}

impl std::fmt::Display for ErroRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(m) => write!(f, "erro lendo regras de roteamento: {m}"),
            Self::SemDefault => write!(
                f,
                "arquivo de regras sem 'default.provider' -- sem isso, \
                 'nenhuma regra casou' não tem resposta"
            ),
        }
    }
}

impl std::error::Error for ErroRouter {}

/// Condições de casamento — campo ausente é curinga (spec: "regra com `when`
/// parcial casa como curinga no campo ausente").
#[derive(Debug, Clone, Default)]
pub struct CondicaoRegra {
    pub client: Option<String>,
    pub project: Option<String>,
}

impl CondicaoRegra {
    fn casa(&self, client_id: &str, project: Option<&str>) -> bool {
        let client_ok = self.client.as_deref().is_none_or(|c| c == client_id);
        let project_ok = self.project.as_deref().is_none_or(|p| Some(p) == project);
        client_ok && project_ok
    }
}

#[derive(Debug, Clone)]
pub struct Regra {
    pub when: CondicaoRegra,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct RegrasDeRoteamento {
    pub regras: Vec<Regra>,
    pub default_provider: String,
}

fn condicao_de(v: &serde_json::Value) -> CondicaoRegra {
    CondicaoRegra {
        client: v.get("client").and_then(|x| x.as_str()).map(str::to_string),
        project: v
            .get("project")
            .and_then(|x| x.as_str())
            .map(str::to_string),
    }
}

/// Carrega `routing/rules.json` — mesmo padrão de `eval::carregar_caso`
/// (navegação via `serde_json::Value`, sem derive).
pub fn carregar_regras(caminho: &Path) -> Result<RegrasDeRoteamento, ErroRouter> {
    let texto = std::fs::read_to_string(caminho).map_err(|e| ErroRouter::Json(e.to_string()))?;
    let v: serde_json::Value =
        serde_json::from_str(&texto).map_err(|e| ErroRouter::Json(e.to_string()))?;

    let default_provider = v
        .get("default")
        .and_then(|d| d.get("provider"))
        .and_then(|p| p.as_str())
        .ok_or(ErroRouter::SemDefault)?
        .to_string();

    let regras = v
        .get("rules")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let provider = item.get("then")?.get("provider")?.as_str()?;
                    Some(Regra {
                        when: item.get("when").map(condicao_de).unwrap_or_default(),
                        provider: provider.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(RegrasDeRoteamento {
        regras,
        default_provider,
    })
}

/// Decisão completa: o provider e, quando veio de regra (não do `default`),
/// a regra responsável — precisa da regra em si, não só do valor do
/// provider, porque regra e `default` podem coincidir no mesmo provider sem
/// que isso signifique que foi o `default` que decidiu.
pub struct Decisao<'a> {
    pub provider: &'a str,
    pub regra: Option<&'a Regra>,
}

/// Primeira regra que casa vence; nenhuma casando usa o `default` (spec:
/// "Regra decide o provider quando não há override", "Nenhuma regra casa usa
/// o default").
pub fn decidir<'a>(
    regras: &'a RegrasDeRoteamento,
    client_id: &str,
    project: Option<&str>,
) -> Decisao<'a> {
    let regra = regras
        .regras
        .iter()
        .find(|r| r.when.casa(client_id, project));
    let provider = regra
        .map(|r| r.provider.as_str())
        .unwrap_or(&regras.default_provider);
    Decisao { provider, regra }
}

/// Conveniência para quando só o provider importa (mesma decisão de
/// `decidir`, sem a informação de qual regra bateu).
pub fn decidir_provider<'a>(
    regras: &'a RegrasDeRoteamento,
    client_id: &str,
    project: Option<&str>,
) -> &'a str {
    decidir(regras, client_id, project).provider
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
        let dir = crate::testutil::dir_temporario_unico(&format!("router-{sufixo}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn primeira_regra_que_casa_decide_entre_varias() {
        let dir = dir_temp("primeira-que-casa");
        let caminho = escrever(
            &dir,
            "rules.json",
            r#"{
                "default": {"provider": "codex"},
                "rules": [
                    {"when": {"client": "xpto"}, "then": {"provider": "claude"}},
                    {"when": {"client": "xpto", "project": "checkout"}, "then": {"provider": "gemini"}}
                ]
            }"#,
        );
        let regras = carregar_regras(&caminho).unwrap();
        // A primeira regra já casa em client="xpto" independente do project
        // -- "checkout" nunca chega a ser avaliado, prova que é a ordem que
        // decide, não a regra "mais específica".
        assert_eq!(
            decidir_provider(&regras, "xpto", Some("checkout")),
            "claude"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn decidir_reporta_a_regra_mesmo_quando_provider_coincide_com_o_default() {
        // Regressão: comparar só o valor do provider decidido contra
        // `default_provider` para inferir a origem é errado quando os dois
        // apontam pro mesmo provider -- teria dito "default" mesmo tendo
        // sido a regra que bateu. `decidir` precisa devolver a regra em si.
        let dir = dir_temp("regra-igual-ao-default");
        let caminho = escrever(
            &dir,
            "rules.json",
            r#"{
                "default": {"provider": "codex"},
                "rules": [
                    {"when": {"project": "router-teste"}, "then": {"provider": "codex"}}
                ]
            }"#,
        );
        let regras = carregar_regras(&caminho).unwrap();
        let decisao = decidir(&regras, "qualquer", Some("router-teste"));
        assert_eq!(decisao.provider, "codex");
        assert!(
            decisao.regra.is_some(),
            "deveria reportar que foi a regra que bateu, não o default"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn regra_com_when_parcial_casa_como_curinga_no_campo_ausente() {
        let dir = dir_temp("curinga");
        let caminho = escrever(
            &dir,
            "rules.json",
            r#"{
                "default": {"provider": "codex"},
                "rules": [
                    {"when": {"project": "checkout"}, "then": {"provider": "claude"}}
                ]
            }"#,
        );
        let regras = carregar_regras(&caminho).unwrap();
        // Sem "client" na condição -- casa pra qualquer cliente, desde que o
        // project bata.
        assert_eq!(
            decidir_provider(&regras, "qualquer-cliente", Some("checkout")),
            "claude"
        );
        assert_eq!(
            decidir_provider(&regras, "outro", Some("checkout")),
            "claude"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn nenhuma_regra_casando_usa_default() {
        let dir = dir_temp("usa-default");
        let caminho = escrever(
            &dir,
            "rules.json",
            r#"{
                "default": {"provider": "codex"},
                "rules": [
                    {"when": {"client": "outro-cliente"}, "then": {"provider": "claude"}}
                ]
            }"#,
        );
        let regras = carregar_regras(&caminho).unwrap();
        assert_eq!(decidir_provider(&regras, "xpto", None), "codex");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn arquivo_sem_default_falha_ao_carregar() {
        let dir = dir_temp("sem-default");
        let caminho = escrever(&dir, "rules.json", r#"{"rules": []}"#);
        let erro = carregar_regras(&caminho).unwrap_err();
        assert!(matches!(erro, ErroRouter::SemDefault));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn arquivo_ausente_falha_com_erro_claro() {
        let erro = carregar_regras(Path::new("/caminho/que/nao/existe/rules.json")).unwrap_err();
        assert!(matches!(erro, ErroRouter::Json(_)));
    }
}
