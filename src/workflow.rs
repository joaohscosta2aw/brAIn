//! Workflow Engine (blueprint §15, D-3): máquina de estados determinística
//! sobre fases definidas como dado. Fronteira Workflow×Reasoning (§15.5)
//! respeitada por construção — `avancar` é pura, nunca chama provider; cada
//! fase não-terminal executa como um `execucao::iniciar_run` normal.

use crate::domain::{
    ContextoAtivo, EntradaDeFase, Instante, RunRegistrado, StatusRun, StatusWorkflowRun,
    WorkflowRunRegistrado,
};
use crate::storage::{NovaEntradaDeFase, NovoWorkflowRun, Store};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub enum ErroWorkflow {
    Json(String),
    FaseDesconhecida(String),
}

impl std::fmt::Display for ErroWorkflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(m) => write!(f, "erro lendo definição de workflow: {m}"),
            Self::FaseDesconhecida(id) => write!(f, "fase '{id}' não existe na definição"),
        }
    }
}

impl std::error::Error for ErroWorkflow {}

#[derive(Debug, Clone)]
pub struct FaseDef {
    pub id: String,
    pub role: Option<String>,
    pub model_pointer: Option<String>,
    pub gates: Vec<String>,
    pub on_success: Option<String>,
    pub on_failure: Option<String>,
    pub max_entries: Option<i64>,
    pub on_max_entries: Option<String>,
    pub terminal: bool,
    pub requires_approval: bool,
}

#[derive(Debug, Clone)]
pub struct WorkflowDef {
    pub id: String,
    pub version: i64,
    pub fases: Vec<FaseDef>,
    pub max_total_phases: i64,
    pub max_wall_seconds: Option<i64>,
}

impl WorkflowDef {
    pub fn fase(&self, id: &str) -> Option<&FaseDef> {
        self.fases.iter().find(|f| f.id == id)
    }
}

fn fase_de(v: &serde_json::Value) -> Option<FaseDef> {
    Some(FaseDef {
        id: v.get("id")?.as_str()?.to_string(),
        role: v.get("role").and_then(|x| x.as_str()).map(str::to_string),
        model_pointer: v
            .get("model_pointer")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        gates: v
            .get("gates")
            .and_then(|g| g.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|g| g.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        on_success: v
            .get("on_success")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        on_failure: v
            .get("on_failure")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        max_entries: v.get("max_entries").and_then(|x| x.as_i64()),
        on_max_entries: v
            .get("on_max_entries")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        terminal: v.get("terminal").and_then(|x| x.as_bool()).unwrap_or(false),
        requires_approval: v
            .get("requires_approval")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
    })
}

/// Carrega `workflows/<id>.json` — mesmo padrão de `router::carregar_regras`
/// (navegação via `serde_json::Value`, sem derive).
pub fn carregar_workflow(caminho: &Path) -> Result<WorkflowDef, ErroWorkflow> {
    let texto = std::fs::read_to_string(caminho).map_err(|e| ErroWorkflow::Json(e.to_string()))?;
    carregar_workflow_de_texto(&texto)
}

/// Parseia a definição a partir de um texto JSON já em memória — usado tanto
/// para ler do disco (`carregar_workflow`) quanto para reconstruir a
/// definição a partir do snapshot persistido em `workflow_run.definicao_json`
/// (spec state-machine: "Versão do workflow é congelada no início do run" —
/// `brian workflow approve`, uma chamada de CLI separada, nunca relê o
/// arquivo do disco, só esse snapshot).
pub fn carregar_workflow_de_texto(texto: &str) -> Result<WorkflowDef, ErroWorkflow> {
    let v: serde_json::Value =
        serde_json::from_str(texto).map_err(|e| ErroWorkflow::Json(e.to_string()))?;

    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .ok_or_else(|| ErroWorkflow::Json("campo 'id' ausente".into()))?
        .to_string();
    let version = v.get("version").and_then(|x| x.as_i64()).unwrap_or(1);
    let fases = v
        .get("phases")
        .and_then(|p| p.as_array())
        .map(|arr| arr.iter().filter_map(fase_de).collect())
        .unwrap_or_default();
    let max_total_phases = v
        .get("max_total_phases")
        .and_then(|x| x.as_i64())
        .unwrap_or(8);
    let max_wall_seconds = v.get("max_wall_seconds").and_then(|x| x.as_i64());

    Ok(WorkflowDef {
        id,
        version,
        fases,
        max_total_phases,
        max_wall_seconds,
    })
}

/// `role` → `model_pointer` default (spec phase-execution: "Role da fase
/// resolve para um model pointer") — só usado quando a fase não declara
/// `model_pointer` explícito.
pub fn resolver_model_pointer_do_role(role: &str) -> Option<&'static str> {
    match role {
        "builder" => Some("coding"),
        "planner" => Some("reasoning"),
        "reviewer" => Some("review"),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseOutcome {
    Success,
    Failure,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transicao {
    ProximaFase(String),
    Pausar(String),
    Encerrar(String),
}

/// Estado mínimo necessário para decidir a próxima transição — só dados já
/// em memória, sem I/O (design.md, blueprint §15.5).
pub struct EstadoTransicao<'a> {
    pub current_phase: &'a str,
    pub total_phases: i64,
    pub started_at: Instante,
    pub agora: Instante,
    /// Quantas vezes cada fase já foi entrada até agora, incluindo a
    /// entrada que está prestes a ser decidida.
    pub contagem_por_fase: &'a HashMap<String, i64>,
}

/// Única função de transição do sistema — nenhuma chamada de I/O aqui
/// (spec state-machine: "Transição é determinística e nunca chama provider
/// diretamente"). Limites primeiro (spec: "Limites do workflow encerram o
/// run"), depois pausa por aprovação, depois `max_entries` da fase de
/// destino (design.md, decisão sobre a ambiguidade do pseudocódigo do
/// blueprint).
pub fn avancar(def: &WorkflowDef, estado: &EstadoTransicao, outcome: PhaseOutcome) -> Transicao {
    if estado.total_phases >= def.max_total_phases {
        return Transicao::Encerrar("limite de fases atingido".to_string());
    }
    if let Some(max_wall) = def.max_wall_seconds
        && (estado.agora.0 - estado.started_at.0) >= max_wall
    {
        return Transicao::Encerrar("limite de tempo atingido".to_string());
    }

    let Some(fase) = def.fase(estado.current_phase) else {
        return Transicao::Encerrar(format!(
            "fase '{}' não existe na definição",
            estado.current_phase
        ));
    };

    if outcome == PhaseOutcome::Success && fase.requires_approval {
        return Transicao::Pausar(format!("fase '{}' requer aprovação", fase.id));
    }

    let proxima_id = match outcome {
        PhaseOutcome::Success => &fase.on_success,
        PhaseOutcome::Failure => &fase.on_failure,
    };
    let Some(proxima_id) = proxima_id else {
        return Transicao::Encerrar(format!(
            "fase '{}' não declara transição para esse outcome",
            fase.id
        ));
    };

    let Some(proxima) = def.fase(proxima_id) else {
        return Transicao::Encerrar(format!("fase de destino '{proxima_id}' não existe"));
    };

    if let Some(max) = proxima.max_entries {
        let vezes = estado
            .contagem_por_fase
            .get(proxima_id.as_str())
            .copied()
            .unwrap_or(0);
        if vezes >= max {
            let fallback = proxima
                .on_max_entries
                .clone()
                .unwrap_or_else(|| "escalate".to_string());
            return Transicao::ProximaFase(fallback);
        }
    }

    Transicao::ProximaFase(proxima_id.clone())
}

fn gerar_id(prefixo: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{prefixo}-{nanos}")
}

/// Executa uma fase não-terminal como um run real via
/// `execucao::iniciar_run` — spec phase-execution: "Fase não-terminal
/// executa como um run real". Provider resolvido pela mesma infraestrutura
/// de `routing/provider-rules`/`routing/historical-scoring` (fase não
/// inventa segunda forma de escolher provider); `model_pointer` explícito
/// da fase vence o default do `role` (spec: "Model_pointer explícito da
/// fase vence o default do role").
#[allow(clippy::too_many_arguments)]
fn executar_fase(
    store: &dyn Store,
    contexto: &ContextoAtivo,
    repo: &Path,
    cwd: &Path,
    fase: &FaseDef,
    tarefa: &str,
    scored: bool,
    agora: Instante,
) -> Result<(RunRegistrado, PhaseOutcome), String> {
    let (provider_id, _origem) = if scored {
        let (p, o, _scores) = crate::comandos::resolver_provider_por_score(store)?;
        (p, Some(o))
    } else {
        crate::comandos::resolver_provider(store, cwd, None)?
    };

    let model_pointer_do_role = fase
        .role
        .as_deref()
        .and_then(resolver_model_pointer_do_role);
    let model_pointer = fase.model_pointer.as_deref().or(model_pointer_do_role);
    let (modelo, _origem_modelo) =
        crate::comandos::resolver_modelo(cwd, &provider_id, None, model_pointer)?;

    // Gates da fase concatenados num único comando -- reaproveita 100% o
    // `--gate` de execucao::iniciar_run (spec: "Gates da fase reaproveitam
    // o gate determinístico existente"), sem gate composto novo.
    let gate = if fase.gates.is_empty() {
        None
    } else {
        Some(fase.gates.join(" && "))
    };

    let run = crate::execucao::iniciar_run(
        store,
        Some(contexto),
        repo,
        crate::execucao::PedidoRun {
            provider_id: &provider_id,
            model: modelo.as_deref(),
            tarefa,
            gate: gate.as_deref(),
            base_commit: None,
        },
        agora,
    )
    .map_err(|e| e.to_string())?;

    let outcome = match run.status {
        StatusRun::Concluido => PhaseOutcome::Success,
        _ => PhaseOutcome::Failure,
    };
    Ok((run, outcome))
}

/// Status final de `workflow_run` ao atingir uma fase terminal — convenção
/// desta change (design.md não travou isso explicitamente): `"done"` é
/// sucesso, qualquer outra fase terminal (`"escalate"`, `"fail"`, ...) é
/// tratada como falha/necessidade de intervenção, nunca como sucesso
/// silencioso.
fn status_de_fase_terminal(fase_id: &str) -> StatusWorkflowRun {
    if fase_id == "done" {
        StatusWorkflowRun::Completed
    } else {
        StatusWorkflowRun::Failed
    }
}

/// Roda a máquina de estados até bater fase terminal, limite, ou pausa por
/// aprovação. Cria o `workflow_run` antes de qualquer fase rodar (D-12,
/// mesma disciplina de `execucao::iniciar_run`) — o laço em si é
/// `continuar_workflow`, compartilhado com `aprovar_workflow_run`.
#[allow(clippy::too_many_arguments)]
pub fn rodar_workflow(
    store: &dyn Store,
    contexto: Option<&ContextoAtivo>,
    repo: &Path,
    cwd: &Path,
    def: &WorkflowDef,
    definicao_json: &str,
    tarefa: &str,
    scored: bool,
    agora: Instante,
) -> Result<WorkflowRunRegistrado, String> {
    let contexto_ref = contexto.ok_or("nenhum contexto ativo")?;
    let primeira_fase = def.fases.first().ok_or("workflow sem fases")?;

    let wf_id = gerar_id("wfrun");
    store
        .criar_workflow_run(NovoWorkflowRun {
            id: wf_id.clone(),
            client_id: contexto_ref.client_id.clone(),
            project: contexto_ref.project.clone(),
            workflow_id: def.id.clone(),
            workflow_version: def.version,
            definicao_json: definicao_json.to_string(),
            tarefa: tarefa.to_string(),
            current_phase: primeira_fase.id.clone(),
            started_at: agora,
        })
        .map_err(|e| e.to_string())?;

    continuar_workflow(
        store,
        contexto,
        repo,
        cwd,
        def,
        &wf_id,
        &primeira_fase.id,
        0,
        tarefa,
        scored,
        agora,
    )
}

/// Retoma um `workflow_run` pausado — spec human-approval: "Aprovação
/// explícita retoma o workflow pausado". Só age sobre workflows com
/// `status = Paused`.
#[allow(clippy::too_many_arguments)]
pub fn aprovar_workflow_run(
    store: &dyn Store,
    contexto: Option<&ContextoAtivo>,
    repo: &Path,
    cwd: &Path,
    workflow_run_id: &str,
    scored: bool,
    agora: Instante,
) -> Result<WorkflowRunRegistrado, String> {
    let contexto = contexto.ok_or("nenhum contexto ativo")?;
    let wf = store
        .workflow_run(workflow_run_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("workflow_run '{workflow_run_id}' não encontrado"))?;

    if wf.status != StatusWorkflowRun::Paused {
        return Err(format!(
            "workflow_run '{workflow_run_id}' não está pausado (status atual: {:?})",
            wf.status
        ));
    }

    // Reconstrói a definição a partir do snapshot persistido na criação, não
    // do arquivo no disco -- spec: "Versão do workflow é congelada no início
    // do run".
    let def = carregar_workflow_de_texto(&wf.definicao_json).map_err(|e| e.to_string())?;
    let fase_pausada = def
        .fase(&wf.current_phase)
        .ok_or_else(|| format!("fase '{}' não existe na definição", wf.current_phase))?;
    let Some(proxima_id) = &fase_pausada.on_success else {
        return Err(format!(
            "fase '{}' não declara on_success para retomar",
            fase_pausada.id
        ));
    };

    store
        .atualizar_workflow_run(
            workflow_run_id,
            proxima_id,
            StatusWorkflowRun::Running,
            None,
            wf.total_phases,
            None,
        )
        .map_err(|e| e.to_string())?;

    // Continua o laço a partir da fase seguinte -- reaproveita a mesma
    // máquina de `rodar_workflow`, agora já com o `workflow_run` existente.
    // `tarefa` vem do próprio workflow_run (spec/design: nunca re-digitada).
    continuar_workflow(
        store,
        Some(contexto),
        repo,
        cwd,
        &def,
        workflow_run_id,
        proxima_id,
        wf.total_phases,
        &wf.tarefa,
        scored,
        agora,
    )
}

/// Núcleo do laço, compartilhado entre `rodar_workflow` (workflow_run novo)
/// e `aprovar_workflow_run` (workflow_run existente retomado).
#[allow(clippy::too_many_arguments)]
fn continuar_workflow(
    store: &dyn Store,
    contexto: Option<&ContextoAtivo>,
    repo: &Path,
    cwd: &Path,
    def: &WorkflowDef,
    wf_id: &str,
    fase_inicial: &str,
    total_phases_inicial: i64,
    tarefa: &str,
    scored: bool,
    agora: Instante,
) -> Result<WorkflowRunRegistrado, String> {
    let contexto = contexto.ok_or("nenhum contexto ativo")?;
    let mut contagem_por_fase: HashMap<String, i64> = HashMap::new();
    let mut fase_atual_id = fase_inicial.to_string();
    let mut total_phases = total_phases_inicial;
    let started_at = store
        .workflow_run(wf_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("workflow_run '{wf_id}' não encontrado"))?
        .started_at;

    loop {
        let Some(fase) = def.fase(&fase_atual_id) else {
            return Err(format!("fase '{fase_atual_id}' não existe na definição"));
        };

        if fase.terminal {
            let status = status_de_fase_terminal(&fase.id);
            store
                .atualizar_workflow_run(wf_id, &fase.id, status, None, total_phases, Some(agora))
                .map_err(|e| e.to_string())?;
            return store
                .workflow_run(wf_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "workflow_run sumiu depois de atualizar".to_string());
        }

        *contagem_por_fase.entry(fase.id.clone()).or_insert(0) += 1;
        total_phases += 1;

        let resultado = executar_fase(store, contexto, repo, cwd, fase, tarefa, scored, agora);
        let (outcome, run_id) = match &resultado {
            Ok((run, outcome)) => (*outcome, Some(run.id.clone())),
            Err(_) => (PhaseOutcome::Failure, None),
        };
        let outcome_str = match outcome {
            PhaseOutcome::Success => "success",
            PhaseOutcome::Failure => "failure",
        };

        let entrada = store
            .registrar_entrada_fase(NovaEntradaDeFase {
                id: gerar_id("entrada"),
                workflow_run_id: wf_id.to_string(),
                phase_id: fase.id.clone(),
                run_id,
                entrada_numero: contagem_por_fase[&fase.id],
                started_at: agora,
            })
            .map_err(|e| e.to_string())?;
        store
            .concluir_entrada_fase(&entrada.id, outcome_str, agora)
            .map_err(|e| e.to_string())?;

        let estado = EstadoTransicao {
            current_phase: &fase.id,
            total_phases,
            started_at,
            agora,
            contagem_por_fase: &contagem_por_fase,
        };
        let fase_id_atual = fase.id.clone();
        let transicao = avancar(def, &estado, outcome);

        match transicao {
            Transicao::ProximaFase(proxima) => {
                fase_atual_id = proxima;
                store
                    .atualizar_workflow_run(
                        wf_id,
                        &fase_atual_id,
                        StatusWorkflowRun::Running,
                        None,
                        total_phases,
                        None,
                    )
                    .map_err(|e| e.to_string())?;
            }
            Transicao::Pausar(motivo) => {
                store
                    .atualizar_workflow_run(
                        wf_id,
                        &fase_id_atual,
                        StatusWorkflowRun::Paused,
                        Some(&motivo),
                        total_phases,
                        None,
                    )
                    .map_err(|e| e.to_string())?;
                return store
                    .workflow_run(wf_id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "workflow_run sumiu depois de atualizar".to_string());
            }
            Transicao::Encerrar(motivo) => {
                store
                    .atualizar_workflow_run(
                        wf_id,
                        &fase_id_atual,
                        StatusWorkflowRun::Failed,
                        Some(&motivo),
                        total_phases,
                        Some(agora),
                    )
                    .map_err(|e| e.to_string())?;
                return store
                    .workflow_run(wf_id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "workflow_run sumiu depois de atualizar".to_string());
            }
        }
    }
}

pub fn entradas_do_workflow_run(
    store: &dyn Store,
    workflow_run_id: &str,
) -> Result<Vec<EntradaDeFase>, String> {
    store
        .entradas_do_workflow_run(workflow_run_id)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_temp(sufixo: &str) -> std::path::PathBuf {
        let dir = crate::testutil::dir_temporario_unico(&format!("workflow-{sufixo}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn store_com_contexto_ativo() -> crate::storage::sqlite::SqliteStore {
        let s = crate::storage::sqlite::SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s.upsert_client("xpto").unwrap();
        s.criar_perfil(crate::storage::NovoPerfil {
            id: "p1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            git_author_name: None,
            git_author_email: None,
            github_org: None,
            bindings: vec![],
            created_at: Instante(0),
        })
        .unwrap();
        crate::identidade::conectar(&s, "xpto", Some("checkout-api"), Instante(1)).unwrap();
        s
    }

    fn repo_git_temporario(sufixo: &str) -> std::path::PathBuf {
        let dir = dir_temp(sufixo);
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
        dir
    }

    fn workflow_duas_fases_teste() -> WorkflowDef {
        WorkflowDef {
            id: "duas-fases".into(),
            version: 1,
            max_total_phases: 8,
            max_wall_seconds: Some(1800),
            fases: vec![
                FaseDef {
                    id: "implement".into(),
                    role: Some("builder".into()),
                    model_pointer: None,
                    gates: vec![],
                    on_success: Some("verify".into()),
                    on_failure: Some("fail".into()),
                    max_entries: None,
                    on_max_entries: None,
                    terminal: false,
                    requires_approval: false,
                },
                FaseDef {
                    id: "verify".into(),
                    role: Some("builder".into()),
                    model_pointer: None,
                    gates: vec![],
                    on_success: Some("done".into()),
                    on_failure: Some("fail".into()),
                    max_entries: None,
                    on_max_entries: None,
                    terminal: false,
                    requires_approval: false,
                },
                FaseDef {
                    id: "fail".into(),
                    role: None,
                    model_pointer: None,
                    gates: vec![],
                    on_success: None,
                    on_failure: None,
                    max_entries: None,
                    on_max_entries: None,
                    terminal: true,
                    requires_approval: false,
                },
                FaseDef {
                    id: "done".into(),
                    role: None,
                    model_pointer: None,
                    gates: vec![],
                    on_success: None,
                    on_failure: None,
                    max_entries: None,
                    on_max_entries: None,
                    terminal: true,
                    requires_approval: false,
                },
            ],
        }
    }

    #[test]
    fn workflow_falha_deterministicamente_sem_provider_disponivel() {
        // Sem routing/rules.json em `repo` -- resolver_provider falha ao
        // carregar regras, sem precisar de `codex` real. A fase falha
        // (Failure), transiciona para "fail" (terminal), e o workflow_run
        // termina como Failed -- tudo isso sem nenhum processo externo.
        let s = store_com_contexto_ativo();
        let repo = repo_git_temporario("sem-provider");
        let def = workflow_duas_fases_teste();
        let contexto = s.contexto_ativo().unwrap();

        let wf = rodar_workflow(
            &s,
            contexto.as_ref(),
            &repo,
            &repo,
            &def,
            "{}",
            "tarefa de teste",
            false,
            Instante(0),
        )
        .unwrap();

        assert_eq!(wf.status, StatusWorkflowRun::Failed);
        assert_eq!(wf.current_phase, "fail");
        assert_eq!(
            wf.total_phases, 1,
            "só 'implement' rodou antes de cair em 'fail'"
        );

        let entradas = s.entradas_do_workflow_run(&wf.id).unwrap();
        assert_eq!(entradas.len(), 1);
        assert_eq!(entradas[0].phase_id, "implement");
        assert_eq!(entradas[0].outcome.as_deref(), Some("failure"));
        assert_eq!(
            entradas[0].run_id, None,
            "resolução de provider falhou antes de existir qualquer run"
        );

        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn workflow_run_persistido_antes_de_qualquer_fase_mesmo_se_a_primeira_falhar() {
        // D-12 no nível do workflow: o workflow_run já existe no banco
        // mesmo que a primeira fase falhe irrecuperavelmente.
        let s = store_com_contexto_ativo();
        let repo = repo_git_temporario("d12");
        let def = workflow_duas_fases_teste();
        let contexto = s.contexto_ativo().unwrap();

        let wf = rodar_workflow(
            &s,
            contexto.as_ref(),
            &repo,
            &repo,
            &def,
            "{}",
            "tarefa de teste",
            false,
            Instante(0),
        )
        .unwrap();

        assert!(s.workflow_run(&wf.id).unwrap().is_some());
        std::fs::remove_dir_all(&repo).ok();
    }

    fn workflow_fast_teste() -> WorkflowDef {
        WorkflowDef {
            id: "fast".into(),
            version: 1,
            max_total_phases: 8,
            max_wall_seconds: Some(1800),
            fases: vec![
                FaseDef {
                    id: "implement".into(),
                    role: Some("builder".into()),
                    model_pointer: None,
                    gates: vec![],
                    on_success: Some("verify".into()),
                    on_failure: Some("fail".into()),
                    max_entries: None,
                    on_max_entries: None,
                    terminal: false,
                    requires_approval: false,
                },
                FaseDef {
                    id: "verify".into(),
                    role: Some("builder".into()),
                    model_pointer: Some("quick".into()),
                    gates: vec!["cargo build".into()],
                    on_success: Some("done".into()),
                    on_failure: Some("fix".into()),
                    max_entries: None,
                    on_max_entries: None,
                    terminal: false,
                    requires_approval: false,
                },
                FaseDef {
                    id: "fix".into(),
                    role: Some("builder".into()),
                    model_pointer: None,
                    gates: vec![],
                    on_success: Some("verify".into()),
                    on_failure: Some("escalate".into()),
                    max_entries: Some(2),
                    on_max_entries: Some("escalate".into()),
                    terminal: false,
                    requires_approval: false,
                },
                FaseDef {
                    id: "escalate".into(),
                    role: None,
                    model_pointer: None,
                    gates: vec![],
                    on_success: None,
                    on_failure: None,
                    max_entries: None,
                    on_max_entries: None,
                    terminal: true,
                    requires_approval: false,
                },
                FaseDef {
                    id: "fail".into(),
                    role: None,
                    model_pointer: None,
                    gates: vec![],
                    on_success: None,
                    on_failure: None,
                    max_entries: None,
                    on_max_entries: None,
                    terminal: true,
                    requires_approval: false,
                },
                FaseDef {
                    id: "done".into(),
                    role: None,
                    model_pointer: None,
                    gates: vec![],
                    on_success: None,
                    on_failure: None,
                    max_entries: None,
                    on_max_entries: None,
                    terminal: true,
                    requires_approval: false,
                },
            ],
        }
    }

    #[test]
    fn carregar_workflow_le_todos_os_campos() {
        let dir = dir_temp("carregar");
        let caminho = dir.join("fast.json");
        std::fs::write(
            &caminho,
            r#"{
                "id": "fast",
                "version": 1,
                "phases": [
                    {"id": "implement", "role": "builder", "on_success": "done", "on_failure": "fail"},
                    {"id": "done", "terminal": true},
                    {"id": "fail", "terminal": true}
                ],
                "max_total_phases": 8,
                "max_wall_seconds": 1800
            }"#,
        )
        .unwrap();

        let def = carregar_workflow(&caminho).unwrap();
        assert_eq!(def.id, "fast");
        assert_eq!(def.version, 1);
        assert_eq!(def.fases.len(), 3);
        assert_eq!(def.max_total_phases, 8);
        assert_eq!(def.max_wall_seconds, Some(1800));

        let implement = def.fase("implement").unwrap();
        assert_eq!(implement.role.as_deref(), Some("builder"));
        assert_eq!(implement.on_success.as_deref(), Some("done"));
        assert!(!implement.terminal);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolver_model_pointer_do_role_mapeia_os_tres_roles() {
        assert_eq!(resolver_model_pointer_do_role("builder"), Some("coding"));
        assert_eq!(resolver_model_pointer_do_role("planner"), Some("reasoning"));
        assert_eq!(resolver_model_pointer_do_role("reviewer"), Some("review"));
        assert_eq!(resolver_model_pointer_do_role("desconhecido"), None);
    }

    #[test]
    fn transicao_simples_sucesso_e_falha() {
        let def = workflow_fast_teste();
        let contagem = HashMap::new();
        let estado = EstadoTransicao {
            current_phase: "implement",
            total_phases: 1,
            started_at: Instante(0),
            agora: Instante(10),
            contagem_por_fase: &contagem,
        };

        assert_eq!(
            avancar(&def, &estado, PhaseOutcome::Success),
            Transicao::ProximaFase("verify".to_string())
        );
        assert_eq!(
            avancar(&def, &estado, PhaseOutcome::Failure),
            Transicao::ProximaFase("fail".to_string())
        );
    }

    #[test]
    fn max_entries_escala_para_on_max_entries() {
        let def = workflow_fast_teste();
        let mut contagem = HashMap::new();
        contagem.insert("fix".to_string(), 2); // já entrou 2 vezes, no limite

        let estado = EstadoTransicao {
            current_phase: "verify",
            total_phases: 3,
            started_at: Instante(0),
            agora: Instante(10),
            contagem_por_fase: &contagem,
        };

        // verify.on_failure = fix, mas fix já bateu max_entries=2.
        assert_eq!(
            avancar(&def, &estado, PhaseOutcome::Failure),
            Transicao::ProximaFase("escalate".to_string())
        );
    }

    #[test]
    fn max_total_phases_encerra_antes_de_decidir_proxima_fase() {
        let def = workflow_fast_teste();
        let contagem = HashMap::new();
        let estado = EstadoTransicao {
            current_phase: "implement",
            total_phases: 8, // == max_total_phases
            started_at: Instante(0),
            agora: Instante(10),
            contagem_por_fase: &contagem,
        };

        assert!(matches!(
            avancar(&def, &estado, PhaseOutcome::Success),
            Transicao::Encerrar(_)
        ));
    }

    #[test]
    fn max_wall_seconds_encerra_antes_de_decidir_proxima_fase() {
        let def = workflow_fast_teste();
        let contagem = HashMap::new();
        let estado = EstadoTransicao {
            current_phase: "implement",
            total_phases: 1,
            started_at: Instante(0),
            agora: Instante(1800), // == max_wall_seconds
            contagem_por_fase: &contagem,
        };

        assert!(matches!(
            avancar(&def, &estado, PhaseOutcome::Success),
            Transicao::Encerrar(_)
        ));
    }

    #[test]
    fn requires_approval_pausa_em_vez_de_avancar() {
        let mut def = workflow_fast_teste();
        let verify = def.fases.iter_mut().find(|f| f.id == "verify").unwrap();
        verify.requires_approval = true;

        let contagem = HashMap::new();
        let estado = EstadoTransicao {
            current_phase: "verify",
            total_phases: 2,
            started_at: Instante(0),
            agora: Instante(10),
            contagem_por_fase: &contagem,
        };

        assert!(matches!(
            avancar(&def, &estado, PhaseOutcome::Success),
            Transicao::Pausar(_)
        ));
    }

    #[test]
    fn avancar_nao_faz_io() {
        // Verificação estrutural (mesmo padrão de
        // execucao::iniciar_run_chama_executar_provider_uma_unica_vez):
        // `avancar` não pode conter chamadas a std::fs/std::process/
        // Command -- só recebe dados já em memória.
        let fonte = include_str!("workflow.rs");
        let corpo_avancar = fonte
            .split("pub fn avancar(")
            .nth(1)
            .unwrap()
            .split("\n#[cfg(test)]")
            .next()
            .unwrap();
        assert!(!corpo_avancar.contains("std::fs::"));
        assert!(!corpo_avancar.contains("Command::new"));
    }

    #[test]
    fn aprovar_workflow_run_nunca_le_arquivo_do_disco() {
        // Verificação estrutural (spec: "Versão do workflow é congelada no
        // início do run"): `aprovar_workflow_run` só pode reconstruir a
        // definição via `carregar_workflow_de_texto` (o snapshot
        // persistido), nunca `carregar_workflow` (que lê do disco) nem
        // `std::fs::read`.
        let fonte = include_str!("workflow.rs");
        let corpo = fonte
            .split("pub fn aprovar_workflow_run(")
            .nth(1)
            .unwrap()
            .split("\nfn continuar_workflow")
            .next()
            .unwrap();
        assert!(!corpo.contains("std::fs::read"));
        assert!(!corpo.contains("carregar_workflow("));
        assert!(corpo.contains("carregar_workflow_de_texto("));
    }

    #[test]
    fn aprovar_workflow_nao_pausado_falha_com_erro_claro() {
        let s = store_com_contexto_ativo();
        let repo = repo_git_temporario("aprovar-nao-pausado");
        let def = workflow_duas_fases_teste();
        let contexto = s.contexto_ativo().unwrap();

        s.criar_workflow_run(NovoWorkflowRun {
            id: "wf1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            workflow_id: def.id.clone(),
            workflow_version: def.version,
            definicao_json: "{}".into(),
            tarefa: "tarefa de teste".into(),
            current_phase: "implement".into(),
            started_at: Instante(0),
        })
        .unwrap();

        let erro = aprovar_workflow_run(
            &s,
            contexto.as_ref(),
            &repo,
            &repo,
            "wf1",
            false,
            Instante(10),
        )
        .unwrap_err();
        assert!(erro.contains("não está pausado"));
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn aprovar_workflow_pausado_avanca_corretamente() {
        // A definição é reconstruída por `aprovar_workflow_run` a partir do
        // snapshot persistido (spec: "Versão do workflow é congelada no
        // início do run") -- por isso o JSON, não uma struct Rust em
        // memória, é o que precisa estar em `definicao_json`.
        let s = store_com_contexto_ativo();
        let repo = repo_git_temporario("aprovar-pausado");
        let definicao_json = r#"{
            "id": "com-aprovacao",
            "version": 1,
            "phases": [
                {"id": "review", "on_success": "done", "on_failure": "fail", "requires_approval": true},
                {"id": "done", "terminal": true},
                {"id": "fail", "terminal": true}
            ],
            "max_total_phases": 8
        }"#;
        let contexto = s.contexto_ativo().unwrap();

        s.criar_workflow_run(NovoWorkflowRun {
            id: "wf1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            workflow_id: "com-aprovacao".into(),
            workflow_version: 1,
            definicao_json: definicao_json.into(),
            tarefa: "tarefa de teste".into(),
            current_phase: "review".into(),
            started_at: Instante(0),
        })
        .unwrap();
        s.atualizar_workflow_run(
            "wf1",
            "review",
            StatusWorkflowRun::Paused,
            Some("aguardando aprovação"),
            1,
            None,
        )
        .unwrap();

        let wf = aprovar_workflow_run(
            &s,
            contexto.as_ref(),
            &repo,
            &repo,
            "wf1",
            false,
            Instante(10),
        )
        .unwrap();

        assert_eq!(wf.status, StatusWorkflowRun::Completed);
        assert_eq!(wf.current_phase, "done");
        std::fs::remove_dir_all(&repo).ok();
    }
}
