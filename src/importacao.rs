//! Importação de consumo a partir de adapters de provider (grupo 5).
//!
//! Orquestra: resolve o período ainda não coberto, coleta, calcula custo via
//! `custo::resolver_custo`, grava via `Store::gravar_consumo` (idempotente
//! pela `dedup_key`). Não faz SQL — quem fala com o banco é `storage`.

use crate::custo::resolver_custo;
use crate::domain::{BillingMode, Instante, Money, Tokens, UsageSource};
use crate::storage::{NovoConsumo, Periodo, Result as StorageResult, Store};

/// Hierarquia de integração (D-4). `headless_json` nunca é substituído por
/// `pty` a não ser que nada melhor esteja disponível — a ordem aqui é a
/// ordem de preferência, não apenas um rótulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TierIntegracao {
    Pty,
    SessionFiles,
    HeadlessJson,
}

/// Um consumo bruto, como o adapter o observou — antes de dedup, custo
/// resolvido ou gravação. `identificador_estavel` é o que o provider oferece
/// como ID de chamada, quando oferece (task 5.2).
#[derive(Debug, Clone)]
pub struct ConsumoColetado {
    pub identificador_estavel: Option<String>,
    pub provider_id: String,
    pub model: String,
    pub tokens: Tokens,
    pub custo_pago: Option<Money>,
    pub billing_mode: BillingMode,
    pub usage_source: UsageSource,
    /// Referência de sessão, usada no fallback de impressão digital quando
    /// não há identificador estável (task 5.3).
    pub session_ref: Option<String>,
    pub occurred_at: Instante,
    pub client_id: Option<String>,
}

/// Falha de coleta de um provider específico. Isolada: não impede a
/// importação dos demais (task 5.6).
#[derive(Debug)]
pub struct ErroColeta {
    pub motivo: String,
}

/// O que um adapter de provider implementa.
///
/// Cada adapter declara seu próprio tier e campos — o núcleo não presume
/// paridade entre providers (design.md: "capacidade declarada é honesta;
/// capacidade presumida vira número inventado").
pub trait ColetorDeUso {
    fn provider_id(&self) -> &str;
    fn tier(&self) -> TierIntegracao;
    /// Campos que este adapter consegue fornecer — insumo da cobertura
    /// declarada da task 6.7. Vazio é uma resposta válida (tier degradado).
    fn campos_disponiveis(&self) -> &[&'static str];
    fn coletar(&self, periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta>;
}

/// Impressão digital de dedup para consumo sem identificador estável de
/// provider (task 5.3). FNV-1a: determinística, sem dependência nova para
/// uma chave que não precisa resistir a ataque, só a colisão acidental.
fn fingerprint(c: &ConsumoColetado) -> String {
    let base = format!(
        "{}|{}|{}|{:?}|{:?}|{:?}|{:?}|{}",
        c.provider_id,
        c.model,
        c.occurred_at.0,
        c.tokens.input,
        c.tokens.cache,
        c.tokens.output,
        c.tokens.reasoning,
        c.session_ref.as_deref().unwrap_or(""),
    );

    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in base.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fp:{hash:016x}")
}

fn dedup_key(c: &ConsumoColetado) -> String {
    match &c.identificador_estavel {
        Some(id) => format!("{}:{id}", c.provider_id),
        None => fingerprint(c),
    }
}

/// Resultado da importação de um provider.
#[derive(Debug)]
pub struct ResultadoProvider {
    pub provider_id: String,
    pub gravados: usize,
    /// `Some` se a coleta falhou. Os demais providers seguem normalmente
    /// (task 5.6) — este campo é o que torna a falha visível, não um estado
    /// de importação parcial silencioso.
    pub erro: Option<String>,
}

pub fn importar(
    store: &dyn Store,
    coletores: &[Box<dyn ColetorDeUso>],
    periodo: Periodo,
) -> StorageResult<Vec<ResultadoProvider>> {
    let mut resultados = Vec::with_capacity(coletores.len());

    for coletor in coletores {
        // Task 5.5: só o período ainda não coberto, quando já há consumo
        // importado desse provider dentro da janela pedida.
        let desde_efetivo = match store.ultimo_consumo_importado(coletor.provider_id())? {
            Some(ultimo) if ultimo.0 >= periodo.desde.0 => Instante(ultimo.0 + 1),
            _ => periodo.desde,
        };
        let periodo_efetivo = Periodo {
            desde: desde_efetivo,
            ate: periodo.ate,
        };

        match coletor.coletar(periodo_efetivo) {
            Ok(itens) => {
                let mut gravados = 0;
                for item in itens {
                    let catalogo = store.preco_vigente(&item.model, item.occurred_at)?;
                    let (custo, cost_source) =
                        resolver_custo(item.custo_pago, &item.tokens, catalogo.as_ref());

                    store.gravar_consumo(NovoConsumo {
                        dedup_key: dedup_key(&item),
                        provider_id: item.provider_id,
                        model: item.model,
                        tokens: item.tokens,
                        custo_pago: custo.pago,
                        custo_equivalente_api: custo.equivalente_api,
                        billing_mode: item.billing_mode,
                        usage_source: item.usage_source,
                        cost_source,
                        client_id: item.client_id,
                        occurred_at: item.occurred_at,
                    })?;
                    gravados += 1;
                }
                resultados.push(ResultadoProvider {
                    provider_id: coletor.provider_id().to_string(),
                    gravados,
                    erro: None,
                });
            }
            Err(e) => resultados.push(ResultadoProvider {
                provider_id: coletor.provider_id().to_string(),
                gravados: 0,
                erro: Some(e.motivo),
            }),
        }
    }

    Ok(resultados)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::sqlite::SqliteStore;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn store() -> SqliteStore {
        let s = SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s
    }

    fn item(provider: &str, id_estavel: Option<&str>, occurred_at: i64) -> ConsumoColetado {
        ConsumoColetado {
            identificador_estavel: id_estavel.map(String::from),
            provider_id: provider.into(),
            model: "opus".into(),
            tokens: Tokens {
                input: Some(100),
                cache: None,
                output: Some(50),
                reasoning: None,
            },
            custo_pago: None,
            billing_mode: BillingMode::Api,
            usage_source: UsageSource::Provider,
            session_ref: Some("sess1".into()),
            occurred_at: Instante(occurred_at),
            client_id: None,
        }
    }

    fn periodo_aberto() -> Periodo {
        Periodo {
            desde: Instante(0),
            ate: None,
        }
    }

    /// Coletor de teste: devolve itens fixos e, opcionalmente, registra num
    /// `Rc<RefCell<>>` compartilhado o período com que foi chamado — usado
    /// pelo teste de cobertura parcial (5.5) para observar sem `unsafe`.
    struct ColetorFixo {
        provider: &'static str,
        itens: Vec<ConsumoColetado>,
        periodos_recebidos: Option<Rc<RefCell<Vec<Periodo>>>>,
    }

    impl ColetorDeUso for ColetorFixo {
        fn provider_id(&self) -> &str {
            self.provider
        }
        fn tier(&self) -> TierIntegracao {
            TierIntegracao::HeadlessJson
        }
        fn campos_disponiveis(&self) -> &[&'static str] {
            &["tokens", "custo"]
        }
        fn coletar(&self, periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta> {
            if let Some(registro) = &self.periodos_recebidos {
                registro.borrow_mut().push(periodo);
            }
            Ok(self.itens.clone())
        }
    }

    struct ColetorQuebrado {
        provider: &'static str,
    }

    impl ColetorDeUso for ColetorQuebrado {
        fn provider_id(&self) -> &str {
            self.provider
        }
        fn tier(&self) -> TierIntegracao {
            TierIntegracao::SessionFiles
        }
        fn campos_disponiveis(&self) -> &[&'static str] {
            &[]
        }
        fn coletar(&self, _periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta> {
            Err(ErroColeta {
                motivo: "fonte fora do ar".into(),
            })
        }
    }

    #[test]
    fn dedup_por_identificador_estavel_e_idempotente() {
        let s = store();
        let coletores: Vec<Box<dyn ColetorDeUso>> = vec![Box::new(ColetorFixo {
            provider: "claude",
            itens: vec![item("claude", Some("call-1"), 100)],
            periodos_recebidos: None,
        })];

        importar(&s, &coletores, periodo_aberto()).unwrap();
        importar(&s, &coletores, periodo_aberto()).unwrap();

        let todos = s.consumo_no_periodo(periodo_aberto()).unwrap();
        assert_eq!(todos.len(), 1, "reimportar não duplica");
    }

    #[test]
    fn fingerprint_e_estavel_para_o_mesmo_item() {
        let a = item("claude", None, 100);
        let b = item("claude", None, 100);
        assert_eq!(fingerprint(&a), fingerprint(&b));
    }

    #[test]
    fn fingerprint_difere_para_instantes_diferentes() {
        let a = item("claude", None, 100);
        let b = item("claude", None, 200);
        assert_ne!(fingerprint(&a), fingerprint(&b));
    }

    #[test]
    fn falha_de_um_provider_nao_impede_os_demais() {
        let s = store();
        let bom: Box<dyn ColetorDeUso> = Box::new(ColetorFixo {
            provider: "codex",
            itens: vec![item("codex", Some("c1"), 100)],
            periodos_recebidos: None,
        });
        let quebrado: Box<dyn ColetorDeUso> = Box::new(ColetorQuebrado { provider: "gemini" });

        let resultados = importar(&s, &[bom, quebrado], periodo_aberto()).unwrap();

        let codex = resultados
            .iter()
            .find(|r| r.provider_id == "codex")
            .unwrap();
        assert_eq!(codex.gravados, 1);
        assert!(codex.erro.is_none());

        let gemini = resultados
            .iter()
            .find(|r| r.provider_id == "gemini")
            .unwrap();
        assert_eq!(gemini.gravados, 0);
        assert_eq!(gemini.erro.as_deref(), Some("fonte fora do ar"));
    }

    #[test]
    fn importa_apenas_periodo_ainda_nao_coberto() {
        let s = store();

        // Primeira importação: grava um consumo em occurred_at=100.
        let primeira: Vec<Box<dyn ColetorDeUso>> = vec![Box::new(ColetorFixo {
            provider: "claude",
            itens: vec![item("claude", Some("call-1"), 100)],
            periodos_recebidos: None,
        })];
        importar(&s, &primeira, periodo_aberto()).unwrap();

        // Segunda importação: o coletor deve receber `desde` = 101, não 0 —
        // o período de 0 a 100 já está coberto.
        let recebidos = Rc::new(RefCell::new(Vec::new()));
        let segunda: Vec<Box<dyn ColetorDeUso>> = vec![Box::new(ColetorFixo {
            provider: "claude",
            itens: vec![item("claude", Some("call-2"), 200)],
            periodos_recebidos: Some(Rc::clone(&recebidos)),
        })];
        importar(&s, &segunda, periodo_aberto()).unwrap();

        assert_eq!(recebidos.borrow()[0].desde, Instante(101));
    }
}
