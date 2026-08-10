//! Ciclo de vida de um run: worktree isolado (D-7), persistência antes de
//! efeito colateral (D-12), execução do provider, recuperação de órfão,
//! trilha de proveniência no commit (grupos 4-6).
//!
//! Não faz SQL — recebe/grava via `Store`. Mesma separação de
//! `capacidade.rs`/`identidade.rs`/`continuidade.rs`.

use crate::domain::{ContextoAtivo, Instante, RunRegistrado, StatusRun};
use crate::storage::{NovoEvento, NovoRun, Result as StorageResult, Store};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub enum ErroExecucao {
    SemContextoAtivo,
    /// Provider sem execução não-interativa verificada como segura
    /// (design.md: "só codex executa tarefas nesta change").
    ProviderNaoSuportado(String),
    Git(String),
    Storage(String),
}

impl std::fmt::Display for ErroExecucao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SemContextoAtivo => write!(f, "nenhum contexto ativo"),
            Self::ProviderNaoSuportado(p) => write!(
                f,
                "'{p}' não tem execução não-interativa verificada como segura nesta change"
            ),
            Self::Git(m) => write!(f, "erro de git: {m}"),
            Self::Storage(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for ErroExecucao {}

/// Providers com execução não-interativa verificada como segura — mesma
/// disciplina de honestidade de `PROVIDERS_ISOLAMENTO_VERIFICADO`
/// (`identidade.rs`) e `providers_sem_fonte` (`capacidade.rs`): só cresce
/// quando alguém verifica de verdade (design.md).
pub const PROVIDERS_EXECUCAO_VERIFICADA: &[&str] = &["codex", "grok"];

fn git(repo: &Path, args: &[&str]) -> Result<String, ErroExecucao> {
    let saida = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| ErroExecucao::Git(format!("erro executando git {args:?}: {e}")))?;
    if !saida.status.success() {
        return Err(ErroExecucao::Git(format!(
            "git {args:?} falhou: {}",
            String::from_utf8_lossy(&saida.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&saida.stdout).trim().to_string())
}

/// Commit atual em `repo` (`HEAD`) — usado como `base_commit` do run quando o
/// operador não especifica um.
pub fn commit_atual(repo: &Path) -> Result<String, ErroExecucao> {
    git(repo, &["rev-parse", "HEAD"])
}

/// Cria o worktree dedicado do run (spec isolated-run: "Worktree dedicado a
/// partir de um commit base"). Caminho fixo fora do repositório do usuário
/// (blueprint §109.2, design.md).
pub fn criar_worktree(
    repo: &Path,
    base_commit: &str,
    run_id: &str,
) -> Result<(PathBuf, String), ErroExecucao> {
    let worktree_path = dirs_home()
        .join(".brian")
        .join("worktrees")
        .join(format!("run_{run_id}"));
    let branch = format!("brian/run_{run_id}");

    git(
        repo,
        &[
            "worktree",
            "add",
            worktree_path.to_str().unwrap_or_default(),
            "-b",
            &branch,
            base_commit,
        ],
    )?;

    Ok((worktree_path, branch))
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn registrar_evento(
    store: &dyn Store,
    run_id: &str,
    tipo: &str,
    detalhe: Option<String>,
    agora: Instante,
) {
    // Falha ao registrar evento nunca impede o run de continuar -- o evento é
    // observabilidade, não parte do caminho crítico (design.md: log local
    // simples, não um subsistema do qual o run depende para funcionar).
    let _ = store.registrar_evento_run(NovoEvento {
        id: gerar_id(),
        run_id: run_id.to_string(),
        tipo: tipo.to_string(),
        detalhe,
        ocorrido_em: agora,
    });
}

fn gerar_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("run-{nanos}")
}

/// Resultado bruto da execução do provider — só o que basta para decidir
/// sucesso/falha; saída completa não é guardada (blueprint §39.3: "saída
/// bruta... não vai para o trace").
struct ResultadoProvider {
    sucesso: bool,
    resumo_stderr: String,
}

fn executar_provider(
    provider_id: &str,
    worktree: &Path,
    tarefa: &str,
    model: Option<&str>,
) -> Result<ResultadoProvider, ErroExecucao> {
    if !PROVIDERS_EXECUCAO_VERIFICADA.contains(&provider_id) {
        return Err(ErroExecucao::ProviderNaoSuportado(provider_id.to_string()));
    }

    match provider_id {
        "codex" => executar_codex(worktree, tarefa, model),
        "grok" => executar_grok(worktree, tarefa, model),
        _ => unreachable!("já filtrado por PROVIDERS_EXECUCAO_VERIFICADA acima"),
    }
}

fn saida_para_resultado(saida: std::process::Output) -> ResultadoProvider {
    ResultadoProvider {
        sucesso: saida.status.success(),
        resumo_stderr: String::from_utf8_lossy(&saida.stderr)
            .lines()
            .take(3)
            .collect::<Vec<_>>()
            .join(" | "),
    }
}

/// `codex exec -s workspace-write -C <worktree> <tarefa>`: sandbox do
/// próprio Codex restringe escrita ao workspace, sem flag "dangerously"
/// (design.md, task 4.4). `-m` só é passado quando o operador escolheu um
/// modelo explicitamente -- sem isso, Codex usa o próprio default, que o
/// trailer de proveniência registra como "default" (honesto: Brian não
/// sabe qual modelo o default do Codex resolve para).
fn executar_codex(
    worktree: &Path,
    tarefa: &str,
    model: Option<&str>,
) -> Result<ResultadoProvider, ErroExecucao> {
    let mut args = vec!["exec", "-s", "workspace-write", "-C"];
    let worktree_str = worktree.to_str().unwrap_or_default();
    args.push(worktree_str);
    if let Some(m) = model {
        args.push("-m");
        args.push(m);
    }
    args.push(tarefa);

    let saida = Command::new("codex")
        .args(&args)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| ErroExecucao::Git(format!("erro executando codex exec: {e}")))?;

    Ok(saida_para_resultado(saida))
}

/// Monta os argumentos de `grok` -- separado de `executar_grok` para ser
/// testável sem depender do binário real instalado.
fn montar_args_grok<'a>(
    worktree: &'a Path,
    tarefa: &'a str,
    model: Option<&'a str>,
) -> Vec<&'a str> {
    let mut args = vec!["--cwd", worktree.to_str().unwrap_or_default()];
    if let Some(m) = model {
        args.push("-m");
        args.push(m);
    }
    args.push("-p");
    args.push(tarefa);
    args.push("--permission-mode");
    args.push("bypassPermissions");
    args
}

/// `grok --cwd <worktree> -p <tarefa> --permission-mode bypassPermissions
/// [-m <model>]` -- `bypassPermissions` é o modo confirmado manualmente
/// (design.md, grok-execucao-verificada) que roda de ponta a ponta sem
/// travar esperando aprovação interativa. Diferente do Codex, o Grok não
/// cria commit próprio -- `aplicar_trailers_se_houver_commit_novo` já
/// trata isso como no-op (nenhuma mudança necessária aqui).
fn executar_grok(
    worktree: &Path,
    tarefa: &str,
    model: Option<&str>,
) -> Result<ResultadoProvider, ErroExecucao> {
    let args = montar_args_grok(worktree, tarefa, model);

    let saida = Command::new("grok")
        .args(&args)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| ErroExecucao::Git(format!("erro executando grok: {e}")))?;

    Ok(saida_para_resultado(saida))
}

/// Resultado bruto do gate determinístico — mesmo formato de
/// `ResultadoProvider` (spec deterministic-gate).
struct ResultadoGate {
    sucesso: bool,
    resumo_saida: String,
}

/// Roda o gate (`sh -c <comando>`) dentro do worktree — só chamado quando o
/// provider já teve sucesso (spec: "Provider já falho não chega a rodar o
/// gate").
fn rodar_gate(worktree: &Path, comando: &str) -> Result<ResultadoGate, ErroExecucao> {
    let saida = Command::new("sh")
        .arg("-c")
        .arg(comando)
        .current_dir(worktree)
        .output()
        .map_err(|e| ErroExecucao::Git(format!("erro executando gate: {e}")))?;

    let resumo_saida = if saida.status.success() {
        String::new()
    } else {
        format!(
            "{}{}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        )
        .lines()
        .take(3)
        .collect::<Vec<_>>()
        .join(" | ")
    };

    Ok(ResultadoGate {
        sucesso: saida.status.success(),
        resumo_saida,
    })
}

/// Combina resultado do provider + resultado do gate (quando houver) no
/// status final do run — função pura, testável sem spawnar processo nenhum
/// (spec deterministic-gate, isolated-run: "Run bem-sucedido registra o
/// resultado").
fn decidir_status_final(
    resultado_provider: &Result<ResultadoProvider, ErroExecucao>,
    resultado_gate: Option<&Result<ResultadoGate, ErroExecucao>>,
) -> (StatusRun, Option<String>) {
    match resultado_provider {
        Ok(rp) if rp.sucesso => match resultado_gate {
            None => (StatusRun::Concluido, None),
            Some(Ok(rg)) if rg.sucesso => (StatusRun::Concluido, None),
            Some(Ok(rg)) => (StatusRun::Falhou, Some(rg.resumo_saida.clone())),
            Some(Err(e)) => (StatusRun::Falhou, Some(e.to_string())),
        },
        Ok(rp) => (StatusRun::Falhou, Some(rp.resumo_stderr.clone())),
        Err(e) => (StatusRun::Falhou, Some(e.to_string())),
    }
}

/// Trailers de proveniência (spec provenance-trail: "Commit gerado carrega
/// trailers de proveniência"). Custo fica "unknown": esta change não calcula
/// custo do run — isso é o pipeline de `client-cost-attribution`, que lê o
/// consumo relatado pelo provider via `brian import`, não `brian run`.
fn trailers_texto(
    run_id: &str,
    contexto: &ContextoAtivo,
    provider_id: &str,
    model: &str,
) -> String {
    format!(
        "Brian-Run: {run_id}\nBrian-Client: {}\nBrian-Context: {}\nBrian-Provider: {provider_id}\nBrian-Model: {model}\nBrian-Cost-USD: unknown",
        contexto.client_id,
        contexto.project.as_deref().unwrap_or("sem-projeto"),
    )
}

/// Se o provider criou um commit novo no worktree, acrescenta os trailers de
/// proveniência via `--amend` (spec: "Trilha é legível sem acesso ao banco").
/// Sem commit novo, não faz nada (spec: "Run sem alteração não força commit
/// vazio").
fn aplicar_trailers_se_houver_commit_novo(
    worktree: &Path,
    commit_antes: &str,
    run_id: &str,
    contexto: &ContextoAtivo,
    provider_id: &str,
    model: &str,
) -> Result<(), ErroExecucao> {
    let commit_depois = commit_atual(worktree)?;
    if commit_depois == commit_antes {
        return Ok(());
    }

    let mensagem_original = git(worktree, &["log", "-1", "--format=%B"])?;
    let nova_mensagem = format!(
        "{}\n\n{}",
        mensagem_original,
        trailers_texto(run_id, contexto, provider_id, model)
    );
    git(worktree, &["commit", "--amend", "-m", &nova_mensagem])?;
    Ok(())
}

/// O que rodar num run — separado dos parâmetros de infraestrutura
/// (`store`/`contexto`/`repo`/`agora`) só para não estourar o limite de
/// argumentos do clippy.
pub struct PedidoRun<'a> {
    pub provider_id: &'a str,
    pub model: Option<&'a str>,
    pub tarefa: &'a str,
    pub gate: Option<&'a str>,
    /// `None` resolve `HEAD` do repo (comportamento padrão). `Some` fixa o
    /// commit base explicitamente — necessário para casos de eval, que
    /// precisam rodar sempre contra o mesmo ponto de partida, não o `HEAD`
    /// corrente do repositório de fixture.
    pub base_commit: Option<&'a str>,
}

/// `None` resolve `HEAD` do repo (comportamento padrão, já existente antes
/// desta capability). `Some` fixa o commit explicitamente — spec isolated-run:
/// "Commit base pode ser fixado explicitamente".
fn resolver_base_commit(
    repo: &Path,
    override_explicito: Option<&str>,
) -> Result<String, ErroExecucao> {
    match override_explicito {
        Some(c) => Ok(c.to_string()),
        None => commit_atual(repo),
    }
}

/// Inicia um run: resolve o commit base, persiste o registro **antes** de
/// criar o worktree ou invocar o provider (D-12, spec isolated-run: "Run
/// persistido antes de qualquer efeito colateral"), então executa.
pub fn iniciar_run(
    store: &dyn Store,
    contexto: Option<&ContextoAtivo>,
    repo: &Path,
    pedido: PedidoRun,
    agora: Instante,
) -> Result<RunRegistrado, ErroExecucao> {
    let PedidoRun {
        provider_id,
        model,
        tarefa,
        gate,
        base_commit,
    } = pedido;
    let contexto = contexto.ok_or(ErroExecucao::SemContextoAtivo)?;

    // Provider não suportado recusa antes de qualquer persistência -- não é
    // "falha ao invocar", é uma tarefa que nunca deveria ter começado.
    if !PROVIDERS_EXECUCAO_VERIFICADA.contains(&provider_id) {
        return Err(ErroExecucao::ProviderNaoSuportado(provider_id.to_string()));
    }

    let base_commit = resolver_base_commit(repo, base_commit)?;
    let run_id = gerar_id();

    // Efeito colateral zero até aqui -- a partir daqui, worktree e processo
    // do provider já existem no mundo real, então o run já precisa estar
    // gravado (D-12).
    let mut run = store
        .criar_run(NovoRun {
            id: run_id.clone(),
            client_id: contexto.client_id.clone(),
            project: contexto.project.clone(),
            base_commit: base_commit.clone(),
            worktree_path: String::new(), // preenchido abaixo, após criar_run
            branch: String::new(),
            provider_id: provider_id.to_string(),
            started_at: agora,
        })
        .map_err(|e| ErroExecucao::Storage(e.to_string()))?;

    let worktree_resultado = criar_worktree(repo, &base_commit, &run_id);
    let Ok((worktree_path, branch)) = worktree_resultado else {
        let motivo = worktree_resultado.err().map(|e| e.to_string());
        registrar_evento(store, &run_id, "worktree.create.failed", motivo, agora);
        store
            .atualizar_status_run(&run_id, StatusRun::Falhou, Some(agora), None)
            .map_err(|e| ErroExecucao::Storage(e.to_string()))?;
        run.status = StatusRun::Falhou;
        run.finished_at = Some(agora);
        return Ok(run);
    };
    registrar_evento(
        store,
        &run_id,
        "worktree.create",
        Some(worktree_path.display().to_string()),
        agora,
    );

    // Gravado no banco (não só no `run` em memória): se o processo `brian
    // run` morrer logo em seguida, `recover`/`worktree list` (que leem do
    // banco) ainda precisam saber onde o worktree está.
    let worktree_path_str = worktree_path.display().to_string();
    store
        .definir_worktree_run(&run_id, &worktree_path_str, &branch)
        .map_err(|e| ErroExecucao::Storage(e.to_string()))?;
    run.worktree_path = worktree_path_str;
    run.branch = branch;

    // PID rastreado é o do processo `brian run` em si, não do processo
    // filho `codex` -- `iniciar_run` bloqueia esperando o provider terminar,
    // então é a morte *deste* processo (ex.: `kill -9` no `brian run`) que
    // deixaria o run preso em `em_execucao` para sempre sem essa checagem.
    // O filho `codex`, se sobreviver ao pai, não é o que importa para
    // "este run nunca vai ser finalizado sozinho" (spec orphan-recovery).
    let pid_do_processo = std::process::id();
    let _ = store.definir_pid_run(&run_id, pid_do_processo);
    run.pid = Some(pid_do_processo);

    registrar_evento(
        store,
        &run_id,
        "provider.execute",
        Some(provider_id.to_string()),
        agora,
    );

    let commit_antes = commit_atual(&worktree_path)?;
    let resultado = executar_provider(provider_id, &worktree_path, tarefa, model);
    let provider_sucesso = matches!(&resultado, Ok(r) if r.sucesso);

    // Trailers seguem só o sucesso do provider, não o do gate (design.md:
    // um commit real do provider carrega proveniência mesmo que o gate
    // reprove depois -- mais honesto para quem for inspecionar a falha).
    if provider_sucesso {
        let modelo_registrado = model.unwrap_or("default");
        if let Err(e) = aplicar_trailers_se_houver_commit_novo(
            &worktree_path,
            &commit_antes,
            &run_id,
            contexto,
            provider_id,
            modelo_registrado,
        ) {
            registrar_evento(
                store,
                &run_id,
                "provenance.trailers.failed",
                Some(e.to_string()),
                agora,
            );
        }
    }

    // Gate só roda com provider já bem-sucedido (spec: "Provider já falho
    // não chega a rodar o gate").
    let resultado_gate = if provider_sucesso {
        gate.map(|comando| rodar_gate(&worktree_path, comando))
    } else {
        None
    };

    if let Some(rg) = &resultado_gate {
        let detalhe_evento = match rg {
            Ok(r) if r.sucesso => None,
            Ok(r) => Some(r.resumo_saida.clone()),
            Err(e) => Some(e.to_string()),
        };
        registrar_evento(store, &run_id, "gate.run", detalhe_evento, agora);
    }

    let (status_final, detalhe) = decidir_status_final(&resultado, resultado_gate.as_ref());

    registrar_evento(store, &run_id, "provider.finished", detalhe, agora);
    store
        .atualizar_status_run(&run_id, status_final, Some(agora), None)
        .map_err(|e| ErroExecucao::Storage(e.to_string()))?;

    run.status = status_final;
    run.finished_at = Some(agora);
    Ok(run)
}

/// Testa se um processo está vivo via `kill -0` (POSIX: não mata, só
/// verifica existência/permissão — task 5.1, sem dependência nova).
pub fn processo_vivo(pid: u32) -> bool {
    let saida = Command::new("kill").args(["-0", &pid.to_string()]).output();
    matches!(saida, Ok(s) if s.status.success())
}

/// Runs em execução cujo processo não está mais vivo (spec orphan-recovery:
/// "Detecção de run órfão").
pub fn runs_orfaos(store: &dyn Store) -> StorageResult<Vec<RunRegistrado>> {
    Ok(store
        .runs_em_execucao()?
        .into_iter()
        .filter(|r| !r.pid.is_some_and(processo_vivo))
        .collect())
}

/// Finaliza um run órfão sem reexecutar a tarefa (spec: "Finalização de
/// órfão nunca duplica custo") — só marca `Abandonado`, worktree preservado.
pub fn recuperar(store: &dyn Store, run_id: &str, agora: Instante) -> Result<(), ErroExecucao> {
    registrar_evento(store, run_id, "run.recovered", None, agora);
    store
        .atualizar_status_run(run_id, StatusRun::Abandonado, Some(agora), None)
        .map_err(|e| ErroExecucao::Storage(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::sqlite::SqliteStore;

    fn store() -> SqliteStore {
        let s = SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s
    }

    #[test]
    fn montar_args_grok_sem_model_usa_bypass_permissions() {
        let worktree = Path::new("/tmp/wt");
        let args = montar_args_grok(worktree, "corrige o bug", None);
        assert_eq!(
            args,
            vec![
                "--cwd",
                "/tmp/wt",
                "-p",
                "corrige o bug",
                "--permission-mode",
                "bypassPermissions",
            ]
        );
    }

    #[test]
    fn montar_args_grok_com_model_inclui_flag_m() {
        let worktree = Path::new("/tmp/wt");
        let args = montar_args_grok(worktree, "corrige o bug", Some("grok-4"));
        assert_eq!(
            args,
            vec![
                "--cwd",
                "/tmp/wt",
                "-m",
                "grok-4",
                "-p",
                "corrige o bug",
                "--permission-mode",
                "bypassPermissions",
            ]
        );
    }

    #[test]
    fn grok_esta_na_lista_de_providers_verificados() {
        assert!(PROVIDERS_EXECUCAO_VERIFICADA.contains(&"grok"));
    }

    fn contexto() -> ContextoAtivo {
        ContextoAtivo {
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            identity_profile_id: "p1".into(),
            connected_at: Instante(0),
        }
    }

    fn repo_git_temporario() -> PathBuf {
        let dir = crate::testutil::dir_temporario_unico("execucao");
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str]| {
            Command::new("git")
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

    #[test]
    fn iniciar_run_sem_contexto_ativo_e_erro() {
        let s = store();
        let repo = repo_git_temporario();
        let erro = iniciar_run(
            &s,
            None,
            &repo,
            PedidoRun {
                provider_id: "codex",
                model: None,
                tarefa: "tarefa",
                gate: None,
                base_commit: None,
            },
            Instante(0),
        )
        .unwrap_err();
        assert!(matches!(erro, ErroExecucao::SemContextoAtivo));
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn iniciar_run_com_provider_nao_suportado_e_erro_e_nao_persiste() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let repo = repo_git_temporario();
        let erro = iniciar_run(
            &s,
            Some(&contexto()),
            &repo,
            PedidoRun {
                provider_id: "claude",
                model: None,
                tarefa: "tarefa",
                gate: None,
                base_commit: None,
            },
            Instante(0),
        )
        .unwrap_err();
        assert!(matches!(erro, ErroExecucao::ProviderNaoSuportado(_)));
        assert!(
            s.runs_em_execucao().unwrap().is_empty(),
            "provider recusado não deveria ter chegado a persistir nada"
        );
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn criar_worktree_isola_do_repositorio_principal() {
        let repo = repo_git_temporario();
        let base = commit_atual(&repo).unwrap();
        let (worktree_path, branch) = criar_worktree(&repo, &base, "teste123").unwrap();

        assert!(worktree_path.exists());
        assert_ne!(worktree_path, repo, "worktree nunca é a árvore principal");
        assert!(branch.starts_with("brian/run_"));

        // Alterar o worktree não deve alterar a árvore principal.
        std::fs::write(worktree_path.join("novo.txt"), "conteudo").unwrap();
        assert!(!repo.join("novo.txt").exists());

        let _ = Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(&repo)
            .output();
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn dois_runs_simultaneos_recebem_worktrees_e_branches_distintos() {
        // Spec isolated-run, "Runs concorrentes não colidem".
        let repo = repo_git_temporario();
        let base = commit_atual(&repo).unwrap();
        let (worktree_a, branch_a) = criar_worktree(&repo, &base, "run-a").unwrap();
        let (worktree_b, branch_b) = criar_worktree(&repo, &base, "run-b").unwrap();

        assert_ne!(worktree_a, worktree_b);
        assert_ne!(branch_a, branch_b);

        std::fs::write(worktree_a.join("so_a.txt"), "a").unwrap();
        assert!(
            !worktree_b.join("so_a.txt").exists(),
            "arquivo do run A não pode aparecer no worktree do run B"
        );

        for wt in [&worktree_a, &worktree_b] {
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force", wt.to_str().unwrap()])
                .current_dir(&repo)
                .output();
        }
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn resolver_base_commit_sem_override_usa_head() {
        let repo = repo_git_temporario();
        let head = commit_atual(&repo).unwrap();
        assert_eq!(resolver_base_commit(&repo, None).unwrap(), head);
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn resolver_base_commit_com_override_usa_o_commit_explicito() {
        let repo = repo_git_temporario();
        let head = commit_atual(&repo).unwrap();

        // Avança o HEAD depois de fixar o commit -- o override precisa
        // continuar valendo o commit antigo, não o HEAD novo.
        std::fs::write(repo.join("novo.txt"), "conteudo").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&repo)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "segundo commit"])
            .current_dir(&repo)
            .output()
            .unwrap();
        let novo_head = commit_atual(&repo).unwrap();
        assert_ne!(head, novo_head);

        assert_eq!(resolver_base_commit(&repo, Some(&head)).unwrap(), head);
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn resolucao_de_commit_base_invalida_falha_sem_persistir_nada() {
        // `iniciar_run` resolve `base_commit` antes de `criar_run` -- se
        // isso falhar (repo inexistente), não há efeito colateral nenhum
        // ainda, então nada deve ficar persistido.
        let s = store();
        s.upsert_client("xpto").unwrap();
        let nao_e_repo = crate::testutil::dir_temporario_unico("nao-repo");
        std::fs::create_dir_all(&nao_e_repo).unwrap();

        let erro = iniciar_run(
            &s,
            Some(&contexto()),
            &nao_e_repo,
            PedidoRun {
                provider_id: "codex",
                model: None,
                tarefa: "tarefa",
                gate: None,
                base_commit: None,
            },
            Instante(0),
        )
        .unwrap_err();
        assert!(matches!(erro, ErroExecucao::Git(_)));
        assert!(s.runs_em_execucao().unwrap().is_empty());

        std::fs::remove_dir_all(&nao_e_repo).ok();
    }

    #[test]
    fn criar_worktree_com_commit_base_invalido_falha_sem_panico() {
        // Cobre o mecanismo por trás de "Falha ao invocar o provider ainda
        // deixa rastro" (spec isolated-run): quando a criação do worktree
        // falha (aqui, commit base inexistente contra um repo válido),
        // `criar_worktree` devolve erro em vez de entrar em pânico --
        // `iniciar_run` já trata esse `Err` marcando o run como `Falhou`
        // (ver ramo `let Ok(...) = worktree_resultado else` no código).
        let repo = repo_git_temporario();
        let erro =
            criar_worktree(&repo, "0000000000000000000000000000000000dead", "run-x").unwrap_err();
        assert!(matches!(erro, ErroExecucao::Git(_)));
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn processo_vivo_detecta_o_proprio_processo() {
        assert!(processo_vivo(std::process::id()));
    }

    #[test]
    fn processo_vivo_falso_para_pid_improvavel() {
        // PID muito alto, improvável de existir -- mesma disciplina de teste
        // usada em `arquivos_tocados` (diretório sem git = comportamento
        // negativo verificável sem simular o caso real inteiro).
        assert!(!processo_vivo(999_999));
    }

    #[test]
    fn recuperar_marca_abandonado_sem_reexecutar() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(NovoRun {
            id: "run1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            base_commit: "abc".into(),
            worktree_path: "/tmp/x".into(),
            branch: "brian/run_run1".into(),
            provider_id: "codex".into(),
            started_at: Instante(0),
        })
        .unwrap();

        recuperar(&s, "run1", Instante(100)).unwrap();

        let run = s.run("run1").unwrap().unwrap();
        assert_eq!(run.status, StatusRun::Abandonado);
        assert_eq!(run.finished_at, Some(Instante(100)));
    }

    #[test]
    fn runs_orfaos_encontra_apenas_processos_mortos() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(NovoRun {
            id: "run-vivo".into(),
            client_id: "xpto".into(),
            project: None,
            base_commit: "abc".into(),
            worktree_path: "/tmp/x".into(),
            branch: "brian/run_run-vivo".into(),
            provider_id: "codex".into(),
            started_at: Instante(0),
        })
        .unwrap();
        s.definir_pid_run("run-vivo", std::process::id()).unwrap();

        s.criar_run(NovoRun {
            id: "run-morto".into(),
            client_id: "xpto".into(),
            project: None,
            base_commit: "abc".into(),
            worktree_path: "/tmp/y".into(),
            branch: "brian/run_run-morto".into(),
            provider_id: "codex".into(),
            started_at: Instante(0),
        })
        .unwrap();
        s.definir_pid_run("run-morto", 999_999).unwrap();

        let orfaos = runs_orfaos(&s).unwrap();
        assert_eq!(orfaos.len(), 1);
        assert_eq!(orfaos[0].id, "run-morto");
    }

    #[test]
    fn commit_novo_ganha_trailers_de_proveniencia_preservando_mensagem() {
        let repo = repo_git_temporario();
        let commit_antes = commit_atual(&repo).unwrap();

        std::fs::write(repo.join("feature.txt"), "novidade").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&repo)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "adiciona feature"])
            .current_dir(&repo)
            .output()
            .unwrap();

        aplicar_trailers_se_houver_commit_novo(
            &repo,
            &commit_antes,
            "run-xyz",
            &contexto(),
            "codex",
            "default",
        )
        .unwrap();

        let mensagem = git(&repo, &["log", "-1", "--format=%B"]).unwrap();
        assert!(mensagem.starts_with("adiciona feature"));
        assert!(mensagem.contains("Brian-Run: run-xyz"));
        assert!(mensagem.contains("Brian-Client: xpto"));
        assert!(mensagem.contains("Brian-Context: checkout-api"));
        assert!(mensagem.contains("Brian-Provider: codex"));
        assert!(mensagem.contains("Brian-Model: default"));
        assert!(mensagem.contains("Brian-Cost-USD: unknown"));

        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn sem_commit_novo_nenhum_commit_e_forjado() {
        let repo = repo_git_temporario();
        let commit_antes = commit_atual(&repo).unwrap();

        aplicar_trailers_se_houver_commit_novo(
            &repo,
            &commit_antes,
            "run-xyz",
            &contexto(),
            "codex",
            "default",
        )
        .unwrap();

        let commit_depois = commit_atual(&repo).unwrap();
        assert_eq!(
            commit_antes, commit_depois,
            "nenhum commit deveria ter sido criado"
        );

        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn rodar_gate_com_comando_que_passa() {
        let repo = repo_git_temporario();
        let r = rodar_gate(&repo, "true").unwrap();
        assert!(r.sucesso);
        assert!(r.resumo_saida.is_empty());
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn rodar_gate_com_comando_que_falha() {
        let repo = repo_git_temporario();
        let r = rodar_gate(&repo, "echo 'deu ruim' >&2; exit 1").unwrap();
        assert!(!r.sucesso);
        assert!(r.resumo_saida.contains("deu ruim"));
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn rodar_gate_roda_no_worktree_e_nao_no_diretorio_atual() {
        let repo = repo_git_temporario();
        // README.md só existe dentro do repo de teste -- se o gate rodasse
        // em outro cwd, `test -f README.md` falharia.
        let r = rodar_gate(&repo, "test -f README.md").unwrap();
        assert!(r.sucesso);
        std::fs::remove_dir_all(&repo).ok();
    }

    fn resultado_provider_ok() -> Result<ResultadoProvider, ErroExecucao> {
        Ok(ResultadoProvider {
            sucesso: true,
            resumo_stderr: String::new(),
        })
    }

    fn resultado_provider_falho() -> Result<ResultadoProvider, ErroExecucao> {
        Ok(ResultadoProvider {
            sucesso: false,
            resumo_stderr: "provider quebrou".into(),
        })
    }

    #[test]
    fn decidir_status_provider_ok_sem_gate_e_concluido() {
        // Spec deterministic-gate: "Ausência de gate preserva o
        // comportamento anterior".
        let (status, detalhe) = decidir_status_final(&resultado_provider_ok(), None);
        assert_eq!(status, StatusRun::Concluido);
        assert_eq!(detalhe, None);
    }

    #[test]
    fn decidir_status_provider_ok_gate_ok_e_concluido() {
        let gate_ok = Ok(ResultadoGate {
            sucesso: true,
            resumo_saida: String::new(),
        });
        let (status, detalhe) = decidir_status_final(&resultado_provider_ok(), Some(&gate_ok));
        assert_eq!(status, StatusRun::Concluido);
        assert_eq!(detalhe, None);
    }

    #[test]
    fn decidir_status_provider_ok_gate_falho_e_falhou() {
        // Spec: "Gate reprovado marca o run como falho mesmo com provider
        // verde".
        let gate_falho = Ok(ResultadoGate {
            sucesso: false,
            resumo_saida: "teste quebrou".into(),
        });
        let (status, detalhe) = decidir_status_final(&resultado_provider_ok(), Some(&gate_falho));
        assert_eq!(status, StatusRun::Falhou);
        assert_eq!(detalhe, Some("teste quebrou".into()));
    }

    #[test]
    fn decidir_status_provider_falho_ignora_gate() {
        // Spec: "Provider já falho não chega a rodar o gate" -- mesmo que
        // `resultado_gate` seja `None` (porque `iniciar_run` nem chamou
        // `rodar_gate`), o motivo registrado é o do provider.
        let (status, detalhe) = decidir_status_final(&resultado_provider_falho(), None);
        assert_eq!(status, StatusRun::Falhou);
        assert_eq!(detalhe, Some("provider quebrou".into()));
    }

    #[test]
    fn iniciar_run_chama_executar_provider_uma_unica_vez() {
        // Spec deterministic-gate: "Gate reprovado nunca reexecuta o
        // provider automaticamente" -- verificação estrutural (task 2.4):
        // só existe um call site de `executar_provider` em todo o arquivo,
        // então nenhum caminho (incluindo falha de gate) pode invocá-lo de
        // novo sem alguém adicionar um segundo call site visível aqui.
        let fonte = include_str!("execucao.rs");
        // Montado em partes para o padrão não bater consigo mesmo aqui.
        let alvo = ["let result", "ado = execu", "tar_provider("].concat();
        let chamadas = fonte.matches(&alvo).count();
        assert_eq!(
            chamadas, 1,
            "executar_provider deve ter exatamente um call site"
        );
    }
}
