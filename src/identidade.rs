//! Identity Manager: isolamento de provider por variável de ambiente e troca
//! de contexto (grupos 4-5).
//!
//! Não faz SQL — recebe o que já foi lido/gravado via `Store`. Mesma
//! separação já usada em `capacidade.rs`: função pura de decisão, glue fina
//! com storage.

use crate::domain::{ContextoAtivo, Instante, PerfilIdentidade};
use crate::storage::{NovoPerfil, Result as StorageResult, Store};

/// Variável de ambiente que isola a configuração de um provider, quando
/// existe uma documentada e confirmada (task 4.2). Providers sem entrada
/// aqui não têm mecanismo de isolamento conhecido — `None`, nunca um valor
/// inventado.
///
/// Confirmado nesta sessão contra `--help`/documentação oficial de cada CLI:
/// `codex --help` ("Layer $CODEX_HOME/..."), docs oficiais do Claude Code
/// (`CLAUDE_CONFIG_DIR`, code.claude.com/docs/en/env-vars). `agy` (Gemini) não
/// tem variável dedicada — deriva de `$HOME` — por isso fica de fora daqui
/// até uma forma mais segura de isolar existir; ver design.md, escopo
/// desta change.
pub fn env_var_do_provider(provider_id: &str) -> Option<&'static str> {
    match provider_id {
        "codex" => Some("CODEX_HOME"),
        "claude" => Some("CLAUDE_CONFIG_DIR"),
        _ => None,
    }
}

/// Providers com isolamento **verificado** — duas identidades simultâneas
/// testadas e confirmadas funcionando (task 4.1). Começa vazio
/// deliberadamente: nenhum provider foi testado dessa forma ainda nesta
/// change. Mesmo padrão de honestidade de `adapters::cobertura_v0_0` — a
/// lista não reflete "tem variável de ambiente conhecida" (isso é
/// `env_var_do_provider`), reflete "alguém rodou duas identidades ao mesmo
/// tempo e confirmou que não vazou uma pra outra".
pub const PROVIDERS_ISOLAMENTO_VERIFICADO: &[&str] = &[];

/// Resultado de montar as variáveis de ambiente para um provider vinculado a
/// um perfil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariavelDeAmbiente {
    /// Isolamento verificado — `Command` do processo filho deve receber esta
    /// variável.
    Aplicar { var: &'static str, valor: String },
    /// Provider sem isolamento verificado — identidade paralela recusada
    /// para ele especificamente (spec provider-isolation).
    Recusado {
        provider_id: String,
        motivo: &'static str,
    },
}

/// Monta as variáveis de ambiente para todos os providers vinculados a um
/// perfil, uma decisão por provider — falha em um não impede os demais
/// (spec: "demais providers do mesmo contexto continuam funcionando").
///
/// `verificados` é injetado (não lê `PROVIDERS_ISOLAMENTO_VERIFICADO`
/// diretamente) para que o caminho "Aplicar" seja testável sem depender de
/// uma constante global vazia — produção sempre chama com
/// `PROVIDERS_ISOLAMENTO_VERIFICADO`.
pub fn montar_variaveis_de_ambiente(
    perfil: &PerfilIdentidade,
    verificados: &[&str],
) -> Vec<VariavelDeAmbiente> {
    perfil
        .bindings
        .iter()
        .map(|binding| {
            if !verificados.contains(&binding.provider_id.as_str()) {
                return VariavelDeAmbiente::Recusado {
                    provider_id: binding.provider_id.clone(),
                    motivo: "isolamento não verificado para este provider",
                };
            }
            match env_var_do_provider(&binding.provider_id) {
                Some(var) => VariavelDeAmbiente::Aplicar {
                    var,
                    valor: binding.config_home.clone(),
                },
                None => VariavelDeAmbiente::Recusado {
                    provider_id: binding.provider_id.clone(),
                    motivo: "nenhuma variável de ambiente de isolamento conhecida",
                },
            }
        })
        .collect()
}

#[derive(Debug)]
pub enum ErroConexao {
    ClienteInexistente,
    ProjetoAmbiguo { projetos: Vec<Option<String>> },
    PerfilInexistente,
}

impl std::fmt::Display for ErroConexao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClienteInexistente => write!(f, "cliente não existe"),
            Self::ProjetoAmbiguo { projetos } => {
                let nomes: Vec<&str> = projetos
                    .iter()
                    .map(|p| p.as_deref().unwrap_or("(sem projeto)"))
                    .collect();
                write!(
                    f,
                    "cliente tem múltiplos projetos, especifique um: {}",
                    nomes.join(", ")
                )
            }
            Self::PerfilInexistente => write!(f, "projeto não configurado para este cliente"),
        }
    }
}

/// Conecta a um cliente/projeto: resolve o perfil (erro se ambíguo ou
/// inexistente) e grava o contexto ativo. Espelha `capacidade::montar_janela`
/// — decisão pura de qual perfil usar, storage só grava (spec
/// context-switching: "Conectar a cliente com múltiplos projetos, sem
/// especificar qual").
pub fn conectar(
    store: &dyn Store,
    client_id: &str,
    project: Option<&str>,
    agora: Instante,
) -> Result<ContextoAtivo, ErroConexao> {
    if !store
        .client_exists(client_id)
        .map_err(|_| ErroConexao::ClienteInexistente)?
    {
        return Err(ErroConexao::ClienteInexistente);
    }

    let perfis = store
        .perfis_do_cliente(client_id)
        .map_err(|_| ErroConexao::PerfilInexistente)?;

    let perfil = match project {
        Some(p) => perfis
            .iter()
            .find(|perfil| perfil.project.as_deref() == Some(p))
            .ok_or(ErroConexao::PerfilInexistente)?,
        None => match perfis.len() {
            0 => return Err(ErroConexao::PerfilInexistente),
            1 => &perfis[0],
            _ => {
                return Err(ErroConexao::ProjetoAmbiguo {
                    projetos: perfis.iter().map(|p| p.project.clone()).collect(),
                });
            }
        },
    };

    let contexto = ContextoAtivo {
        client_id: client_id.to_string(),
        project: perfil.project.clone(),
        identity_profile_id: perfil.id.clone(),
        connected_at: agora,
    };

    store
        .conectar(contexto.clone())
        .map_err(|_| ErroConexao::PerfilInexistente)?;

    Ok(contexto)
}

/// Encerra o contexto ativo. No-op sem contexto ativo (spec:
/// "Desconectar sem contexto ativo").
pub fn desconectar(store: &dyn Store) -> StorageResult<()> {
    store.desconectar()
}

/// Constrói um novo perfil a partir de dados já validados por quem chama
/// (CLI) — função fina, só delega ao storage (task 5.1 cita `conectar`, este
/// é o complemento de criação usado por `context init`).
pub fn criar_perfil(store: &dyn Store, novo: NovoPerfil) -> StorageResult<PerfilIdentidade> {
    store.criar_perfil(novo)
}

/// Nomes das variáveis de ambiente que carregam a identidade Git — sempre
/// via env, nunca `git config --local` (design.md: "identidade Git via
/// variável de ambiente, não git config --local").
const VARS_GIT: &[&str] = &[
    "GIT_AUTHOR_NAME",
    "GIT_AUTHOR_EMAIL",
    "GIT_COMMITTER_NAME",
    "GIT_COMMITTER_EMAIL",
];

/// Linhas `export VAR=valor` para `eval "$(brian connect ...)"` — nunca abre
/// subshell nem escreve em `.bashrc`/`.zshrc` (design.md, decisão
/// "`brian connect` imprime exports"). Função pura: só monta texto a partir
/// do que já foi resolvido, sem tocar em processo nem em I/O.
pub fn linhas_export(perfil: &PerfilIdentidade) -> Vec<String> {
    let mut linhas = Vec::new();

    if let Some(nome) = &perfil.git_author_name {
        linhas.push(format!("export GIT_AUTHOR_NAME={}", shell_quote(nome)));
        linhas.push(format!("export GIT_COMMITTER_NAME={}", shell_quote(nome)));
    }
    if let Some(email) = &perfil.git_author_email {
        linhas.push(format!("export GIT_AUTHOR_EMAIL={}", shell_quote(email)));
        linhas.push(format!("export GIT_COMMITTER_EMAIL={}", shell_quote(email)));
    }

    for aplicavel in montar_variaveis_de_ambiente(perfil, PROVIDERS_ISOLAMENTO_VERIFICADO) {
        if let VariavelDeAmbiente::Aplicar { var, valor } = aplicavel {
            linhas.push(format!("export {var}={}", shell_quote(&valor)));
        }
    }

    linhas
}

/// Linhas `unset VAR` — reverte exatamente o que `linhas_export` teria
/// exportado para o mesmo perfil (spec: "desconectar... providers voltam a
/// usar a configuração padrão do desktop").
pub fn linhas_unset(perfil: &PerfilIdentidade) -> Vec<String> {
    let mut vars: Vec<&str> = VARS_GIT.to_vec();
    for aplicavel in montar_variaveis_de_ambiente(perfil, PROVIDERS_ISOLAMENTO_VERIFICADO) {
        if let VariavelDeAmbiente::Aplicar { var, .. } = aplicavel {
            vars.push(var);
        }
    }
    vars.into_iter().map(|v| format!("unset {v}")).collect()
}

/// Aspas simples com escape mínimo — shell POSIX. Sem dependência nova: o
/// conjunto de valores aqui é nome/email/caminho, não entrada arbitrária de
/// usuário externo.
fn shell_quote(valor: &str) -> String {
    format!("'{}'", valor.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ProviderBinding;
    use crate::storage::sqlite::SqliteStore;

    fn store() -> SqliteStore {
        let s = SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s
    }

    fn perfil(provider_id: &str, config_home: &str) -> PerfilIdentidade {
        PerfilIdentidade {
            id: "p1".into(),
            client_id: "xpto".into(),
            project: None,
            git_author_name: None,
            git_author_email: None,
            github_org: None,
            bindings: vec![ProviderBinding {
                provider_id: provider_id.into(),
                config_home: config_home.into(),
            }],
        }
    }

    #[test]
    fn env_var_conhecida_para_codex_e_claude() {
        assert_eq!(env_var_do_provider("codex"), Some("CODEX_HOME"));
        assert_eq!(env_var_do_provider("claude"), Some("CLAUDE_CONFIG_DIR"));
    }

    #[test]
    fn env_var_desconhecida_para_provider_nao_declarado() {
        assert_eq!(env_var_do_provider("grok"), None);
    }

    #[test]
    fn provider_sem_isolamento_verificado_e_recusado() {
        // Spec provider-isolation, "Provider sem isolamento verificado":
        // com a lista de verificados vazia, todo provider recusa, mesmo
        // tendo variável de ambiente conhecida.
        let resultado = montar_variaveis_de_ambiente(&perfil("codex", "/tmp/xpto/codex"), &[]);
        assert_eq!(resultado.len(), 1);
        assert!(matches!(resultado[0], VariavelDeAmbiente::Recusado { .. }));
    }

    #[test]
    fn provider_com_isolamento_verificado_aplica_variavel_de_ambiente() {
        // Spec provider-isolation, "Provider com isolamento verificado": com
        // o provider na lista de verificados, a variável e o valor do
        // caminho isolado são aplicados de verdade. `PROVIDERS_ISOLAMENTO_VERIFICADO`
        // fica vazio em produção nesta change (nada testado com hardware
        // real ainda) -- injetar a lista aqui é o que torna esse caminho
        // testável sem esperar essa verificação acontecer.
        let resultado =
            montar_variaveis_de_ambiente(&perfil("codex", "/tmp/xpto/codex"), &["codex"]);
        assert_eq!(
            resultado[0],
            VariavelDeAmbiente::Aplicar {
                var: "CODEX_HOME",
                valor: "/tmp/xpto/codex".to_string(),
            }
        );
    }

    #[test]
    fn provider_sem_env_var_conhecida_e_recusado_com_motivo_proprio() {
        // Verificado mas sem variável de ambiente conhecida (grok não está
        // em env_var_do_provider) -- ainda recusa, motivo diferente.
        let resultado = montar_variaveis_de_ambiente(&perfil("grok", "/tmp/xpto/grok"), &["grok"]);
        match &resultado[0] {
            VariavelDeAmbiente::Recusado { provider_id, .. } => assert_eq!(provider_id, "grok"),
            _ => panic!("esperado Recusado"),
        }
    }

    #[test]
    fn falha_de_um_provider_nao_impede_os_demais_na_montagem() {
        let perfil = PerfilIdentidade {
            id: "p1".into(),
            client_id: "xpto".into(),
            project: None,
            git_author_name: None,
            git_author_email: None,
            github_org: None,
            bindings: vec![
                ProviderBinding {
                    provider_id: "codex".into(),
                    config_home: "/tmp/a".into(),
                },
                ProviderBinding {
                    provider_id: "grok".into(),
                    config_home: "/tmp/b".into(),
                },
            ],
        };
        let resultado = montar_variaveis_de_ambiente(&perfil, &["codex", "grok"]);
        assert_eq!(
            resultado.len(),
            2,
            "os dois providers aparecem no resultado"
        );
    }

    #[test]
    fn conectar_cliente_com_perfil_unico() {
        let s = store();
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

        let ctx = conectar(&s, "xpto", None, Instante(1000)).unwrap();
        assert_eq!(ctx.project.as_deref(), Some("checkout-api"));
    }

    #[test]
    fn conectar_cliente_com_multiplos_projetos_sem_especificar_e_ambiguo() {
        let s = store();
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
        s.criar_perfil(crate::storage::NovoPerfil {
            id: "p2".into(),
            client_id: "xpto".into(),
            project: Some("billing-api".into()),
            git_author_name: None,
            git_author_email: None,
            github_org: None,
            bindings: vec![],
            created_at: Instante(0),
        })
        .unwrap();

        let erro = conectar(&s, "xpto", None, Instante(1000)).unwrap_err();
        assert!(matches!(erro, ErroConexao::ProjetoAmbiguo { .. }));
        assert_eq!(
            s.contexto_ativo().unwrap(),
            None,
            "nenhum contexto parcial fica ativo"
        );
    }

    #[test]
    fn conectar_cliente_inexistente_e_erro_explicito() {
        let s = store();
        let erro = conectar(&s, "fantasma", None, Instante(1000)).unwrap_err();
        assert!(matches!(erro, ErroConexao::ClienteInexistente));
    }

    #[test]
    fn conectar_falha_preserva_contexto_anterior() {
        // Spec context-switching, "Conectar a cliente ou projeto
        // inexistente": "o contexto anterior (se houver) permanece ativo".
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();
        s.criar_perfil(crate::storage::NovoPerfil {
            id: "p-xpto".into(),
            client_id: "xpto".into(),
            project: None,
            git_author_name: None,
            git_author_email: None,
            github_org: None,
            bindings: vec![],
            created_at: Instante(0),
        })
        .unwrap();
        conectar(&s, "xpto", None, Instante(1)).unwrap();

        // Tentativa de conectar a cliente inexistente falha.
        assert!(conectar(&s, "fantasma", None, Instante(2)).is_err());
        // Tentativa de conectar a projeto inexistente do cliente acme (que
        // nem tem perfil algum) também falha.
        assert!(conectar(&s, "acme", Some("nao-existe"), Instante(3)).is_err());

        let ativo = s.contexto_ativo().unwrap().unwrap();
        assert_eq!(
            ativo.client_id, "xpto",
            "conexão que falhou não deve ter tocado o contexto ativo anterior"
        );
    }

    #[test]
    fn conectar_projeto_inexistente_e_erro_explicito() {
        let s = store();
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

        let erro = conectar(&s, "xpto", Some("nao-existe"), Instante(1000)).unwrap_err();
        assert!(matches!(erro, ErroConexao::PerfilInexistente));
    }

    #[test]
    fn desconectar_sem_contexto_ativo_e_no_op() {
        let s = store();
        assert!(desconectar(&s).is_ok());
    }

    #[test]
    fn conectar_troca_sequencial_nao_retem_identidade_anterior() {
        // Spec context-switching, "Isolamento entre contextos por
        // construção": trocar de A pra B, o contexto lido depois é o de B.
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();
        s.criar_perfil(crate::storage::NovoPerfil {
            id: "p-xpto".into(),
            client_id: "xpto".into(),
            project: None,
            git_author_name: None,
            git_author_email: None,
            github_org: None,
            bindings: vec![],
            created_at: Instante(0),
        })
        .unwrap();
        s.criar_perfil(crate::storage::NovoPerfil {
            id: "p-acme".into(),
            client_id: "acme".into(),
            project: None,
            git_author_name: None,
            git_author_email: None,
            github_org: None,
            bindings: vec![],
            created_at: Instante(0),
        })
        .unwrap();

        conectar(&s, "xpto", None, Instante(1)).unwrap();
        conectar(&s, "acme", None, Instante(2)).unwrap();

        let ativo = s.contexto_ativo().unwrap().unwrap();
        assert_eq!(ativo.client_id, "acme");
    }

    fn perfil_completo() -> PerfilIdentidade {
        PerfilIdentidade {
            id: "p1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            git_author_name: Some("Joao Costa".into()),
            git_author_email: Some("joao@xpto.com.br".into()),
            github_org: Some("xpto-org".into()),
            bindings: vec![ProviderBinding {
                provider_id: "codex".into(),
                config_home: "/tmp/xpto/codex".into(),
            }],
        }
    }

    #[test]
    fn linhas_export_inclui_identidade_git() {
        let linhas = linhas_export(&perfil_completo());
        assert!(linhas.contains(&"export GIT_AUTHOR_NAME='Joao Costa'".to_string()));
        assert!(linhas.contains(&"export GIT_AUTHOR_EMAIL='joao@xpto.com.br'".to_string()));
        assert!(linhas.contains(&"export GIT_COMMITTER_NAME='Joao Costa'".to_string()));
        assert!(linhas.contains(&"export GIT_COMMITTER_EMAIL='joao@xpto.com.br'".to_string()));
    }

    #[test]
    fn linhas_export_nao_inclui_provider_sem_isolamento_verificado() {
        // PROVIDERS_ISOLAMENTO_VERIFICADO está vazio nesta change -- nenhuma
        // export de provider deve aparecer, só identidade Git.
        let linhas = linhas_export(&perfil_completo());
        assert!(!linhas.iter().any(|l| l.contains("CODEX_HOME")));
    }

    #[test]
    fn linhas_export_escapa_aspas_simples() {
        let mut perfil = perfil_completo();
        perfil.git_author_name = Some("O'Brien".into());
        let linhas = linhas_export(&perfil);
        assert!(linhas.iter().any(|l| l.contains(r"O'\''Brien")));
    }

    #[test]
    fn linhas_unset_cobre_as_mesmas_variaveis_git() {
        let unset = linhas_unset(&perfil_completo());
        assert!(unset.contains(&"unset GIT_AUTHOR_NAME".to_string()));
        assert!(unset.contains(&"unset GIT_AUTHOR_EMAIL".to_string()));
        assert!(unset.contains(&"unset GIT_COMMITTER_NAME".to_string()));
        assert!(unset.contains(&"unset GIT_COMMITTER_EMAIL".to_string()));
    }

    #[test]
    fn linhas_export_sem_identidade_git_e_so_providers() {
        let perfil = PerfilIdentidade {
            id: "p1".into(),
            client_id: "xpto".into(),
            project: None,
            git_author_name: None,
            git_author_email: None,
            github_org: None,
            bindings: vec![],
        };
        assert!(linhas_export(&perfil).is_empty());
    }
}
