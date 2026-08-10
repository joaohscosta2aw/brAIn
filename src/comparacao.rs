//! Comparação pareada (blueprint §38.4): mesma tarefa em múltiplos
//! providers, resultado lado a lado, escolha humana explícita — o
//! mecanismo útil enquanto o histórico real (`routing/historical-scoring`)
//! ainda não tem `n` suficiente. Cada candidato é um `execucao::iniciar_run`
//! normal, sem alteração no motor de execução.

use crate::domain::{CandidatoComparacao, ComparacaoRegistrada, ContextoAtivo, Instante};
use crate::execucao::{self, PedidoRun};
use crate::storage::{NovaComparacao, NovoCandidatoComparacao, Store};

fn gerar_id(prefixo: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{prefixo}-{nanos}")
}

/// Providers pedidos para comparação que não têm execução verificada —
/// vazio quando todos são válidos.
fn providers_invalidos(providers: &[&str]) -> Vec<String> {
    providers
        .iter()
        .filter(|p| !execucao::PROVIDERS_EXECUCAO_VERIFICADA.contains(p))
        .map(|p| p.to_string())
        .collect()
}

/// Roda a mesma tarefa em cada `providers`, sequencialmente, cada um em
/// worktree isolado (spec: "Dois providers geram dois runs isolados"). Toda
/// a lista é validada ANTES de rodar qualquer candidato — spec: "Candidato
/// inválido falha a comparação inteira, sem pular silenciosamente".
#[allow(clippy::too_many_arguments)]
pub fn rodar_comparacao(
    store: &dyn Store,
    contexto: Option<&ContextoAtivo>,
    repo: &std::path::Path,
    providers: &[&str],
    tarefa: &str,
    gate: Option<&str>,
    agora: Instante,
) -> Result<(ComparacaoRegistrada, Vec<CandidatoComparacao>), String> {
    let contexto = contexto.ok_or("nenhum contexto ativo")?;

    let invalidos = providers_invalidos(providers);
    if !invalidos.is_empty() {
        return Err(format!(
            "provider(s) sem execução verificada, comparação recusada: {}",
            invalidos.join(", ")
        ));
    }

    let comparacao_id = gerar_id("cmp");
    let comparacao = store
        .criar_comparacao(NovaComparacao {
            id: comparacao_id.clone(),
            client_id: contexto.client_id.clone(),
            project: contexto.project.clone(),
            tarefa: tarefa.to_string(),
            started_at: agora,
        })
        .map_err(|e| e.to_string())?;

    let mut candidatos = Vec::with_capacity(providers.len());
    for &provider_id in providers {
        let run = execucao::iniciar_run(
            store,
            Some(contexto),
            repo,
            PedidoRun {
                provider_id,
                model: None,
                tarefa,
                gate,
                base_commit: None,
            },
            agora,
        )
        .map_err(|e| e.to_string())?;

        let candidato = store
            .registrar_candidato_comparacao(NovoCandidatoComparacao {
                id: gerar_id("cand"),
                comparacao_id: comparacao_id.clone(),
                provider_id: provider_id.to_string(),
                run_id: Some(run.id.clone()),
            })
            .map_err(|e| e.to_string())?;
        candidatos.push(candidato);
    }

    Ok((comparacao, candidatos))
}

/// Registra a escolha do operador — nunca decide sozinho, nunca reexecuta
/// nada (spec: "Escolha do vencedor é sempre uma ação explícita separada").
pub fn escolher_vencedor(
    store: &dyn Store,
    comparacao_id: &str,
    provider_id: &str,
    agora: Instante,
) -> Result<(), String> {
    store
        .comparacao(comparacao_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("comparacao '{comparacao_id}' não encontrada"))?;
    store
        .definir_vencedor_comparacao(comparacao_id, provider_id, agora)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let dir = crate::testutil::dir_temporario_unico(&format!("comparacao-{sufixo}"));
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
        dir
    }

    #[test]
    fn provider_invalido_falha_antes_de_rodar_qualquer_candidato() {
        let s = store_com_contexto_ativo();
        let repo = repo_git_temporario("invalido");
        let contexto = s.contexto_ativo().unwrap();

        let erro = rodar_comparacao(
            &s,
            contexto.as_ref(),
            &repo,
            &["codex", "claude"],
            "tarefa",
            None,
            Instante(0),
        )
        .unwrap_err();
        assert!(erro.contains("claude"));
        assert!(
            s.comparacao("qualquer").unwrap().is_none(),
            "nada deveria ter sido persistido"
        );
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn escolher_vencedor_de_comparacao_inexistente_falha_claro() {
        let s = store_com_contexto_ativo();
        let erro = escolher_vencedor(&s, "fantasma", "codex", Instante(10)).unwrap_err();
        assert!(erro.contains("fantasma"));
    }

    #[test]
    fn escolher_vencedor_valido_persiste_corretamente() {
        let s = store_com_contexto_ativo();
        s.criar_comparacao(crate::storage::NovaComparacao {
            id: "cmp1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            tarefa: "tarefa".into(),
            started_at: Instante(0),
        })
        .unwrap();

        escolher_vencedor(&s, "cmp1", "codex", Instante(10)).unwrap();

        let cmp = s.comparacao("cmp1").unwrap().unwrap();
        assert_eq!(cmp.vencedor_provider_id.as_deref(), Some("codex"));
    }

    #[test]
    fn comparacao_recem_criada_nao_tem_vencedor() {
        let s = store_com_contexto_ativo();
        s.criar_comparacao(crate::storage::NovaComparacao {
            id: "cmp1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            tarefa: "tarefa".into(),
            started_at: Instante(0),
        })
        .unwrap();

        let cmp = s.comparacao("cmp1").unwrap().unwrap();
        assert_eq!(cmp.vencedor_provider_id, None);
    }
}
