//! Recall de memória (continuity/memory-recall, blueprint §35): seleção
//! orçada das notas do Context ativo para injeção no prompt de um
//! `brian run` -- distinto do Continuity Pack (`continuidade::montar_pacote`),
//! que continua completo e só avisa quando grande. Aqui o destino é um
//! prompt, então o orçamento corta de verdade, silenciosamente.

use crate::domain::{CategoriaNota, ContextoAtivo, NotaDeMemoria};
use crate::storage::Store;

/// blueprint §35.2: `max_items: 8`, `max_tokens: 4000` -- caracteres como
/// proxy de tokens, mesma convenção de `continuidade`/`context_governor`.
#[derive(Debug, Clone, Copy)]
pub struct OrcamentoRecall {
    pub max_items: usize,
    pub max_caracteres: usize,
}

impl Default for OrcamentoRecall {
    fn default() -> Self {
        Self {
            max_items: 8,
            max_caracteres: 4000,
        }
    }
}

#[derive(Debug)]
pub enum ErroRecall {
    Storage(String),
}

impl std::fmt::Display for ErroRecall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for ErroRecall {}

fn tamanho(nota: &NotaDeMemoria) -> usize {
    nota.texto.len() + nota.rationale.as_deref().map(str::len).unwrap_or(0)
}

/// Seleciona um subconjunto orçado de `notas` (já esperadas em ordem de
/// recência, mesmo contrato de `Store::notas_do_contexto`) -- `decisao`
/// primeiro (spec: "sempre priorizadas quando existem"), depois as demais
/// por recência; corta por `max_items` e por `max_caracteres` acumulado,
/// o que vier primeiro. Uma nota individual gigante ainda entra (nunca
/// trunca o texto de uma nota), só limita o que mais entra depois dela.
pub fn selecionar_para_recall(
    notas: Vec<NotaDeMemoria>,
    orcamento: &OrcamentoRecall,
) -> Vec<NotaDeMemoria> {
    let (decisoes, demais): (Vec<_>, Vec<_>) = notas
        .into_iter()
        .partition(|n| n.categoria == CategoriaNota::Decisao);

    let mut selecionadas = Vec::new();
    let mut total_caracteres = 0usize;
    for nota in decisoes.into_iter().chain(demais) {
        if selecionadas.len() >= orcamento.max_items {
            break;
        }
        if !selecionadas.is_empty() && total_caracteres + tamanho(&nota) > orcamento.max_caracteres
        {
            break;
        }
        total_caracteres += tamanho(&nota);
        selecionadas.push(nota);
    }
    selecionadas
}

fn rotulo_categoria(c: CategoriaNota) -> &'static str {
    match c {
        CategoriaNota::Objetivo => "objetivo",
        CategoriaNota::Decisao => "decisao",
        CategoriaNota::Analise => "analise",
        CategoriaNota::TentativaFalha => "tentativa_falha",
        CategoriaNota::ProximoPasso => "proximo_passo",
        CategoriaNota::Nota => "nota",
    }
}

/// Formata as notas selecionadas como bloco de texto simples, na ordem em
/// que foram selecionadas -- string vazia se `notas` estiver vazio (spec:
/// "Recall vazio quando o Context não tem notas").
pub fn formatar_recall(notas: &[NotaDeMemoria]) -> String {
    if notas.is_empty() {
        return String::new();
    }
    let mut saida = String::from("Memória relevante deste Context:\n");
    for nota in notas {
        saida.push_str("- [");
        saida.push_str(rotulo_categoria(nota.categoria));
        saida.push_str("] ");
        saida.push_str(&nota.texto);
        if let Some(rationale) = &nota.rationale {
            saida.push_str(" (motivo: ");
            saida.push_str(rationale);
            saida.push(')');
        }
        saida.push('\n');
    }
    saida
}

/// Busca as notas do Context ativo, seleciona e formata -- string vazia
/// quando não há notas.
pub fn montar_recall(
    store: &dyn Store,
    contexto: &ContextoAtivo,
    orcamento: &OrcamentoRecall,
) -> Result<String, ErroRecall> {
    let notas = store
        .notas_do_contexto(&contexto.client_id, contexto.project.as_deref())
        .map_err(|e| ErroRecall::Storage(e.to_string()))?;
    // Nota já substituída nunca entra no recall junto com a que a
    // substituiu (continuity/memory-supersede, spec: "Recall exclui
    // notas já superseded").
    let vigentes: Vec<_> = notas
        .into_iter()
        .filter(|n| n.superseded_by.is_none())
        .collect();
    let selecionadas = selecionar_para_recall(vigentes, orcamento);
    Ok(formatar_recall(&selecionadas))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Instante;

    fn nota(id: &str, categoria: CategoriaNota, texto: &str, created_at: i64) -> NotaDeMemoria {
        NotaDeMemoria {
            id: id.into(),
            client_id: "xpto".into(),
            project: None,
            categoria,
            texto: texto.into(),
            rationale: None,
            created_at: Instante(created_at),
            superseded_by: None,
        }
    }

    #[test]
    fn corta_por_max_items() {
        let notas: Vec<_> = (0..12)
            .map(|i| nota(&format!("n{i}"), CategoriaNota::Nota, "texto curto", i))
            .collect();
        let orcamento = OrcamentoRecall {
            max_items: 8,
            max_caracteres: 100_000,
        };
        let selecionadas = selecionar_para_recall(notas, &orcamento);
        assert_eq!(selecionadas.len(), 8);
    }

    #[test]
    fn corta_por_max_caracteres_mesmo_com_max_items_disponivel() {
        let notas = vec![
            nota("n1", CategoriaNota::Nota, &"a".repeat(30), 0),
            nota("n2", CategoriaNota::Nota, &"b".repeat(30), 1),
            nota("n3", CategoriaNota::Nota, &"c".repeat(30), 2),
        ];
        let orcamento = OrcamentoRecall {
            max_items: 8,
            max_caracteres: 50,
        };
        let selecionadas = selecionar_para_recall(notas, &orcamento);
        assert_eq!(selecionadas.len(), 1);
    }

    #[test]
    fn decisao_antiga_sobrevive_ao_corte_com_notas_mais_recentes() {
        let notas = vec![
            nota("velha-decisao", CategoriaNota::Decisao, "decisão antiga", 0),
            nota("recente1", CategoriaNota::Nota, "nota recente 1", 10),
            nota("recente2", CategoriaNota::Nota, "nota recente 2", 11),
        ];
        let orcamento = OrcamentoRecall {
            max_items: 2,
            max_caracteres: 100_000,
        };
        let selecionadas = selecionar_para_recall(notas, &orcamento);
        assert!(selecionadas.iter().any(|n| n.id == "velha-decisao"));
    }

    #[test]
    fn lista_vazia_produz_string_vazia() {
        assert_eq!(formatar_recall(&[]), "");
    }

    #[test]
    fn nota_individual_gigante_ainda_entra_sozinha() {
        let notas = vec![nota("gigante", CategoriaNota::Nota, &"x".repeat(10_000), 0)];
        let orcamento = OrcamentoRecall {
            max_items: 8,
            max_caracteres: 100,
        };
        let selecionadas = selecionar_para_recall(notas, &orcamento);
        assert_eq!(selecionadas.len(), 1);
    }

    #[test]
    fn formatar_recall_inclui_categoria_e_motivo() {
        let mut n = nota("d1", CategoriaNota::Decisao, "usar codex", 0);
        n.rationale = Some("mais confiável".into());
        let saida = formatar_recall(&[n]);
        assert!(saida.contains("[decisao]"));
        assert!(saida.contains("usar codex"));
        assert!(saida.contains("motivo: mais confiável"));
    }

    #[test]
    fn montar_recall_exclui_nota_superseded() {
        use crate::storage::sqlite::SqliteStore;

        let s = SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s.upsert_client("xpto").unwrap();
        crate::continuidade::registrar_nota(
            &s,
            Some(&contexto_teste()),
            "n1".into(),
            CategoriaNota::Decisao,
            "usar claude".into(),
            Some("mais barato".into()),
            Instante(0),
        )
        .unwrap();
        crate::continuidade::supersede(
            &s,
            Some(&contexto_teste()),
            "n2".into(),
            CategoriaNota::Decisao,
            "usar codex".into(),
            Some("claude ficou instável".into()),
            "n1",
            Instante(10),
        )
        .unwrap();

        let recall = montar_recall(&s, &contexto_teste(), &OrcamentoRecall::default()).unwrap();
        assert!(recall.contains("usar codex"));
        assert!(!recall.contains("usar claude"));
    }

    fn contexto_teste() -> ContextoAtivo {
        ContextoAtivo {
            client_id: "xpto".into(),
            project: None,
            identity_profile_id: "p1".into(),
            connected_at: Instante(0),
        }
    }
}
