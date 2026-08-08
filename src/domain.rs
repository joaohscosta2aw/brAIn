//! Tipos do domínio.
//!
//! Fixados pelos specs em `openspec/changes/client-cost-attribution/specs/`.
//! Não há SQL aqui — a persistência é problema de `storage`.

// Os consumidores destes tipos chegam no grupo 2 de tasks (ledger). Até lá o
// módulo é legitimamente não-usado. Este allow sai quando a task 2.1 gravar o
// primeiro registro — se ainda estiver aqui depois disso, é sinal de que algo
// nunca encontrou consumidor e deveria ser apagado.
#![allow(dead_code)]

use std::fmt;

/// Origem das contagens de token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageSource {
    /// Reportado pelo próprio provider.
    Provider,
    /// Derivado do que o Brian observou.
    BrianMeasured,
    /// Estimado, sem observação direta.
    Estimated,
}

/// Origem do valor de custo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostSource {
    /// Valor informado pelo provider. Prevalece sobre catálogo (D-6).
    Provider,
    /// Derivado do catálogo de preço vigente.
    Catalog,
    /// Sem custo do provider e sem entrada de catálogo para o modelo.
    Unknown,
}

/// Modo de cobrança do provider naquela chamada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingMode {
    Api,
    Subscription,
    Credits,
    Mixed,
    Unknown,
}

/// Se o consumo tem dono.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributionStatus {
    Attributed,
    /// Alarme, nunca estado normal silencioso.
    Unattributed,
}

/// Contagens de token de uma chamada.
///
/// Cada campo é `Option` porque **ausente e zero são fatos distintos**: um
/// provider que não expõe tokens de reasoning é diferente de uma chamada que
/// não consumiu nenhum. Confundi-los corrompe o ledger em silêncio.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tokens {
    pub input: Option<u64>,
    pub cache: Option<u64>,
    pub output: Option<u64>,
    pub reasoning: Option<u64>,
}

impl Tokens {
    /// Soma das categorias conhecidas. Categorias ausentes não entram — o total
    /// de um registro incompleto é menor, nunca inventado.
    pub fn total_conhecido(&self) -> u64 {
        [self.input, self.cache, self.output, self.reasoning]
            .iter()
            .filter_map(|t| *t)
            .sum()
    }

    /// Se alguma categoria não foi reportada.
    pub fn tem_ausente(&self) -> bool {
        [self.input, self.cache, self.output, self.reasoning]
            .iter()
            .any(|t| t.is_none())
    }
}

/// Valor monetário em micro-unidades da moeda, para evitar ponto flutuante em
/// caminho de dinheiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money(pub i64);

impl Money {
    pub const ZERO: Money = Money(0);

    /// Converte um valor em unidades da moeda.
    ///
    /// Devolve `None` quando o valor não é representável: `NaN`, infinito, ou
    /// fora da faixa de micro-unidades.
    ///
    /// **Não use `as` direto aqui.** O cast de `f64` para `i64` em Rust satura
    /// silenciosamente — `NaN` vira `0` e infinito vira `i64::MAX`. Num caminho
    /// de faturamento isso transforma custo desconhecido em cobrança de zero,
    /// que é exatamente o que o spec proíbe ao separar ausente, zero e
    /// desconhecido.
    pub fn de_unidades(unidades: f64) -> Option<Self> {
        if !unidades.is_finite() {
            return None;
        }
        let micros = (unidades * 1_000_000.0).round();
        if micros < i64::MIN as f64 || micros > i64::MAX as f64 {
            return None;
        }
        Some(Money(micros as i64))
    }

    pub fn em_unidades(&self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.em_unidades())
    }
}

/// Os dois valores monetários de um consumo.
///
/// **Coexistem e servem a propósitos diferentes** (BRIAN-BLUEPRINT-V1.md §42):
/// o pago é base de custo, o equivalente é base de faturamento. Um nunca
/// substitui o outro, e o equivalente jamais é apresentado como valor pago.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Custo {
    /// O que efetivamente se paga por esta chamada. Ausente sob assinatura.
    pub pago: Option<Money>,
    /// O que estes tokens custariam a preço de tabela. Ausente se o modelo não
    /// estiver no catálogo.
    pub equivalente_api: Option<Money>,
}

/// Instante absoluto em UTC, em segundos desde a época.
///
/// Chaves de janela são derivadas na leitura, não pré-calculadas — janelas são
/// objeto da change seguinte e ainda podem mudar de definição.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instante(pub i64);

/// Uma chamada de provider observada. Unidade de verdade do consumo.
#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub id: String,
    pub provider_id: String,
    pub model: String,
    pub tokens: Tokens,
    pub custo: Custo,
    pub billing_mode: BillingMode,
    pub usage_source: UsageSource,
    pub cost_source: CostSource,
    pub client_id: Option<String>,
    pub attribution_status: AttributionStatus,
    pub occurred_at: Instante,
}

impl UsageRecord {
    /// As invariantes de integridade do ledger que este registro pode violar
    /// sozinho. Violação é defeito de release, não débito técnico.
    pub fn violacoes(&self) -> Vec<&'static str> {
        let mut v = Vec::new();

        if self.client_id.is_none() && self.attribution_status == AttributionStatus::Attributed {
            v.push("atribuído sem cliente");
        }
        if self.client_id.is_some() && self.attribution_status == AttributionStatus::Unattributed {
            v.push("marcado não-atribuído mas tem cliente");
        }
        if self.cost_source == CostSource::Provider && self.custo.pago.is_none() {
            v.push("cost_source=provider sem valor pago");
        }
        if self.cost_source == CostSource::Unknown && self.custo.equivalente_api.is_some() {
            v.push("cost_source=unknown mas há equivalente calculado");
        }

        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> UsageRecord {
        UsageRecord {
            id: "r1".into(),
            provider_id: "claude".into(),
            model: "m".into(),
            tokens: Tokens::default(),
            custo: Custo::default(),
            billing_mode: BillingMode::Api,
            usage_source: UsageSource::Provider,
            cost_source: CostSource::Unknown,
            client_id: Some("xpto".into()),
            attribution_status: AttributionStatus::Attributed,
            occurred_at: Instante(0),
        }
    }

    #[test]
    fn ausente_nao_e_zero() {
        let t = Tokens {
            input: Some(10),
            cache: None,
            output: Some(5),
            reasoning: None,
        };
        assert_eq!(t.total_conhecido(), 15);
        assert!(t.tem_ausente(), "categorias ausentes devem ser detectáveis");

        let zerado = Tokens {
            input: Some(0),
            cache: Some(0),
            output: Some(0),
            reasoning: Some(0),
        };
        assert_eq!(zerado.total_conhecido(), 0);
        assert!(!zerado.tem_ausente(), "consumo zero não é ausência de dado");
    }

    #[test]
    fn pago_e_equivalente_coexistem() {
        let c = Custo {
            pago: Some(Money::de_unidades(1.84).unwrap()),
            equivalente_api: Some(Money::de_unidades(4.61).unwrap()),
        };
        assert_ne!(c.pago, c.equivalente_api);
        assert!(c.pago.is_some() && c.equivalente_api.is_some());
    }

    #[test]
    fn assinatura_tem_equivalente_sem_pago() {
        let c = Custo {
            pago: None,
            equivalente_api: Some(Money::de_unidades(4.61).unwrap()),
        };
        assert!(c.pago.is_none(), "assinatura não tem custo por chamada");
        assert!(
            c.equivalente_api.is_some(),
            "os tokens são conhecidos, logo o equivalente é calculável"
        );
    }

    #[test]
    fn valor_nao_representavel_nunca_vira_zero() {
        // Um cast `as` direto devolveria 0 para NaN e i64::MAX para infinito,
        // transformando custo desconhecido em cobranca. Caminho do dinheiro e RED.
        assert_eq!(Money::de_unidades(f64::NAN), None);
        assert_eq!(Money::de_unidades(f64::INFINITY), None);
        assert_eq!(Money::de_unidades(f64::NEG_INFINITY), None);
        assert_eq!(Money::de_unidades(1e300), None);
        assert_eq!(Money::de_unidades(-1e300), None);

        assert_eq!(Money::de_unidades(0.0), Some(Money::ZERO));
        assert_eq!(Money::de_unidades(1.84), Some(Money(1_840_000)));
    }

    #[test]
    fn atribuido_sem_cliente_e_violacao() {
        let mut r = base();
        r.client_id = None;
        assert!(r.violacoes().contains(&"atribuído sem cliente"));
    }

    #[test]
    fn nao_atribuido_com_cliente_e_violacao() {
        let mut r = base();
        r.attribution_status = AttributionStatus::Unattributed;
        assert!(
            r.violacoes()
                .contains(&"marcado não-atribuído mas tem cliente")
        );
    }

    #[test]
    fn registro_integro_nao_tem_violacao() {
        assert!(base().violacoes().is_empty());
    }

    #[test]
    fn unknown_com_equivalente_e_violacao() {
        let mut r = base();
        r.custo.equivalente_api = Some(Money::de_unidades(1.0).unwrap());
        assert!(
            r.violacoes()
                .contains(&"cost_source=unknown mas há equivalente calculado")
        );
    }
}
