//! Continuity Pack: notas de memória, montagem do pack, handoff (grupos 3-5).
//!
//! Não faz SQL — recebe o que já foi lido/gravado via `Store`. Mesma separação
//! já usada em `capacidade.rs`/`identidade.rs`: função pura de decisão, glue
//! fina com storage.

use crate::domain::{
    ArquivoTocado, CategoriaNota, ContextoAtivo, Instante, NotaDeMemoria, PactoDeContinuidade,
};
use crate::storage::{NovaNota, Store};

#[derive(Debug)]
pub enum ErroNota {
    SemContextoAtivo,
    DecisaoSemRationale,
    NotaDeOutroContext(String),
    Storage(String),
}

impl std::fmt::Display for ErroNota {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SemContextoAtivo => write!(f, "nenhum contexto ativo"),
            Self::DecisaoSemRationale => write!(f, "decisão exige --why"),
            Self::NotaDeOutroContext(id) => write!(
                f,
                "nota '{id}' não pertence ao Context ativo -- supersede recusado"
            ),
            Self::Storage(m) => write!(f, "{m}"),
        }
    }
}

/// Registra uma nota sob o Context ativo. Erro explícito sem contexto ativo
/// (spec memory-notes: "Registrar nota sem contexto ativo") — recusado aqui,
/// antes de qualquer contato com storage, não como um efeito colateral de uma
/// falha de FK.
///
/// Decisão sem `rationale` é recusada nesta camada também (spec: "Registrar
/// decisão sem motivo") — a segunda barreira em `Store::registrar_nota` é
/// defesa em profundidade, não a única linha.
#[allow(clippy::too_many_arguments)]
pub fn registrar_nota(
    store: &dyn Store,
    contexto: Option<&ContextoAtivo>,
    id: String,
    categoria: CategoriaNota,
    texto: String,
    rationale: Option<String>,
    agora: Instante,
) -> Result<NotaDeMemoria, ErroNota> {
    let contexto = contexto.ok_or(ErroNota::SemContextoAtivo)?;

    if matches!(categoria, CategoriaNota::Decisao) && rationale.is_none() {
        return Err(ErroNota::DecisaoSemRationale);
    }

    store
        .registrar_nota(NovaNota {
            id,
            client_id: contexto.client_id.clone(),
            project: contexto.project.clone(),
            categoria,
            texto,
            rationale,
            created_at: agora,
        })
        .map_err(|e| ErroNota::Storage(e.to_string()))
}

/// Registra uma nota nova que substitui `supersedes_id` -- valida que a
/// nota anterior pertence ao Context ativo *antes* de gravar qualquer
/// coisa (spec memory-supersede: "Supersede de nota de outro Context é
/// recusado"), grava a nota nova via `registrar_nota` reaproveitado, e só
/// então marca a anterior. A nota anterior nunca é editada, só ganha o
/// ponteiro (D-14).
#[allow(clippy::too_many_arguments)]
pub fn supersede(
    store: &dyn Store,
    contexto: Option<&ContextoAtivo>,
    id: String,
    categoria: CategoriaNota,
    texto: String,
    rationale: Option<String>,
    supersedes_id: &str,
    agora: Instante,
) -> Result<NotaDeMemoria, ErroNota> {
    let ctx = contexto.ok_or(ErroNota::SemContextoAtivo)?;

    let notas_do_context = store
        .notas_do_contexto(&ctx.client_id, ctx.project.as_deref())
        .map_err(|e| ErroNota::Storage(e.to_string()))?;
    if !notas_do_context.iter().any(|n| n.id == supersedes_id) {
        return Err(ErroNota::NotaDeOutroContext(supersedes_id.to_string()));
    }

    let nova = registrar_nota(store, contexto, id, categoria, texto, rationale, agora)?;

    store
        .marcar_superseded(supersedes_id, &nova.id)
        .map_err(|e| ErroNota::Storage(e.to_string()))?;

    Ok(nova)
}

/// Arquivos alterados no repositório em `cwd`, via `git status --porcelain`.
/// Sem repositório Git (ou `git` indisponível): lista vazia, não erro (spec
/// pack: "Repositório sem alterações" e design.md, "sem repositório Git no
/// cwd, a seção fica vazia, não é erro").
pub fn arquivos_tocados(cwd: &std::path::Path) -> Vec<ArquivoTocado> {
    let saida = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(cwd)
        .output();

    let Ok(saida) = saida else {
        return Vec::new();
    };
    if !saida.status.success() {
        return Vec::new(); // não é repositório Git, ou outro problema local -- não inventa
    }

    String::from_utf8_lossy(&saida.stdout)
        .lines()
        .filter_map(|linha| {
            if linha.len() < 4 {
                return None;
            }
            let (status, path) = linha.split_at(2);
            Some(ArquivoTocado {
                path: path.trim().to_string(),
                status: status.to_string(),
            })
        })
        .collect()
}

/// Tamanho de referência do pack, em caracteres — estimativa, não contagem de
/// tokens real (design.md: "Orçamento em caracteres, rotulado como
/// estimativa"). ~8000 caracteres é uma referência conservadora, não um
/// limite físico de nenhum provider.
const ORCAMENTO_CARACTERES: usize = 8000;

/// Monta o pack a partir das notas do Context ativo e dos arquivos tocados no
/// `cwd` informado. Função pura sobre dados já obtidos — quem chama já leu as
/// notas e calculou `arquivos_tocados`.
pub fn montar_pacote(
    contexto: &ContextoAtivo,
    notas: Vec<NotaDeMemoria>,
    arquivos_tocados: Vec<ArquivoTocado>,
) -> PactoDeContinuidade {
    // Achado real ao testar contra este próprio repositório (task 7.3): uma
    // sessão de trabalho longa produz dezenas de arquivos tocados, que dominam
    // o tamanho do pack tanto quanto ou mais que as notas -- medir só as
    // notas subestimava o pack inteiro na prática.
    let tamanho_notas: usize = notas
        .iter()
        .map(|n| n.texto.len() + n.rationale.as_deref().map(str::len).unwrap_or(0))
        .sum();
    let tamanho_arquivos: usize = arquivos_tocados.iter().map(|a| a.path.len() + 4).sum();
    let tamanho_estimado = tamanho_notas + tamanho_arquivos;

    let aviso_orcamento = (tamanho_estimado > ORCAMENTO_CARACTERES).then(|| {
        format!(
            "pack com ~{tamanho_estimado} caracteres, acima da referência de {ORCAMENTO_CARACTERES}"
        )
    });

    PactoDeContinuidade {
        client_id: contexto.client_id.clone(),
        project: contexto.project.clone(),
        notas,
        arquivos_tocados,
        aviso_orcamento,
    }
}

#[derive(Debug)]
pub enum ErroHandoff {
    SemContextoAtivo,
    Storage(String),
}

impl std::fmt::Display for ErroHandoff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SemContextoAtivo => write!(f, "nenhum contexto ativo"),
            Self::Storage(m) => write!(f, "{m}"),
        }
    }
}

/// Monta o pack do Context ativo para handoff — só dados desse Context, nunca
/// de outro (spec handoff: "Handoff nunca mistura Context").
pub fn handoff(
    store: &dyn Store,
    contexto: Option<&ContextoAtivo>,
    cwd: &std::path::Path,
) -> Result<PactoDeContinuidade, ErroHandoff> {
    let contexto = contexto.ok_or(ErroHandoff::SemContextoAtivo)?;

    let notas = store
        .notas_do_contexto(&contexto.client_id, contexto.project.as_deref())
        .map_err(|e| ErroHandoff::Storage(e.to_string()))?;

    Ok(montar_pacote(contexto, notas, arquivos_tocados(cwd)))
}

/// Formata o pack para apresentação — texto estruturado por seção, arquivos
/// reais citados (spec handoff: "Handoff nunca exige reexplicação do
/// operador").
pub fn formatar_pacote(pacote: &PactoDeContinuidade, provider_destino: Option<&str>) -> String {
    let mut linhas = vec![
        "# Continuity Pack".to_string(),
        format!(
            "cliente: {} / projeto: {}",
            pacote.client_id,
            pacote.project.as_deref().unwrap_or("—")
        ),
    ];
    if let Some(provider) = provider_destino {
        linhas.push(format!("destino: {provider}"));
    }
    if let Some(aviso) = &pacote.aviso_orcamento {
        linhas.push(format!("aviso: {aviso}"));
    }
    linhas.push(String::new());

    for (categoria, titulo) in [
        (CategoriaNota::Objetivo, "## Objetivo"),
        (CategoriaNota::Decisao, "## Decisões"),
        (CategoriaNota::Analise, "## Análise"),
        (CategoriaNota::TentativaFalha, "## Tentativas que falharam"),
        (CategoriaNota::ProximoPasso, "## Próximos passos"),
        (CategoriaNota::Nota, "## Notas"),
    ] {
        let desta_categoria: Vec<&NotaDeMemoria> = pacote
            .notas
            .iter()
            .filter(|n| n.categoria == categoria)
            .collect();
        if desta_categoria.is_empty() {
            continue;
        }
        linhas.push(titulo.to_string());
        for nota in desta_categoria {
            match &nota.rationale {
                Some(why) => linhas.push(format!("- {} (por quê: {why})", nota.texto)),
                None => linhas.push(format!("- {}", nota.texto)),
            }
        }
        linhas.push(String::new());
    }

    linhas.push("## Arquivos tocados".to_string());
    if pacote.arquivos_tocados.is_empty() {
        linhas.push("(nenhuma alteração pendente no repositório)".to_string());
    } else {
        for arquivo in &pacote.arquivos_tocados {
            linhas.push(format!("- [{}] {}", arquivo.status, arquivo.path));
        }
    }

    linhas.join("\n")
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

    fn contexto() -> ContextoAtivo {
        ContextoAtivo {
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            identity_profile_id: "p1".into(),
            connected_at: Instante(0),
        }
    }

    #[test]
    fn registrar_nota_sem_contexto_ativo_e_erro() {
        let s = store();
        let erro = registrar_nota(
            &s,
            None,
            "n1".into(),
            CategoriaNota::Nota,
            "texto".into(),
            None,
            Instante(0),
        )
        .unwrap_err();
        assert!(matches!(erro, ErroNota::SemContextoAtivo));
    }

    #[test]
    fn registrar_decisao_sem_rationale_e_recusado_antes_do_storage() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let erro = registrar_nota(
            &s,
            Some(&contexto()),
            "n1".into(),
            CategoriaNota::Decisao,
            "texto".into(),
            None,
            Instante(0),
        )
        .unwrap_err();
        assert!(matches!(erro, ErroNota::DecisaoSemRationale));
    }

    #[test]
    fn registrar_nota_com_contexto_ativo_funciona() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let nota = registrar_nota(
            &s,
            Some(&contexto()),
            "n1".into(),
            CategoriaNota::Objetivo,
            "tornar refund idempotente".into(),
            None,
            Instante(0),
        )
        .unwrap();
        assert_eq!(nota.client_id, "xpto");
        assert_eq!(nota.project.as_deref(), Some("checkout-api"));
    }

    #[test]
    fn supersede_do_mesmo_context_funciona_end_to_end() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        registrar_nota(
            &s,
            Some(&contexto()),
            "n1".into(),
            CategoriaNota::Decisao,
            "usar claude".into(),
            Some("mais barato".into()),
            Instante(0),
        )
        .unwrap();

        let nova = supersede(
            &s,
            Some(&contexto()),
            "n2".into(),
            CategoriaNota::Decisao,
            "usar codex".into(),
            Some("claude ficou instável".into()),
            "n1",
            Instante(10),
        )
        .unwrap();
        assert_eq!(nova.id, "n2");

        let notas = s.notas_do_contexto("xpto", Some("checkout-api")).unwrap();
        let anterior = notas.iter().find(|n| n.id == "n1").unwrap();
        assert_eq!(anterior.texto, "usar claude");
        assert_eq!(anterior.superseded_by.as_deref(), Some("n2"));
    }

    #[test]
    fn supersede_de_nota_de_outro_context_e_recusado_sem_gravar_nada() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();
        registrar_nota(
            &s,
            Some(&ContextoAtivo {
                client_id: "acme".into(),
                project: None,
                identity_profile_id: "p2".into(),
                connected_at: Instante(0),
            }),
            "n1".into(),
            CategoriaNota::Nota,
            "nota da acme".into(),
            None,
            Instante(0),
        )
        .unwrap();

        let erro = supersede(
            &s,
            Some(&contexto()),
            "n2".into(),
            CategoriaNota::Nota,
            "tentativa cruzando context".into(),
            None,
            "n1",
            Instante(10),
        )
        .unwrap_err();
        assert!(matches!(erro, ErroNota::NotaDeOutroContext(_)));

        let notas_xpto = s.notas_do_contexto("xpto", Some("checkout-api")).unwrap();
        assert!(
            notas_xpto.is_empty(),
            "nenhuma nota nova deveria ter sido gravada"
        );
    }

    #[test]
    fn supersede_de_id_inexistente_e_recusado() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let erro = supersede(
            &s,
            Some(&contexto()),
            "n2".into(),
            CategoriaNota::Nota,
            "texto".into(),
            None,
            "fantasma",
            Instante(0),
        )
        .unwrap_err();
        assert!(matches!(erro, ErroNota::NotaDeOutroContext(_)));
    }

    #[test]
    fn arquivos_tocados_de_diretorio_sem_git_e_vazio() {
        let dir = crate::testutil::dir_temporario_unico("sem-git");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(arquivos_tocados(&dir).is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn arquivos_tocados_de_repositorio_git_real_com_alteracao() {
        let dir = crate::testutil::dir_temporario_unico("git-real");
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
        std::fs::write(dir.join("arquivo.txt"), "conteudo inicial").unwrap();
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "inicial"]);
        std::fs::write(dir.join("arquivo.txt"), "conteudo alterado").unwrap();
        std::fs::write(dir.join("novo.txt"), "novo arquivo").unwrap();

        let tocados = arquivos_tocados(&dir);
        assert_eq!(tocados.len(), 2, "1 modificado + 1 novo");
        assert!(tocados.iter().any(|a| a.path == "arquivo.txt"));
        assert!(tocados.iter().any(|a| a.path == "novo.txt"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn arquivos_tocados_sem_alteracoes_e_vazio() {
        let dir = crate::testutil::dir_temporario_unico("git-limpo");
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
        std::fs::write(dir.join("arquivo.txt"), "conteudo").unwrap();
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "inicial"]);

        assert!(arquivos_tocados(&dir).is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn montar_pacote_agrupa_notas_por_categoria() {
        let ctx = contexto();
        let notas = vec![
            NotaDeMemoria {
                id: "n1".into(),
                client_id: "xpto".into(),
                project: Some("checkout-api".into()),
                categoria: CategoriaNota::Objetivo,
                texto: "objetivo".into(),
                rationale: None,
                created_at: Instante(0),
                superseded_by: None,
            },
            NotaDeMemoria {
                id: "n2".into(),
                client_id: "xpto".into(),
                project: Some("checkout-api".into()),
                categoria: CategoriaNota::Decisao,
                texto: "decisão".into(),
                rationale: Some("motivo".into()),
                created_at: Instante(1),
                superseded_by: None,
            },
        ];
        let pacote = montar_pacote(&ctx, notas, Vec::new());
        assert_eq!(pacote.notas.len(), 2);
        assert_eq!(pacote.aviso_orcamento, None);
    }

    #[test]
    fn montar_pacote_sem_notas_nao_e_erro() {
        let pacote = montar_pacote(&contexto(), Vec::new(), Vec::new());
        assert!(pacote.notas.is_empty());
    }

    #[test]
    fn montar_pacote_acima_do_orcamento_avisa() {
        let nota = NotaDeMemoria {
            id: "n1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            categoria: CategoriaNota::Analise,
            texto: "x".repeat(ORCAMENTO_CARACTERES + 1),
            rationale: None,
            created_at: Instante(0),
            superseded_by: None,
        };
        let pacote = montar_pacote(&contexto(), vec![nota], Vec::new());
        assert!(pacote.aviso_orcamento.is_some());
        assert_eq!(
            pacote.notas[0].texto.len(),
            ORCAMENTO_CARACTERES + 1,
            "conteúdo não é truncado, só sinalizado"
        );
    }

    #[test]
    fn montar_pacote_acima_do_orcamento_por_arquivos_tocados_avisa() {
        // Achado real (task 7.3): notas curtas, mas dezenas de arquivos
        // tocados -- o aviso deve considerar isso, não só o texto das notas.
        let muitos_arquivos: Vec<ArquivoTocado> = (0..500)
            .map(|i| ArquivoTocado {
                path: format!("src/arquivo_{i}.rs"),
                status: " M".into(),
            })
            .collect();
        let pacote = montar_pacote(&contexto(), Vec::new(), muitos_arquivos);
        assert!(
            pacote.aviso_orcamento.is_some(),
            "500 arquivos tocados devem estourar o orçamento mesmo sem notas"
        );
    }

    #[test]
    fn handoff_sem_contexto_ativo_e_erro() {
        let s = store();
        let erro = handoff(&s, None, std::path::Path::new("/tmp")).unwrap_err();
        assert!(matches!(erro, ErroHandoff::SemContextoAtivo));
    }

    #[test]
    fn handoff_usa_apenas_notas_do_contexto_ativo() {
        // Spec handoff, "Handoff nunca mistura Context".
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();
        registrar_nota(
            &s,
            Some(&contexto()),
            "n1".into(),
            CategoriaNota::Nota,
            "nota do xpto".into(),
            None,
            Instante(0),
        )
        .unwrap();
        let ctx_acme = ContextoAtivo {
            client_id: "acme".into(),
            project: None,
            identity_profile_id: "p2".into(),
            connected_at: Instante(0),
        };
        registrar_nota(
            &s,
            Some(&ctx_acme),
            "n2".into(),
            CategoriaNota::Nota,
            "nota do acme".into(),
            None,
            Instante(0),
        )
        .unwrap();

        let pacote = handoff(&s, Some(&contexto()), std::path::Path::new("/tmp")).unwrap();
        assert_eq!(pacote.notas.len(), 1);
        assert_eq!(pacote.notas[0].texto, "nota do xpto");
    }

    #[test]
    fn formatar_pacote_cita_arquivos_reais() {
        let pacote = PactoDeContinuidade {
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            notas: vec![NotaDeMemoria {
                id: "n1".into(),
                client_id: "xpto".into(),
                project: Some("checkout-api".into()),
                categoria: CategoriaNota::Objetivo,
                texto: "tornar refund idempotente".into(),
                rationale: None,
                created_at: Instante(0),
                superseded_by: None,
            }],
            arquivos_tocados: vec![ArquivoTocado {
                path: "src/payment/RefundService.ts".into(),
                status: " M".into(),
            }],
            aviso_orcamento: None,
        };
        let texto = formatar_pacote(&pacote, Some("codex"));
        assert!(texto.contains("tornar refund idempotente"));
        assert!(texto.contains("src/payment/RefundService.ts"));
        assert!(texto.contains("destino: codex"));
    }
}
