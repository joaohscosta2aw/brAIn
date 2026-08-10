//! Fronteira de armazenamento.
//!
//! **D-9: todo SQL vive aqui dentro.** Nenhum outro módulo pode conter consulta,
//! nome de tabela ou detalhe de esquema — o restante do sistema conversa apenas
//! com as traits definidas neste módulo.
//!
//! `scripts/verificar-invariantes.sh` verifica isso mecanicamente a cada push.

pub mod sqlite;

use crate::domain::{
    BillingMode, CandidatoComparacao, CategoriaNota, ClasseSecret, ComparacaoRegistrada,
    ContextoAtivo, CostSource, CredencialRegistrada, EntradaDeFase, EventoDeRun,
    ExecucaoExperimento, Instante, Money, NotaDeMemoria, PerfilIdentidade, ProviderBinding,
    RunRegistrado, StatusRun, StatusWorkflowRun, Tokens, UsageRecord, UsageSource,
    WorkflowRunRegistrado,
};
use std::fmt;

/// Erro de armazenamento visto de fora do módulo.
///
/// Deliberadamente opaco quanto ao mecanismo: quem chama não deve poder
/// distinguir um erro de SQLite de um erro de qualquer outra implementação,
/// sob pena de vazar o detalhe que D-1 e D-9 existem para conter.
#[derive(Debug)]
pub enum StorageError {
    /// A entidade referenciada não existe. Ex.: atribuir consumo a um cliente
    /// inexistente, que o spec manda recusar sem alterar o registro.
    NotFound(String),
    /// O dado apresentado viola uma invariante do ledger.
    Invalid(String),
    /// Falha do mecanismo de persistência.
    Backend(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(o) => write!(f, "não encontrado: {o}"),
            Self::Invalid(m) => write!(f, "inválido: {m}"),
            Self::Backend(m) => write!(f, "falha de armazenamento: {m}"),
        }
    }
}

impl std::error::Error for StorageError {}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Um `usage_record` ainda sem identidade nem estado de atribuição — o que um
/// coletor de consumo apresenta ao armazenamento para gravação.
///
/// `dedup_key` é o que garante idempotência (spec: "Importação idempotente").
/// Vem do identificador estável do provider quando existe; senão, de uma
/// impressão digital calculada por quem chama (task 5.3). O armazenamento não
/// decide a chave, só a usa: apresentar a mesma chave duas vezes devolve o
/// registro já existente em vez de duplicar.
#[derive(Debug, Clone)]
pub struct NovoConsumo {
    pub dedup_key: String,
    pub provider_id: String,
    pub model: String,
    pub tokens: Tokens,
    pub custo_pago: Option<Money>,
    pub custo_equivalente_api: Option<Money>,
    pub billing_mode: BillingMode,
    pub usage_source: UsageSource,
    pub cost_source: CostSource,
    /// Cliente já conhecido no momento da gravação, se houver. `None` grava
    /// como `unattributed` — nunca como suposição de dono.
    pub client_id: Option<String>,
    pub occurred_at: Instante,
}

/// Uma entrada do catálogo de preço, válida num intervalo de vigência.
///
/// Versionado por vigência para que o equivalente de um consumo passado
/// permaneça reproduzível mesmo que o preço mude depois (design.md).
#[derive(Debug, Clone)]
pub struct EntradaCatalogo {
    pub model: String,
    pub preco_por_1k_tokens: Money,
    pub vigente_desde: Instante,
    /// `None` = ainda vigente.
    pub vigente_ate: Option<Instante>,
}

/// Uma violação de invariante de integridade do ledger, localizada a um
/// registro específico (spec: "identifica qual invariante foi violada e em
/// quais registros").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViolacaoIntegridade {
    pub usage_record_id: String,
    pub descricao: String,
}

/// Uma revisão registrada de um campo de `usage_record` (D-14: correção
/// supersede, não sobrescreve sem rastro).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revisao {
    pub campo: String,
    pub valor_anterior: Option<String>,
    pub revisado_em: Instante,
}

/// Recorte de tempo meio-aberto: `[desde, ate)`. `ate = None` significa "até
/// agora".
#[derive(Debug, Clone, Copy)]
pub struct Periodo {
    pub desde: Instante,
    pub ate: Option<Instante>,
}

/// Plano detectado a registrar. `registrar_plano` decide, comparando com o
/// vigente, se isso abre uma nova vigência ou é um no-op (task 3.3/4.4).
#[derive(Debug, Clone)]
pub struct NovoPlano {
    pub provider_id: String,
    pub billing_mode: BillingMode,
    pub plan_label: Option<String>,
    pub detectado_em: Instante,
}

/// Um plano com sua vigência, como persistido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanoRegistrado {
    pub provider_id: String,
    pub billing_mode: BillingMode,
    pub plan_label: Option<String>,
    pub ativo_desde: Instante,
    /// `None` = ainda vigente.
    pub ativo_ate: Option<Instante>,
    /// Última consulta bem-sucedida à fonte, mesmo quando o plano não
    /// mudou. Envelhecido (bem anterior a agora) é o sinal de que a fonte
    /// pode estar falhando nas importações recentes — spec plan-catalog,
    /// "Consulta de plano falha": "identifica a informação como
    /// potencialmente desatualizada".
    pub verificado_em: Instante,
}

/// Sinal de quota a gravar — upsert por `(provider_id, bucket_id)`.
#[derive(Debug, Clone)]
pub struct NovoQuotaSignal {
    pub provider_id: String,
    pub bucket_id: String,
    pub grupo: String,
    pub remaining_percent: f64,
    pub reset_at: Option<Instante>,
    pub observed_at: Instante,
}

/// Um sinal de quota como persistido — sempre o mais recente por bucket.
#[derive(Debug, Clone, PartialEq)]
pub struct QuotaSignalRegistrado {
    pub provider_id: String,
    pub bucket_id: String,
    pub grupo: String,
    pub remaining_percent: f64,
    pub reset_at: Option<Instante>,
    pub observed_at: Instante,
}

/// Um perfil de identidade a criar.
#[derive(Debug, Clone)]
pub struct NovoPerfil {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub git_author_name: Option<String>,
    pub git_author_email: Option<String>,
    pub github_org: Option<String>,
    pub bindings: Vec<ProviderBinding>,
    pub created_at: Instante,
}

/// Metadados de uma credencial a registrar — nunca o valor (task 3.7/3.8
/// vivem aqui, não em `vault.rs`: bookkeeping é SQL, D-9).
#[derive(Debug, Clone)]
pub struct NovaCredencialMetadados {
    pub id: String,
    pub label: String,
    pub keychain_service: String,
    pub keychain_account: String,
    pub class: ClasseSecret,
    pub created_at: Instante,
    pub expires_at: Option<Instante>,
    pub rotation_policy: Option<String>,
}

/// A fronteira de armazenamento que o núcleo consome.
///
/// Uma única trait, não uma por grupo de tasks: as operações compartilham a
/// mesma transação lógica em vários casos (gravar E verificar integridade;
/// atribuir E preservar o valor anterior), e SQLite não teria como compor
/// transações através de múltiplos objetos de trait sem vazar a conexão para
/// fora de `storage/`.
pub trait Store {
    /// Aplica as migrações ainda não aplicadas. Idempotente.
    fn migrate(&self) -> Result<()>;

    /// Registra um cliente. Idempotente por id: registrar o mesmo id duas
    /// vezes não é erro.
    fn upsert_client(&self, client_id: &str) -> Result<()>;

    fn client_exists(&self, client_id: &str) -> Result<bool>;

    /// Grava um consumo. Se `dedup_key` já existir, devolve o registro
    /// existente sem criar duplicata e sem alterar totais (spec: importação
    /// idempotente).
    fn gravar_consumo(&self, novo: NovoConsumo) -> Result<UsageRecord>;

    /// Atribui (ou reatribui) um registro a um cliente existente. A
    /// atribuição anterior, se houver, fica recuperável via `historico`
    /// (D-14, task 4.6).
    ///
    /// `NotFound` se o registro ou o cliente não existirem — nesse caso o
    /// registro não é alterado (spec: "recusa a operação... registro
    /// permanece não atribuído").
    fn atribuir(&self, usage_record_id: &str, client_id: &str) -> Result<UsageRecord>;

    /// Registra o custo pago informado pelo provider, chegado depois da
    /// gravação inicial. O valor e a fonte anteriores permanecem recuperáveis
    /// via `historico` — não são sobrescritos sem rastro (task 3.6, D-14).
    fn atualizar_custo_pago(&self, usage_record_id: &str, pago: Money) -> Result<UsageRecord>;

    /// Revisões registradas para um `usage_record`, mais recente primeiro.
    fn historico(&self, usage_record_id: &str) -> Result<Vec<Revisao>>;

    /// Instante do consumo mais recente já importado de um provider.
    /// `None` se nada foi importado ainda. Usado para importar apenas o
    /// período ainda não coberto (task 5.5).
    fn ultimo_consumo_importado(&self, provider_id: &str) -> Result<Option<Instante>>;

    /// Consumo sem cliente no período. Vazio quando o ledger está íntegro
    /// (spec: "o resultado é vazio... nenhum alarme").
    fn nao_atribuidos(&self, periodo: Periodo) -> Result<Vec<UsageRecord>>;

    /// Consumo de um cliente no período. Isolamento garantido pela própria
    /// consulta: nunca inclui registros de outro cliente nem não-atribuídos
    /// (spec: "isolamento... por construção, não por filtragem posterior").
    fn consumo_do_cliente(&self, client_id: &str, periodo: Periodo) -> Result<Vec<UsageRecord>>;

    /// Todo o ledger no período, para agregações que não são por cliente
    /// (`--by provider`, `--by model`, exportação completa).
    fn consumo_no_periodo(&self, periodo: Periodo) -> Result<Vec<UsageRecord>>;

    /// Verifica as quatro invariantes de integridade contra o ledger inteiro.
    fn verificar_integridade(&self) -> Result<Vec<ViolacaoIntegridade>>;

    /// Insere ou atualiza uma entrada de catálogo. Não sobrescreve entradas
    /// passadas: uma nova vigência fecha a anterior (`vigente_ate`) em vez de
    /// apagá-la, preservando reprodutibilidade histórica.
    fn upsert_catalogo(&self, entrada: EntradaCatalogo) -> Result<()>;

    /// Preço vigente para um modelo num instante. `None` se o modelo nunca
    /// esteve no catálogo naquele instante.
    fn preco_vigente(&self, model: &str, em: Instante) -> Result<Option<EntradaCatalogo>>;

    /// Registra o plano detectado de um provider. Se o billing_mode/label
    /// relatado é igual ao vigente, não cria vigência redundante (evita uma
    /// linha nova a cada import quando nada mudou). Se difere, fecha a
    /// vigência anterior e abre uma nova (spec plan-catalog: "Plano detectado
    /// muda").
    fn registrar_plano(&self, novo: NovoPlano) -> Result<()>;

    /// Plano vigente de um provider agora. `None` se nunca foi detectado.
    fn plano_vigente(&self, provider_id: &str) -> Result<Option<PlanoRegistrado>>;

    /// Plano vigente de um provider num instante passado (spec plan-catalog:
    /// "Janela histórica usa o plano vigente à época").
    fn plano_vigente_em(&self, provider_id: &str, em: Instante) -> Result<Option<PlanoRegistrado>>;

    /// Grava ou atualiza o sinal de quota mais recente de um bucket.
    fn upsert_quota_signal(&self, sinal: NovoQuotaSignal) -> Result<()>;

    /// Sinais de quota mais recentes de um provider, um por bucket.
    fn quota_signals(&self, provider_id: &str) -> Result<Vec<QuotaSignalRegistrado>>;

    /// Cria um perfil de identidade. `Invalid` se já existir um perfil para o
    /// mesmo `(client_id, project)` (constraint `UNIQUE` da migração).
    fn criar_perfil(&self, novo: NovoPerfil) -> Result<PerfilIdentidade>;

    /// Um perfil por id. `None` se não existir.
    fn perfil(&self, id: &str) -> Result<Option<PerfilIdentidade>>;

    /// Todos os perfis de um cliente — a contagem decide ambiguidade de
    /// `connect` sem `--project` (spec context-switching: "múltiplos
    /// projetos, sem especificar qual"). Essa decisão é de quem chama, não
    /// desta consulta.
    fn perfis_do_cliente(&self, client_id: &str) -> Result<Vec<PerfilIdentidade>>;

    /// Ativa um contexto, substituindo o anterior se houver (singleton —
    /// design.md: "contexto ativo persistido no SQLite").
    fn conectar(&self, contexto: ContextoAtivo) -> Result<()>;

    /// Encerra o contexto ativo. No-op se não houver um.
    fn desconectar(&self) -> Result<()>;

    /// O contexto ativo agora. `None` se nenhum.
    fn contexto_ativo(&self) -> Result<Option<ContextoAtivo>>;

    /// Registra metadados de uma credencial — nunca o valor (spec vault: "Só
    /// referência é persistida").
    fn registrar_credencial(&self, nova: NovaCredencialMetadados) -> Result<CredencialRegistrada>;

    /// Metadados de uma credencial por id, sem resolver o valor.
    fn credencial(&self, id: &str) -> Result<Option<CredencialRegistrada>>;

    fn listar_credenciais(&self) -> Result<Vec<CredencialRegistrada>>;

    /// Atualiza `last_used_at` após uma resolução bem-sucedida (spec vault:
    /// "Metadados de uso e expiração são rastreados").
    fn atualizar_ultimo_uso_credencial(&self, id: &str, em: Instante) -> Result<()>;

    /// Grava uma nota de memória. `Invalid` se `categoria = Decisao` sem
    /// `rationale` (spec memory-notes: "Decisão exige o porquê") — a
    /// recusa também acontece antes disso em `continuidade::registrar_nota`;
    /// aqui é a segunda barreira, não a única (defesa em profundidade barata).
    fn registrar_nota(&self, nova: NovaNota) -> Result<NotaDeMemoria>;

    /// Notas de um Context, mais recente primeiro. `client_id`/`project`
    /// obrigatórios — sem filtro opcional que alguém possa esquecer de
    /// aplicar (spec: "Isolamento entre Contexts por construção").
    fn notas_do_contexto(
        &self,
        client_id: &str,
        project: Option<&str>,
    ) -> Result<Vec<NotaDeMemoria>>;

    /// Marca `nota_id` como substituída por `superseded_by` -- nunca
    /// altera `texto`/`rationale`/`categoria` da nota original (D-14,
    /// continuity/memory-supersede).
    fn marcar_superseded(&self, nota_id: &str, superseded_by: &str) -> Result<()>;

    /// Grava o registro do run. Chamado antes de qualquer efeito colateral —
    /// D-12, design.md: "Run persistido antes de qualquer efeito colateral".
    fn criar_run(&self, novo: NovoRun) -> Result<RunRegistrado>;

    fn run(&self, id: &str) -> Result<Option<RunRegistrado>>;

    /// PID gravado assim que o processo do provider existe — não no momento
    /// da criação do run, que precede o processo (spec isolated-run).
    fn definir_pid_run(&self, run_id: &str, pid: u32) -> Result<()>;

    /// Caminho do worktree e branch gravados assim que `criar_worktree`
    /// termina — só existem depois de `criar_run` (D-12), mas precisam estar
    /// no banco para `recover`/`worktree list` funcionarem após a morte do
    /// processo `brian run` (o valor em memória não sobrevive ao crash).
    fn definir_worktree_run(&self, run_id: &str, worktree_path: &str, branch: &str) -> Result<()>;

    /// Atualiza status e, quando aplicável, `finished_at`/custo equivalente.
    /// `finished_at = None` mantém o run em aberto (uso interno de
    /// `definir_pid_run` não passa por aqui).
    fn atualizar_status_run(
        &self,
        run_id: &str,
        status: StatusRun,
        finished_at: Option<Instante>,
        custo_equivalente: Option<Money>,
    ) -> Result<()>;

    /// Runs com `status = EmExecucao` — o que `brian recover` varre em busca
    /// de órfãos (spec orphan-recovery).
    fn runs_em_execucao(&self) -> Result<Vec<RunRegistrado>>;

    /// Runs com `status = Abandonado` — o que `brian worktree list` mostra
    /// junto aos ativos (task 7.3).
    fn runs_abandonados(&self) -> Result<Vec<RunRegistrado>>;

    /// Runs `Concluido` ou `Falhou` de um cliente — histórico que alimenta
    /// `routing/historical-scoring` (nunca de outro cliente, spec: "Score é
    /// calculado sobre runs reais do cliente").
    fn runs_finalizados_do_cliente(&self, client_id: &str) -> Result<Vec<RunRegistrado>>;

    fn registrar_evento_run(&self, novo: NovoEvento) -> Result<()>;

    fn eventos_do_run(&self, run_id: &str) -> Result<Vec<EventoDeRun>>;

    /// Grava o workflow_run — antes de qualquer fase rodar (D-12).
    fn criar_workflow_run(&self, novo: NovoWorkflowRun) -> Result<WorkflowRunRegistrado>;

    fn workflow_run(&self, id: &str) -> Result<Option<WorkflowRunRegistrado>>;

    /// Atualiza fase atual, status, motivo de pausa, total de fases e,
    /// quando aplicável, `finished_at`.
    #[allow(clippy::too_many_arguments)]
    fn atualizar_workflow_run(
        &self,
        id: &str,
        current_phase: &str,
        status: StatusWorkflowRun,
        pause_reason: Option<&str>,
        total_phases: i64,
        finished_at: Option<Instante>,
    ) -> Result<()>;

    fn registrar_entrada_fase(&self, nova: NovaEntradaDeFase) -> Result<EntradaDeFase>;

    /// Fecha uma entrada de fase já registrada com o outcome e `ended_at`.
    fn concluir_entrada_fase(
        &self,
        entrada_id: &str,
        outcome: &str,
        ended_at: Instante,
    ) -> Result<()>;

    /// Histórico de fases de um workflow_run, em ordem (blueprint §15.6:
    /// phase_history).
    fn entradas_do_workflow_run(&self, workflow_run_id: &str) -> Result<Vec<EntradaDeFase>>;

    /// Grava a comparação — antes de qualquer candidato rodar (D-12).
    fn criar_comparacao(&self, nova: NovaComparacao) -> Result<ComparacaoRegistrada>;

    fn comparacao(&self, id: &str) -> Result<Option<ComparacaoRegistrada>>;

    fn registrar_candidato_comparacao(
        &self,
        novo: NovoCandidatoComparacao,
    ) -> Result<CandidatoComparacao>;

    fn candidatos_da_comparacao(&self, comparacao_id: &str) -> Result<Vec<CandidatoComparacao>>;

    /// Só grava `vencedor_provider_id` — spec: "Escolha do vencedor é
    /// sempre uma ação explícita separada".
    fn definir_vencedor_comparacao(
        &self,
        comparacao_id: &str,
        provider_id: &str,
        finished_at: Instante,
    ) -> Result<()>;

    /// Grava uma execução do experimento H-1 -- liga case_id + braço ao
    /// `run` real que os executou.
    fn registrar_execucao_experimento(
        &self,
        nova: NovaExecucaoExperimento,
    ) -> Result<ExecucaoExperimento>;

    /// Todas as execuções do experimento, opcionalmente filtradas por
    /// braço.
    fn execucoes_do_experimento(&self, braco: Option<&str>) -> Result<Vec<ExecucaoExperimento>>;
}

/// Um run a criar — `status` sempre nasce `EmExecucao`, `pid`/`finished_at`/
/// `custo_equivalente` sempre nascem ausentes (task 3.1).
#[derive(Debug, Clone)]
pub struct NovoRun {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub base_commit: String,
    pub worktree_path: String,
    pub branch: String,
    pub provider_id: String,
    pub started_at: Instante,
}

/// Um evento de run a gravar.
#[derive(Debug, Clone)]
pub struct NovoEvento {
    pub id: String,
    pub run_id: String,
    pub tipo: String,
    pub detalhe: Option<String>,
    pub ocorrido_em: Instante,
}

/// Um workflow_run a gravar — persistido antes de qualquer fase rodar
/// (D-12, mesma disciplina de `NovoRun`).
#[derive(Debug, Clone)]
pub struct NovoWorkflowRun {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub workflow_id: String,
    pub workflow_version: i64,
    pub definicao_json: String,
    pub tarefa: String,
    pub current_phase: String,
    pub started_at: Instante,
}

/// Uma entrada de fase a gravar.
#[derive(Debug, Clone)]
pub struct NovaEntradaDeFase {
    pub id: String,
    pub workflow_run_id: String,
    pub phase_id: String,
    pub run_id: Option<String>,
    pub entrada_numero: i64,
    pub started_at: Instante,
}

/// Uma comparação a criar — `vencedor_provider_id` sempre nasce ausente.
#[derive(Debug, Clone)]
pub struct NovaComparacao {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub tarefa: String,
    pub started_at: Instante,
}

/// Um candidato de comparação a gravar.
#[derive(Debug, Clone)]
pub struct NovoCandidatoComparacao {
    pub id: String,
    pub comparacao_id: String,
    pub provider_id: String,
    pub run_id: Option<String>,
}

/// Uma execução do experimento H-1 a gravar.
#[derive(Debug, Clone)]
pub struct NovaExecucaoExperimento {
    pub id: String,
    pub case_id: String,
    pub braco: String,
    pub run_id: String,
    pub started_at: Instante,
}

/// Uma nota de memória a gravar.
#[derive(Debug, Clone)]
pub struct NovaNota {
    pub id: String,
    pub client_id: String,
    pub project: Option<String>,
    pub categoria: CategoriaNota,
    pub texto: String,
    pub rationale: Option<String>,
    pub created_at: Instante,
}

#[cfg(test)]
pub(crate) mod test_util {
    use super::*;

    pub fn novo_consumo(dedup_key: &str, provider: &str, model: &str) -> NovoConsumo {
        NovoConsumo {
            dedup_key: dedup_key.to_string(),
            provider_id: provider.to_string(),
            model: model.to_string(),
            tokens: Tokens {
                input: Some(100),
                cache: None,
                output: Some(50),
                reasoning: None,
            },
            custo_pago: None,
            custo_equivalente_api: None,
            billing_mode: BillingMode::Api,
            usage_source: UsageSource::Provider,
            cost_source: CostSource::Unknown,
            client_id: None,
            occurred_at: Instante(1_700_000_000),
        }
    }
}
