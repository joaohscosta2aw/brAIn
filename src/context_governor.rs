//! Experimento H-1 (context-governor-experiment, blueprint §18): decide se
//! pré-montar um pacote de contexto mínimo (`ContextPackage::Curated`)
//! reduz o custo de um change bem-sucedido -- sem grafo de código real
//! (blueprint §21-25 não existem no Brian ainda), sem medição de custo em
//! USD (duração é proxy declarado). H-1 é uma hipótese, não um pilar (D-5):
//! nada no caminho padrão de `brian run` depende do resultado. Cada
//! execução de braço é um `execucao::iniciar_run` normal, sem alteração no
//! motor de execução -- ver design.md.

use crate::domain::{ContextoAtivo, Instante, NotaDeMemoria, RunRegistrado, StatusRun};
use crate::execucao::{self, ErroExecucao, PedidoRun};
use crate::storage::{NovaExecucaoExperimento, Store};
use std::path::{Path, PathBuf};

/// Contexto sintético do experimento -- nunca o Context ativo do operador
/// (mesma disciplina de `eval::CLIENT_ID_EVAL`).
pub const CLIENT_ID_H1: &str = "h1-experiment";

/// Orçamento de caracteres do pacote curado -- mesma referência de
/// `continuidade::ORCAMENTO_CARACTERES` (~8000), proxy simples de
/// "orçamento de tokens" já que Brian não tem tokenizer real.
const ORCAMENTO_CARACTERES: usize = 8000;

/// Palavras curtas demais ou comuns demais para renderem busca útil --
/// lista mínima em português e inglês, não um dicionário de stopwords
/// completo (YAGNI: só o suficiente para não poluir a busca com "de", "the",
/// "para", etc.).
const STOPWORDS: &[&str] = &[
    "para", "como", "esta", "esse", "essa", "isso", "quando", "onde", "sobre", "entre", "depois",
    "antes", "sempre", "nunca", "cada", "todo", "toda", "todos", "todas", "that", "this", "with",
    "from", "when", "where", "which", "should", "would", "could", "into", "then", "than", "also",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Braco {
    A,
    B,
    C,
}

impl Braco {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::C => "c",
        }
    }

    pub fn de_str(s: &str) -> Option<Self> {
        match s {
            "a" | "A" => Some(Self::A),
            "b" | "B" => Some(Self::B),
            "c" | "C" => Some(Self::C),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ErroExperimento {
    Storage(String),
    Execucao(ErroExecucao),
}

impl std::fmt::Display for ErroExperimento {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(m) => write!(f, "{m}"),
            Self::Execucao(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ErroExperimento {}

/// Extrai palavras-chave de uma tarefa para busca simbólica -- palavras com
/// 4+ letras, sem stopwords, deduplicadas. Função pura, sem I/O.
pub fn extrair_palavras_chave(tarefa: &str) -> Vec<String> {
    let mut vistas = std::collections::BTreeSet::new();
    tarefa
        .split(|c: char| !c.is_alphanumeric())
        .map(|p| p.to_lowercase())
        .filter(|p| p.chars().count() >= 4)
        .filter(|p| !STOPWORDS.contains(&p.as_str()))
        .filter(|p| vistas.insert(p.clone()))
        .collect()
}

/// Roda `grep -rl` por cada palavra-chave em `repo`, deduplica os arquivos
/// encontrados. Aproximação grosseira e deliberada de busca relevante --
/// não é um grafo de código (design.md, risco declarado).
pub fn buscar_arquivos_relevantes(repo: &Path, palavras: &[String]) -> Vec<String> {
    let mut arquivos = std::collections::BTreeSet::new();
    for palavra in palavras {
        let saida = std::process::Command::new("grep")
            .args(["-rl", "--exclude-dir=.git", "-e", palavra])
            .arg(repo)
            .output();
        let Ok(saida) = saida else { continue };
        for linha in String::from_utf8_lossy(&saida.stdout).lines() {
            arquivos.insert(linha.to_string());
        }
    }
    arquivos.into_iter().collect()
}

/// `git log -3 -p` no repo do case -- aproximação de "o que mudou
/// recentemente", não o diff da própria tarefa (que ainda não aconteceu).
/// `None` quando o repositório não tem histórico (comando falha ou saída
/// vazia).
pub fn diff_recente(repo: &Path) -> Option<String> {
    let saida = std::process::Command::new("git")
        .args(["-C", &repo.to_string_lossy(), "log", "-3", "-p"])
        .output()
        .ok()?;
    if !saida.status.success() {
        return None;
    }
    let texto = String::from_utf8_lossy(&saida.stdout).into_owned();
    (!texto.trim().is_empty()).then_some(texto)
}

/// Corta uma string no orçamento de caracteres, respeitando fronteira de
/// char (nunca quebra um caractere multi-byte no meio).
fn truncar(texto: &str, orcamento: usize) -> String {
    match texto.char_indices().nth(orcamento) {
        Some((idx, _)) => texto[..idx].to_string(),
        None => texto.to_string(),
    }
}

/// Monta o pacote curado -- só de fontes que existem de fato no Brian:
/// busca por palavra-chave, diff recente, notas de memória já existentes
/// (spec: "Pacote curado é montado sem grafo de código real").
pub fn montar_pacote_curado(
    repo: &Path,
    tarefa: &str,
    notas: &[NotaDeMemoria],
    orcamento_caracteres: usize,
) -> String {
    let palavras = extrair_palavras_chave(tarefa);
    let arquivos = buscar_arquivos_relevantes(repo, &palavras);

    let mut pacote = String::new();
    if !arquivos.is_empty() {
        pacote.push_str("Arquivos possivelmente relevantes (busca por palavra-chave):\n");
        for arquivo in &arquivos {
            pacote.push_str("- ");
            pacote.push_str(arquivo);
            pacote.push('\n');
        }
    }
    if let Some(diff) = diff_recente(repo) {
        pacote.push_str("\nDiff recente do repositório:\n");
        pacote.push_str(&diff);
    }
    if !notas.is_empty() {
        pacote.push_str("\nNotas de memória do contexto:\n");
        for nota in notas {
            pacote.push_str("- ");
            pacote.push_str(&nota.texto);
            pacote.push('\n');
        }
    }

    truncar(&pacote, orcamento_caracteres)
}

/// Formata a mesma tarefa-base de acordo com o braço -- spec: "Cada braço
/// formata a mesma tarefa de forma diferente".
pub fn formatar_tarefa_por_braco(tarefa: &str, pacote: Option<&str>, braco: Braco) -> String {
    match (braco, pacote) {
        (Braco::A, _) | (_, None) => tarefa.to_string(),
        (Braco::B, Some(pacote)) => format!(
            "{tarefa}\n\nContexto (use SOMENTE isto, não explore o resto do repositório):\n{pacote}"
        ),
        (Braco::C, Some(pacote)) => {
            format!("{tarefa}\n\nContexto (ponto de partida — explore mais se precisar):\n{pacote}")
        }
    }
}

fn contexto_h1(case_id: &str) -> ContextoAtivo {
    ContextoAtivo {
        client_id: CLIENT_ID_H1.to_string(),
        project: Some(case_id.to_string()),
        identity_profile_id: String::new(),
        connected_at: Instante(0),
    }
}

fn gerar_id(prefixo: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{prefixo}-{nanos}")
}

/// Roda uma execução do experimento para um case/braço: monta o pacote
/// curado (só para B/C -- spec: "Braço A não recebe pacote curado"),
/// formata a tarefa, delega a `execucao::iniciar_run` inalterado, registra
/// a ligação case/braço/run.
#[allow(clippy::too_many_arguments)]
pub fn rodar_execucao_experimento(
    store: &dyn Store,
    repo: &Path,
    case_id: &str,
    braco: Braco,
    tarefa_base: &str,
    provider_id: &str,
    gate: Option<&str>,
    base_commit: Option<&str>,
    agora: Instante,
) -> Result<RunRegistrado, ErroExperimento> {
    store
        .upsert_client(CLIENT_ID_H1)
        .map_err(|e| ErroExperimento::Storage(e.to_string()))?;

    let contexto = contexto_h1(case_id);

    let pacote = match braco {
        Braco::A => None,
        Braco::B | Braco::C => {
            let notas = store
                .notas_do_contexto(CLIENT_ID_H1, Some(case_id))
                .map_err(|e| ErroExperimento::Storage(e.to_string()))?;
            Some(montar_pacote_curado(
                repo,
                tarefa_base,
                &notas,
                ORCAMENTO_CARACTERES,
            ))
        }
    };
    let tarefa_formatada = formatar_tarefa_por_braco(tarefa_base, pacote.as_deref(), braco);

    let run = execucao::iniciar_run(
        store,
        Some(&contexto),
        repo,
        PedidoRun {
            provider_id,
            model: None,
            tarefa: &tarefa_formatada,
            gate,
            base_commit,
        },
        agora,
    )
    .map_err(ErroExperimento::Execucao)?;

    store
        .registrar_execucao_experimento(NovaExecucaoExperimento {
            id: gerar_id("exp"),
            case_id: case_id.to_string(),
            braco: braco.as_str().to_string(),
            run_id: run.id.clone(),
            started_at: agora,
        })
        .map_err(|e| ErroExperimento::Storage(e.to_string()))?;

    Ok(run)
}

/// Resultado agregado de um braço -- mesma disciplina de
/// `router::Score`/`routing/historical-scoring`: `n` sempre visível ao lado
/// da taxa e da duração.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoBraco {
    pub braco: String,
    pub taxa_sucesso: f64,
    pub n: u32,
    pub duracao_media_segundos: Option<f64>,
}

/// Calcula taxa de sucesso e duração média por braço -- mesma fórmula de
/// `router::calcular_scores`, agrupando por braço em vez de provider.
pub fn calcular_resultado_por_braco(execucoes: &[(String, RunRegistrado)]) -> Vec<ResultadoBraco> {
    let mut bracos: Vec<String> = execucoes.iter().map(|(b, _)| b.clone()).collect();
    bracos.sort();
    bracos.dedup();

    bracos
        .into_iter()
        .map(|braco| {
            let do_braco: Vec<&RunRegistrado> = execucoes
                .iter()
                .filter(|(b, _)| *b == braco)
                .map(|(_, r)| r)
                .collect();
            let n = do_braco.len() as u32;
            let concluidos = do_braco
                .iter()
                .filter(|r| r.status == StatusRun::Concluido)
                .count();
            let taxa_sucesso = if n == 0 {
                0.0
            } else {
                concluidos as f64 / n as f64
            };

            let duracoes: Vec<i64> = do_braco
                .iter()
                .filter_map(|r| Some(r.finished_at?.0 - r.started_at.0))
                .collect();
            let duracao_media_segundos = (!duracoes.is_empty())
                .then(|| duracoes.iter().sum::<i64>() as f64 / duracoes.len() as f64);

            ResultadoBraco {
                braco,
                taxa_sucesso,
                n,
                duracao_media_segundos,
            }
        })
        .collect()
}

/// Nota de limitação hardcoded -- spec: "Relatório nunca esconde que custo
/// em USD não é medido" é uma garantia, não uma opção que alguém possa
/// desligar.
const NOTA_LIMITACAO: &str = "Nota: custo em USD não é medido nesta implementação; \
duração (tempo de parede) é usada como proxy imperfeito.";

/// Formata o relatório do experimento H-1 -- sempre inclui `n` por braço e a
/// nota de limitação de métrica, mesmo com resultados vazios.
pub fn formatar_relatorio_h1(resultados: &[ResultadoBraco]) -> String {
    let mut saida = String::from("Relatório do experimento H-1 (Context Governor)\n\n");
    if resultados.is_empty() {
        saida.push_str("Nenhuma execução registrada ainda.\n\n");
    } else {
        for r in resultados {
            let duracao = r
                .duracao_media_segundos
                .map(|d| format!("{d:.1}s"))
                .unwrap_or_else(|| "—".to_string());
            saida.push_str(&format!(
                "braço {}: taxa {:.0}% (n={}), duração média {duracao}\n",
                r.braco,
                r.taxa_sucesso * 100.0,
                r.n
            ));
        }
        saida.push('\n');
    }
    saida.push_str(NOTA_LIMITACAO);
    saida.push('\n');
    saida
}

/// Uma tarefa sintética do experimento -- dado, não código (mesmo padrão de
/// `eval::CasoEval`). Rotulada como sintética em `experiments/h1-tasks.json`,
/// nunca apresentada como histórico real do autor.
#[derive(Debug, Clone)]
pub struct CasoExperimento {
    pub id: String,
    pub description: String,
    pub tipo: String,
    pub fixture_repo: PathBuf,
    pub base_commit: Option<String>,
    pub tarefa: String,
    pub provider_id: String,
    pub gate: Option<String>,
}

fn campo_texto<'a>(v: &'a serde_json::Value, campo: &str) -> Result<&'a str, String> {
    v.get(campo)
        .and_then(|x| x.as_str())
        .ok_or_else(|| format!("caso de experimento sem campo obrigatório: {campo}"))
}

/// Carrega os cases de `experiments/h1-tasks.json` -- mesma navegação via
/// `serde_json::Value` sem derive de `eval::carregar_caso`.
pub fn carregar_casos(caminho: &Path) -> Result<Vec<CasoExperimento>, String> {
    let texto = std::fs::read_to_string(caminho)
        .map_err(|e| format!("erro lendo {}: {e}", caminho.display()))?;
    let v: serde_json::Value = serde_json::from_str(&texto)
        .map_err(|e| format!("erro lendo {}: {e}", caminho.display()))?;
    let itens = v
        .as_array()
        .ok_or_else(|| format!("{} não é uma lista de cases", caminho.display()))?;

    itens
        .iter()
        .map(|item| {
            Ok(CasoExperimento {
                id: campo_texto(item, "id")?.to_string(),
                description: campo_texto(item, "description")?.to_string(),
                tipo: campo_texto(item, "tipo")?.to_string(),
                fixture_repo: PathBuf::from(campo_texto(item, "fixture_repo")?),
                base_commit: item
                    .get("base_commit")
                    .and_then(|x| x.as_str())
                    .map(str::to_string),
                tarefa: campo_texto(item, "task")?.to_string(),
                provider_id: item
                    .get("provider")
                    .and_then(|x| x.as_str())
                    .unwrap_or("codex")
                    .to_string(),
                gate: item
                    .get("gate")
                    .and_then(|x| x.as_str())
                    .map(str::to_string),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_git_temporario(sufixo: &str) -> std::path::PathBuf {
        let dir = crate::testutil::dir_temporario_unico(&format!("context-governor-{sufixo}"));
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
        std::fs::write(dir.join("refund.rs"), "fn refund() {}").unwrap();
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "inicial"]);
        dir
    }

    #[test]
    fn extrair_palavras_chave_ignora_stopwords_e_palavras_curtas() {
        let palavras = extrair_palavras_chave("Torne a operação de refund idempotente para o de");
        assert!(palavras.contains(&"torne".to_string()));
        assert!(palavras.contains(&"operação".to_string()));
        assert!(palavras.contains(&"refund".to_string()));
        assert!(palavras.contains(&"idempotente".to_string()));
        assert!(!palavras.contains(&"para".to_string()));
        assert!(!palavras.iter().any(|p| p.chars().count() < 4));
    }

    #[test]
    fn extrair_palavras_chave_deduplica() {
        let palavras = extrair_palavras_chave("refund refund refund");
        assert_eq!(palavras, vec!["refund".to_string()]);
    }

    #[test]
    fn buscar_arquivos_relevantes_encontra_e_deduplica() {
        let repo = repo_git_temporario("busca");
        let arquivos =
            buscar_arquivos_relevantes(&repo, &["refund".to_string(), "refund".to_string()]);
        assert_eq!(arquivos.len(), 1);
        assert!(arquivos[0].ends_with("refund.rs"));
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn montar_pacote_curado_respeita_orcamento() {
        let repo = repo_git_temporario("orcamento");
        let pacote = montar_pacote_curado(&repo, "tarefa sobre refund", &[], 20);
        assert!(pacote.chars().count() <= 20);
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn formatar_tarefa_por_braco_a_nao_recebe_pacote() {
        let tarefa = formatar_tarefa_por_braco("faz a tarefa", Some("pacote curado"), Braco::A);
        assert_eq!(tarefa, "faz a tarefa");
        assert!(!tarefa.contains("pacote curado"));
    }

    #[test]
    fn formatar_tarefa_por_braco_b_e_c_compartilham_pacote_com_instrucoes_diferentes() {
        let b = formatar_tarefa_por_braco("faz a tarefa", Some("pacote curado"), Braco::B);
        let c = formatar_tarefa_por_braco("faz a tarefa", Some("pacote curado"), Braco::C);
        assert!(b.contains("pacote curado"));
        assert!(c.contains("pacote curado"));
        assert_ne!(b, c);
    }

    #[test]
    fn calcular_resultado_por_braco_reporta_n_e_taxa() {
        let run = |status: StatusRun| RunRegistrado {
            id: "r".into(),
            client_id: CLIENT_ID_H1.into(),
            project: None,
            base_commit: "abc".into(),
            worktree_path: "/tmp/x".into(),
            branch: "brian/run_r".into(),
            provider_id: "codex".into(),
            pid: None,
            status,
            custo_equivalente: None,
            started_at: Instante(0),
            finished_at: Some(Instante(10)),
        };
        let execucoes = vec![
            ("a".to_string(), run(StatusRun::Concluido)),
            ("a".to_string(), run(StatusRun::Falhou)),
            ("b".to_string(), run(StatusRun::Concluido)),
        ];
        let resultados = calcular_resultado_por_braco(&execucoes);
        let a = resultados.iter().find(|r| r.braco == "a").unwrap();
        assert_eq!(a.n, 2);
        assert!((a.taxa_sucesso - 0.5).abs() < 1e-9);
        let b = resultados.iter().find(|r| r.braco == "b").unwrap();
        assert_eq!(b.n, 1);
        assert_eq!(b.taxa_sucesso, 1.0);
    }

    #[test]
    fn formatar_relatorio_h1_sempre_contem_nota_de_limitacao() {
        assert!(formatar_relatorio_h1(&[]).contains("custo em USD não é medido"));
        let resultados = vec![ResultadoBraco {
            braco: "a".into(),
            taxa_sucesso: 1.0,
            n: 3,
            duracao_media_segundos: Some(12.5),
        }];
        let relatorio = formatar_relatorio_h1(&resultados);
        assert!(relatorio.contains("custo em USD não é medido"));
        assert!(relatorio.contains("n=3"));
    }

    #[test]
    fn carregar_casos_le_lista_de_tarefas_sinteticas() {
        let dir = crate::testutil::dir_temporario_unico("h1-tasks");
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("h1-tasks.json");
        std::fs::write(
            &caminho,
            r#"[
                {"id": "h1-01", "description": "d", "tipo": "bug_pequeno",
                 "fixture_repo": "/tmp/fixtures/h1-01", "task": "faz X"}
            ]"#,
        )
        .unwrap();

        let casos = carregar_casos(&caminho).unwrap();
        assert_eq!(casos.len(), 1);
        assert_eq!(casos[0].id, "h1-01");
        assert_eq!(casos[0].tipo, "bug_pequeno");
        assert_eq!(casos[0].provider_id, "codex");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rodar_execucao_experimento_braco_a_nao_contem_pacote_na_tarefa() {
        use crate::storage::sqlite::SqliteStore;

        let s = SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        let repo = repo_git_temporario("run-a");

        // `base_commit` inválido faz a criação do worktree falhar de forma
        // determinística -- não depende de `codex` estar instalado para
        // produzir um run real (Falhou), mesmo padrão de `eval::rodar_caso`.
        let run = rodar_execucao_experimento(
            &s,
            &repo,
            "caso-1",
            Braco::A,
            "tarefa de teste",
            "codex",
            None,
            Some("commit-invalido"),
            Instante(0),
        )
        .unwrap();
        assert_eq!(run.client_id, CLIENT_ID_H1);
        assert_eq!(run.project.as_deref(), Some("caso-1"));
        assert_eq!(run.status, StatusRun::Falhou);

        let execucoes = s.execucoes_do_experimento(Some("a")).unwrap();
        assert_eq!(execucoes.len(), 1);
        assert_eq!(execucoes[0].case_id, "caso-1");

        std::fs::remove_dir_all(&repo).ok();
    }
}
