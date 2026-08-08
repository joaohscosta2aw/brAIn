//! Fronteira de armazenamento.
//!
//! **D-9: todo SQL vive aqui dentro.** Nenhum outro módulo pode conter consulta,
//! nome de tabela ou detalhe de esquema — o restante do sistema conversa apenas
//! com as traits definidas neste módulo.
//!
//! `scripts/verificar-invariantes.sh` verifica isso mecanicamente a cada push.

pub mod sqlite;

use crate::domain::{BillingMode, CostSource, Instante, Money, Tokens, UsageRecord, UsageSource};
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
