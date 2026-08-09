//! Brian Core — plano de controle de engenharia de IA.
//!
//! Comportamento aprovado vive em `openspec/`.

pub mod adapters;
pub mod capacidade;
pub mod comandos;
pub mod continuidade;
pub mod custo;
pub mod domain;
pub mod eval;
pub mod execucao;
pub mod identidade;
pub mod importacao;
pub mod router;
pub mod storage;
#[cfg(test)]
mod testutil;
pub mod vault;

use capacidade::ColetorDeCapacidade;
use clap::Parser;
use comandos::{
    Cli, Comando, ComandoContext, ComandoEval, ComandoMemory, ComandoPlans, ComandoVault,
    ComandoWorktree,
};
use importacao::ColetorDeUso;
use std::path::PathBuf;
use std::process::ExitCode;
use storage::{Store, sqlite::SqliteStore};

/// Caminho do banco (docs/harness/ambiente.md).
fn caminho_banco() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".brian").join("brian.db")
}

/// Coletores registrados para o diretório de trabalho atual. Cada um é
/// independente (task 5.6): falha de um não impede os demais.
fn coletores(cwd: PathBuf) -> Vec<Box<dyn ColetorDeUso>> {
    vec![
        Box::new(adapters::claude::ClaudeAdapter::new(cwd.clone())),
        Box::new(adapters::codex::CodexAdapter::new(cwd.clone())),
        Box::new(adapters::copilot::CopilotAdapter::new(cwd.clone())),
        Box::new(adapters::grok::GrokAdapter::new(cwd.clone())),
        Box::new(adapters::gemini::GeminiAdapter::new()),
        Box::new(adapters::qwen::QwenAdapter::deepseek(cwd.clone())),
        Box::new(adapters::qwen::QwenAdapter::zai(cwd.clone())),
        Box::new(adapters::qwen::QwenAdapter::kimi(cwd)),
    ]
}

/// Coletores de plano/quota — só os três providers com fonte verificada
/// (capacidade::PROVIDERS_VERIFICADOS). Grok/Copilot/Qwen não entram: sem
/// fonte, documentado em `capacidade::providers_sem_fonte`, não um
/// `ColetorDeCapacidade` que sempre falharia.
fn coletores_capacidade(cwd: PathBuf) -> Vec<Box<dyn ColetorDeCapacidade>> {
    vec![
        Box::new(adapters::claude::ClaudeAdapter::new(cwd.clone())),
        Box::new(adapters::codex::CodexAdapter::new(cwd)),
        Box::new(adapters::gemini::GeminiAdapter::new()),
    ]
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let db_path = caminho_banco();
    if let Some(pai) = db_path.parent()
        && let Err(e) = std::fs::create_dir_all(pai)
    {
        eprintln!("erro: não foi possível criar {}: {e}", pai.display());
        return ExitCode::FAILURE;
    }

    let store = match SqliteStore::open(db_path.to_string_lossy().as_ref()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("erro abrindo banco: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = store.migrate() {
        eprintln!("erro aplicando migrações: {e}");
        return ExitCode::FAILURE;
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let resultado = match cli.comando {
        Comando::Import { desde, ate } => {
            comandos::executar_import(&store, &coletores(cwd), desde, ate)
        }
        Comando::Attribute {
            usage_record_id,
            client,
        } => comandos::executar_attribute(&store, &usage_record_id, &client),
        Comando::Costs {
            client,
            period,
            by,
            unattributed,
            export,
        } => comandos::executar_costs(&store, client, period, by, unattributed, export),
        Comando::ImportCapacity => {
            comandos::executar_import_capacity(&store, &coletores_capacidade(cwd))
        }
        Comando::Capacity { provider } => comandos::executar_capacity(&store, provider),
        Comando::Plans(ComandoPlans::List) => comandos::executar_plans_list(&store),
        Comando::Plans(ComandoPlans::Allocation { provider, period }) => {
            comandos::executar_plans_allocation(&store, &provider, period)
        }
        Comando::Connect { alvo } => comandos::executar_connect(&store, &alvo),
        Comando::Disconnect => comandos::executar_disconnect(&store),
        Comando::Whoami => comandos::executar_whoami(&store, &coletores_capacidade(cwd)),
        Comando::Context(ComandoContext::List { client }) => {
            comandos::executar_context_list(&store, &client)
        }
        Comando::Context(ComandoContext::Show { id }) => {
            comandos::executar_context_show(&store, &id)
        }
        Comando::Context(ComandoContext::Init {
            client,
            project,
            git_name,
            git_email,
            github_org,
        }) => comandos::executar_context_init(
            &store, client, project, git_name, git_email, github_org,
        ),
        Comando::Vault(ComandoVault::List) => comandos::executar_vault_list(&store),
        Comando::Memory(ComandoMemory::Note { texto }) => {
            comandos::executar_memory_note(&store, texto)
        }
        Comando::Memory(ComandoMemory::Decide { texto, why }) => {
            comandos::executar_memory_decide(&store, texto, why)
        }
        Comando::Continuity => comandos::executar_continuity_show(&store, &cwd),
        Comando::Handoff { provider } => comandos::executar_handoff(&store, &cwd, &provider),
        Comando::Run {
            tarefa,
            provider,
            model,
            gate,
            explain_only,
        } => comandos::executar_run(
            &store,
            &cwd,
            provider.as_deref(),
            model.as_deref(),
            &tarefa,
            gate.as_deref(),
            explain_only,
        ),
        Comando::Recover { run, all } => comandos::executar_recover(&store, run.as_deref(), all),
        Comando::Worktree(ComandoWorktree::List) => comandos::executar_worktree_list(&store),
        Comando::Eval(ComandoEval::Run { case, dir }) => {
            comandos::executar_eval_run(&store, &dir, case.as_deref())
        }
    };

    match resultado {
        Ok(saida) => {
            println!("{saida}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::FAILURE
        }
    }
}
