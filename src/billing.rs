//! Chargeback por cliente (capacity/chargeback, blueprint §44, D-16):
//! markup configurável aplicado ao custo interno já apurado
//! (`Store::consumo_do_cliente` + `comandos::agregar`) -- Brian calcula,
//! nunca decide preço (política comercial é da organização).

use crate::domain::Money;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub enum ErroBilling {
    Json(String),
    SemMarkupConfigurado(String),
}

impl std::fmt::Display for ErroBilling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(m) => write!(f, "erro lendo billing: {m}"),
            Self::SemMarkupConfigurado(client_id) => write!(
                f,
                "cliente '{client_id}' sem markup configurado em billing/clients.json -- \
                 chargeback recusado (nunca assume markup 1.0 silenciosamente)"
            ),
        }
    }
}

impl std::error::Error for ErroBilling {}

/// Config de billing de um cliente -- `markup` é obrigatório para
/// chargeback existir; `minimum_monthly`/`includes_infrastructure` são
/// opcionais.
#[derive(Debug, Clone)]
pub struct ConfigBillingCliente {
    pub markup: f64,
    pub minimum_monthly: Option<Money>,
    pub includes_infrastructure: bool,
}

/// Carrega `billing/clients.json`. Arquivo ausente devolve mapa vazio
/// (mesmo padrão de `budget::carregar_budgets`), não erro. Erro só se o
/// arquivo existir e não for JSON válido.
pub fn carregar_billing(
    caminho: &Path,
) -> Result<HashMap<String, ConfigBillingCliente>, ErroBilling> {
    let texto = match std::fs::read_to_string(caminho) {
        Ok(t) => t,
        Err(_) => return Ok(HashMap::new()),
    };
    let v: serde_json::Value =
        serde_json::from_str(&texto).map_err(|e| ErroBilling::Json(e.to_string()))?;

    let clientes = v
        .get("clients")
        .and_then(|c| c.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(client_id, cfg)| {
                    let markup = cfg.get("markup").and_then(|x| x.as_f64())?;
                    let minimum_monthly = cfg
                        .get("minimum_monthly_usd")
                        .and_then(|x| x.as_f64())
                        .and_then(Money::de_unidades);
                    let includes_infrastructure = cfg
                        .get("includes_infrastructure")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(false);
                    Some((
                        client_id.clone(),
                        ConfigBillingCliente {
                            markup,
                            minimum_monthly,
                            includes_infrastructure,
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(clientes)
}

/// Relatório de chargeback -- custo interno e valor faturável sempre
/// juntos (spec: "Relatório mostra custo interno e valor faturável lado a
/// lado").
#[derive(Debug, Clone, PartialEq)]
pub struct RelatorioChargeback {
    pub client_id: String,
    pub custo_interno: Option<Money>,
    pub markup: f64,
    pub valor_faturavel: Money,
    pub piso_aplicado: bool,
    pub includes_infrastructure: bool,
}

/// Calcula o chargeback -- `Err` se `config` for `None` (spec: "Chargeback
/// exige markup configurado explicitamente"). Markup aplicado só sobre
/// `custo_interno` (o `equivalente` já apurado, nunca `pago` -- decisão de
/// escopo em design.md).
pub fn calcular_chargeback(
    client_id: &str,
    config: Option<&ConfigBillingCliente>,
    custo_interno: Option<Money>,
) -> Result<RelatorioChargeback, ErroBilling> {
    let config = config.ok_or_else(|| ErroBilling::SemMarkupConfigurado(client_id.to_string()))?;

    let base = custo_interno.unwrap_or(Money::ZERO);
    let marcado_up = Money::de_unidades(base.em_unidades() * config.markup).unwrap_or(Money::ZERO);

    let (valor_faturavel, piso_aplicado) = match config.minimum_monthly {
        Some(piso) if piso > marcado_up => (piso, true),
        _ => (marcado_up, false),
    };

    Ok(RelatorioChargeback {
        client_id: client_id.to_string(),
        custo_interno,
        markup: config.markup,
        valor_faturavel,
        piso_aplicado,
        includes_infrastructure: config.includes_infrastructure,
    })
}

/// Formata o relatório -- custo interno e valor faturável sempre
/// presentes, piso sinalizado quando aplicado.
pub fn formatar_chargeback(r: &RelatorioChargeback) -> String {
    let custo = r
        .custo_interno
        .map(|m| m.to_string())
        .unwrap_or_else(|| "—".to_string());
    let mut saida = format!(
        "cliente: {}\ncusto interno (equivalente): {custo}\nmarkup: {:.2}x\nvalor faturável: {}",
        r.client_id, r.markup, r.valor_faturavel
    );
    if r.piso_aplicado {
        saida.push_str("\npiso mensal aplicado");
    }
    if r.includes_infrastructure {
        saida.push_str("\n(inclui infraestrutura -- rótulo informativo, não medido no cálculo)");
    }
    saida
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carregar_billing_de_arquivo_ausente_e_mapa_vazio() {
        let billing = carregar_billing(Path::new("/caminho/que/nao/existe.json")).unwrap();
        assert!(billing.is_empty());
    }

    #[test]
    fn carregar_billing_com_json_invalido_e_erro_claro() {
        let dir = crate::testutil::dir_temporario_unico("billing-invalido");
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("clients.json");
        std::fs::write(&caminho, "{ nao é json").unwrap();
        let erro = carregar_billing(&caminho).unwrap_err();
        assert!(matches!(erro, ErroBilling::Json(_)));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn carregar_billing_le_todos_os_campos() {
        let dir = crate::testutil::dir_temporario_unico("billing-le");
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("clients.json");
        std::fs::write(
            &caminho,
            r#"{"clients": {"xpto": {"markup": 1.6, "minimum_monthly_usd": 2000, "includes_infrastructure": true}}}"#,
        )
        .unwrap();
        let billing = carregar_billing(&caminho).unwrap();
        let c = billing.get("xpto").unwrap();
        assert_eq!(c.markup, 1.6);
        assert_eq!(c.minimum_monthly, Money::de_unidades(2000.0));
        assert!(c.includes_infrastructure);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn sem_markup_configurado_e_erro_claro() {
        let erro = calcular_chargeback("xpto", None, Money::de_unidades(100.0)).unwrap_err();
        assert!(matches!(erro, ErroBilling::SemMarkupConfigurado(_)));
    }

    #[test]
    fn markup_aplicado_corretamente_sobre_custo_interno() {
        let config = ConfigBillingCliente {
            markup: 1.6,
            minimum_monthly: None,
            includes_infrastructure: false,
        };
        let r = calcular_chargeback("xpto", Some(&config), Money::de_unidades(100.0)).unwrap();
        assert_eq!(r.valor_faturavel, Money::de_unidades(160.0).unwrap());
        assert!(!r.piso_aplicado);
    }

    #[test]
    fn piso_mensal_aplica_quando_custo_vezes_markup_fica_abaixo() {
        let config = ConfigBillingCliente {
            markup: 1.6,
            minimum_monthly: Money::de_unidades(2000.0),
            includes_infrastructure: false,
        };
        let r = calcular_chargeback("xpto", Some(&config), Money::de_unidades(100.0)).unwrap();
        assert_eq!(r.valor_faturavel, Money::de_unidades(2000.0).unwrap());
        assert!(r.piso_aplicado);
    }

    #[test]
    fn sem_piso_configurado_nunca_aciona_piso() {
        let config = ConfigBillingCliente {
            markup: 1.6,
            minimum_monthly: None,
            includes_infrastructure: false,
        };
        let r = calcular_chargeback("xpto", Some(&config), Money::de_unidades(1.0)).unwrap();
        assert!(!r.piso_aplicado);
    }

    #[test]
    fn cliente_sem_consumo_no_periodo_nao_quebra_o_calculo() {
        let config = ConfigBillingCliente {
            markup: 1.6,
            minimum_monthly: Money::de_unidades(2000.0),
            includes_infrastructure: false,
        };
        let r = calcular_chargeback("xpto", Some(&config), None).unwrap();
        assert_eq!(r.custo_interno, None);
        assert_eq!(r.valor_faturavel, Money::de_unidades(2000.0).unwrap());
        assert!(r.piso_aplicado);
    }
}
