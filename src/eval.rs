//! Harness de eval mínimo (grupo 112 do blueprint, pré-requisito de D-13):
//! casos como dado, cada tentativa é um `run` real via `execucao::iniciar_run`,
//! taxa de sucesso classificada com a banda de instabilidade do blueprint
//! §112.4. Sem grading assistido por LLM, sem comparação entre providers —
//! ver design.md.

use crate::domain::{ContextoAtivo, Instante, RunRegistrado, StatusRun};
use crate::execucao::{self, ErroExecucao, PedidoRun};
use crate::storage::Store;
use std::path::{Path, PathBuf};

/// Cliente sintético de eval — nunca o contexto ativo do operador (spec
/// eval-harness: "Runs de eval nunca são atribuídos a cliente real").
pub const CLIENT_ID_EVAL: &str = "eval";

/// Quantas vezes um caso roda por padrão — blueprint §112.4, sem flag de
/// configuração (design.md: N diferente só quando houver necessidade real).
const N_TENTATIVAS: u32 = 3;

#[derive(Debug)]
pub enum ErroEval {
    Json(String),
    CampoObrigatorioAusente(String),
    Execucao(ErroExecucao),
    Storage(String),
}

impl std::fmt::Display for ErroEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(m) => write!(f, "erro lendo caso de eval: {m}"),
            Self::CampoObrigatorioAusente(campo) => {
                write!(f, "caso de eval sem campo obrigatório: {campo}")
            }
            Self::Execucao(e) => write!(f, "{e}"),
            Self::Storage(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for ErroEval {}

/// Um caso de eval — dado, não código (spec: "Caso de eval é dado, não
/// código").
#[derive(Debug, Clone)]
pub struct CasoEval {
    pub id: String,
    pub description: String,
    pub fixture_repo: PathBuf,
    pub base_commit: String,
    pub tarefa: String,
    pub provider_id: String,
    pub model: Option<String>,
    pub gate: Option<String>,
}

fn campo_texto<'a>(v: &'a serde_json::Value, campo: &str) -> Result<&'a str, ErroEval> {
    v.get(campo)
        .and_then(|x| x.as_str())
        .ok_or_else(|| ErroEval::CampoObrigatorioAusente(campo.to_string()))
}

/// Carrega um caso de um único arquivo JSON.
pub fn carregar_caso(caminho: &Path) -> Result<CasoEval, ErroEval> {
    let texto = std::fs::read_to_string(caminho).map_err(|e| ErroEval::Json(e.to_string()))?;
    let v: serde_json::Value =
        serde_json::from_str(&texto).map_err(|e| ErroEval::Json(e.to_string()))?;

    let fixture = v
        .get("fixture")
        .ok_or_else(|| ErroEval::CampoObrigatorioAusente("fixture".to_string()))?;

    Ok(CasoEval {
        id: campo_texto(&v, "id")?.to_string(),
        description: campo_texto(&v, "description")?.to_string(),
        fixture_repo: PathBuf::from(campo_texto(fixture, "repo")?),
        base_commit: campo_texto(fixture, "base_commit")?.to_string(),
        tarefa: campo_texto(&v, "task")?.to_string(),
        provider_id: campo_texto(&v, "provider")?.to_string(),
        model: v.get("model").and_then(|x| x.as_str()).map(str::to_string),
        gate: v.get("gate").and_then(|x| x.as_str()).map(str::to_string),
    })
}

/// Carrega todo `.json` de um diretório (spec: "Novo caso sem tocar código").
pub fn carregar_casos_do_diretorio(dir: &Path) -> Result<Vec<CasoEval>, ErroEval> {
    let entradas = std::fs::read_dir(dir).map_err(|e| ErroEval::Json(e.to_string()))?;
    let mut casos = Vec::new();
    for entrada in entradas {
        let entrada = entrada.map_err(|e| ErroEval::Json(e.to_string()))?;
        let caminho = entrada.path();
        if caminho.extension().and_then(|e| e.to_str()) == Some("json") {
            casos.push(carregar_caso(&caminho)?);
        }
    }
    casos.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(casos)
}

/// Contexto sintético de eval — `project` é o id do caso, para poder filtrar
/// runs de um caso específico sem tabela nova (design.md).
fn contexto_eval(caso: &CasoEval) -> ContextoAtivo {
    ContextoAtivo {
        client_id: CLIENT_ID_EVAL.to_string(),
        project: Some(caso.id.clone()),
        identity_profile_id: String::new(),
        connected_at: Instante(0),
    }
}

/// Roda um caso `N_TENTATIVAS` vezes — cada tentativa é um run real (spec:
/// "Cada tentativa é um run rastreado normalmente").
pub fn rodar_caso(
    store: &dyn Store,
    caso: &CasoEval,
    agora: Instante,
) -> Result<Vec<RunRegistrado>, ErroEval> {
    store
        .upsert_client(CLIENT_ID_EVAL)
        .map_err(|e| ErroEval::Storage(e.to_string()))?;

    let contexto = contexto_eval(caso);
    let mut runs = Vec::with_capacity(N_TENTATIVAS as usize);
    for _ in 0..N_TENTATIVAS {
        let run = execucao::iniciar_run(
            store,
            Some(&contexto),
            &caso.fixture_repo,
            PedidoRun {
                provider_id: &caso.provider_id,
                model: caso.model.as_deref(),
                tarefa: &caso.tarefa,
                gate: caso.gate.as_deref(),
                base_commit: Some(&caso.base_commit),
            },
            agora,
        )
        .map_err(ErroEval::Execucao)?;
        runs.push(run);
    }
    Ok(runs)
}

/// Fração de tentativas com `status == Concluido` (spec: "Tentativa passa só
/// por critério determinístico").
pub fn taxa_de_sucesso(runs: &[RunRegistrado]) -> f64 {
    if runs.is_empty() {
        return 0.0;
    }
    let aprovadas = runs
        .iter()
        .filter(|r| r.status == StatusRun::Concluido)
        .count();
    aprovadas as f64 / runs.len() as f64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Veredito {
    Passou,
    Falhou,
    Instavel,
}

/// Limites exatos do blueprint §112.4: banda `[0.34, 0.66]` é instável (spec:
/// "Taxa intermediária é marcada instável, não passa nem falha").
pub fn classificar(taxa: f64) -> Veredito {
    if taxa > 0.66 {
        Veredito::Passou
    } else if taxa < 0.34 {
        Veredito::Falhou
    } else {
        Veredito::Instavel
    }
}

pub fn formatar_relatorio(caso: &CasoEval, runs: &[RunRegistrado]) -> String {
    let taxa = taxa_de_sucesso(runs);
    let veredito = classificar(taxa);
    let aprovadas = runs
        .iter()
        .filter(|r| r.status == StatusRun::Concluido)
        .count();
    format!(
        "{} — {:?} ({aprovadas}/{} tentativas, taxa {:.0}%)",
        caso.id,
        veredito,
        runs.len(),
        taxa * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(status: StatusRun) -> RunRegistrado {
        RunRegistrado {
            id: "r".into(),
            client_id: CLIENT_ID_EVAL.into(),
            project: None,
            base_commit: "abc".into(),
            worktree_path: "/tmp/x".into(),
            branch: "brian/run_r".into(),
            provider_id: "codex".into(),
            pid: None,
            status,
            custo_equivalente: None,
            started_at: Instante(0),
            finished_at: None,
        }
    }

    #[test]
    fn carregar_caso_le_todos_os_campos() {
        let dir = crate::testutil::dir_temporario_unico("eval");
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("caso.json");
        std::fs::write(
            &caminho,
            r#"{
                "id": "refund-idempotency",
                "description": "Torna refund idempotente",
                "fixture": {"repo": "/tmp/fixtures/checkout-api", "base_commit": "a3f21b8"},
                "task": "Torne a operação de refund idempotente",
                "provider": "codex",
                "model": "gpt-5-codex",
                "gate": "npm test -- RefundService"
            }"#,
        )
        .unwrap();

        let caso = carregar_caso(&caminho).unwrap();
        assert_eq!(caso.id, "refund-idempotency");
        assert_eq!(caso.description, "Torna refund idempotente");
        assert_eq!(
            caso.fixture_repo,
            PathBuf::from("/tmp/fixtures/checkout-api")
        );
        assert_eq!(caso.base_commit, "a3f21b8");
        assert_eq!(caso.tarefa, "Torne a operação de refund idempotente");
        assert_eq!(caso.provider_id, "codex");
        assert_eq!(caso.model.as_deref(), Some("gpt-5-codex"));
        assert_eq!(caso.gate.as_deref(), Some("npm test -- RefundService"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn carregar_caso_sem_campo_obrigatorio_falha_com_erro_claro() {
        let dir = crate::testutil::dir_temporario_unico("eval-incompleto");
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("caso.json");
        std::fs::write(&caminho, r#"{"id": "sem-tarefa"}"#).unwrap();

        let erro = carregar_caso(&caminho).unwrap_err();
        assert!(matches!(erro, ErroEval::CampoObrigatorioAusente(_)));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn carregar_caso_json_malformado_falha_sem_panico() {
        let dir = crate::testutil::dir_temporario_unico("eval-malformado");
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("caso.json");
        std::fs::write(&caminho, "{ nao é json").unwrap();

        let erro = carregar_caso(&caminho).unwrap_err();
        assert!(matches!(erro, ErroEval::Json(_)));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn carregar_casos_do_diretorio_le_so_json_em_ordem() {
        let dir = crate::testutil::dir_temporario_unico("eval-dir");
        std::fs::create_dir_all(&dir).unwrap();
        let corpo = |id: &str| {
            format!(
                r#"{{"id": "{id}", "description": "d", "fixture": {{"repo": "/tmp/r", "base_commit": "abc"}}, "task": "t", "provider": "codex"}}"#
            )
        };
        std::fs::write(dir.join("b.json"), corpo("b-caso")).unwrap();
        std::fs::write(dir.join("a.json"), corpo("a-caso")).unwrap();
        std::fs::write(dir.join("leia-me.txt"), "não é caso de eval").unwrap();

        let casos = carregar_casos_do_diretorio(&dir).unwrap();
        assert_eq!(casos.len(), 2);
        assert_eq!(casos[0].id, "a-caso");
        assert_eq!(casos[1].id, "b-caso");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn taxa_de_sucesso_conta_so_concluidos() {
        let runs = vec![
            run(StatusRun::Concluido),
            run(StatusRun::Concluido),
            run(StatusRun::Falhou),
        ];
        assert!((taxa_de_sucesso(&runs) - (2.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn taxa_de_sucesso_com_lista_vazia_e_zero() {
        assert_eq!(taxa_de_sucesso(&[]), 0.0);
    }

    #[test]
    fn classificar_nos_limites_exatos_e_fora_deles() {
        assert_eq!(classificar(1.0), Veredito::Passou);
        assert_eq!(classificar(0.67), Veredito::Passou);
        assert_eq!(classificar(0.66), Veredito::Instavel);
        assert_eq!(classificar(0.5), Veredito::Instavel);
        assert_eq!(classificar(0.34), Veredito::Instavel);
        assert_eq!(classificar(0.33), Veredito::Falhou);
        assert_eq!(classificar(0.0), Veredito::Falhou);
    }

    #[test]
    fn rodar_caso_produz_tres_runs_com_client_e_project_de_eval() {
        use crate::storage::sqlite::SqliteStore;

        let s = SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();

        let dir = crate::testutil::dir_temporario_unico("eval-run");
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&dir)
                .output()
                .unwrap()
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "teste@teste.com"]);
        git(&["config", "user.name", "Teste"]);
        std::fs::write(dir.join("README.md"), "inicial").unwrap();
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "inicial"]);

        let caso = CasoEval {
            id: "caso-teste".into(),
            description: "d".into(),
            fixture_repo: dir.clone(),
            base_commit: "commit-invalido".into(),
            tarefa: "t".into(),
            provider_id: "codex".into(),
            model: None,
            gate: None,
        };

        // `base_commit` inválido faz a criação do worktree falhar de forma
        // determinística em todas as tentativas -- não depende de `codex`
        // estar instalado para produzir 3 runs reais (todos `Falhou`).
        let runs = rodar_caso(&s, &caso, Instante(0)).unwrap();
        assert_eq!(runs.len(), 3);
        for r in &runs {
            assert_eq!(r.client_id, CLIENT_ID_EVAL);
            assert_eq!(r.project.as_deref(), Some("caso-teste"));
            assert_eq!(r.status, StatusRun::Falhou);
        }

        std::fs::remove_dir_all(&dir).ok();
    }
}
