//! Comandos da CLI (grupo 7). `brian import`, `brian attribute`, `brian costs`.
//!
//! Lógica de agregação e formatação fica em funções puras, separadas da
//! parte fina que só conecta ao `Store` — testável sem banco (mesmo padrão
//! usado nos adapters).

use crate::capacidade::{self, ColetorDeCapacidade, montar_janela, providers_sem_fonte};
use crate::continuidade;
use crate::domain::{
    BillingMode, CategoriaNota, CostSource, FonteCapacidade, Instante, JanelaDeCapacidade, Money,
    RunRegistrado, TipoJanela, UsageRecord,
};
use crate::execucao;
use crate::identidade;
use crate::importacao::{ColetorDeUso, importar};
use crate::router;
use crate::storage::{NovoPerfil, Periodo, Store};
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
    /// Importa plano e sinais de quota dos providers com fonte própria.
    ImportCapacity,
    /// Consulta capacidade por provider: plano, janela, %, restante, reset, burn.
    Capacity {
        #[arg(long)]
        provider: Option<String>,
    },
    /// Gerencia planos detectados.
    #[command(subcommand)]
    Plans(ComandoPlans),
    /// Ativa cliente/projeto: identidade Git, isolamento de provider,
    /// namespace de memória. Imprime `export` para `eval` no shell.
    Connect {
        /// `<cliente>` ou `<cliente>/<projeto>`.
        alvo: String,
    },
    /// Encerra o contexto ativo. Imprime `unset` para `eval` no shell.
    Disconnect,
    /// Contexto ativo: cliente, projeto, identidade Git e, por provider, a
    /// conta autenticada.
    Whoami,
    /// Gerencia perfis de identidade (cliente/projeto/provider bindings).
    #[command(subcommand)]
    Context(ComandoContext),
    /// Gerencia credenciais do Vault (metadados — nunca o valor).
    #[command(subcommand)]
    Vault(ComandoVault),
    /// Gerencia notas de memória do Context ativo.
    #[command(subcommand)]
    Memory(ComandoMemory),
    /// Mostra o Continuity Pack do Context ativo, sem handoff.
    Continuity,
    /// Monta e apresenta o Continuity Pack para o próximo provider.
    Handoff {
        #[arg(long = "to")]
        provider: String,
    },
    /// Executa uma tarefa num worktree isolado (D-7), rastreando o run.
    Run {
        tarefa: String,
        /// Vence qualquer regra de `routing/rules.json` (blueprint §11.5).
        /// Sem isso, o provider é decidido por regra.
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        model: Option<String>,
        /// Comando shell rodado no worktree após o provider — só conclui o
        /// run se sair com sucesso.
        #[arg(long)]
        gate: Option<String>,
        /// Mostra qual provider seria escolhido e por quê, sem criar
        /// worktree nem invocar provider nenhum.
        #[arg(long)]
        explain_only: bool,
    },
    /// Lista runs órfãos e finaliza (nunca reexecuta) — `--run <id>` ou `--all`.
    Recover {
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Gerencia worktrees de runs.
    #[command(subcommand)]
    Worktree(ComandoWorktree),
    /// Harness de eval (pré-requisito de D-13).
    #[command(subcommand)]
    Eval(ComandoEval),
}

#[derive(Subcommand)]
pub enum ComandoWorktree {
    /// Worktrees de runs ativos ou abandonados, com status do run associado.
    List,
}

#[derive(Subcommand)]
pub enum ComandoEval {
    /// Roda casos de eval (3 tentativas cada) e reporta taxa de sucesso.
    Run {
        #[arg(long)]
        case: Option<String>,
        #[arg(long, default_value = "evals/cases")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ComandoMemory {
    /// Registra uma nota simples.
    Note { texto: String },
    /// Registra uma decisão com o motivo.
    Decide {
        texto: String,
        #[arg(long)]
        why: String,
    },
}

#[derive(Subcommand)]
pub enum ComandoContext {
    /// Lista os perfis de identidade de um cliente.
    List {
        #[arg(long)]
        client: String,
    },
    /// Mostra um perfil por id.
    Show { id: String },
    /// Cria um perfil de identidade para cliente/projeto.
    Init {
        #[arg(long)]
        client: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        git_name: Option<String>,
        #[arg(long)]
        git_email: Option<String>,
        #[arg(long)]
        github_org: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ComandoVault {
    /// Metadados de credenciais registradas — nunca o valor.
    List,
}

#[derive(Subcommand)]
pub enum ComandoPlans {
    /// Lista o plano vigente de cada provider (leitura — detecção é automática).
    List,
    /// Rateio de capacidade do plano entre clientes (showback em fração, sem
    /// custo em dólar — nenhuma fonte desta change expõe o preço do plano).
    Allocation {
        #[arg(long)]
        provider: String,
        /// Recorte `AAAA-MM`. Padrão: mês corrente.
        #[arg(long)]
        period: Option<String>,
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

/// Mês corrente como `Periodo` — padrão de `brian plans allocation` quando
/// `--period` não é informado.
fn periodo_mes_atual() -> Periodo {
    let agora = Instante::agora();
    let dias = agora.0.div_euclid(86_400);
    let (y, mo, _d) = data_civil_de_dias(dias);
    periodo_do_mes(&format!("{y:04}-{mo:02}")).expect("mês corrente é sempre um período válido")
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
    // Colunas separadas para pago e equivalente, como em toda outra visão
    // de custo -- uma coluna "custo" genérica escondendo qual dos dois
    // valores está ali é a mesma ambiguidade que §42.2 proíbe.
    let mut linhas =
        vec!["provider\tmodel\ttokens\tcusto_pago\tcusto_equivalente\tinstante".to_string()];
    for r in registros {
        let tokens = r.tokens.total_conhecido();
        linhas.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            r.provider_id,
            r.model,
            tokens,
            fmt_opt_money(r.custo.pago),
            fmt_opt_money(r.custo.equivalente_api),
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

// --- Capacidade (grupos 8/9) --------------------------------------------

fn fmt_opt_percent(p: Option<f64>) -> String {
    match p {
        Some(v) => format!("{v:.1}%"),
        None => "—".to_string(),
    }
}

/// Soma tokens conhecidos de um provider num recorte de tempo. Filtro por
/// provider em Rust, não em SQL: `Store` não tem consulta escopada por
/// provider+período além da lista completa do período (mesmo caminho que
/// `--by provider` já usa em `executar_costs`).
fn somar_tokens_provider(
    store: &dyn Store,
    provider_id: &str,
    desde: Instante,
    ate: Instante,
) -> Result<u64, String> {
    let registros = store
        .consumo_no_periodo(Periodo {
            desde,
            ate: Some(ate),
        })
        .map_err(|e| e.to_string())?;
    Ok(registros
        .iter()
        .filter(|r| r.provider_id == provider_id)
        .map(|r| r.tokens.total_conhecido())
        .sum())
}

fn formatar_linha_janela(j: &JanelaDeCapacidade, plano_label: &str) -> String {
    let fonte = match j.fonte {
        FonteCapacidade::Provider => "provider",
        FonteCapacidade::BrianMeasured => "brian_measured",
    };
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        j.provider_id,
        j.bucket_id,
        plano_label,
        fonte,
        j.consumido_tokens
            .map(|t| t.to_string())
            .unwrap_or_else(|| "—".to_string()),
        fmt_opt_percent(j.used_percent),
        fmt_opt_percent(j.remaining_percent),
        j.resets_at
            .map(formatar_instante)
            .unwrap_or_else(|| "—".to_string()),
        j.burn_tokens_por_hora
            .map(|b| format!("{b:.1}"))
            .unwrap_or_else(|| "—".to_string()),
        // Sempre "—" nesta change: nenhuma fonte dá capacidade em tokens
        // absolutos, só percentual (spec capacity-windows, requirement
        // "Burn rate e projeção de esgotamento"). Campo existe para o dia em
        // que uma fonte passar a dar.
        j.eta_esgotamento
            .map(formatar_instante)
            .unwrap_or_else(|| "—".to_string()),
    )
}

pub fn executar_import_capacity(
    store: &dyn Store,
    coletores: &[Box<dyn ColetorDeCapacidade>],
) -> Result<String, String> {
    let agora = Instante::agora();
    let resultados =
        capacidade::importar_capacidade(store, coletores, agora).map_err(|e| e.to_string())?;
    let mut linhas = vec!["provider\terro".to_string()];
    for r in &resultados {
        linhas.push(format!(
            "{}\t{}",
            r.provider_id,
            r.erro.as_deref().unwrap_or("—")
        ));
    }
    Ok(linhas.join("\n"))
}

pub fn executar_capacity(store: &dyn Store, provider: Option<String>) -> Result<String, String> {
    let agora = Instante::agora();
    let providers: Vec<String> = match &provider {
        Some(p) => vec![p.clone()],
        None => capacidade::PROVIDERS_VERIFICADOS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };

    let mut linhas = vec![
        "provider\tbucket\tplano\tfonte\tconsumido_tokens\tused_percent\trestante_percent\treset\tburn_tokens_hora\teta_esgotamento"
            .to_string(),
    ];

    for provider_id in &providers {
        let plano = store
            .plano_vigente(provider_id)
            .map_err(|e| e.to_string())?;
        let plano_label = plano
            .as_ref()
            .and_then(|p| p.plan_label.clone())
            .unwrap_or_else(|| "—".to_string());

        let sinais = store
            .quota_signals(provider_id)
            .map_err(|e| e.to_string())?;

        let consumido_semana =
            somar_tokens_provider(store, provider_id, Instante(agora.0 - 7 * 86_400), agora)?;
        let consumido_24h =
            somar_tokens_provider(store, provider_id, Instante(agora.0 - 86_400), agora)?;
        let burn = capacidade::calcular_burn(consumido_24h, 86_400);

        if sinais.is_empty() {
            let j = montar_janela(
                provider_id,
                TipoJanela::Semana,
                Some(consumido_semana),
                None,
                burn,
                agora,
            );
            linhas.push(formatar_linha_janela(&j, &plano_label));
        } else {
            for s in &sinais {
                let j = montar_janela(
                    provider_id,
                    TipoJanela::Semana,
                    Some(consumido_semana),
                    Some(s),
                    burn,
                    agora,
                );
                linhas.push(formatar_linha_janela(&j, &plano_label));
            }
        }
    }

    // Provider sem fonte só aparece quando não há filtro específico — pedir
    // um provider por nome que não tem fonte é o operador escolhendo,
    // continua útil informar "sem fonte" ali também.
    if provider.is_none() {
        for excluido in providers_sem_fonte() {
            linhas.push(format!(
                "{}\t—\tsem fonte\t—\t—\t—\t—\t—\t—\t—",
                excluido.provider_id
            ));
        }
    } else if let Some(p) = &provider
        && let Some(excluido) = providers_sem_fonte()
            .into_iter()
            .find(|e| e.provider_id == p.as_str())
    {
        linhas.push(format!(
            "{}\t—\tsem fonte\t—\t—\t—\t—\t—\t—\t—",
            excluido.provider_id
        ));
    }

    Ok(linhas.join("\n"))
}

pub fn executar_plans_list(store: &dyn Store) -> Result<String, String> {
    let mut linhas = vec!["provider\tbilling_mode\tplano\tativo_desde\tverificado_em".to_string()];

    for provider_id in capacidade::PROVIDERS_VERIFICADOS {
        match store
            .plano_vigente(provider_id)
            .map_err(|e| e.to_string())?
        {
            Some(p) => linhas.push(format!(
                "{}\t{:?}\t{}\t{}\t{}",
                provider_id,
                p.billing_mode,
                p.plan_label.as_deref().unwrap_or("—"),
                formatar_instante(p.ativo_desde),
                formatar_instante(p.verificado_em),
            )),
            None => linhas.push(format!("{provider_id}\t—\tsem plano detectado\t—\t—")),
        }
    }

    for excluido in providers_sem_fonte() {
        linhas.push(format!("{}\t—\tsem fonte\t—\t—", excluido.provider_id));
    }

    Ok(linhas.join("\n"))
}

pub fn executar_plans_allocation(
    store: &dyn Store,
    provider: &str,
    period: Option<String>,
) -> Result<String, String> {
    let Some(plano) = store.plano_vigente(provider).map_err(|e| e.to_string())? else {
        return Ok(format!("sem plano detectado para '{provider}'"));
    };
    if plano.billing_mode != BillingMode::Subscription {
        return Ok(format!(
            "rateio não se aplica — billing_mode de '{provider}' é {:?}, não subscription",
            plano.billing_mode
        ));
    }

    let periodo = match &period {
        Some(p) => periodo_do_mes(p).ok_or_else(|| format!("período inválido: {p}"))?,
        None => periodo_mes_atual(),
    };

    let registros: Vec<_> = store
        .consumo_no_periodo(periodo)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|r| r.provider_id == provider)
        .collect();

    let mut tokens_por_cliente: Vec<(String, u64)> = Vec::new();
    let mut tokens_nao_atribuidos = 0u64;
    for r in &registros {
        let tokens = r.tokens.total_conhecido();
        match &r.client_id {
            Some(cliente) => match tokens_por_cliente.iter_mut().find(|(c, _)| c == cliente) {
                Some(entrada) => entrada.1 += tokens,
                None => tokens_por_cliente.push((cliente.clone(), tokens)),
            },
            None => tokens_nao_atribuidos += tokens,
        }
    }

    let rateio = capacidade::calcular_rateio(&tokens_por_cliente, tokens_nao_atribuidos);

    let mut linhas = vec!["client\tfracao_do_plano".to_string()];
    for (cliente, fracao) in &rateio.por_cliente {
        linhas.push(format!("{cliente}\t{:.1}%", fracao * 100.0));
    }
    linhas.push(format!(
        "não_atribuído (tokens, fora do rateio)\t{}",
        rateio.tokens_nao_atribuidos
    ));

    Ok(linhas.join("\n"))
}

// --- Identidade e contexto (grupos 5-6) ---------------------------------

pub fn executar_connect(store: &dyn Store, alvo: &str) -> Result<String, String> {
    let (client, project) = match alvo.split_once('/') {
        Some((c, p)) => (c, Some(p)),
        None => (alvo, None),
    };

    let ctx = identidade::conectar(store, client, project, Instante::agora())
        .map_err(|e| e.to_string())?;

    let perfil = store
        .perfil(&ctx.identity_profile_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "perfil não encontrado após conectar".to_string())?;

    Ok(identidade::linhas_export(&perfil).join("\n"))
}

pub fn executar_disconnect(store: &dyn Store) -> Result<String, String> {
    let Some(ctx) = store.contexto_ativo().map_err(|e| e.to_string())? else {
        return Ok(String::new()); // no-op: nada para desconectar
    };
    let perfil = store
        .perfil(&ctx.identity_profile_id)
        .map_err(|e| e.to_string())?;
    identidade::desconectar(store).map_err(|e| e.to_string())?;

    match perfil {
        Some(p) => Ok(identidade::linhas_unset(&p).join("\n")),
        None => Ok(String::new()),
    }
}

pub fn executar_whoami(
    store: &dyn Store,
    coletores: &[Box<dyn ColetorDeCapacidade>],
) -> Result<String, String> {
    let Some(ctx) = store.contexto_ativo().map_err(|e| e.to_string())? else {
        return Ok("nenhum contexto ativo".to_string());
    };
    let perfil = store
        .perfil(&ctx.identity_profile_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "perfil do contexto ativo não encontrado".to_string())?;

    let mut linhas = vec![
        format!("cliente: {}", ctx.client_id),
        format!("projeto: {}", ctx.project.as_deref().unwrap_or("—")),
        format!("perfil: {}", perfil.id),
    ];
    if let (Some(nome), Some(email)) = (&perfil.git_author_name, &perfil.git_author_email) {
        linhas.push(format!("git: {nome} <{email}>"));
    }
    if let Some(org) = &perfil.github_org {
        linhas.push(format!("github: {org}"));
    }
    linhas.push(String::new());
    linhas.push("provider\tstatus\tconta".to_string());
    for coletor in coletores {
        match coletor.consultar() {
            Ok((plano, _)) => linhas.push(format!(
                "{}\tautenticado\t{}",
                coletor.provider_id(),
                plano.account_email.as_deref().unwrap_or("desconhecida")
            )),
            Err(_) => linhas.push(format!("{}\tnão autenticado\t—", coletor.provider_id())),
        }
    }

    Ok(linhas.join("\n"))
}

pub fn executar_context_list(store: &dyn Store, client: &str) -> Result<String, String> {
    let perfis = store.perfis_do_cliente(client).map_err(|e| e.to_string())?;
    if perfis.is_empty() {
        return Ok(format!("nenhum perfil configurado para '{client}'"));
    }
    let mut linhas = vec!["id\tprojeto\tproviders".to_string()];
    for p in &perfis {
        linhas.push(format!(
            "{}\t{}\t{}",
            p.id,
            p.project.as_deref().unwrap_or("—"),
            p.bindings
                .iter()
                .map(|b| b.provider_id.as_str())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    Ok(linhas.join("\n"))
}

pub fn executar_context_show(store: &dyn Store, id: &str) -> Result<String, String> {
    match store.perfil(id).map_err(|e| e.to_string())? {
        None => Ok(format!("perfil '{id}' não existe")),
        Some(p) => {
            let mut linhas = vec![
                format!("id: {}", p.id),
                format!("cliente: {}", p.client_id),
                format!("projeto: {}", p.project.as_deref().unwrap_or("—")),
                format!(
                    "git: {} <{}>",
                    p.git_author_name.as_deref().unwrap_or("—"),
                    p.git_author_email.as_deref().unwrap_or("—")
                ),
                format!("github: {}", p.github_org.as_deref().unwrap_or("—")),
            ];
            for b in &p.bindings {
                linhas.push(format!("  binding: {} -> {}", b.provider_id, b.config_home));
            }
            Ok(linhas.join("\n"))
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn executar_context_init(
    store: &dyn Store,
    client: String,
    project: Option<String>,
    git_name: Option<String>,
    git_email: Option<String>,
    github_org: Option<String>,
) -> Result<String, String> {
    if !store.client_exists(&client).map_err(|e| e.to_string())? {
        store.upsert_client(&client).map_err(|e| e.to_string())?;
    }
    let id = format!(
        "{client}{}",
        project
            .as_deref()
            .map(|p| format!("-{p}"))
            .unwrap_or_default()
    );
    let perfil = identidade::criar_perfil(
        store,
        NovoPerfil {
            id,
            client_id: client,
            project,
            git_author_name: git_name,
            git_author_email: git_email,
            github_org,
            bindings: Vec::new(),
            created_at: Instante::agora(),
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(format!("perfil criado: {}", perfil.id))
}

pub fn executar_vault_list(store: &dyn Store) -> Result<String, String> {
    let credenciais = store.listar_credenciais().map_err(|e| e.to_string())?;
    if credenciais.is_empty() {
        return Ok("nenhuma credencial registrada".to_string());
    }
    let agora = Instante::agora();
    let mut linhas = vec!["id\tlabel\tclasse\tcriado_em\tultimo_uso\texpiracao".to_string()];
    for c in &credenciais {
        let expiracao = match c.expires_at {
            Some(exp) if c.esta_expirada(agora) => {
                format!("{} (EXPIRADA)", formatar_instante(exp))
            }
            Some(exp) => formatar_instante(exp),
            None => "—".to_string(),
        };
        linhas.push(format!(
            "{}\t{}\t{:?}\t{}\t{}\t{}",
            c.id,
            c.label,
            c.class,
            formatar_instante(c.created_at),
            c.last_used_at
                .map(formatar_instante)
                .unwrap_or_else(|| "—".to_string()),
            expiracao,
        ));
    }
    Ok(linhas.join("\n"))
}

// --- Continuity Pack (grupo 6) -------------------------------------------

fn gerar_id_nota() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("nota-{nanos}")
}

pub fn executar_memory_note(store: &dyn Store, texto: String) -> Result<String, String> {
    let contexto = store.contexto_ativo().map_err(|e| e.to_string())?;
    let nota = continuidade::registrar_nota(
        store,
        contexto.as_ref(),
        gerar_id_nota(),
        CategoriaNota::Nota,
        texto,
        None,
        Instante::agora(),
    )
    .map_err(|e| e.to_string())?;
    Ok(format!("nota registrada: {}", nota.id))
}

pub fn executar_memory_decide(
    store: &dyn Store,
    texto: String,
    why: String,
) -> Result<String, String> {
    let contexto = store.contexto_ativo().map_err(|e| e.to_string())?;
    let nota = continuidade::registrar_nota(
        store,
        contexto.as_ref(),
        gerar_id_nota(),
        CategoriaNota::Decisao,
        texto,
        Some(why),
        Instante::agora(),
    )
    .map_err(|e| e.to_string())?;
    Ok(format!("decisão registrada: {}", nota.id))
}

pub fn executar_continuity_show(
    store: &dyn Store,
    cwd: &std::path::Path,
) -> Result<String, String> {
    let contexto = store.contexto_ativo().map_err(|e| e.to_string())?;
    let pacote = continuidade::handoff(store, contexto.as_ref(), cwd).map_err(|e| e.to_string())?;
    Ok(continuidade::formatar_pacote(&pacote, None))
}

pub fn executar_handoff(
    store: &dyn Store,
    cwd: &std::path::Path,
    provider: &str,
) -> Result<String, String> {
    let contexto = store.contexto_ativo().map_err(|e| e.to_string())?;
    let pacote = continuidade::handoff(store, contexto.as_ref(), cwd).map_err(|e| e.to_string())?;
    Ok(continuidade::formatar_pacote(&pacote, Some(provider)))
}

/// Resolve o provider de um run: override explícito sempre vence (spec
/// routing/provider-rules: "Override explícito sempre vence a regra") — só
/// consulta `routing/rules.json` (relativo a `cwd`, mesmo diretório onde o
/// operador roda `brian run`) quando o operador não especifica um.
///
/// Devolve `(provider, origem)` — `origem` é `None` para override explícito,
/// ou a explicação da regra/`default` para `--explain-only`.
fn resolver_provider(
    store: &dyn Store,
    cwd: &std::path::Path,
    provider: Option<&str>,
) -> Result<(String, Option<String>), String> {
    if let Some(p) = provider {
        return Ok((p.to_string(), None));
    }

    let contexto = store
        .contexto_ativo()
        .map_err(|e| e.to_string())?
        .ok_or("nenhum contexto ativo")?;
    let regras =
        router::carregar_regras(&cwd.join("routing/rules.json")).map_err(|e| e.to_string())?;
    let decisao = router::decidir(&regras, &contexto.client_id, contexto.project.as_deref());
    let origem = match decisao.regra {
        Some(r) => format!(
            "regra when={{client: {:?}, project: {:?}}}",
            r.when.client, r.when.project
        ),
        None => "default".to_string(),
    };
    Ok((decisao.provider.to_string(), Some(origem)))
}

pub fn executar_run(
    store: &dyn Store,
    cwd: &std::path::Path,
    provider: Option<&str>,
    model: Option<&str>,
    tarefa: &str,
    gate: Option<&str>,
    explain_only: bool,
) -> Result<String, String> {
    let (provider_id, origem) = resolver_provider(store, cwd, provider)?;

    if explain_only {
        return Ok(match origem {
            Some(o) => format!("provider escolhido: {provider_id} (via {o})"),
            None => format!("provider escolhido: {provider_id} (override explícito)"),
        });
    }

    let contexto = store.contexto_ativo().map_err(|e| e.to_string())?;
    let run = execucao::iniciar_run(
        store,
        contexto.as_ref(),
        cwd,
        execucao::PedidoRun {
            provider_id: &provider_id,
            model,
            tarefa,
            gate,
            base_commit: None,
        },
        Instante::agora(),
    )
    .map_err(|e| e.to_string())?;
    Ok(format!(
        "run {} — status: {:?}, worktree: {}",
        run.id, run.status, run.worktree_path
    ))
}

fn formatar_run_recover(r: &RunRegistrado) -> String {
    format!(
        "{} — provider: {}, pid: {}, worktree: {}",
        r.id,
        r.provider_id,
        r.pid.map(|p| p.to_string()).unwrap_or_else(|| "?".into()),
        r.worktree_path
    )
}

pub fn executar_recover(store: &dyn Store, run: Option<&str>, all: bool) -> Result<String, String> {
    let orfaos = execucao::runs_orfaos(store).map_err(|e| e.to_string())?;
    if orfaos.is_empty() {
        return Ok("nenhum run órfão encontrado".to_string());
    }

    if let Some(run_id) = run {
        if !orfaos.iter().any(|r| r.id == run_id) {
            return Err(format!("'{run_id}' não é um run órfão"));
        }
        execucao::recuperar(store, run_id, Instante::agora()).map_err(|e| e.to_string())?;
        return Ok(format!("run {run_id} recuperado (abandonado)"));
    }

    if all {
        for r in &orfaos {
            execucao::recuperar(store, &r.id, Instante::agora()).map_err(|e| e.to_string())?;
        }
        return Ok(format!(
            "{} run(s) recuperado(s) (abandonados)",
            orfaos.len()
        ));
    }

    let linhas: Vec<String> = orfaos.iter().map(formatar_run_recover).collect();
    Ok(format!(
        "{} run(s) órfão(s) — use --run <id> ou --all para finalizar:\n{}",
        orfaos.len(),
        linhas.join("\n")
    ))
}

pub fn executar_worktree_list(store: &dyn Store) -> Result<String, String> {
    let mut runs = store.runs_em_execucao().map_err(|e| e.to_string())?;
    runs.extend(store.runs_abandonados().map_err(|e| e.to_string())?);

    if runs.is_empty() {
        return Ok("nenhum worktree ativo ou abandonado".to_string());
    }

    let linhas: Vec<String> = runs
        .iter()
        .map(|r| {
            format!(
                "{} — status: {:?}, branch: {}, worktree: {}",
                r.id, r.status, r.branch, r.worktree_path
            )
        })
        .collect();
    Ok(linhas.join("\n"))
}

pub fn executar_eval_run(
    store: &dyn Store,
    dir: &std::path::Path,
    case: Option<&str>,
) -> Result<String, String> {
    let mut casos = crate::eval::carregar_casos_do_diretorio(dir).map_err(|e| e.to_string())?;
    if let Some(id) = case {
        casos.retain(|c| c.id == id);
        if casos.is_empty() {
            return Err(format!("caso '{id}' não encontrado em {}", dir.display()));
        }
    }
    if casos.is_empty() {
        return Ok(format!(
            "nenhum caso de eval encontrado em {}",
            dir.display()
        ));
    }

    let mut linhas = Vec::with_capacity(casos.len());
    for caso in &casos {
        let runs =
            crate::eval::rodar_caso(store, caso, Instante::agora()).map_err(|e| e.to_string())?;
        linhas.push(crate::eval::formatar_relatorio(caso, &runs));
    }
    Ok(linhas.join("\n"))
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
    fn unattributed_nunca_apresenta_equivalente_como_pago() {
        // Bug real encontrado na auditoria da task 8.6: formatar_unattributed
        // colapsava pago/equivalente numa coluna "custo" genérica via
        // .or() -- se pago fosse None, o equivalente aparecia ali sem
        // rótulo, indistinguível de valor realmente pago.
        let regs = vec![registro(
            "codex",
            "gpt",
            None,
            Some(500_000),
            CostSource::Catalog,
            None, // não-atribuído
        )];
        let saida = formatar_unattributed(&regs);
        let cabecalho = saida.lines().next().unwrap();
        assert!(cabecalho.contains("custo_pago"));
        assert!(cabecalho.contains("custo_equivalente"));

        let campos: Vec<&str> = saida.lines().nth(1).unwrap().split('\t').collect();
        let idx_pago = cabecalho
            .split('\t')
            .position(|c| c == "custo_pago")
            .unwrap();
        let idx_equiv = cabecalho
            .split('\t')
            .position(|c| c == "custo_equivalente")
            .unwrap();
        assert_eq!(
            campos[idx_pago], "—",
            "sem custo pago real, não deve aparecer valor"
        );
        assert_ne!(
            campos[idx_equiv], "—",
            "equivalente calculável deve aparecer, na coluna certa"
        );
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

    // --- brian plans allocation (achado do audit da task 10.1) -----------

    fn store_com_migracoes() -> crate::storage::sqlite::SqliteStore {
        let s = crate::storage::sqlite::SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s
    }

    #[test]
    fn plans_allocation_sem_plano_detectado() {
        let s = store_com_migracoes();
        let saida = executar_plans_allocation(&s, "grok", None).unwrap();
        assert!(saida.contains("sem plano detectado"));
    }

    #[test]
    fn plans_allocation_recusa_billing_mode_api() {
        // Spec plan-cost-allocation, "Rateio se aplica apenas a planos de
        // assinatura".
        let s = store_com_migracoes();
        s.registrar_plano(crate::storage::NovoPlano {
            provider_id: "codex".into(),
            billing_mode: BillingMode::Api,
            plan_label: None,
            detectado_em: Instante(0),
        })
        .unwrap();
        let saida = executar_plans_allocation(&s, "codex", None).unwrap();
        assert!(saida.contains("não se aplica"));
    }

    #[test]
    fn plans_allocation_divide_proporcional_entre_clientes() {
        let s = store_com_migracoes();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();
        s.registrar_plano(crate::storage::NovoPlano {
            provider_id: "codex".into(),
            billing_mode: BillingMode::Subscription,
            plan_label: Some("team".into()),
            detectado_em: Instante(0),
        })
        .unwrap();

        let occurred_at = Instante(
            crate::adapters::tempo::parse_timestamp_iso8601("2026-01-15T00:00:00Z").unwrap(),
        );

        let mut c1 = crate::storage::test_util::novo_consumo("k1", "codex", "gpt");
        c1.client_id = Some("xpto".into());
        c1.occurred_at = occurred_at;
        s.gravar_consumo(c1).unwrap();

        let mut c2 = crate::storage::test_util::novo_consumo("k2", "codex", "gpt");
        c2.client_id = Some("acme".into());
        c2.occurred_at = occurred_at;
        s.gravar_consumo(c2).unwrap();

        // xpto e acme usam a mesma fixture (150 tokens conhecidos cada) --
        // divisão deve ficar 50/50.
        let saida = executar_plans_allocation(&s, "codex", Some("2026-01".to_string())).unwrap();
        assert!(saida.contains("xpto\t50.0%"));
        assert!(saida.contains("acme\t50.0%"));
    }

    // --- brian whoami (achado do audit da task 7.1) -----------------------

    struct ColetorFalso {
        provider_id: &'static str,
        resultado: std::result::Result<crate::domain::PlanoDetectado, ()>,
    }

    impl ColetorDeCapacidade for ColetorFalso {
        fn provider_id(&self) -> &str {
            self.provider_id
        }
        fn consultar(
            &self,
        ) -> std::result::Result<
            (
                crate::domain::PlanoDetectado,
                Vec<crate::domain::SinalDeQuotaColetado>,
            ),
            crate::importacao::ErroColeta,
        > {
            self.resultado
                .clone()
                .map(|plano| (plano, Vec::new()))
                .map_err(|_| crate::importacao::ErroColeta {
                    motivo: "não autenticado".into(),
                })
        }
    }

    #[test]
    fn whoami_sem_contexto_ativo_informa_explicitamente() {
        let s = crate::storage::sqlite::SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        let saida = executar_whoami(&s, &[]).unwrap();
        assert_eq!(saida, "nenhum contexto ativo");
    }

    #[test]
    fn whoami_com_contexto_ativo_mostra_conta_autenticada_por_provider() {
        let s = crate::storage::sqlite::SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s.upsert_client("xpto").unwrap();
        s.criar_perfil(crate::storage::NovoPerfil {
            id: "p1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            git_author_name: Some("Joao Costa".into()),
            git_author_email: Some("joao@xpto.com.br".into()),
            github_org: Some("xpto-org".into()),
            bindings: vec![],
            created_at: Instante(0),
        })
        .unwrap();
        identidade::conectar(&s, "xpto", None, Instante(1)).unwrap();

        let coletores: Vec<Box<dyn ColetorDeCapacidade>> = vec![
            Box::new(ColetorFalso {
                provider_id: "claude",
                resultado: Ok(crate::domain::PlanoDetectado {
                    billing_mode: BillingMode::Subscription,
                    plan_label: Some("pro".into()),
                    account_email: Some("eng@xpto.com.br".into()),
                }),
            }),
            Box::new(ColetorFalso {
                provider_id: "codex",
                resultado: Err(()),
            }),
        ];

        let saida = executar_whoami(&s, &coletores).unwrap();
        assert!(saida.contains("cliente: xpto"));
        assert!(saida.contains("projeto: checkout-api"));
        assert!(saida.contains("git: Joao Costa <joao@xpto.com.br>"));
        assert!(
            saida.contains("claude\tautenticado\teng@xpto.com.br"),
            "spec: mostra a conta autenticada, não só o status. Saída: {saida}"
        );
        assert!(saida.contains("codex\tnão autenticado\t—"));
    }

    fn store_com_contexto_ativo() -> crate::storage::sqlite::SqliteStore {
        let s = crate::storage::sqlite::SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s.upsert_client("xpto").unwrap();
        s.criar_perfil(NovoPerfil {
            id: "p1".into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            git_author_name: None,
            git_author_email: None,
            github_org: None,
            bindings: vec![],
            created_at: Instante(0),
        })
        .unwrap();
        identidade::conectar(&s, "xpto", Some("checkout-api"), Instante(1)).unwrap();
        s
    }

    fn repo_git_temporario_para_run(sufixo: &str) -> std::path::PathBuf {
        let dir = crate::testutil::dir_temporario_unico(&format!("comandos-run-{sufixo}"));
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
        std::fs::write(dir.join("README.md"), "inicial").unwrap();
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "inicial"]);
        dir
    }

    #[test]
    fn executar_run_com_provider_explicito_nao_consulta_regras() {
        let s = store_com_contexto_ativo();
        let repo = repo_git_temporario_para_run("explicito");
        // Sem routing/rules.json em `repo` -- se a implementação tentasse
        // carregar regras mesmo com override, o erro seria "erro lendo
        // regras", não o de provider inválido.
        let erro =
            executar_run(&s, &repo, Some("claude"), None, "tarefa", None, false).unwrap_err();
        assert!(
            erro.contains("não tem execução não-interativa verificada"),
            "esperava erro de provider inválido do execucao::iniciar_run, veio: {erro}"
        );
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn executar_run_sem_provider_usa_decisao_de_regras() {
        let s = store_com_contexto_ativo();
        let repo = repo_git_temporario_para_run("regra");
        std::fs::create_dir_all(repo.join("routing")).unwrap();
        std::fs::write(
            repo.join("routing/rules.json"),
            r#"{"default": {"provider": "claude"}, "rules": []}"#,
        )
        .unwrap();

        let erro = executar_run(&s, &repo, None, None, "tarefa", None, false).unwrap_err();
        assert!(
            erro.contains("não tem execução não-interativa verificada"),
            "provider decidido pela regra (claude) deveria chegar até a validação de \
             execucao::iniciar_run e falhar por lá, veio: {erro}"
        );
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn explain_only_mostra_decisao_sem_criar_run() {
        let s = store_com_contexto_ativo();
        let repo = repo_git_temporario_para_run("explain");
        std::fs::create_dir_all(repo.join("routing")).unwrap();
        std::fs::write(
            repo.join("routing/rules.json"),
            r#"{"default": {"provider": "codex"}, "rules": []}"#,
        )
        .unwrap();

        let saida = executar_run(&s, &repo, None, None, "tarefa", None, true).unwrap();
        assert!(saida.contains("codex"));
        assert!(saida.contains("default"));
        assert!(
            s.runs_em_execucao().unwrap().is_empty(),
            "--explain-only não deveria ter criado run nenhum"
        );
        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn explain_only_reporta_regra_mesmo_quando_provider_igual_ao_default() {
        // Regressão: `store_com_contexto_ativo` usa project="checkout-api",
        // que casa com a regra abaixo -- mesmo o provider da regra sendo
        // igual ao `default`, a explicação precisa dizer "regra", não
        // "default" (mesmo bug coberto em router::tests).
        let s = store_com_contexto_ativo();
        let repo = repo_git_temporario_para_run("explain-regra");
        std::fs::create_dir_all(repo.join("routing")).unwrap();
        std::fs::write(
            repo.join("routing/rules.json"),
            r#"{
                "default": {"provider": "codex"},
                "rules": [
                    {"when": {"project": "checkout-api"}, "then": {"provider": "codex"}}
                ]
            }"#,
        )
        .unwrap();

        let saida = executar_run(&s, &repo, None, None, "tarefa", None, true).unwrap();
        assert!(saida.contains("codex"));
        assert!(
            saida.contains("regra"),
            "regra casou (project=checkout-api) e deveria ser reportada como origem, veio: {saida}"
        );
        std::fs::remove_dir_all(&repo).ok();
    }
}
