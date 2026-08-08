//! Comandos da CLI (grupo 7). `brian import`, `brian attribute`, `brian costs`.
//!
//! Lógica de agregação e formatação fica em funções puras, separadas da
//! parte fina que só conecta ao `Store` — testável sem banco (mesmo padrão
//! usado nos adapters).

use crate::domain::{CostSource, Money, UsageRecord};
use crate::importacao::{ColetorDeUso, importar};
use crate::storage::{Periodo, Store};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "brian", about = "Plano de controle de engenharia de IA")]
pub struct Cli {
    #[command(subcommand)]
    pub comando: Comando,
}

#[derive(Subcommand)]
pub enum Comando {
    /// Importa consumo dos providers configurados.
    Import {
        /// Início do recorte, ISO-8601 (`2026-08-01T00:00:00Z`). Padrão: início dos tempos.
        #[arg(long)]
        desde: Option<String>,
        /// Fim do recorte, ISO-8601. Padrão: sem fim (até agora).
        #[arg(long)]
        ate: Option<String>,
    },
    /// Atribui um registro não-atribuído a um cliente.
    Attribute {
        usage_record_id: String,
        #[arg(long)]
        client: String,
    },
    /// Consulta custo. Exige ao menos um filtro/visão.
    Costs {
        #[arg(long)]
        client: Option<String>,
        /// Recorte `AAAA-MM`. Padrão: sem recorte (todo o histórico).
        #[arg(long)]
        period: Option<String>,
        #[arg(long, value_enum)]
        by: Option<AgrupamentoPor>,
        #[arg(long)]
        unattributed: bool,
        #[arg(long)]
        export: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum AgrupamentoPor {
    Provider,
    Model,
}

/// Formata um instante ISO-8601 UTC a partir de segundos desde a época, sem
/// dependência nova — só a direção inversa do parser em `adapters::tempo`.
fn formatar_instante(i: crate::domain::Instante) -> String {
    let secs = i.0;
    let dias = secs.div_euclid(86_400);
    let resto = secs.rem_euclid(86_400);
    let (h, m, s) = (resto / 3600, (resto % 3600) / 60, resto % 60);
    let (y, mo, d) = data_civil_de_dias(dias);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Inverso de `civil_from_days`: dias desde a época para (ano, mês, dia).
/// Mesma família de algoritmo de Howard Hinnant usada em `adapters::tempo`.
fn data_civil_de_dias(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `"AAAA-MM"` → `[início do mês, início do mês seguinte)`.
pub fn periodo_do_mes(spec: &str) -> Option<Periodo> {
    let (ano, mes) = spec.split_once('-')?;
    let ano: i64 = ano.parse().ok()?;
    let mes: i64 = mes.parse().ok()?;
    if !(1..=12).contains(&mes) {
        return None;
    }
    let inicio = dias_desde_epoca_civil(ano, mes, 1) * 86_400;
    let (prox_ano, prox_mes) = if mes == 12 {
        (ano + 1, 1)
    } else {
        (ano, mes + 1)
    };
    let fim = dias_desde_epoca_civil(prox_ano, prox_mes, 1) * 86_400;
    Some(Periodo {
        desde: crate::domain::Instante(inicio),
        ate: Some(crate::domain::Instante(fim)),
    })
}

fn dias_desde_epoca_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn periodo_aberto() -> Periodo {
    Periodo {
        desde: crate::domain::Instante(i64::MIN),
        ate: None,
    }
}

fn fmt_opt_money(m: Option<Money>) -> String {
    match m {
        Some(v) => v.to_string(),
        None => "—".to_string(),
    }
}

/// Soma pago e equivalente separadamente, mais a contagem/soma de tokens
/// cujo custo é desconhecido — a "receita não faturável" (task 3.8).
///
/// `pago`/`equivalente` são `Option<i64>`, não `i64`: um cliente 100% em
/// assinatura tem `pago = None` (inexistente), não `Some(0)` (zero real).
/// Confundir os dois é exatamente o que o spec proíbe (cost-attribution,
/// "Cliente atendido inteiramente por assinatura").
#[derive(Debug, Default)]
pub struct Agregado {
    pub pago: Option<i64>,
    pub equivalente: Option<i64>,
    pub registros_custo_desconhecido: usize,
}

pub fn agregar(registros: &[UsageRecord]) -> Agregado {
    let mut a = Agregado::default();
    for r in registros {
        if let Some(p) = r.custo.pago {
            a.pago = Some(a.pago.unwrap_or(0) + p.0);
        }
        if let Some(e) = r.custo.equivalente_api {
            a.equivalente = Some(a.equivalente.unwrap_or(0) + e.0);
        }
        if r.cost_source == CostSource::Unknown {
            a.registros_custo_desconhecido += 1;
        }
    }
    a
}

/// Linha de exibição de `costs --client`. Distingue registro-vazio de
/// cliente-inexistente no próprio tipo, não em prosa de erro genérica.
pub enum ResultadoCostsClient {
    ClienteInexistente,
    Ok {
        pago: Option<Money>,
        equivalente: Option<Money>,
        registros: usize,
    },
}

pub fn montar_resultado_client(existe: bool, registros: &[UsageRecord]) -> ResultadoCostsClient {
    if !existe {
        return ResultadoCostsClient::ClienteInexistente;
    }
    let a = agregar(registros);
    ResultadoCostsClient::Ok {
        pago: a.pago.map(Money),
        equivalente: a.equivalente.map(Money),
        registros: registros.len(),
    }
}

pub fn formatar_client(resultado: &ResultadoCostsClient, client_id: &str) -> String {
    match resultado {
        ResultadoCostsClient::ClienteInexistente => {
            format!("erro: cliente '{client_id}' não existe")
        }
        ResultadoCostsClient::Ok {
            pago,
            equivalente,
            registros,
        } => {
            format!(
                "cliente: {client_id}\nregistros: {registros}\npago: {}\nequivalente: {}",
                fmt_opt_money(*pago),
                fmt_opt_money(*equivalente)
            )
        }
    }
}

/// Agrupa por uma chave (`provider_id` ou `model`) preservando ordem de
/// primeira aparição, com soma coerente com o total (task 7.4/7.5).
pub fn agrupar_por<'a>(
    registros: &'a [UsageRecord],
    chave: impl Fn(&'a UsageRecord) -> &'a str,
) -> Vec<(String, Agregado)> {
    let mut ordem: Vec<String> = Vec::new();
    let mut grupos: std::collections::HashMap<String, Vec<&UsageRecord>> =
        std::collections::HashMap::new();

    for r in registros {
        let k = chave(r).to_string();
        if !grupos.contains_key(&k) {
            ordem.push(k.clone());
        }
        grupos.entry(k).or_default().push(r);
    }

    ordem
        .into_iter()
        .map(|k| {
            let recs: Vec<UsageRecord> = grupos[&k].iter().map(|r| (*r).clone()).collect();
            (k, agregar(&recs))
        })
        .collect()
}

pub fn formatar_agrupado(grupos: &[(String, Agregado)], rotulo_coluna: &str) -> String {
    let mut linhas = vec![format!("{rotulo_coluna}\tpago\tequivalente\tregistros")];
    let (mut total_pago, mut total_equiv): (Option<i64>, Option<i64>) = (None, None);
    for (chave, a) in grupos {
        linhas.push(format!(
            "{chave}\t{}\t{}\t{}",
            fmt_opt_money(a.pago.map(Money)),
            fmt_opt_money(a.equivalente.map(Money)),
            a.registros_custo_desconhecido
        ));
        if let Some(p) = a.pago {
            total_pago = Some(total_pago.unwrap_or(0) + p);
        }
        if let Some(e) = a.equivalente {
            total_equiv = Some(total_equiv.unwrap_or(0) + e);
        }
    }
    linhas.push(format!(
        "total\t{}\t{}",
        fmt_opt_money(total_pago.map(Money)),
        fmt_opt_money(total_equiv.map(Money))
    ));
    linhas.join("\n")
}

pub fn formatar_unattributed(registros: &[UsageRecord]) -> String {
    if registros.is_empty() {
        return "nenhum consumo não-atribuído".to_string();
    }
    let mut linhas = vec!["provider\tmodel\ttokens\tcusto\tinstante".to_string()];
    for r in registros {
        let tokens = r.tokens.total_conhecido();
        let custo = fmt_opt_money(r.custo.pago.or(r.custo.equivalente_api));
        linhas.push(format!(
            "{}\t{}\t{}\t{}\t{}",
            r.provider_id,
            r.model,
            tokens,
            custo,
            formatar_instante(r.occurred_at)
        ));
    }
    linhas.join("\n")
}

/// Exportação tabular (task 7.7): colunas separadas para pago e
/// equivalente, mais as três procedências. Formato CSV mínimo — sem
/// dependência nova, os campos não têm vírgula nem quebra de linha.
pub fn formatar_export_csv(registros: &[UsageRecord]) -> String {
    let mut linhas = vec![
        "client_id,provider_id,model,billing_mode,usage_source,cost_source,\
         tokens_total,custo_pago,custo_equivalente,occurred_at"
            .to_string(),
    ];
    for r in registros {
        linhas.push(format!(
            "{},{},{},{:?},{:?},{:?},{},{},{},{}",
            r.client_id.as_deref().unwrap_or(""),
            r.provider_id,
            r.model,
            r.billing_mode,
            r.usage_source,
            r.cost_source,
            r.tokens.total_conhecido(),
            fmt_opt_money(r.custo.pago),
            fmt_opt_money(r.custo.equivalente_api),
            formatar_instante(r.occurred_at),
        ));
    }
    linhas.join("\n")
}

// --- Conexão fina com o Store -------------------------------------------

pub fn executar_import(
    store: &dyn Store,
    coletores: &[Box<dyn ColetorDeUso>],
    desde: Option<String>,
    ate: Option<String>,
) -> Result<String, String> {
    let periodo = Periodo {
        desde: desde
            .and_then(|s| crate::adapters::tempo::parse_timestamp_iso8601(&s))
            .map(crate::domain::Instante)
            .unwrap_or(crate::domain::Instante(i64::MIN)),
        ate: ate
            .and_then(|s| crate::adapters::tempo::parse_timestamp_iso8601(&s))
            .map(crate::domain::Instante),
    };

    let resultados = importar(store, coletores, periodo).map_err(|e| e.to_string())?;
    let mut linhas = vec!["provider\tgravados\terro".to_string()];
    for r in &resultados {
        linhas.push(format!(
            "{}\t{}\t{}",
            r.provider_id,
            r.gravados,
            r.erro.as_deref().unwrap_or("—")
        ));
    }
    Ok(linhas.join("\n"))
}

pub fn executar_attribute(
    store: &dyn Store,
    usage_record_id: &str,
    client_id: &str,
) -> Result<String, String> {
    store
        .atribuir(usage_record_id, client_id)
        .map(|r| format!("atribuído: {} → {}", r.id, client_id))
        .map_err(|e| e.to_string())
}

pub fn executar_costs(
    store: &dyn Store,
    client: Option<String>,
    period: Option<String>,
    by: Option<AgrupamentoPor>,
    unattributed: bool,
    export: Option<PathBuf>,
) -> Result<String, String> {
    let periodo = match &period {
        Some(p) => periodo_do_mes(p).ok_or_else(|| format!("período inválido: {p}"))?,
        None => periodo_aberto(),
    };

    if unattributed {
        let registros = store.nao_atribuidos(periodo).map_err(|e| e.to_string())?;
        return Ok(formatar_unattributed(&registros));
    }

    if let Some(client_id) = &client {
        let existe = store.client_exists(client_id).map_err(|e| e.to_string())?;
        let registros = if existe {
            store
                .consumo_do_cliente(client_id, periodo)
                .map_err(|e| e.to_string())?
        } else {
            Vec::new()
        };
        let resultado = montar_resultado_client(existe, &registros);
        return Ok(formatar_client(&resultado, client_id));
    }

    if let Some(agrupamento) = by {
        let registros = store
            .consumo_no_periodo(periodo)
            .map_err(|e| e.to_string())?;
        let (grupos, rotulo) = match agrupamento {
            AgrupamentoPor::Provider => (
                agrupar_por(&registros, |r| r.provider_id.as_str()),
                "provider",
            ),
            AgrupamentoPor::Model => (agrupar_por(&registros, |r| r.model.as_str()), "model"),
        };
        return Ok(formatar_agrupado(&grupos, rotulo));
    }

    if let Some(caminho) = export {
        let registros = match &client {
            Some(c) => store
                .consumo_do_cliente(c, periodo)
                .map_err(|e| e.to_string())?,
            None => store
                .consumo_no_periodo(periodo)
                .map_err(|e| e.to_string())?,
        };
        let csv = formatar_export_csv(&registros);
        std::fs::write(&caminho, &csv).map_err(|e| e.to_string())?;
        return Ok(format!(
            "exportado: {} ({} linhas)",
            caminho.display(),
            registros.len()
        ));
    }

    Err("informe --client, --by, --unattributed ou --export".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;

    fn registro(
        provider: &str,
        model: &str,
        pago: Option<i64>,
        equiv: Option<i64>,
        cost_source: CostSource,
        client: Option<&str>,
    ) -> UsageRecord {
        UsageRecord {
            id: format!("{provider}-{model}"),
            provider_id: provider.into(),
            model: model.into(),
            tokens: Tokens {
                input: Some(100),
                cache: None,
                output: Some(50),
                reasoning: None,
            },
            custo: Custo {
                pago: pago.map(Money),
                equivalente_api: equiv.map(Money),
            },
            billing_mode: BillingMode::Api,
            usage_source: UsageSource::Provider,
            cost_source,
            client_id: client.map(String::from),
            attribution_status: if client.is_some() {
                AttributionStatus::Attributed
            } else {
                AttributionStatus::Unattributed
            },
            occurred_at: Instante(1_700_000_000),
        }
    }

    #[test]
    fn periodo_do_mes_calcula_limites_corretos() {
        let p = periodo_do_mes("2026-08").unwrap();
        assert_eq!(formatar_instante(p.desde), "2026-08-01T00:00:00Z");
        assert_eq!(formatar_instante(p.ate.unwrap()), "2026-09-01T00:00:00Z");
    }

    #[test]
    fn periodo_do_mes_dezembro_vira_ano() {
        let p = periodo_do_mes("2026-12").unwrap();
        assert_eq!(formatar_instante(p.ate.unwrap()), "2027-01-01T00:00:00Z");
    }

    #[test]
    fn formatar_instante_e_inverso_do_parser_de_adapters() {
        let t = crate::adapters::tempo::parse_timestamp_iso8601("2026-08-08T13:18:29Z").unwrap();
        assert_eq!(formatar_instante(Instante(t)), "2026-08-08T13:18:29Z");
    }

    #[test]
    fn cliente_inexistente_e_distinto_de_sem_consumo() {
        let inexistente = montar_resultado_client(false, &[]);
        assert!(matches!(
            inexistente,
            ResultadoCostsClient::ClienteInexistente
        ));

        let sem_consumo = montar_resultado_client(true, &[]);
        match sem_consumo {
            ResultadoCostsClient::Ok { registros, .. } => assert_eq!(registros, 0),
            _ => panic!("cliente existente sem consumo não é ClienteInexistente"),
        }
    }

    #[test]
    fn agregar_soma_pago_e_equivalente_separadamente() {
        let regs = vec![
            registro(
                "claude",
                "opus",
                Some(1_000_000),
                Some(1_500_000),
                CostSource::Provider,
                Some("xpto"),
            ),
            registro(
                "codex",
                "gpt",
                None,
                Some(500_000),
                CostSource::Catalog,
                Some("xpto"),
            ),
        ];
        let a = agregar(&regs);
        assert_eq!(
            a.pago,
            Some(1_000_000),
            "só o registro com pago conhecido soma"
        );
        assert_eq!(a.equivalente, Some(2_000_000));
    }

    #[test]
    fn agregar_conta_custo_desconhecido_separadamente() {
        let regs = vec![registro(
            "x",
            "y",
            None,
            None,
            CostSource::Unknown,
            Some("c"),
        )];
        let a = agregar(&regs);
        assert_eq!(a.registros_custo_desconhecido, 1);
        assert_eq!(
            a.pago, None,
            "nenhum registro com pago -- inexistente, não zero"
        );
        assert_eq!(a.equivalente, None);
    }

    #[test]
    fn cliente_100_por_cento_assinatura_pago_e_inexistente_nao_zero() {
        // Spec cost-attribution, "Cliente atendido inteiramente por
        // assinatura": custo pago aparece como inexistente, não como zero.
        // Bug real encontrado na auditoria do grupo 8: Agregado.pago era
        // i64 puro, então "nenhum registro pago" e "somou zero" ficavam
        // indistinguíveis -- mostrava "0.00" em vez de "—".
        let regs = vec![registro(
            "codex",
            "gpt",
            None,
            Some(500_000),
            CostSource::Catalog,
            Some("xpto"),
        )];
        let resultado = montar_resultado_client(true, &regs);
        let texto = formatar_client(&resultado, "xpto");
        assert!(
            texto.contains("pago: —"),
            "cliente só com consumo por assinatura deve mostrar pago inexistente, não '0.00'. Saída: {texto}"
        );
    }

    #[test]
    fn grupo_100_por_cento_assinatura_pago_e_inexistente_nao_zero() {
        // Mesmo bug, caminho de agrupamento (--by provider/model).
        let regs = vec![registro(
            "codex",
            "gpt",
            None,
            Some(500_000),
            CostSource::Catalog,
            Some("xpto"),
        )];
        let grupos = agrupar_por(&regs, |r| r.provider_id.as_str());
        let saida = formatar_agrupado(&grupos, "provider");
        assert!(
            !saida.contains("0.00"),
            "nenhuma linha deve mostrar '0.00' quando não há pago real. Saída: {saida}"
        );
    }

    #[test]
    fn agrupar_por_provider_soma_bate_com_total() {
        let regs = vec![
            registro(
                "claude",
                "opus",
                Some(1_000_000),
                Some(1_500_000),
                CostSource::Provider,
                Some("a"),
            ),
            registro(
                "claude",
                "sonnet",
                Some(500_000),
                Some(600_000),
                CostSource::Provider,
                Some("a"),
            ),
            registro(
                "codex",
                "gpt",
                Some(300_000),
                Some(300_000),
                CostSource::Provider,
                Some("a"),
            ),
        ];
        let grupos = agrupar_por(&regs, |r| r.provider_id.as_str());
        assert_eq!(grupos.len(), 2);

        let total_grupos: i64 = grupos.iter().filter_map(|(_, a)| a.pago).sum();
        let total_direto: i64 = agregar(&regs).pago.unwrap_or(0);
        assert_eq!(
            total_grupos, total_direto,
            "soma dos grupos bate com o total"
        );
    }

    #[test]
    fn export_csv_tem_colunas_separadas_para_pago_e_equivalente() {
        let regs = vec![registro(
            "claude",
            "opus",
            Some(1_000_000),
            Some(1_500_000),
            CostSource::Provider,
            Some("xpto"),
        )];
        let csv = formatar_export_csv(&regs);
        let cabecalho: Vec<&str> = csv.lines().next().unwrap().split(',').collect();
        assert!(cabecalho.contains(&"custo_pago"));
        assert!(cabecalho.contains(&"custo_equivalente"));

        let campos: Vec<&str> = csv.lines().nth(1).unwrap().split(',').collect();
        let idx_pago = cabecalho.iter().position(|c| *c == "custo_pago").unwrap();
        let idx_equiv = cabecalho
            .iter()
            .position(|c| *c == "custo_equivalente")
            .unwrap();
        assert_eq!(campos[idx_pago], Money(1_000_000).to_string());
        assert_eq!(campos[idx_equiv], Money(1_500_000).to_string());
        assert_ne!(
            campos[idx_pago], campos[idx_equiv],
            "pago e equivalente devem permanecer distintos, não colapsados"
        );
    }

    #[test]
    fn export_csv_assinatura_tem_pago_vazio_nao_zero() {
        let regs = vec![registro(
            "codex",
            "gpt",
            None,
            Some(50),
            CostSource::Catalog,
            Some("xpto"),
        )];
        let csv = formatar_export_csv(&regs);
        let cabecalho: Vec<&str> = csv.lines().next().unwrap().split(',').collect();
        let idx_pago = cabecalho.iter().position(|c| *c == "custo_pago").unwrap();
        let campos: Vec<&str> = csv.lines().nth(1).unwrap().split(',').collect();
        assert_eq!(
            campos[idx_pago], "—",
            "pago inexistente aparece como travessão, não zero"
        );
    }

    #[test]
    fn unattributed_vazio_tem_mensagem_clara() {
        assert_eq!(formatar_unattributed(&[]), "nenhum consumo não-atribuído");
    }

    #[test]
    fn agrupar_por_model_compara_providers_distintos_numa_base_comum() {
        // Spec cost-attribution, "Desdobramento por modelo": permite
        // comparar preço por token entre modelos de providers distintos.
        let regs = vec![
            registro(
                "claude",
                "opus",
                Some(1_000_000),
                Some(1_500_000),
                CostSource::Provider,
                Some("a"),
            ),
            registro(
                "grok",
                "grok-4.5",
                Some(200_000),
                Some(200_000),
                CostSource::Provider,
                Some("a"),
            ),
        ];
        let grupos = agrupar_por(&regs, |r| r.model.as_str());
        assert_eq!(
            grupos.len(),
            2,
            "cada modelo é um grupo, mesmo de providers diferentes"
        );
        assert!(grupos.iter().any(|(m, _)| m == "opus"));
        assert!(grupos.iter().any(|(m, _)| m == "grok-4.5"));
    }

    #[test]
    fn export_csv_custo_desconhecido_nao_aparece_como_zero() {
        // Spec usage-ledger, "Exportação com custo desconhecido": aparece
        // marcado como desconhecido, nunca como zero.
        let regs = vec![registro(
            "x",
            "y",
            None,
            None,
            CostSource::Unknown,
            Some("xpto"),
        )];
        let csv = formatar_export_csv(&regs);
        let cabecalho: Vec<&str> = csv.lines().next().unwrap().split(',').collect();
        let idx_pago = cabecalho.iter().position(|c| *c == "custo_pago").unwrap();
        let idx_equiv = cabecalho
            .iter()
            .position(|c| *c == "custo_equivalente")
            .unwrap();
        let campos: Vec<&str> = csv.lines().nth(1).unwrap().split(',').collect();
        assert_eq!(campos[idx_pago], "—", "pago desconhecido não é zero");
        assert_eq!(
            campos[idx_equiv], "—",
            "equivalente desconhecido não é zero"
        );
    }
}
