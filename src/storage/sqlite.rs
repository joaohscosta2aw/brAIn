//! Implementação SQLite da fronteira de armazenamento (D-1).
//!
//! Único lugar do repositório que fala SQL (D-9) — `scripts/verificar-invariantes.sh`
//! garante isso a cada push.

use super::{
    EntradaCatalogo, NovoConsumo, Periodo, Result, Revisao, StorageError, Store,
    ViolacaoIntegridade,
};
use crate::domain::{
    AttributionStatus, BillingMode, CostSource, Custo, Instante, Money, Tokens, UsageRecord,
    UsageSource,
};
use rusqlite::{Connection, OptionalExtension, Row, params};
use std::sync::Mutex;

/// Migrações aplicadas em ordem. Cada uma é aplicada no máximo uma vez — a
/// tabela `schema_migration` registra o que já rodou (task 1.3: "idempotentes
/// na reexecução").
const MIGRATIONS: &[(i64, &str)] = &[(1, include_str!("migrations/0001_inicial.sql"))];

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    /// Abre (ou cria) o banco no caminho dado. `:memory:` para testes.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path).map_err(|e| StorageError::Backend(e.to_string()))?;
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        // Envenenamento só ocorre se uma chamada anterior entrar em panic
        // segurando o lock — nesse caso o processo já está em estado inválido
        // e propagar o panic é mais seguro que continuar com dado suspeito.
        self.conn.lock().expect("lock do SQLite envenenado")
    }
}

impl Store for SqliteStore {
    fn migrate(&self) -> Result<()> {
        let conn = self.conn();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migration (
                version INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            );",
        )
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        for (version, sql) in MIGRATIONS {
            let ja_aplicada: bool = conn
                .query_row(
                    "SELECT 1 FROM schema_migration WHERE version = ?1",
                    params![version],
                    |_| Ok(true),
                )
                .optional()
                .map_err(|e| StorageError::Backend(e.to_string()))?
                .unwrap_or(false);

            if ja_aplicada {
                continue;
            }

            conn.execute_batch(sql)
                .map_err(|e| StorageError::Backend(e.to_string()))?;
            conn.execute(
                "INSERT INTO schema_migration (version, applied_at) VALUES (?1, unixepoch())",
                params![version],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        }

        Ok(())
    }

    fn upsert_client(&self, client_id: &str) -> Result<()> {
        self.conn()
            .execute(
                "INSERT OR IGNORE INTO client (id) VALUES (?1)",
                params![client_id],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(())
    }

    fn client_exists(&self, client_id: &str) -> Result<bool> {
        self.conn()
            .query_row(
                "SELECT 1 FROM client WHERE id = ?1",
                params![client_id],
                |_| Ok(true),
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
            .map(|r| r.unwrap_or(false))
    }

    fn gravar_consumo(&self, novo: NovoConsumo) -> Result<UsageRecord> {
        let conn = self.conn();

        conn.execute(
            "INSERT OR IGNORE INTO provider (id) VALUES (?1)",
            params![novo.provider_id],
        )
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        if let Some(client_id) = &novo.client_id {
            conn.execute(
                "INSERT OR IGNORE INTO client (id) VALUES (?1)",
                params![client_id],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        }

        let attribution_status = if novo.client_id.is_some() {
            AttributionStatus::Attributed
        } else {
            AttributionStatus::Unattributed
        };

        // dedup_key é a chave de idempotência (spec: "reimportar janela já
        // coberta não cria duplicata"). Usá-la também como id evita inventar
        // um gerador de identificador antes de existir necessidade real.
        conn.execute(
            "INSERT INTO usage_record (
                id, dedup_key, provider_id, model,
                tokens_input, tokens_cache, tokens_output, tokens_reasoning,
                custo_pago_micros, custo_equivalente_micros,
                billing_mode, usage_source, cost_source,
                client_id, attribution_status, occurred_at
            ) VALUES (?1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT (dedup_key) DO NOTHING",
            params![
                novo.dedup_key,
                novo.provider_id,
                novo.model,
                novo.tokens.input,
                novo.tokens.cache,
                novo.tokens.output,
                novo.tokens.reasoning,
                novo.custo_pago.map(|m| m.0),
                novo.custo_equivalente_api.map(|m| m.0),
                billing_mode_to_str(novo.billing_mode),
                usage_source_to_str(novo.usage_source),
                cost_source_to_str(novo.cost_source),
                novo.client_id,
                attribution_status_to_str(attribution_status),
                novo.occurred_at.0,
            ],
        )
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        conn.query_row(
            "SELECT * FROM usage_record WHERE dedup_key = ?1",
            params![novo.dedup_key],
            row_to_usage_record,
        )
        .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn atribuir(&self, usage_record_id: &str, client_id: &str) -> Result<UsageRecord> {
        let conn = self.conn();

        let cliente_existe: bool = conn
            .query_row(
                "SELECT 1 FROM client WHERE id = ?1",
                params![client_id],
                |_| Ok(true),
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))?
            .unwrap_or(false);
        if !cliente_existe {
            return Err(StorageError::NotFound(format!("cliente {client_id}")));
        }

        // Lida antes de sobrescrever: se já havia um cliente, essa reatribuição
        // precisa deixar rastro (task 4.6, D-14).
        let cliente_anterior: Option<String> = conn
            .query_row(
                "SELECT client_id FROM usage_record WHERE id = ?1",
                params![usage_record_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))?
            .ok_or_else(|| StorageError::NotFound(format!("usage_record {usage_record_id}")))?;

        conn.execute(
            "UPDATE usage_record
             SET client_id = ?1, attribution_status = 'attributed'
             WHERE id = ?2",
            params![client_id, usage_record_id],
        )
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        if let Some(anterior) = cliente_anterior {
            registrar_revisao(&conn, usage_record_id, "client_id", Some(&anterior))?;
        }

        conn.query_row(
            "SELECT * FROM usage_record WHERE id = ?1",
            params![usage_record_id],
            row_to_usage_record,
        )
        .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn atualizar_custo_pago(&self, usage_record_id: &str, pago: Money) -> Result<UsageRecord> {
        let conn = self.conn();

        let anterior: Option<i64> = conn
            .query_row(
                "SELECT custo_pago_micros FROM usage_record WHERE id = ?1",
                params![usage_record_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))?
            .ok_or_else(|| StorageError::NotFound(format!("usage_record {usage_record_id}")))?;

        conn.execute(
            "UPDATE usage_record
             SET custo_pago_micros = ?1, cost_source = 'provider'
             WHERE id = ?2",
            params![pago.0, usage_record_id],
        )
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        registrar_revisao(
            &conn,
            usage_record_id,
            "custo_pago_micros",
            anterior.map(|v| v.to_string()).as_deref(),
        )?;

        conn.query_row(
            "SELECT * FROM usage_record WHERE id = ?1",
            params![usage_record_id],
            row_to_usage_record,
        )
        .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn historico(&self, usage_record_id: &str) -> Result<Vec<Revisao>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT campo, valor_anterior, revisado_em
                 FROM usage_record_revisao
                 WHERE usage_record_id = ?1
                 ORDER BY id DESC",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![usage_record_id], |row| {
                Ok(Revisao {
                    campo: row.get(0)?,
                    valor_anterior: row.get(1)?,
                    revisado_em: Instante(row.get(2)?),
                })
            })
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn nao_atribuidos(&self, periodo: Periodo) -> Result<Vec<UsageRecord>> {
        self.consultar_periodo(
            "SELECT * FROM usage_record
             WHERE attribution_status = 'unattributed'
               AND occurred_at >= ?1 AND (?2 IS NULL OR occurred_at < ?2)
             ORDER BY occurred_at",
            periodo,
        )
    }

    fn consumo_do_cliente(&self, client_id: &str, periodo: Periodo) -> Result<Vec<UsageRecord>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT * FROM usage_record
                 WHERE client_id = ?1
                   AND occurred_at >= ?2 AND (?3 IS NULL OR occurred_at < ?3)
                 ORDER BY occurred_at",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(
                params![client_id, periodo.desde.0, periodo.ate.map(|i| i.0)],
                row_to_usage_record,
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn consumo_no_periodo(&self, periodo: Periodo) -> Result<Vec<UsageRecord>> {
        self.consultar_periodo(
            "SELECT * FROM usage_record
             WHERE occurred_at >= ?1 AND (?2 IS NULL OR occurred_at < ?2)
             ORDER BY occurred_at",
            periodo,
        )
    }

    fn verificar_integridade(&self) -> Result<Vec<ViolacaoIntegridade>> {
        let todos = self.consumo_no_periodo(Periodo {
            desde: Instante(i64::MIN),
            ate: None,
        })?;

        Ok(todos
            .iter()
            .flat_map(|r| {
                r.violacoes()
                    .into_iter()
                    .map(|descricao| ViolacaoIntegridade {
                        usage_record_id: r.id.clone(),
                        descricao: descricao.to_string(),
                    })
            })
            .collect())
    }

    fn upsert_catalogo(&self, entrada: EntradaCatalogo) -> Result<()> {
        let conn = self.conn();

        // Fecha qualquer entrada ainda vigente do mesmo modelo cuja vigência
        // começou antes desta — não apaga histórico, só encerra o intervalo
        // (design.md: "reproduzível após atualização de preços").
        conn.execute(
            "UPDATE price_catalog
             SET vigente_ate = ?1
             WHERE model = ?2 AND vigente_ate IS NULL AND vigente_desde < ?1",
            params![entrada.vigente_desde.0, entrada.model],
        )
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        conn.execute(
            "INSERT INTO price_catalog (model, preco_micros, vigente_desde, vigente_ate)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (model, vigente_desde) DO UPDATE SET
                preco_micros = excluded.preco_micros,
                vigente_ate = excluded.vigente_ate",
            params![
                entrada.model,
                entrada.preco_por_1k_tokens.0,
                entrada.vigente_desde.0,
                entrada.vigente_ate.map(|i| i.0),
            ],
        )
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(())
    }

    fn ultimo_consumo_importado(&self, provider_id: &str) -> Result<Option<Instante>> {
        self.conn()
            .query_row(
                "SELECT MAX(occurred_at) FROM usage_record WHERE provider_id = ?1",
                params![provider_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
            .map(|r| r.flatten().map(Instante))
    }

    fn preco_vigente(&self, model: &str, em: Instante) -> Result<Option<EntradaCatalogo>> {
        self.conn()
            .query_row(
                "SELECT model, preco_micros, vigente_desde, vigente_ate
                 FROM price_catalog
                 WHERE model = ?1 AND vigente_desde <= ?2
                   AND (vigente_ate IS NULL OR vigente_ate > ?2)
                 ORDER BY vigente_desde DESC LIMIT 1",
                params![model, em.0],
                |row| {
                    Ok(EntradaCatalogo {
                        model: row.get(0)?,
                        preco_por_1k_tokens: Money(row.get(1)?),
                        vigente_desde: Instante(row.get(2)?),
                        vigente_ate: row.get::<_, Option<i64>>(3)?.map(Instante),
                    })
                },
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }
}

impl SqliteStore {
    fn consultar_periodo(&self, sql: &str, periodo: Periodo) -> Result<Vec<UsageRecord>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(
                params![periodo.desde.0, periodo.ate.map(|i| i.0)],
                row_to_usage_record,
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }
}

fn registrar_revisao(
    conn: &Connection,
    usage_record_id: &str,
    campo: &str,
    valor_anterior: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO usage_record_revisao (usage_record_id, campo, valor_anterior, revisado_em)
         VALUES (?1, ?2, ?3, unixepoch())",
        params![usage_record_id, campo, valor_anterior],
    )
    .map_err(|e| StorageError::Backend(e.to_string()))?;
    Ok(())
}

fn row_to_usage_record(row: &Row) -> rusqlite::Result<UsageRecord> {
    Ok(UsageRecord {
        id: row.get("id")?,
        provider_id: row.get("provider_id")?,
        model: row.get("model")?,
        tokens: Tokens {
            input: row.get("tokens_input")?,
            cache: row.get("tokens_cache")?,
            output: row.get("tokens_output")?,
            reasoning: row.get("tokens_reasoning")?,
        },
        custo: Custo {
            pago: row.get::<_, Option<i64>>("custo_pago_micros")?.map(Money),
            equivalente_api: row
                .get::<_, Option<i64>>("custo_equivalente_micros")?
                .map(Money),
        },
        billing_mode: str_to_billing_mode(&row.get::<_, String>("billing_mode")?),
        usage_source: str_to_usage_source(&row.get::<_, String>("usage_source")?),
        cost_source: str_to_cost_source(&row.get::<_, String>("cost_source")?),
        client_id: row.get("client_id")?,
        attribution_status: str_to_attribution_status(&row.get::<_, String>("attribution_status")?),
        occurred_at: Instante(row.get("occurred_at")?),
    })
}

fn billing_mode_to_str(m: BillingMode) -> &'static str {
    match m {
        BillingMode::Api => "api",
        BillingMode::Subscription => "subscription",
        BillingMode::Credits => "credits",
        BillingMode::Mixed => "mixed",
        BillingMode::Unknown => "unknown",
    }
}

fn str_to_billing_mode(s: &str) -> BillingMode {
    match s {
        "api" => BillingMode::Api,
        "subscription" => BillingMode::Subscription,
        "credits" => BillingMode::Credits,
        "mixed" => BillingMode::Mixed,
        _ => BillingMode::Unknown,
    }
}

fn usage_source_to_str(s: UsageSource) -> &'static str {
    match s {
        UsageSource::Provider => "provider",
        UsageSource::BrianMeasured => "brian_measured",
        UsageSource::Estimated => "estimated",
    }
}

fn str_to_usage_source(s: &str) -> UsageSource {
    match s {
        "provider" => UsageSource::Provider,
        "brian_measured" => UsageSource::BrianMeasured,
        _ => UsageSource::Estimated,
    }
}

fn cost_source_to_str(s: CostSource) -> &'static str {
    match s {
        CostSource::Provider => "provider",
        CostSource::Catalog => "catalog",
        CostSource::Unknown => "unknown",
    }
}

fn str_to_cost_source(s: &str) -> CostSource {
    match s {
        "provider" => CostSource::Provider,
        "catalog" => CostSource::Catalog,
        _ => CostSource::Unknown,
    }
}

fn attribution_status_to_str(s: AttributionStatus) -> &'static str {
    match s {
        AttributionStatus::Attributed => "attributed",
        AttributionStatus::Unattributed => "unattributed",
    }
}

fn str_to_attribution_status(s: &str) -> AttributionStatus {
    match s {
        "attributed" => AttributionStatus::Attributed,
        _ => AttributionStatus::Unattributed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_util::novo_consumo;

    fn store() -> SqliteStore {
        let s = SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s
    }

    #[test]
    fn migrate_e_idempotente() {
        let s = store();
        s.migrate().unwrap();
        s.migrate().unwrap();
    }

    #[test]
    fn gravar_consumo_e_le_de_volta() {
        let s = store();
        let r = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        assert_eq!(r.provider_id, "claude");
        assert_eq!(r.attribution_status, AttributionStatus::Unattributed);
    }

    #[test]
    fn provider_reporta_consumo_completo_todos_os_campos_voltam() {
        // Spec usage-ledger, cenário "Provider reporta consumo completo":
        // um usage_record é criado com TODOS os campos preenchidos e
        // occurred_at do momento da chamada -- não só 2 campos, os 4
        // categorias de token, modelo e instante inteiros.
        let s = store();
        let mut consumo = novo_consumo("k1", "claude", "opus");
        consumo.tokens = Tokens {
            input: Some(100),
            cache: Some(50),
            output: Some(30),
            reasoning: Some(10),
        };
        consumo.occurred_at = Instante(1_700_000_123);

        let r = s.gravar_consumo(consumo).unwrap();

        assert_eq!(r.model, "opus");
        assert_eq!(r.tokens.input, Some(100));
        assert_eq!(r.tokens.cache, Some(50));
        assert_eq!(r.tokens.output, Some(30));
        assert_eq!(r.tokens.reasoning, Some(10));
        assert_eq!(r.occurred_at, Instante(1_700_000_123));
    }

    #[test]
    fn usage_source_brian_measured_sobrevive_ao_roundtrip() {
        // Spec: "Tokens medidos pelo Brian" -- usage_source=brian_measured
        // quando o provider não reporta contagens diretamente.
        let s = store();
        let mut consumo = novo_consumo("k1", "claude", "opus");
        consumo.usage_source = UsageSource::BrianMeasured;
        let r = s.gravar_consumo(consumo).unwrap();
        assert_eq!(r.usage_source, UsageSource::BrianMeasured);
    }

    #[test]
    fn dedup_key_repetida_nao_duplica() {
        let s = store();
        let a = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        let b = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        assert_eq!(a.id, b.id);

        let todos = s
            .consumo_no_periodo(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();
        assert_eq!(todos.len(), 1, "reimportar a mesma chave não duplica");
    }

    #[test]
    fn atribuir_a_cliente_inexistente_nao_altera_registro() {
        let s = store();
        let r = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        let err = s.atribuir(&r.id, "fantasma").unwrap_err();
        assert!(matches!(err, StorageError::NotFound(_)));

        let ainda = s
            .nao_atribuidos(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();
        assert_eq!(ainda.len(), 1, "registro continua não-atribuído");
    }

    #[test]
    fn atribuir_a_cliente_existente_funciona() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let r = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        let atualizado = s.atribuir(&r.id, "xpto").unwrap();
        assert_eq!(atualizado.client_id.as_deref(), Some("xpto"));
        assert_eq!(atualizado.attribution_status, AttributionStatus::Attributed);
    }

    #[test]
    fn isolamento_entre_clientes() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();
        let a = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        let b = s
            .gravar_consumo(novo_consumo("k2", "claude", "opus"))
            .unwrap();
        s.atribuir(&a.id, "xpto").unwrap();
        s.atribuir(&b.id, "acme").unwrap();

        let periodo = Periodo {
            desde: Instante(0),
            ate: None,
        };
        let de_xpto = s.consumo_do_cliente("xpto", periodo).unwrap();
        assert_eq!(de_xpto.len(), 1);
        assert_eq!(de_xpto[0].client_id.as_deref(), Some("xpto"));
    }

    #[test]
    fn catalogo_versionado_preserva_preco_historico() {
        let s = store();
        s.upsert_catalogo(EntradaCatalogo {
            model: "opus".into(),
            preco_por_1k_tokens: Money(15_000_000),
            vigente_desde: Instante(1000),
            vigente_ate: None,
        })
        .unwrap();
        s.upsert_catalogo(EntradaCatalogo {
            model: "opus".into(),
            preco_por_1k_tokens: Money(20_000_000),
            vigente_desde: Instante(2000),
            vigente_ate: None,
        })
        .unwrap();

        let antigo = s.preco_vigente("opus", Instante(1500)).unwrap().unwrap();
        assert_eq!(antigo.preco_por_1k_tokens, Money(15_000_000));

        let novo = s.preco_vigente("opus", Instante(2500)).unwrap().unwrap();
        assert_eq!(novo.preco_por_1k_tokens, Money(20_000_000));
    }

    #[test]
    fn ausente_sobrevive_ao_roundtrip_do_banco() {
        // Task 2.2: ausente e zero sao fatos distintos. Prova que o banco nao
        // colapsa NULL em 0 na volta -- nao so a memoria (ja coberto em
        // domain.rs), mas o caminho real de persistencia.
        let s = store();
        let mut consumo = novo_consumo("k1", "claude", "opus");
        consumo.tokens.reasoning = None;
        consumo.tokens.output = Some(0);
        let gravado = s.gravar_consumo(consumo).unwrap();

        assert_eq!(gravado.tokens.reasoning, None, "ausente permanece ausente");
        assert_eq!(
            gravado.tokens.output,
            Some(0),
            "zero permanece zero, nao vira ausente"
        );
    }

    #[test]
    fn custo_pago_tardio_preserva_valor_anterior_recuperavel() {
        let s = store();
        let mut consumo = novo_consumo("k1", "claude", "opus");
        consumo.custo_equivalente_api = Some(Money(15_000_000));
        let r = s.gravar_consumo(consumo).unwrap();
        assert_eq!(r.custo.pago, None);

        let atualizado = s.atualizar_custo_pago(&r.id, Money(14_500_000)).unwrap();
        assert_eq!(atualizado.custo.pago, Some(Money(14_500_000)));
        assert_eq!(atualizado.cost_source, CostSource::Provider);
        // O equivalente não foi apagado pela chegada do custo pago.
        assert_eq!(atualizado.custo.equivalente_api, Some(Money(15_000_000)));

        let historico = s.historico(&r.id).unwrap();
        assert_eq!(historico.len(), 1);
        assert_eq!(historico[0].campo, "custo_pago_micros");
        assert_eq!(
            historico[0].valor_anterior, None,
            "não havia valor pago antes"
        );
    }

    #[test]
    fn reatribuicao_preserva_cliente_anterior_recuperavel() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();
        let r = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        s.atribuir(&r.id, "xpto").unwrap();
        let reatribuido = s.atribuir(&r.id, "acme").unwrap();

        assert_eq!(reatribuido.client_id.as_deref(), Some("acme"));

        let historico = s.historico(&r.id).unwrap();
        assert_eq!(historico.len(), 1);
        assert_eq!(historico[0].campo, "client_id");
        assert_eq!(historico[0].valor_anterior.as_deref(), Some("xpto"));
    }

    #[test]
    fn atualizar_custo_de_registro_inexistente_e_notfound() {
        let s = store();
        let err = s
            .atualizar_custo_pago("fantasma", Money(1_000_000))
            .unwrap_err();
        assert!(matches!(err, StorageError::NotFound(_)));
    }

    #[test]
    fn consulta_de_nao_atribuidos_lista_registros_orfaos() {
        let s = store();
        s.gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        s.gravar_consumo(novo_consumo("k2", "codex", "gpt"))
            .unwrap();

        let orfaos = s
            .nao_atribuidos(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();
        assert_eq!(orfaos.len(), 2);
        assert!(orfaos.iter().all(|r| r.client_id.is_none()));
    }

    #[test]
    fn ledger_integro_nao_tem_consumo_sem_dono() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let r = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        s.atribuir(&r.id, "xpto").unwrap();

        let orfaos = s
            .nao_atribuidos(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();
        assert!(
            orfaos.is_empty(),
            "ledger totalmente atribuído não deve soar alarme"
        );
    }

    #[test]
    fn verificar_integridade_detecta_violacao() {
        let s = store();
        // Insere direto via SQL para simular um dado corrompido que o
        // caminho normal de gravar_consumo não produziria.
        s.conn()
            .execute("INSERT INTO provider (id) VALUES ('claude')", params![])
            .unwrap();
        s.conn()
            .execute(
                "INSERT INTO usage_record (
                    id, dedup_key, provider_id, model, billing_mode,
                    usage_source, cost_source, client_id, attribution_status, occurred_at
                ) VALUES ('r1','r1','claude','m','api','provider','unknown',NULL,'attributed',0)",
                params![],
            )
            .unwrap();

        let violacoes = s.verificar_integridade().unwrap();
        assert!(violacoes.iter().any(|v| v.usage_record_id == "r1"));
    }
}
