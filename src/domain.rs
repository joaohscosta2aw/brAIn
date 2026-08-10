//! Tipos do domínio.
//!
//! Fixados pelos specs em `openspec/changes/client-cost-attribution/specs/`.
//! Não há SQL aqui — a persistência é problema de `storage`.

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

impl Instante {
    /// Instante atual, em segundos desde a época UTC. `unwrap_or(0)` só é
    /// alcançável com relógio do sistema antes de 1970 — mesma tolerância já
    /// usada em `adapters::gemini`.
    pub fn agora() -> Self {
        Instante(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        )
    }
}

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

/// Uma das janelas de capacidade suportadas (capacity-windows-and-plans).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoJanela {
    Dia,
    Semana,
    Mes,
    CicloDoPlano,
}

/// Plano detectado na fonte de um provider — o que um `ColetorDeCapacidade`
/// devolve, antes de virar registro com vigência (essa parte é do storage).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanoDetectado {
    pub billing_mode: BillingMode,
    /// Identificador do plano relatado pela fonte (ex.: "pro", "plus").
    /// `None` quando a fonte não distingue por nome.
    pub plan_label: Option<String>,
    /// Conta autenticada, quando a fonte relata (spec context-switching,
    /// "Consulta do contexto ativo mostra a conta autenticada" — mostrar
    /// *qual* conta, não só que há autenticação, é o que evita rodar
    /// trabalho de cliente na conta errada).
    pub account_email: Option<String>,
}

/// Um sinal de percentual restante de cota, como um `ColetorDeCapacidade` o
/// observou — antes de virar `quota_signal` gravado.
#[derive(Debug, Clone, PartialEq)]
pub struct SinalDeQuotaColetado {
    pub bucket_id: String,
    pub grupo: String,
    pub remaining_percent: f64,
    pub reset_at: Option<Instante>,
}

/// De onde veio um valor de janela de capacidade. Nunca um nível inferior
/// apresentado com a autoridade de um superior (spec capacity-windows,
/// "Hierarquia de fonte da capacidade").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FonteCapacidade {
    /// Percentual/reset relatado pela fonte do próprio provider.
    Provider,
    /// Só o consumo medido pelo ledger está disponível — sem percentual.
    BrianMeasured,
}

/// Estado de uma janela de capacidade de um provider, pronto para
/// apresentação. `capacidade_tokens` fica `None` sempre que a fonte não
/// expõe contagem absoluta (todos os providers desta change dão apenas
/// percentual) — nunca inventado a partir do percentual.
#[derive(Debug, Clone, PartialEq)]
pub struct JanelaDeCapacidade {
    pub provider_id: String,
    pub bucket_id: String,
    pub tipo: TipoJanela,
    pub consumido_tokens: Option<u64>,
    pub capacidade_tokens: Option<u64>,
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    pub resets_at: Option<Instante>,
    pub burn_tokens_por_hora: Option<f64>,
    pub eta_esgotamento: Option<Instante>,
    pub fonte: FonteCapacidade,
}

/// Vínculo de um provider a um caminho de configuração isolado — o que vira
/// variável de ambiente do processo filho (`CODEX_HOME=<config_home>`, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderBinding {
    pub provider_id: String,
    pub config_home: String,
}

/// Um perfil de identidade: cliente (e opcionalmente projeto), identidade Git
/// e bindings de provider. `project = None` é o perfil "padrão" do cliente,
/// usado quando ele só tem um projeto configurado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerfilIdentidade {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub git_author_name: Option<String>,
    pub git_author_email: Option<String>,
    pub github_org: Option<String>,
    pub bindings: Vec<ProviderBinding>,
}

/// O contexto ativo agora — singleton (spec context-switching: "trocar
/// contexto por construção, não por filtro").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextoAtivo {
    pub client_id: String,
    pub project: Option<String>,
    pub identity_profile_id: String,
    pub connected_at: Instante,
}

/// Classe de secret — determina exigência de autenticação biométrica na
/// resolução (spec vault: "Classe de secret determina exigência de
/// autenticação").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClasseSecret {
    Low,
    Medium,
    High,
    Critical,
}

impl ClasseSecret {
    /// `high`/`critical` exigem Touch ID antes de liberar o valor.
    pub fn exige_biometria(&self) -> bool {
        matches!(self, ClasseSecret::High | ClasseSecret::Critical)
    }
}

/// Uma credencial registrada no Vault — referência e metadados, nunca o
/// valor (spec vault: "Só referência é persistida, nunca o valor do
/// secret").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredencialRegistrada {
    pub id: String,
    pub label: String,
    pub keychain_service: String,
    pub keychain_account: String,
    pub class: ClasseSecret,
    pub created_at: Instante,
    pub expires_at: Option<Instante>,
    pub last_used_at: Option<Instante>,
    pub rotation_policy: Option<String>,
}

impl CredencialRegistrada {
    /// `true` quando `expires_at` existe e já passou. Spec vault: "Consulta
    /// de credencial expirada alerta, não bloqueia sem explicação" — quem
    /// exibe a credencial decide o que fazer com isso, esta função só
    /// responde o fato.
    pub fn esta_expirada(&self, agora: Instante) -> bool {
        self.expires_at.is_some_and(|exp| exp <= agora)
    }
}

/// Categoria de uma nota de memória — como o Continuity Pack agrupa notas na
/// apresentação (spec pack: "Pack montado a partir das notas do Context ativo").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoriaNota {
    Objetivo,
    Decisao,
    Analise,
    TentativaFalha,
    ProximoPasso,
    Nota,
}

/// Uma nota de memória, escopada por Context (`client_id` + `project`, os
/// mesmos campos de `ContextoAtivo` — design.md: "Context de memória é
/// (client_id, project), reaproveitado de active_context"). Append-only: não
/// há método de edição/remoção em nenhuma camada (D-14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotaDeMemoria {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub categoria: CategoriaNota,
    pub texto: String,
    /// Só preenchido para `CategoriaNota::Decisao` — spec memory-notes:
    /// "Decisão exige o porquê".
    pub rationale: Option<String>,
    pub created_at: Instante,
}

/// Um arquivo alterado no repositório do Context, como `git status` relatou —
/// nunca inventado (spec pack: "Arquivos tocados vêm do repositório real").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArquivoTocado {
    pub path: String,
    /// Código de duas letras do `git status --porcelain` (`" M"`, `"??"`,
    /// `"A "`, etc.) — preservado tal como o Git relatou, sem reinterpretar.
    pub status: String,
}

/// O Continuity Pack montado — notas agrupadas por categoria, arquivos
/// tocados reais, aviso de orçamento quando aplicável. Nunca contém
/// transcript bruto de provider (spec pack).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PactoDeContinuidade {
    pub client_id: String,
    pub project: Option<String>,
    pub notas: Vec<NotaDeMemoria>,
    pub arquivos_tocados: Vec<ArquivoTocado>,
    /// `Some` quando o pack montado excede o tamanho de referência — nunca
    /// trunca conteúdo, só sinaliza (spec pack: "Pack acima do orçamento").
    pub aviso_orcamento: Option<String>,
}

/// Estado de um run — o primeiro subsistema do Brian que executa, não só
/// observa (isolated-tracked-run).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusRun {
    EmExecucao,
    Concluido,
    Falhou,
    /// Processo morreu (SIGKILL não é capturável) e foi finalizado por
    /// `brian recover` sem reexecutar a tarefa (spec orphan-recovery:
    /// "Finalização de órfão nunca duplica custo").
    Abandonado,
}

/// Um run: worktree isolado (D-7), persistido antes de qualquer efeito
/// colateral (D-12). `pid` é `None` até o processo do provider existir —
/// ausente e "processo não iniciado" são o mesmo fato aqui, não confundido
/// com zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRegistrado {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub base_commit: String,
    pub worktree_path: String,
    pub branch: String,
    pub provider_id: String,
    pub pid: Option<u32>,
    pub status: StatusRun,
    pub custo_equivalente: Option<Money>,
    pub started_at: Instante,
    pub finished_at: Option<Instante>,
}

/// Um evento do log local de um run — não OTel completo (design.md, "Log de
/// eventos local, não OTel completo"), só o suficiente para reconstruir o
/// que aconteceu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventoDeRun {
    pub run_id: String,
    pub tipo: String,
    pub detalhe: Option<String>,
    pub ocorrido_em: Instante,
}

/// Estado de um `workflow_run` — máquina de estados sobre fases
/// (workflow-engine, blueprint §15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusWorkflowRun {
    Pending,
    Running,
    /// Aguardando aprovação explícita do operador (spec human-approval:
    /// "Fase com aprovação obrigatória pausa o workflow").
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Um workflow em execução: `workflow_version` é congelado no momento da
/// criação (design.md, blueprint §15.6/§113) — nunca reavaliado depois.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunRegistrado {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub workflow_id: String,
    pub workflow_version: i64,
    /// Texto bruto do arquivo de definição no momento da criação — toda
    /// decisão de transição posterior lê esta cópia, nunca o arquivo no
    /// disco de novo (spec state-machine: "Versão do workflow é congelada
    /// no início do run").
    pub definicao_json: String,
    /// Gravada uma vez, reaproveitada em toda fase e em `approve` — nunca
    /// re-digitada pelo operador.
    pub tarefa: String,
    pub current_phase: String,
    pub status: StatusWorkflowRun,
    pub pause_reason: Option<String>,
    pub total_phases: i64,
    pub started_at: Instante,
    pub finished_at: Option<Instante>,
}

/// Uma entrada no histórico de fases (blueprint §15.6: phase_history).
/// `run_id` é `None` para fases terminais, que não executam
/// `execucao::iniciar_run` (spec phase-execution).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntradaDeFase {
    pub id: String,
    pub workflow_run_id: String,
    pub phase_id: String,
    pub run_id: Option<String>,
    pub outcome: Option<String>,
    pub entrada_numero: i64,
    pub started_at: Instante,
    pub ended_at: Option<Instante>,
}

/// Uma comparação pareada (paired-comparison, blueprint §38.4):
/// `vencedor_provider_id` é `None` até o operador escolher explicitamente —
/// nunca preenchido pela execução dos candidatos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparacaoRegistrada {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub tarefa: String,
    pub vencedor_provider_id: Option<String>,
    pub started_at: Instante,
    pub finished_at: Option<Instante>,
}

/// Um candidato de comparação — liga um provider ao `run` real que o
/// executou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidatoComparacao {
    pub id: String,
    pub comparacao_id: String,
    pub provider_id: String,
    pub run_id: Option<String>,
}

/// Uma execução do experimento H-1 (context-governor-experiment): liga um
/// case sintético e um braço (A/B/C) ao `run` real que os executou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecucaoExperimento {
    pub id: String,
    pub case_id: String,
    pub braco: String,
    pub run_id: String,
    pub started_at: Instante,
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

    fn credencial(expires_at: Option<Instante>) -> CredencialRegistrada {
        CredencialRegistrada {
            id: "c1".into(),
            label: "teste".into(),
            keychain_service: "brian".into(),
            keychain_account: "xpto/c1".into(),
            class: ClasseSecret::Low,
            created_at: Instante(0),
            expires_at,
            last_used_at: None,
            rotation_policy: None,
        }
    }

    #[test]
    fn credencial_sem_expiracao_nunca_esta_expirada() {
        assert!(!credencial(None).esta_expirada(Instante(1_000_000)));
    }

    #[test]
    fn credencial_com_expiracao_futura_nao_esta_expirada() {
        assert!(!credencial(Some(Instante(2000))).esta_expirada(Instante(1000)));
    }

    #[test]
    fn credencial_com_expiracao_passada_esta_expirada() {
        assert!(credencial(Some(Instante(1000))).esta_expirada(Instante(2000)));
    }
}
