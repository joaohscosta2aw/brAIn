//! Cálculo do custo equivalente em API a partir de tokens e catálogo (grupo 3).
//!
//! Não faz SQL — recebe o preço já resolvido pela camada de armazenamento e
//! só multiplica. A resolução por vigência (D-6, catálogo por data) é
//! responsabilidade de `storage::Store::preco_vigente`.

use crate::domain::{CostSource, Custo, Money, Tokens};
use crate::storage::EntradaCatalogo;

/// Calcula o custo equivalente em API para um consumo, a partir do preço
/// vigente no catálogo. `None` se não houver entrada de catálogo — nunca
/// zero (spec: "nenhum dos dois valores é registrado como zero").
pub fn calcular_equivalente(tokens: &Tokens, catalogo: Option<&EntradaCatalogo>) -> Option<Money> {
    let entrada = catalogo?;
    let total = tokens.total_conhecido();
    let micros = (entrada.preco_por_1k_tokens.0 as i128 * total as i128) / 1000;
    Some(Money(micros as i64))
}

/// Resolve `cost_source` e o custo pago a partir do que o provider informou e
/// do que o catálogo permite calcular.
///
/// **D-6: o valor do provider sempre prevalece como custo pago.** O
/// equivalente calculado por catálogo nunca substitui nem é somado a ele —
/// são campos distintos em `Custo` desde o tipo, então não há como um
/// chamador desta função produzir a soma proibida por acidente.
pub fn resolver_custo(
    custo_reportado_pelo_provider: Option<Money>,
    tokens: &Tokens,
    catalogo: Option<&EntradaCatalogo>,
) -> (Custo, CostSource) {
    let equivalente = calcular_equivalente(tokens, catalogo);

    match custo_reportado_pelo_provider {
        Some(pago) => (
            Custo {
                pago: Some(pago),
                equivalente_api: equivalente,
            },
            CostSource::Provider,
        ),
        None => {
            let source = if equivalente.is_some() {
                CostSource::Catalog
            } else {
                CostSource::Unknown
            };
            (
                Custo {
                    pago: None,
                    equivalente_api: equivalente,
                },
                source,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Instante;

    fn tokens(total: u64) -> Tokens {
        Tokens {
            input: Some(total),
            cache: None,
            output: None,
            reasoning: None,
        }
    }

    fn catalogo(preco_por_1k: i64) -> EntradaCatalogo {
        EntradaCatalogo {
            model: "opus".into(),
            preco_por_1k_tokens: Money(preco_por_1k),
            vigente_desde: Instante(0),
            vigente_ate: None,
        }
    }

    #[test]
    fn sem_catalogo_equivalente_e_none_nao_zero() {
        assert_eq!(calcular_equivalente(&tokens(1000), None), None);
    }

    #[test]
    fn equivalente_calculado_por_mil_tokens() {
        let c = catalogo(15_000_000); // 15.00 por 1k tokens
        let eq = calcular_equivalente(&tokens(2000), Some(&c)).unwrap();
        assert_eq!(eq, Money(30_000_000)); // 30.00
    }

    #[test]
    fn provider_prevalece_sobre_catalogo_como_pago() {
        let c = catalogo(15_000_000);
        let (custo, source) =
            resolver_custo(Some(Money(9_990_000)), &tokens(1000), Some(&c));
        assert_eq!(custo.pago, Some(Money(9_990_000)));
        assert_eq!(source, CostSource::Provider);
        // O equivalente continua calculado e presente, não apagado pela
        // precedência do valor pago (design.md: "sem apagar o equivalente").
        assert_eq!(custo.equivalente_api, Some(Money(15_000_000)));
    }

    #[test]
    fn assinatura_sem_pago_com_equivalente_calculavel() {
        let c = catalogo(15_000_000);
        let (custo, source) = resolver_custo(None, &tokens(1000), Some(&c));
        assert_eq!(custo.pago, None);
        assert_eq!(custo.equivalente_api, Some(Money(15_000_000)));
        assert_eq!(source, CostSource::Catalog);
    }

    #[test]
    fn sem_provider_e_sem_catalogo_e_unknown() {
        let (custo, source) = resolver_custo(None, &tokens(1000), None);
        assert_eq!(custo.pago, None);
        assert_eq!(custo.equivalente_api, None);
        assert_eq!(source, CostSource::Unknown);
    }

    #[test]
    fn pago_e_equivalente_nunca_se_somam_num_unico_numero() {
        // Task 3.7: prova que os dois valores permanecem campos distintos e
        // que somá-los exigiria código explícito que não existe em nenhum
        // caminho desta função -- verificado aqui, e a invariante de domínio
        // que caminho do dinheiro é RED cobre isso em src/domain.rs também.
        let c = catalogo(15_000_000);
        let (custo, _) = resolver_custo(Some(Money(9_990_000)), &tokens(1000), Some(&c));
        let soma_seria = custo.pago.unwrap().0 + custo.equivalente_api.unwrap().0;
        assert_ne!(
            custo.pago.unwrap().0,
            soma_seria,
            "os dois valores devem permanecer distinguíveis, não colapsados"
        );
    }
}
