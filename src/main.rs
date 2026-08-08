//! Brian Core — plano de controle de engenharia de IA.
//!
//! Comportamento aprovado vive em `openspec/`.

pub mod adapters;
pub mod comandos;
pub mod custo;
pub mod domain;
pub mod importacao;
pub mod storage;

use clap::Parser;
use comandos::{Cli, Comando};
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
