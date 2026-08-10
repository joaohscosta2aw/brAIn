//! Orçamento mensal por cliente (capacity/budget-alerts, blueprint §45,
//! D-16). Config opt-in (`budgets/clients.json`): cliente sem entrada
//! nunca é bloqueado nem alertado. Este módulo só calcula status a partir
//! de dados já apurados por quem chama (`Store::consumo_do_cliente` +
//! `comandos::agregar`) — não sabe de SQL nem de período (D-9).

use crate::domain::Money;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub enum ErroBudget {
    Json(String),
}

impl std::fmt::Display for ErroBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(m) => write!(f, "erro lendo orçamentos: {m}"),
        }
    }
}

impl std::error::Error for ErroBudget {}

/// Orçamento mensal de um cliente -- todos os campos opcionais (spec:
/// cliente pode limitar só por USD, só por tokens, ou nem estar presente).
#[derive(Debug, Clone, Default)]
pub struct BudgetCliente {
    pub monthly_usd_equivalent: Option<Money>,
    pub monthly_tokens: Option<u64>,
    pub alert_at_percent: Vec<u8>,
}

/// Carrega `budgets/clients.json`. Arquivo ausente é o estado padrão
/// esperado (orçamento é opt-in) -- devolve mapa vazio, não erro. Erro só
/// se o arquivo existir e não for JSON válido.
pub fn carregar_budgets(caminho: &Path) -> Result<HashMap<String, BudgetCliente>, ErroBudget> {
    let texto = match std::fs::read_to_string(caminho) {
        Ok(t) => t,
        Err(_) => return Ok(HashMap::new()),
    };
    let v: serde_json::Value =
        serde_json::from_str(&texto).map_err(|e| ErroBudget::Json(e.to_string()))?;

    let clientes = v
        .get("clients")
        .and_then(|c| c.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(client_id, cfg)| {
                    let monthly_usd_equivalent = cfg
                        .get("monthly_usd_equivalent")
                        .and_then(|x| x.as_f64())
                        .and_then(Money::de_unidades);
                    let monthly_tokens = cfg.get("monthly_tokens").and_then(|x| x.as_u64());
                    let alert_at_percent = cfg
                        .get("alert_at_percent")
                        .and_then(|x| x.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|p| p.as_u64())
                                .map(|p| p as u8)
                                .collect()
                        })
                        .unwrap_or_default();
                    (
                        client_id.clone(),
                        BudgetCliente {
                            monthly_usd_equivalent,
                            monthly_tokens,
                            alert_at_percent,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(clientes)
}

/// Status de orçamento calculado -- gasto e limite sempre lado a lado
/// (spec: "Relatório de orçamento nunca esconde o gasto real por trás do
/// limite").
#[derive(Debug, Clone, PartialEq)]
pub struct StatusBudget {
    pub client_id: String,
    pub gasto_usd: Option<Money>,
    pub limite_usd: Option<Money>,
    pub gasto_tokens: u64,
    pub limite_tokens: Option<u64>,
    /// Subconjunto de `alert_at_percent` já ultrapassado pelo % de gasto
    /// USD.
    pub alertas_cruzados: Vec<u8>,
    /// `true` se o gasto já atingiu ou excedeu o limite de USD OU de
    /// tokens -- qualquer um dos dois configurados que estoure já basta.
    pub limite_excedido: bool,
}

/// Função pura, sem I/O -- calcula o status a partir do orçamento
/// configurado (ou ausente) e do gasto já apurado por quem chama.
pub fn calcular_status(
    client_id: &str,
    budget: Option<&BudgetCliente>,
    gasto_usd: Option<Money>,
    gasto_tokens: u64,
) -> StatusBudget {
    let Some(budget) = budget else {
        return StatusBudget {
            client_id: client_id.to_string(),
            gasto_usd,
            limite_usd: None,
            gasto_tokens,
            limite_tokens: None,
            alertas_cruzados: Vec::new(),
            limite_excedido: false,
        };
    };

    let excedeu_usd = match (gasto_usd, budget.monthly_usd_equivalent) {
        (Some(g), Some(limite)) => g >= limite,
        _ => false,
    };
    let excedeu_tokens = match budget.monthly_tokens {
        Some(limite) => gasto_tokens >= limite,
        None => false,
    };

    let percent_usd = match (gasto_usd, budget.monthly_usd_equivalent) {
        (Some(g), Some(limite)) if limite.em_unidades() > 0.0 => {
            Some(g.em_unidades() / limite.em_unidades() * 100.0)
        }
        _ => None,
    };
    let mut alertas_cruzados: Vec<u8> = match percent_usd {
        Some(p) => budget
            .alert_at_percent
            .iter()
            .copied()
            .filter(|&limiar| p >= limiar as f64)
            .collect(),
        None => Vec::new(),
    };
    alertas_cruzados.sort_unstable();

    StatusBudget {
        client_id: client_id.to_string(),
        gasto_usd,
        limite_usd: budget.monthly_usd_equivalent,
        gasto_tokens,
        limite_tokens: budget.monthly_tokens,
        alertas_cruzados,
        limite_excedido: excedeu_usd || excedeu_tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carregar_budgets_de_arquivo_ausente_e_mapa_vazio() {
        let budgets = carregar_budgets(Path::new("/caminho/que/nao/existe.json")).unwrap();
        assert!(budgets.is_empty());
    }

    #[test]
    fn carregar_budgets_com_json_invalido_e_erro_claro() {
        let dir = crate::testutil::dir_temporario_unico("budget-invalido");
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("clients.json");
        std::fs::write(&caminho, "{ nao é json").unwrap();

        let erro = carregar_budgets(&caminho).unwrap_err();
        assert!(matches!(erro, ErroBudget::Json(_)));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn carregar_budgets_le_todos_os_campos() {
        let dir = crate::testutil::dir_temporario_unico("budget-le");
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("clients.json");
        std::fs::write(
            &caminho,
            r#"{"clients": {"xpto": {"monthly_usd_equivalent": 500, "monthly_tokens": 50000000, "alert_at_percent": [50, 80, 95]}}}"#,
        )
        .unwrap();

        let budgets = carregar_budgets(&caminho).unwrap();
        let b = budgets.get("xpto").unwrap();
        assert_eq!(b.monthly_usd_equivalent, Money::de_unidades(500.0));
        assert_eq!(b.monthly_tokens, Some(50_000_000));
        assert_eq!(b.alert_at_percent, vec![50, 80, 95]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cliente_sem_budget_nunca_tem_limite_excedido_nem_alertas() {
        let status = calcular_status("xpto", None, Money::de_unidades(10_000.0), 999_999_999);
        assert!(!status.limite_excedido);
        assert!(status.alertas_cruzados.is_empty());
    }

    #[test]
    fn limiares_cruzados_nos_limites_exatos() {
        let budget = BudgetCliente {
            monthly_usd_equivalent: Money::de_unidades(100.0),
            monthly_tokens: None,
            alert_at_percent: vec![50, 80, 95],
        };
        let status = calcular_status("xpto", Some(&budget), Money::de_unidades(80.0), 0);
        assert_eq!(status.alertas_cruzados, vec![50, 80]);
        assert!(!status.limite_excedido);
    }

    #[test]
    fn limite_excedido_por_usd() {
        let budget = BudgetCliente {
            monthly_usd_equivalent: Money::de_unidades(100.0),
            monthly_tokens: None,
            alert_at_percent: vec![],
        };
        let status = calcular_status("xpto", Some(&budget), Money::de_unidades(100.0), 0);
        assert!(status.limite_excedido);
    }

    #[test]
    fn limite_excedido_por_tokens() {
        let budget = BudgetCliente {
            monthly_usd_equivalent: None,
            monthly_tokens: Some(1000),
            alert_at_percent: vec![],
        };
        let status = calcular_status("xpto", Some(&budget), None, 1000);
        assert!(status.limite_excedido);
    }

    #[test]
    fn dentro_do_limite_nao_excede() {
        let budget = BudgetCliente {
            monthly_usd_equivalent: Money::de_unidades(100.0),
            monthly_tokens: Some(1_000_000),
            alert_at_percent: vec![],
        };
        let status = calcular_status("xpto", Some(&budget), Money::de_unidades(50.0), 500_000);
        assert!(!status.limite_excedido);
    }
}
