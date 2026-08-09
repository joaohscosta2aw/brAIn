//! Cálculo de janela de capacidade e rateio de custo de plano (grupos 6-7).
//!
//! Funções puras: recebem o que já foi lido do ledger/plano/sinal de quota,
//! nunca tocam storage nem processo externo diretamente. `ColetorDeCapacidade`
//! e `importar_capacidade` são o espelho, no eixo de plano/quota, de
//! `ColetorDeUso`/`importar` (`src/importacao.rs`).

use crate::domain::{
    FonteCapacidade, Instante, JanelaDeCapacidade, PlanoDetectado, SinalDeQuotaColetado, TipoJanela,
};
use crate::importacao::ErroColeta;
use crate::storage::{
    NovoPlano, NovoQuotaSignal, QuotaSignalRegistrado, Result as StorageResult, Store,
};

/// O que um provider com fonte própria de plano/quota implementa.
pub trait ColetorDeCapacidade {
    fn provider_id(&self) -> &str;
    /// Consulta a fonte do provider. Devolve o plano detectado e os sinais
    /// de quota observados (pode ser vazio — nem toda fonte dá percentual,
    /// ex.: Claude só dá o plano).
    fn consultar(&self) -> Result<(PlanoDetectado, Vec<SinalDeQuotaColetado>), ErroColeta>;
}

/// Resultado da consulta de capacidade de um provider — mesmo formato de
/// `ResultadoProvider` em `importacao.rs`, isolado por provider (task
/// 3.2/4.5/5.4: falha de um não impede os demais).
#[derive(Debug)]
pub struct ResultadoCapacidadeProvider {
    pub provider_id: String,
    pub erro: Option<String>,
}

/// Consulta e grava plano + sinais de quota de cada provider com fonte.
pub fn importar_capacidade(
    store: &dyn Store,
    coletores: &[Box<dyn ColetorDeCapacidade>],
    agora: Instante,
) -> StorageResult<Vec<ResultadoCapacidadeProvider>> {
    let mut resultados = Vec::with_capacity(coletores.len());

    for coletor in coletores {
        match coletor.consultar() {
            Ok((plano, sinais)) => {
                store.registrar_plano(NovoPlano {
                    provider_id: coletor.provider_id().to_string(),
                    billing_mode: plano.billing_mode,
                    plan_label: plano.plan_label,
                    detectado_em: agora,
                })?;
                for s in sinais {
                    store.upsert_quota_signal(NovoQuotaSignal {
                        provider_id: coletor.provider_id().to_string(),
                        bucket_id: s.bucket_id,
                        grupo: s.grupo,
                        remaining_percent: s.remaining_percent,
                        reset_at: s.reset_at,
                        observed_at: agora,
                    })?;
                }
                resultados.push(ResultadoCapacidadeProvider {
                    provider_id: coletor.provider_id().to_string(),
                    erro: None,
                });
            }
            Err(e) => resultados.push(ResultadoCapacidadeProvider {
                provider_id: coletor.provider_id().to_string(),
                erro: Some(e.motivo),
            }),
        }
    }

    Ok(resultados)
}

/// Monta a janela de capacidade de um bucket a partir do que já foi
/// coletado: consumo medido pelo ledger, sinal de quota mais recente (se
/// houver) e burn já calculado.
///
/// Sem sinal de quota, `used_percent`/`remaining_percent`/`resets_at` ficam
/// ausentes — nunca zero nem inventados (spec capacity-windows, "Janela sem
/// capacidade conhecida").
pub fn montar_janela(
    provider_id: &str,
    tipo: TipoJanela,
    consumido_tokens: Option<u64>,
    sinal: Option<&QuotaSignalRegistrado>,
    burn_tokens_por_hora: Option<f64>,
    agora: Instante,
) -> JanelaDeCapacidade {
    let (bucket_id, used_percent, remaining_percent, resets_at, fonte) = match sinal {
        Some(s) => (
            s.bucket_id.clone(),
            Some(100.0 - s.remaining_percent),
            Some(s.remaining_percent),
            s.reset_at,
            FonteCapacidade::Provider,
        ),
        None => (
            provider_id.to_string(),
            None,
            None,
            None,
            FonteCapacidade::BrianMeasured,
        ),
    };

    // Nenhuma fonte desta change dá capacidade em tokens absolutos — só
    // percentual. Campo existe para o dia em que uma fonte passar a dar.
    let capacidade_tokens: Option<u64> = None;
    let eta_esgotamento = projetar_esgotamento(
        capacidade_tokens,
        consumido_tokens,
        burn_tokens_por_hora,
        agora,
    );

    JanelaDeCapacidade {
        provider_id: provider_id.to_string(),
        bucket_id,
        tipo,
        consumido_tokens,
        capacidade_tokens,
        used_percent,
        remaining_percent,
        resets_at,
        burn_tokens_por_hora,
        eta_esgotamento,
        fonte,
    }
}

/// Taxa de consumo recente em tokens/hora, a partir de um total já somado
/// pelo chamador sobre uma janela recente do ledger (ex.: últimas 24h).
pub fn calcular_burn(tokens_no_periodo: u64, duracao_segundos: i64) -> Option<f64> {
    if duracao_segundos <= 0 {
        return None;
    }
    Some(tokens_no_periodo as f64 / (duracao_segundos as f64 / 3600.0))
}

/// Projeta o instante de esgotamento — só quando capacidade (em tokens) e
/// burn são ambos conhecidos e o burn é positivo (spec: "capacidade
/// desconhecida não produz projeção inventada"). Projeção linear simples,
/// não é ML (design.md/BRIAN-BLUEPRINT-V1.md §13.6).
fn projetar_esgotamento(
    capacidade_tokens: Option<u64>,
    consumido_tokens: Option<u64>,
    burn_tokens_por_hora: Option<f64>,
    agora: Instante,
) -> Option<Instante> {
    let capacidade = capacidade_tokens?;
    let consumido = consumido_tokens?;
    let burn = burn_tokens_por_hora?;
    if burn <= 0.0 {
        return None;
    }
    let restante = capacidade.saturating_sub(consumido) as f64;
    let horas = restante / burn;
    Some(Instante(agora.0 + (horas * 3600.0) as i64))
}

/// Rateio de custo de plano entre clientes, proporcional ao consumo de cada
/// um no mês corrente do plano. `unattributed` fica de fora do denominador e
/// é reportado à parte (spec plan-cost-allocation: "Consumo não atribuído
/// não é rateado entre clientes").
#[derive(Debug, Clone, PartialEq)]
pub struct RateioPlano {
    /// Fração (0.0..=1.0) do consumo atribuído total que cada cliente
    /// representa. Nunca dólar — nenhuma fonte desta change (Claude, Codex,
    /// Gemini) expõe o preço do plano, e este projeto não introduz
    /// declaração manual de custo (design.md, decisão do autor).
    pub por_cliente: Vec<(String, f64)>,
    pub tokens_nao_atribuidos: u64,
}

/// O chamador decide se o rateio se aplica antes de invocar, verificando
/// `billing_mode` do plano vigente (spec: "Rateio se aplica apenas a planos
/// de assinatura") — esta função sempre calcula a fração, a recusa é do
/// caminho de chamada, não desta função pura.
pub fn calcular_rateio(
    tokens_por_cliente: &[(String, u64)],
    tokens_nao_atribuidos: u64,
) -> RateioPlano {
    let total: u64 = tokens_por_cliente.iter().map(|(_, t)| *t).sum();

    let por_cliente = if total == 0 {
        Vec::new()
    } else {
        tokens_por_cliente
            .iter()
            .map(|(cliente, tokens)| (cliente.clone(), *tokens as f64 / total as f64))
            .collect()
    };

    RateioPlano {
        por_cliente,
        tokens_nao_atribuidos,
    }
}

/// Providers com fonte própria de plano/quota verificada nesta change —
/// exatamente os que têm um `ColetorDeCapacidade` real (`main::coletores_capacidade`).
pub const PROVIDERS_VERIFICADOS: &[&str] = &["claude", "codex", "gemini"];

/// Um provider sem fonte de plano/quota, com o motivo investigado — mesmo
/// padrão de `StatusCobertura::SemFonteUtilizavel` da change anterior
/// (`adapters::mod::cobertura_v0_0`), aqui restrito ao eixo de capacidade.
/// Nunca desaparece em silêncio das superfícies de capacidade (spec
/// plan-catalog: "Provider sem fonte de plano/quota é excluído e
/// documentado").
pub struct ProviderSemFonte {
    pub provider_id: &'static str,
    pub motivo: &'static str,
}

/// Investigado em três lugares por provider antes de excluir (design.md):
/// `--help`, código-fonte do projeto quando aberto, documentação oficial.
pub fn providers_sem_fonte() -> Vec<ProviderSemFonte> {
    vec![
        ProviderSemFonte {
            provider_id: "grok",
            motivo: "Lista completa de subcomandos em xai-org/grok-build \
                     (crates/codegen/xai-grok-pager/src/app/cli.rs), --help e guia \
                     oficial de autenticação -- nenhum comando de conta/plano/cota \
                     existe. `inspect` é config de diretório, `doctor` é ambiente, \
                     `sessions` é histórico local.",
        },
        ProviderSemFonte {
            provider_id: "github-copilot",
            motivo: "`/usage` existe mas é interativo e por sessão, não por conta -- \
                     não scriptável. Confirmado nos tópicos de ajuda oficiais \
                     `billing`/`limits` e no --help: nenhum subcomando headless de \
                     conta/plano.",
        },
        ProviderSemFonte {
            provider_id: "qwen-deepseek",
            motivo: "Mesma fonte local que os demais backends qwen -- ver qwen-zai.",
        },
        ProviderSemFonte {
            provider_id: "qwen-zai",
            motivo: "--help e documentação oficial (qwenlm.github.io/qwen-code-docs): \
                     só /auth, /model, /doctor interativos. Nenhum endpoint de \
                     consulta de plano/quota.",
        },
        ProviderSemFonte {
            provider_id: "qwen-kimi",
            motivo: "Mesma fonte local que os demais backends qwen -- ver qwen-zai.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::QuotaSignalRegistrado;

    fn sinal(
        bucket_id: &str,
        remaining_percent: f64,
        reset_at: Option<i64>,
    ) -> QuotaSignalRegistrado {
        QuotaSignalRegistrado {
            provider_id: "codex".into(),
            bucket_id: bucket_id.into(),
            grupo: "rate_limits".into(),
            remaining_percent,
            reset_at: reset_at.map(Instante),
            observed_at: Instante(0),
        }
    }

    #[test]
    fn janela_com_sinal_usa_fonte_provider() {
        let s = sinal("primary", 75.0, Some(1_000));
        let j = montar_janela(
            "codex",
            TipoJanela::Semana,
            Some(500),
            Some(&s),
            None,
            Instante(0),
        );
        assert_eq!(j.fonte, FonteCapacidade::Provider);
        assert_eq!(j.used_percent, Some(25.0));
        assert_eq!(j.remaining_percent, Some(75.0));
        assert_eq!(j.resets_at, Some(Instante(1_000)));
    }

    #[test]
    fn janela_sem_sinal_nao_inventa_percentual() {
        let j = montar_janela(
            "grok",
            TipoJanela::Semana,
            Some(1_000),
            None,
            Some(50.0),
            Instante(0),
        );
        assert_eq!(j.fonte, FonteCapacidade::BrianMeasured);
        assert_eq!(
            j.used_percent, None,
            "sem sinal, percentual é ausente, não zero"
        );
        assert_eq!(j.remaining_percent, None);
        assert_eq!(j.resets_at, None);
        assert_eq!(
            j.consumido_tokens,
            Some(1_000),
            "consumo medido continua presente"
        );
    }

    #[test]
    fn burn_calculado_a_partir_de_tokens_e_duracao() {
        let burn = calcular_burn(2_400, 24 * 3600).unwrap();
        assert_eq!(burn, 100.0, "2400 tokens em 24h = 100 tokens/hora");
    }

    #[test]
    fn burn_com_duracao_zero_e_none() {
        assert_eq!(calcular_burn(100, 0), None);
    }

    #[test]
    fn projecao_com_capacidade_e_burn_conhecidos() {
        let eta = projetar_esgotamento(Some(1_000), Some(500), Some(100.0), Instante(0));
        // Restam 500 tokens a 100/hora = 5h = 18000s.
        assert_eq!(eta, Some(Instante(18_000)));
    }

    #[test]
    fn projecao_sem_capacidade_conhecida_e_none() {
        assert_eq!(
            projetar_esgotamento(None, Some(500), Some(100.0), Instante(0)),
            None,
            "capacidade desconhecida não produz projeção inventada"
        );
    }

    #[test]
    fn projecao_sem_burn_conhecido_e_none() {
        assert_eq!(
            projetar_esgotamento(Some(1_000), Some(500), None, Instante(0)),
            None
        );
    }

    #[test]
    fn rateio_dois_clientes_dividem_proporcional() {
        let r = calcular_rateio(&[("xpto".to_string(), 300), ("acme".to_string(), 700)], 0);
        let xpto = r.por_cliente.iter().find(|(c, _)| c == "xpto").unwrap().1;
        let acme = r.por_cliente.iter().find(|(c, _)| c == "acme").unwrap().1;
        assert_eq!(xpto, 0.3);
        assert_eq!(acme, 0.7);
    }

    #[test]
    fn rateio_cliente_unico_consome_tudo() {
        let r = calcular_rateio(&[("xpto".to_string(), 100)], 0);
        assert_eq!(r.por_cliente, vec![("xpto".to_string(), 1.0)]);
    }

    #[test]
    fn rateio_exclui_nao_atribuido_do_denominador() {
        // Spec: consumo não atribuído fica de fora do rateio entre clientes,
        // reportado à parte -- não dilui a fração de quem tem dono.
        let r = calcular_rateio(
            &[("xpto".to_string(), 100)],
            900, // consumo não atribuído, bem maior que o atribuído
        );
        assert_eq!(
            r.por_cliente,
            vec![("xpto".to_string(), 1.0)],
            "xpto é o único no denominador, recebe 100% da fração atribuível"
        );
        assert_eq!(r.tokens_nao_atribuidos, 900);
    }

    #[test]
    fn rateio_sem_consumo_atribuido_nenhum_e_vazio() {
        let r = calcular_rateio(&[], 500);
        assert!(r.por_cliente.is_empty());
        assert_eq!(r.tokens_nao_atribuidos, 500);
    }
}
