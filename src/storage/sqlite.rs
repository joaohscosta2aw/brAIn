//! Implementação SQLite da fronteira de armazenamento (D-1).
//!
//! Único lugar do repositório que fala SQL (D-9) — `scripts/verificar-invariantes.sh`
//! garante isso a cada push.

use super::{
    EntradaCatalogo, NovaComparacao, NovaCredencialMetadados, NovaEntradaDeFase,
    NovaExecucaoExperimento, NovaNota, NovoCandidatoComparacao, NovoConsumo, NovoEvento,
    NovoPerfil, NovoPlano, NovoQuotaSignal, NovoRun, NovoWorkflowRun, Periodo, PlanoRegistrado,
    QuotaSignalRegistrado, Result, Revisao, StorageError, Store, ViolacaoIntegridade,
};
use crate::domain::{
    AttributionStatus, BillingMode, CandidatoComparacao, CategoriaNota, ClasseSecret,
    ComparacaoRegistrada, ContextoAtivo, CostSource, CredencialRegistrada, Custo, EntradaDeFase,
    EventoDeRun, ExecucaoExperimento, Instante, Money, NotaDeMemoria, PerfilIdentidade,
    ProviderBinding, RunRegistrado, StatusRun, StatusWorkflowRun, Tokens, UsageRecord, UsageSource,
    WorkflowRunRegistrado,
};
use rusqlite::{Connection, OptionalExtension, Row, params};
use std::sync::Mutex;

/// Migrações aplicadas em ordem. Cada uma é aplicada no máximo uma vez — a
/// tabela `schema_migration` registra o que já rodou (task 1.3: "idempotentes
/// na reexecução").
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("migrations/0001_inicial.sql")),
    (2, include_str!("migrations/0002_capacidade.sql")),
    (3, include_str!("migrations/0003_identidade.sql")),
    (4, include_str!("migrations/0004_continuidade.sql")),
    (5, include_str!("migrations/0005_execucao.sql")),
    (6, include_str!("migrations/0006_workflow.sql")),
    (7, include_str!("migrations/0007_comparacao.sql")),
    (8, include_str!("migrations/0008_experimento.sql")),
    (9, include_str!("migrations/0009_memoria_supersede.sql")),
];

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

    fn registrar_plano(&self, novo: NovoPlano) -> Result<()> {
        let conn = self.conn();

        let atual: Option<(String, Option<String>)> = conn
            .query_row(
                "SELECT billing_mode, plan_label FROM provider_plan
                 WHERE provider_id = ?1 AND ativo_ate IS NULL",
                params![novo.provider_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        let mudou = match &atual {
            None => true,
            Some((bm, label)) => {
                bm.as_str() != billing_mode_to_str(novo.billing_mode)
                    || label.as_deref() != novo.plan_label.as_deref()
            }
        };
        if !mudou {
            // Plano confirmado igual ao vigente: não abre nova vigência, mas
            // registra que a fonte foi consultada agora com sucesso (spec:
            // "identifica a informação como potencialmente desatualizada" —
            // sem isto, uma fonte que passa a falhar silenciosamente nunca
            // deixaria rastro de que o valor exibido está velho).
            if atual.is_some() {
                conn.execute(
                    "UPDATE provider_plan SET verificado_em = ?1
                     WHERE provider_id = ?2 AND ativo_ate IS NULL",
                    params![novo.detectado_em.0, novo.provider_id],
                )
                .map_err(|e| StorageError::Backend(e.to_string()))?;
            }
            return Ok(());
        }

        if atual.is_some() {
            conn.execute(
                "UPDATE provider_plan SET ativo_ate = ?1
                 WHERE provider_id = ?2 AND ativo_ate IS NULL",
                params![novo.detectado_em.0, novo.provider_id],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        }

        conn.execute(
            "INSERT INTO provider_plan
                (provider_id, billing_mode, plan_label, ativo_desde, ativo_ate, verificado_em)
             VALUES (?1, ?2, ?3, ?4, NULL, ?4)",
            params![
                novo.provider_id,
                billing_mode_to_str(novo.billing_mode),
                novo.plan_label,
                novo.detectado_em.0,
            ],
        )
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(())
    }

    fn plano_vigente(&self, provider_id: &str) -> Result<Option<PlanoRegistrado>> {
        self.conn()
            .query_row(
                "SELECT provider_id, billing_mode, plan_label, ativo_desde, ativo_ate, verificado_em
                 FROM provider_plan WHERE provider_id = ?1 AND ativo_ate IS NULL",
                params![provider_id],
                row_to_plano,
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn plano_vigente_em(&self, provider_id: &str, em: Instante) -> Result<Option<PlanoRegistrado>> {
        self.conn()
            .query_row(
                "SELECT provider_id, billing_mode, plan_label, ativo_desde, ativo_ate, verificado_em
                 FROM provider_plan
                 WHERE provider_id = ?1 AND ativo_desde <= ?2
                   AND (ativo_ate IS NULL OR ativo_ate > ?2)
                 ORDER BY ativo_desde DESC LIMIT 1",
                params![provider_id, em.0],
                row_to_plano,
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn upsert_quota_signal(&self, sinal: NovoQuotaSignal) -> Result<()> {
        self.conn()
            .execute(
                "INSERT INTO quota_signal
                    (provider_id, bucket_id, grupo, remaining_percent, reset_at, observed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT (provider_id, bucket_id) DO UPDATE SET
                    grupo = excluded.grupo,
                    remaining_percent = excluded.remaining_percent,
                    reset_at = excluded.reset_at,
                    observed_at = excluded.observed_at",
                params![
                    sinal.provider_id,
                    sinal.bucket_id,
                    sinal.grupo,
                    sinal.remaining_percent,
                    sinal.reset_at.map(|i| i.0),
                    sinal.observed_at.0,
                ],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(())
    }

    fn quota_signals(&self, provider_id: &str) -> Result<Vec<QuotaSignalRegistrado>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT provider_id, bucket_id, grupo, remaining_percent, reset_at, observed_at
                 FROM quota_signal WHERE provider_id = ?1
                 ORDER BY bucket_id",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![provider_id], row_to_quota_signal)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn criar_perfil(&self, novo: NovoPerfil) -> Result<PerfilIdentidade> {
        let conn = self.conn();

        conn.execute(
            "INSERT INTO identity_profile
                (id, client_id, project, git_author_name, git_author_email, github_org, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                novo.id,
                novo.client_id,
                novo.project,
                novo.git_author_name,
                novo.git_author_email,
                novo.github_org,
                novo.created_at.0,
            ],
        )
        .map_err(|e| StorageError::Invalid(e.to_string()))?;

        for binding in &novo.bindings {
            conn.execute(
                "INSERT INTO provider_binding (identity_profile_id, provider_id, config_home)
                 VALUES (?1, ?2, ?3)",
                params![novo.id, binding.provider_id, binding.config_home],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        }

        Ok(PerfilIdentidade {
            id: novo.id,
            client_id: novo.client_id,
            project: novo.project,
            git_author_name: novo.git_author_name,
            git_author_email: novo.git_author_email,
            github_org: novo.github_org,
            bindings: novo.bindings,
        })
    }

    fn perfil(&self, id: &str) -> Result<Option<PerfilIdentidade>> {
        let conn = self.conn();
        let base = conn
            .query_row(
                "SELECT id, client_id, project, git_author_name, git_author_email, github_org
                 FROM identity_profile WHERE id = ?1",
                params![id],
                row_to_perfil_base,
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        let Some(mut perfil) = base else {
            return Ok(None);
        };
        perfil.bindings = self.bindings_do_perfil(&conn, id)?;
        Ok(Some(perfil))
    }

    fn perfis_do_cliente(&self, client_id: &str) -> Result<Vec<PerfilIdentidade>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, client_id, project, git_author_name, git_author_email, github_org
                 FROM identity_profile WHERE client_id = ?1 ORDER BY project",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![client_id], row_to_perfil_base)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let mut perfis = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        for perfil in &mut perfis {
            perfil.bindings = self.bindings_do_perfil(&conn, &perfil.id)?;
        }
        Ok(perfis)
    }

    fn conectar(&self, contexto: ContextoAtivo) -> Result<()> {
        self.conn()
            .execute(
                "INSERT INTO active_context (id, client_id, project, identity_profile_id, connected_at)
                 VALUES (1, ?1, ?2, ?3, ?4)
                 ON CONFLICT (id) DO UPDATE SET
                    client_id = excluded.client_id,
                    project = excluded.project,
                    identity_profile_id = excluded.identity_profile_id,
                    connected_at = excluded.connected_at",
                params![
                    contexto.client_id,
                    contexto.project,
                    contexto.identity_profile_id,
                    contexto.connected_at.0,
                ],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(())
    }

    fn desconectar(&self) -> Result<()> {
        self.conn()
            .execute("DELETE FROM active_context WHERE id = 1", params![])
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(())
    }

    fn contexto_ativo(&self) -> Result<Option<ContextoAtivo>> {
        self.conn()
            .query_row(
                "SELECT client_id, project, identity_profile_id, connected_at
                 FROM active_context WHERE id = 1",
                params![],
                |row| {
                    Ok(ContextoAtivo {
                        client_id: row.get(0)?,
                        project: row.get(1)?,
                        identity_profile_id: row.get(2)?,
                        connected_at: Instante(row.get(3)?),
                    })
                },
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn registrar_credencial(&self, nova: NovaCredencialMetadados) -> Result<CredencialRegistrada> {
        self.conn()
            .execute(
                "INSERT INTO credential_ref
                    (id, label, keychain_service, keychain_account, class, created_at, expires_at, rotation_policy)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    nova.id,
                    nova.label,
                    nova.keychain_service,
                    nova.keychain_account,
                    classe_secret_to_str(nova.class),
                    nova.created_at.0,
                    nova.expires_at.map(|i| i.0),
                    nova.rotation_policy,
                ],
            )
            .map_err(|e| StorageError::Invalid(e.to_string()))?;

        Ok(CredencialRegistrada {
            id: nova.id,
            label: nova.label,
            keychain_service: nova.keychain_service,
            keychain_account: nova.keychain_account,
            class: nova.class,
            created_at: nova.created_at,
            expires_at: nova.expires_at,
            last_used_at: None,
            rotation_policy: nova.rotation_policy,
        })
    }

    fn credencial(&self, id: &str) -> Result<Option<CredencialRegistrada>> {
        self.conn()
            .query_row(
                "SELECT id, label, keychain_service, keychain_account, class, created_at,
                        expires_at, last_used_at, rotation_policy
                 FROM credential_ref WHERE id = ?1",
                params![id],
                row_to_credencial,
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn listar_credenciais(&self) -> Result<Vec<CredencialRegistrada>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, label, keychain_service, keychain_account, class, created_at,
                        expires_at, last_used_at, rotation_policy
                 FROM credential_ref ORDER BY created_at",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![], row_to_credencial)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn atualizar_ultimo_uso_credencial(&self, id: &str, em: Instante) -> Result<()> {
        let alterado = self
            .conn()
            .execute(
                "UPDATE credential_ref SET last_used_at = ?1 WHERE id = ?2",
                params![em.0, id],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        if alterado == 0 {
            return Err(StorageError::NotFound(format!("credencial {id}")));
        }
        Ok(())
    }

    fn registrar_nota(&self, nova: NovaNota) -> Result<NotaDeMemoria> {
        if matches!(nova.categoria, CategoriaNota::Decisao) && nova.rationale.is_none() {
            return Err(StorageError::Invalid("decisão exige rationale".to_string()));
        }

        self.conn()
            .execute(
                "INSERT INTO memory_note (id, client_id, project, categoria, texto, rationale, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    nova.id,
                    nova.client_id,
                    nova.project,
                    categoria_nota_to_str(nova.categoria),
                    nova.texto,
                    nova.rationale,
                    nova.created_at.0,
                ],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(NotaDeMemoria {
            id: nova.id,
            client_id: nova.client_id,
            project: nova.project,
            categoria: nova.categoria,
            texto: nova.texto,
            rationale: nova.rationale,
            created_at: nova.created_at,
            superseded_by: None,
        })
    }

    fn marcar_superseded(&self, nota_id: &str, superseded_by: &str) -> Result<()> {
        let alterado = self
            .conn()
            .execute(
                "UPDATE memory_note SET superseded_by = ?1 WHERE id = ?2",
                params![superseded_by, nota_id],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        if alterado == 0 {
            return Err(StorageError::NotFound(format!("nota {nota_id}")));
        }
        Ok(())
    }

    fn notas_do_contexto(
        &self,
        client_id: &str,
        project: Option<&str>,
    ) -> Result<Vec<NotaDeMemoria>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, client_id, project, categoria, texto, rationale, created_at, superseded_by
                 FROM memory_note
                 WHERE client_id = ?1 AND project IS ?2
                 ORDER BY created_at DESC",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![client_id, project], row_to_nota)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn criar_run(&self, novo: NovoRun) -> Result<RunRegistrado> {
        self.conn()
            .execute(
                "INSERT INTO run
                    (id, client_id, project, base_commit, worktree_path, branch,
                     provider_id, pid, status, custo_equivalente_micros, started_at, finished_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, NULL, ?9, NULL)",
                params![
                    novo.id,
                    novo.client_id,
                    novo.project,
                    novo.base_commit,
                    novo.worktree_path,
                    novo.branch,
                    novo.provider_id,
                    status_run_to_str(StatusRun::EmExecucao),
                    novo.started_at.0,
                ],
            )
            .map_err(|e| StorageError::Invalid(e.to_string()))?;

        Ok(RunRegistrado {
            id: novo.id,
            client_id: novo.client_id,
            project: novo.project,
            base_commit: novo.base_commit,
            worktree_path: novo.worktree_path,
            branch: novo.branch,
            provider_id: novo.provider_id,
            pid: None,
            status: StatusRun::EmExecucao,
            custo_equivalente: None,
            started_at: novo.started_at,
            finished_at: None,
        })
    }

    fn run(&self, id: &str) -> Result<Option<RunRegistrado>> {
        self.conn()
            .query_row(
                "SELECT id, client_id, project, base_commit, worktree_path, branch,
                        provider_id, pid, status, custo_equivalente_micros, started_at, finished_at
                 FROM run WHERE id = ?1",
                params![id],
                row_to_run,
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn definir_pid_run(&self, run_id: &str, pid: u32) -> Result<()> {
        let alterado = self
            .conn()
            .execute(
                "UPDATE run SET pid = ?1 WHERE id = ?2",
                params![pid, run_id],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        if alterado == 0 {
            return Err(StorageError::NotFound(format!("run {run_id}")));
        }
        Ok(())
    }

    fn definir_worktree_run(&self, run_id: &str, worktree_path: &str, branch: &str) -> Result<()> {
        let alterado = self
            .conn()
            .execute(
                "UPDATE run SET worktree_path = ?1, branch = ?2 WHERE id = ?3",
                params![worktree_path, branch, run_id],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        if alterado == 0 {
            return Err(StorageError::NotFound(format!("run {run_id}")));
        }
        Ok(())
    }

    fn atualizar_status_run(
        &self,
        run_id: &str,
        status: StatusRun,
        finished_at: Option<Instante>,
        custo_equivalente: Option<Money>,
    ) -> Result<()> {
        let alterado = self
            .conn()
            .execute(
                "UPDATE run SET status = ?1, finished_at = ?2, custo_equivalente_micros = ?3
                 WHERE id = ?4",
                params![
                    status_run_to_str(status),
                    finished_at.map(|i| i.0),
                    custo_equivalente.map(|m| m.0),
                    run_id,
                ],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        if alterado == 0 {
            return Err(StorageError::NotFound(format!("run {run_id}")));
        }
        Ok(())
    }

    fn runs_em_execucao(&self) -> Result<Vec<RunRegistrado>> {
        self.runs_por_status(StatusRun::EmExecucao)
    }

    fn runs_abandonados(&self) -> Result<Vec<RunRegistrado>> {
        self.runs_por_status(StatusRun::Abandonado)
    }

    fn runs_finalizados_do_cliente(&self, client_id: &str) -> Result<Vec<RunRegistrado>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, client_id, project, base_commit, worktree_path, branch,
                        provider_id, pid, status, custo_equivalente_micros, started_at, finished_at
                 FROM run WHERE client_id = ?1 AND status IN (?2, ?3)",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(
                params![
                    client_id,
                    status_run_to_str(StatusRun::Concluido),
                    status_run_to_str(StatusRun::Falhou)
                ],
                row_to_run,
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn registrar_evento_run(&self, novo: NovoEvento) -> Result<()> {
        self.conn()
            .execute(
                "INSERT INTO run_event (id, run_id, tipo, detalhe, ocorrido_em)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    novo.id,
                    novo.run_id,
                    novo.tipo,
                    novo.detalhe,
                    novo.ocorrido_em.0
                ],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(())
    }

    fn eventos_do_run(&self, run_id: &str) -> Result<Vec<EventoDeRun>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT run_id, tipo, detalhe, ocorrido_em FROM run_event
                 WHERE run_id = ?1 ORDER BY ocorrido_em",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok(EventoDeRun {
                    run_id: row.get(0)?,
                    tipo: row.get(1)?,
                    detalhe: row.get(2)?,
                    ocorrido_em: Instante(row.get(3)?),
                })
            })
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn criar_workflow_run(&self, novo: NovoWorkflowRun) -> Result<WorkflowRunRegistrado> {
        self.conn()
            .execute(
                "INSERT INTO workflow_run
                    (id, client_id, project, workflow_id, workflow_version, definicao_json, tarefa,
                     current_phase, status, pause_reason, total_phases, started_at, finished_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, 0, ?10, NULL)",
                params![
                    novo.id,
                    novo.client_id,
                    novo.project,
                    novo.workflow_id,
                    novo.workflow_version,
                    novo.definicao_json,
                    novo.tarefa,
                    novo.current_phase,
                    status_workflow_run_to_str(StatusWorkflowRun::Running),
                    novo.started_at.0,
                ],
            )
            .map_err(|e| StorageError::Invalid(e.to_string()))?;

        Ok(WorkflowRunRegistrado {
            id: novo.id,
            client_id: novo.client_id,
            project: novo.project,
            workflow_id: novo.workflow_id,
            workflow_version: novo.workflow_version,
            definicao_json: novo.definicao_json,
            tarefa: novo.tarefa,
            current_phase: novo.current_phase,
            status: StatusWorkflowRun::Running,
            pause_reason: None,
            total_phases: 0,
            started_at: novo.started_at,
            finished_at: None,
        })
    }

    fn workflow_run(&self, id: &str) -> Result<Option<WorkflowRunRegistrado>> {
        self.conn()
            .query_row(
                "SELECT id, client_id, project, workflow_id, workflow_version, definicao_json, tarefa,
                        current_phase, status, pause_reason, total_phases, started_at, finished_at
                 FROM workflow_run WHERE id = ?1",
                params![id],
                row_to_workflow_run,
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn atualizar_workflow_run(
        &self,
        id: &str,
        current_phase: &str,
        status: StatusWorkflowRun,
        pause_reason: Option<&str>,
        total_phases: i64,
        finished_at: Option<Instante>,
    ) -> Result<()> {
        let alterado = self
            .conn()
            .execute(
                "UPDATE workflow_run
                 SET current_phase = ?1, status = ?2, pause_reason = ?3,
                     total_phases = ?4, finished_at = ?5
                 WHERE id = ?6",
                params![
                    current_phase,
                    status_workflow_run_to_str(status),
                    pause_reason,
                    total_phases,
                    finished_at.map(|i| i.0),
                    id,
                ],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        if alterado == 0 {
            return Err(StorageError::NotFound(format!("workflow_run {id}")));
        }
        Ok(())
    }

    fn registrar_entrada_fase(&self, nova: NovaEntradaDeFase) -> Result<EntradaDeFase> {
        self.conn()
            .execute(
                "INSERT INTO workflow_phase_entry
                    (id, workflow_run_id, phase_id, run_id, outcome, entrada_numero, started_at, ended_at)
                 VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, NULL)",
                params![
                    nova.id,
                    nova.workflow_run_id,
                    nova.phase_id,
                    nova.run_id,
                    nova.entrada_numero,
                    nova.started_at.0,
                ],
            )
            .map_err(|e| StorageError::Invalid(e.to_string()))?;

        Ok(EntradaDeFase {
            id: nova.id,
            workflow_run_id: nova.workflow_run_id,
            phase_id: nova.phase_id,
            run_id: nova.run_id,
            outcome: None,
            entrada_numero: nova.entrada_numero,
            started_at: nova.started_at,
            ended_at: None,
        })
    }

    fn concluir_entrada_fase(
        &self,
        entrada_id: &str,
        outcome: &str,
        ended_at: Instante,
    ) -> Result<()> {
        let alterado = self
            .conn()
            .execute(
                "UPDATE workflow_phase_entry SET outcome = ?1, ended_at = ?2 WHERE id = ?3",
                params![outcome, ended_at.0, entrada_id],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        if alterado == 0 {
            return Err(StorageError::NotFound(format!(
                "workflow_phase_entry {entrada_id}"
            )));
        }
        Ok(())
    }

    fn entradas_do_workflow_run(&self, workflow_run_id: &str) -> Result<Vec<EntradaDeFase>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, workflow_run_id, phase_id, run_id, outcome, entrada_numero, started_at, ended_at
                 FROM workflow_phase_entry WHERE workflow_run_id = ?1 ORDER BY started_at",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![workflow_run_id], row_to_entrada_fase)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn criar_comparacao(&self, nova: NovaComparacao) -> Result<ComparacaoRegistrada> {
        self.conn()
            .execute(
                "INSERT INTO comparacao
                    (id, client_id, project, tarefa, vencedor_provider_id, started_at, finished_at)
                 VALUES (?1, ?2, ?3, ?4, NULL, ?5, NULL)",
                params![
                    nova.id,
                    nova.client_id,
                    nova.project,
                    nova.tarefa,
                    nova.started_at.0,
                ],
            )
            .map_err(|e| StorageError::Invalid(e.to_string()))?;

        Ok(ComparacaoRegistrada {
            id: nova.id,
            client_id: nova.client_id,
            project: nova.project,
            tarefa: nova.tarefa,
            vencedor_provider_id: None,
            started_at: nova.started_at,
            finished_at: None,
        })
    }

    fn comparacao(&self, id: &str) -> Result<Option<ComparacaoRegistrada>> {
        self.conn()
            .query_row(
                "SELECT id, client_id, project, tarefa, vencedor_provider_id, started_at, finished_at
                 FROM comparacao WHERE id = ?1",
                params![id],
                row_to_comparacao,
            )
            .optional()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn registrar_candidato_comparacao(
        &self,
        novo: NovoCandidatoComparacao,
    ) -> Result<CandidatoComparacao> {
        self.conn()
            .execute(
                "INSERT INTO comparacao_candidato (id, comparacao_id, provider_id, run_id)
                 VALUES (?1, ?2, ?3, ?4)",
                params![novo.id, novo.comparacao_id, novo.provider_id, novo.run_id],
            )
            .map_err(|e| StorageError::Invalid(e.to_string()))?;

        Ok(CandidatoComparacao {
            id: novo.id,
            comparacao_id: novo.comparacao_id,
            provider_id: novo.provider_id,
            run_id: novo.run_id,
        })
    }

    fn candidatos_da_comparacao(&self, comparacao_id: &str) -> Result<Vec<CandidatoComparacao>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, comparacao_id, provider_id, run_id
                 FROM comparacao_candidato WHERE comparacao_id = ?1",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![comparacao_id], row_to_candidato_comparacao)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn definir_vencedor_comparacao(
        &self,
        comparacao_id: &str,
        provider_id: &str,
        finished_at: Instante,
    ) -> Result<()> {
        let alterado = self
            .conn()
            .execute(
                "UPDATE comparacao SET vencedor_provider_id = ?1, finished_at = ?2 WHERE id = ?3",
                params![provider_id, finished_at.0, comparacao_id],
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        if alterado == 0 {
            return Err(StorageError::NotFound(format!(
                "comparacao {comparacao_id}"
            )));
        }
        Ok(())
    }

    fn registrar_execucao_experimento(
        &self,
        nova: NovaExecucaoExperimento,
    ) -> Result<ExecucaoExperimento> {
        self.conn()
            .execute(
                "INSERT INTO experimento_execucao (id, case_id, braco, run_id, started_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    nova.id,
                    nova.case_id,
                    nova.braco,
                    nova.run_id,
                    nova.started_at.0
                ],
            )
            .map_err(|e| StorageError::Invalid(e.to_string()))?;

        Ok(ExecucaoExperimento {
            id: nova.id,
            case_id: nova.case_id,
            braco: nova.braco,
            run_id: nova.run_id,
            started_at: nova.started_at,
        })
    }

    fn execucoes_do_experimento(&self, braco: Option<&str>) -> Result<Vec<ExecucaoExperimento>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, case_id, braco, run_id, started_at
                 FROM experimento_execucao
                 WHERE ?1 IS NULL OR braco = ?1",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![braco], row_to_execucao_experimento)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }
}

impl SqliteStore {
    fn runs_por_status(&self, status: StatusRun) -> Result<Vec<RunRegistrado>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, client_id, project, base_commit, worktree_path, branch,
                        provider_id, pid, status, custo_equivalente_micros, started_at, finished_at
                 FROM run WHERE status = ?1",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![status_run_to_str(status)], row_to_run)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn bindings_do_perfil(
        &self,
        conn: &Connection,
        identity_profile_id: &str,
    ) -> Result<Vec<ProviderBinding>> {
        let mut stmt = conn
            .prepare(
                "SELECT provider_id, config_home FROM provider_binding
                 WHERE identity_profile_id = ?1 ORDER BY provider_id",
            )
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let rows = stmt
            .query_map(params![identity_profile_id], |row| {
                Ok(ProviderBinding {
                    provider_id: row.get(0)?,
                    config_home: row.get(1)?,
                })
            })
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

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

fn row_to_perfil_base(row: &Row) -> rusqlite::Result<PerfilIdentidade> {
    Ok(PerfilIdentidade {
        id: row.get(0)?,
        client_id: row.get(1)?,
        project: row.get(2)?,
        git_author_name: row.get(3)?,
        git_author_email: row.get(4)?,
        github_org: row.get(5)?,
        bindings: Vec::new(), // preenchido por bindings_do_perfil()
    })
}

fn row_to_credencial(row: &Row) -> rusqlite::Result<CredencialRegistrada> {
    Ok(CredencialRegistrada {
        id: row.get(0)?,
        label: row.get(1)?,
        keychain_service: row.get(2)?,
        keychain_account: row.get(3)?,
        class: str_to_classe_secret(&row.get::<_, String>(4)?),
        created_at: Instante(row.get(5)?),
        expires_at: row.get::<_, Option<i64>>(6)?.map(Instante),
        last_used_at: row.get::<_, Option<i64>>(7)?.map(Instante),
        rotation_policy: row.get(8)?,
    })
}

fn row_to_run(row: &Row) -> rusqlite::Result<RunRegistrado> {
    Ok(RunRegistrado {
        id: row.get(0)?,
        client_id: row.get(1)?,
        project: row.get(2)?,
        base_commit: row.get(3)?,
        worktree_path: row.get(4)?,
        branch: row.get(5)?,
        provider_id: row.get(6)?,
        pid: row.get::<_, Option<i64>>(7)?.map(|p| p as u32),
        status: str_to_status_run(&row.get::<_, String>(8)?),
        custo_equivalente: row.get::<_, Option<i64>>(9)?.map(Money),
        started_at: Instante(row.get(10)?),
        finished_at: row.get::<_, Option<i64>>(11)?.map(Instante),
    })
}

fn status_run_to_str(s: StatusRun) -> &'static str {
    match s {
        StatusRun::EmExecucao => "em_execucao",
        StatusRun::Concluido => "concluido",
        StatusRun::Falhou => "falhou",
        StatusRun::Abandonado => "abandonado",
    }
}

fn str_to_status_run(s: &str) -> StatusRun {
    match s {
        "em_execucao" => StatusRun::EmExecucao,
        "concluido" => StatusRun::Concluido,
        "falhou" => StatusRun::Falhou,
        _ => StatusRun::Abandonado,
    }
}

fn row_to_workflow_run(row: &Row) -> rusqlite::Result<WorkflowRunRegistrado> {
    Ok(WorkflowRunRegistrado {
        id: row.get(0)?,
        client_id: row.get(1)?,
        project: row.get(2)?,
        workflow_id: row.get(3)?,
        workflow_version: row.get(4)?,
        definicao_json: row.get(5)?,
        tarefa: row.get(6)?,
        current_phase: row.get(7)?,
        status: str_to_status_workflow_run(&row.get::<_, String>(8)?),
        pause_reason: row.get(9)?,
        total_phases: row.get(10)?,
        started_at: Instante(row.get(11)?),
        finished_at: row.get::<_, Option<i64>>(12)?.map(Instante),
    })
}

fn row_to_entrada_fase(row: &Row) -> rusqlite::Result<EntradaDeFase> {
    Ok(EntradaDeFase {
        id: row.get(0)?,
        workflow_run_id: row.get(1)?,
        phase_id: row.get(2)?,
        run_id: row.get(3)?,
        outcome: row.get(4)?,
        entrada_numero: row.get(5)?,
        started_at: Instante(row.get(6)?),
        ended_at: row.get::<_, Option<i64>>(7)?.map(Instante),
    })
}

fn row_to_comparacao(row: &Row) -> rusqlite::Result<ComparacaoRegistrada> {
    Ok(ComparacaoRegistrada {
        id: row.get(0)?,
        client_id: row.get(1)?,
        project: row.get(2)?,
        tarefa: row.get(3)?,
        vencedor_provider_id: row.get(4)?,
        started_at: Instante(row.get(5)?),
        finished_at: row.get::<_, Option<i64>>(6)?.map(Instante),
    })
}

fn row_to_candidato_comparacao(row: &Row) -> rusqlite::Result<CandidatoComparacao> {
    Ok(CandidatoComparacao {
        id: row.get(0)?,
        comparacao_id: row.get(1)?,
        provider_id: row.get(2)?,
        run_id: row.get(3)?,
    })
}

fn row_to_execucao_experimento(row: &Row) -> rusqlite::Result<ExecucaoExperimento> {
    Ok(ExecucaoExperimento {
        id: row.get(0)?,
        case_id: row.get(1)?,
        braco: row.get(2)?,
        run_id: row.get(3)?,
        started_at: Instante(row.get(4)?),
    })
}

fn status_workflow_run_to_str(s: StatusWorkflowRun) -> &'static str {
    match s {
        StatusWorkflowRun::Pending => "pending",
        StatusWorkflowRun::Running => "running",
        StatusWorkflowRun::Paused => "paused",
        StatusWorkflowRun::Completed => "completed",
        StatusWorkflowRun::Failed => "failed",
        StatusWorkflowRun::Cancelled => "cancelled",
    }
}

fn str_to_status_workflow_run(s: &str) -> StatusWorkflowRun {
    match s {
        "pending" => StatusWorkflowRun::Pending,
        "running" => StatusWorkflowRun::Running,
        "paused" => StatusWorkflowRun::Paused,
        "completed" => StatusWorkflowRun::Completed,
        "failed" => StatusWorkflowRun::Failed,
        _ => StatusWorkflowRun::Cancelled,
    }
}

fn row_to_nota(row: &Row) -> rusqlite::Result<NotaDeMemoria> {
    Ok(NotaDeMemoria {
        id: row.get(0)?,
        client_id: row.get(1)?,
        project: row.get(2)?,
        categoria: str_to_categoria_nota(&row.get::<_, String>(3)?),
        texto: row.get(4)?,
        rationale: row.get(5)?,
        created_at: Instante(row.get(6)?),
        superseded_by: row.get(7)?,
    })
}

fn categoria_nota_to_str(c: CategoriaNota) -> &'static str {
    match c {
        CategoriaNota::Objetivo => "objetivo",
        CategoriaNota::Decisao => "decisao",
        CategoriaNota::Analise => "analise",
        CategoriaNota::TentativaFalha => "tentativa_falha",
        CategoriaNota::ProximoPasso => "proximo_passo",
        CategoriaNota::Nota => "nota",
    }
}

fn str_to_categoria_nota(s: &str) -> CategoriaNota {
    match s {
        "objetivo" => CategoriaNota::Objetivo,
        "decisao" => CategoriaNota::Decisao,
        "analise" => CategoriaNota::Analise,
        "tentativa_falha" => CategoriaNota::TentativaFalha,
        "proximo_passo" => CategoriaNota::ProximoPasso,
        _ => CategoriaNota::Nota,
    }
}

fn classe_secret_to_str(c: ClasseSecret) -> &'static str {
    match c {
        ClasseSecret::Low => "low",
        ClasseSecret::Medium => "medium",
        ClasseSecret::High => "high",
        ClasseSecret::Critical => "critical",
    }
}

fn str_to_classe_secret(s: &str) -> ClasseSecret {
    match s {
        "low" => ClasseSecret::Low,
        "medium" => ClasseSecret::Medium,
        "high" => ClasseSecret::High,
        _ => ClasseSecret::Critical,
    }
}

fn row_to_plano(row: &Row) -> rusqlite::Result<PlanoRegistrado> {
    Ok(PlanoRegistrado {
        provider_id: row.get("provider_id")?,
        billing_mode: str_to_billing_mode(&row.get::<_, String>("billing_mode")?),
        plan_label: row.get("plan_label")?,
        ativo_desde: Instante(row.get("ativo_desde")?),
        ativo_ate: row.get::<_, Option<i64>>("ativo_ate")?.map(Instante),
        verificado_em: Instante(row.get("verificado_em")?),
    })
}

fn row_to_quota_signal(row: &Row) -> rusqlite::Result<QuotaSignalRegistrado> {
    Ok(QuotaSignalRegistrado {
        provider_id: row.get("provider_id")?,
        bucket_id: row.get("bucket_id")?,
        grupo: row.get("grupo")?,
        remaining_percent: row.get("remaining_percent")?,
        reset_at: row.get::<_, Option<i64>>("reset_at")?.map(Instante),
        observed_at: Instante(row.get("observed_at")?),
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
    fn consulta_de_cliente_nao_inclui_nao_atribuidos() {
        // Spec cost-attribution, "Consulta de cliente não inclui não
        // atribuídos" -- modo de falha diferente de vazar para OUTRO
        // cliente (já coberto acima): aqui o risco é vazar consumo SEM
        // dono nenhum para dentro da visão de um cliente real.
        let s = store();
        s.upsert_client("xpto").unwrap();
        let atribuido = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        s.atribuir(&atribuido.id, "xpto").unwrap();
        // Órfão no mesmo período -- client_id NULL.
        s.gravar_consumo(novo_consumo("k2", "codex", "gpt"))
            .unwrap();

        let de_xpto = s
            .consumo_do_cliente(
                "xpto",
                Periodo {
                    desde: Instante(0),
                    ate: None,
                },
            )
            .unwrap();

        assert_eq!(de_xpto.len(), 1, "só o registro atribuído a xpto");
        assert!(
            de_xpto
                .iter()
                .all(|r| r.client_id.as_deref() == Some("xpto"))
        );
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

    #[test]
    fn integridade_de_ledger_populado_realista() {
        // Task 8.2: um ledger com mistura real de estados -- órfãos,
        // custo desconhecido, atribuídos, custo tardio via provider --
        // todos legítimos, nenhum deveria disparar violação. Só a linha
        // deliberadamente corrompida (inserida via SQL bruto, fora do
        // caminho normal) deve aparecer.
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();

        // 1. Consumo atribuído normalmente.
        let r1 = s
            .gravar_consumo(novo_consumo("k1", "claude", "opus"))
            .unwrap();
        s.atribuir(&r1.id, "xpto").unwrap();

        // 2. Consumo órfão (unattributed) -- estado legítimo, é o próprio
        // alarme que o sistema deve saber sinalizar, não uma violação.
        s.gravar_consumo(novo_consumo("k2", "codex", "gpt"))
            .unwrap();

        // 3. Consumo com custo desconhecido, atribuído.
        let mut c3 = novo_consumo("k3", "grok", "grok-4.5");
        c3.client_id = Some("acme".to_string());
        c3.cost_source = crate::domain::CostSource::Unknown;
        s.gravar_consumo(c3).unwrap();

        // 4. Consumo cujo custo pago chega depois (supersessão), atribuído.
        let r4 = s
            .gravar_consumo(novo_consumo("k4", "grok", "grok-4.5"))
            .unwrap();
        s.atribuir(&r4.id, "acme").unwrap();
        s.atualizar_custo_pago(&r4.id, Money(1_000_000)).unwrap();

        // 5. A única linha realmente corrompida -- inserida fora do
        // caminho normal, exatamente como no teste anterior.
        s.conn()
            .execute("INSERT INTO provider (id) VALUES ('quebrado')", params![])
            .unwrap();
        s.conn()
            .execute(
                "INSERT INTO usage_record (
                    id, dedup_key, provider_id, model, billing_mode,
                    usage_source, cost_source, client_id, attribution_status, occurred_at
                ) VALUES ('corrompido','corrompido','quebrado','m','api','provider','unknown',NULL,'attributed',0)",
                params![],
            )
            .unwrap();

        let violacoes = s.verificar_integridade().unwrap();
        assert_eq!(
            violacoes.len(),
            1,
            "só a linha corrompida deve violar; órfão e custo desconhecido são estados legítimos. Violações: {violacoes:?}"
        );
        assert_eq!(violacoes[0].usage_record_id, "corrompido");
    }

    #[test]
    fn registrar_plano_novo_abre_vigencia() {
        let s = store();
        s.registrar_plano(NovoPlano {
            provider_id: "claude".into(),
            billing_mode: BillingMode::Subscription,
            plan_label: Some("pro".into()),
            detectado_em: Instante(100),
        })
        .unwrap();

        let vigente = s.plano_vigente("claude").unwrap().unwrap();
        assert_eq!(vigente.plan_label.as_deref(), Some("pro"));
        assert_eq!(vigente.ativo_desde, Instante(100));
        assert_eq!(vigente.ativo_ate, None);
    }

    #[test]
    fn registrar_plano_igual_ao_vigente_e_no_op() {
        let s = store();
        s.registrar_plano(NovoPlano {
            provider_id: "claude".into(),
            billing_mode: BillingMode::Subscription,
            plan_label: Some("pro".into()),
            detectado_em: Instante(100),
        })
        .unwrap();
        s.registrar_plano(NovoPlano {
            provider_id: "claude".into(),
            billing_mode: BillingMode::Subscription,
            plan_label: Some("pro".into()),
            detectado_em: Instante(200),
        })
        .unwrap();

        let vigente = s.plano_vigente("claude").unwrap().unwrap();
        assert_eq!(
            vigente.ativo_desde,
            Instante(100),
            "plano relatado igual ao vigente não deve abrir nova vigência"
        );
        assert_eq!(
            vigente.verificado_em,
            Instante(200),
            "spec plan-catalog 'Consulta de plano falha': verificado_em avança a cada \
             checagem bem-sucedida mesmo sem mudança, para sinalizar informação \
             potencialmente desatualizada quando parar de avançar"
        );
    }

    #[test]
    fn registrar_plano_diferente_fecha_anterior_e_abre_novo() {
        let s = store();
        s.registrar_plano(NovoPlano {
            provider_id: "claude".into(),
            billing_mode: BillingMode::Subscription,
            plan_label: Some("pro".into()),
            detectado_em: Instante(100),
        })
        .unwrap();
        s.registrar_plano(NovoPlano {
            provider_id: "claude".into(),
            billing_mode: BillingMode::Subscription,
            plan_label: Some("max".into()),
            detectado_em: Instante(200),
        })
        .unwrap();

        let vigente = s.plano_vigente("claude").unwrap().unwrap();
        assert_eq!(vigente.plan_label.as_deref(), Some("max"));
        assert_eq!(vigente.ativo_desde, Instante(200));

        let historico = s
            .plano_vigente_em("claude", Instante(150))
            .unwrap()
            .unwrap();
        assert_eq!(
            historico.plan_label.as_deref(),
            Some("pro"),
            "janela histórica usa o plano vigente à época"
        );
    }

    #[test]
    fn plano_vigente_de_provider_sem_plano_e_none() {
        let s = store();
        assert_eq!(s.plano_vigente("grok").unwrap(), None);
    }

    #[test]
    fn upsert_quota_signal_atualiza_o_mesmo_bucket() {
        let s = store();
        s.upsert_quota_signal(NovoQuotaSignal {
            provider_id: "gemini".into(),
            bucket_id: "gemini-weekly".into(),
            grupo: "Gemini Models".into(),
            remaining_percent: 92.0,
            reset_at: Some(Instante(1000)),
            observed_at: Instante(500),
        })
        .unwrap();
        s.upsert_quota_signal(NovoQuotaSignal {
            provider_id: "gemini".into(),
            bucket_id: "gemini-weekly".into(),
            grupo: "Gemini Models".into(),
            remaining_percent: 80.0,
            reset_at: Some(Instante(1000)),
            observed_at: Instante(600),
        })
        .unwrap();

        let sinais = s.quota_signals("gemini").unwrap();
        assert_eq!(sinais.len(), 1, "upsert por bucket não duplica");
        assert_eq!(sinais[0].remaining_percent, 80.0);
    }

    #[test]
    fn quota_signals_lista_multiplos_buckets_do_mesmo_provider() {
        let s = store();
        s.upsert_quota_signal(NovoQuotaSignal {
            provider_id: "codex".into(),
            bucket_id: "primary".into(),
            grupo: "rate_limits".into(),
            remaining_percent: 75.0,
            reset_at: None,
            observed_at: Instante(1),
        })
        .unwrap();
        s.upsert_quota_signal(NovoQuotaSignal {
            provider_id: "codex".into(),
            bucket_id: "secondary".into(),
            grupo: "rate_limits".into(),
            remaining_percent: 50.0,
            reset_at: None,
            observed_at: Instante(1),
        })
        .unwrap();

        let sinais = s.quota_signals("codex").unwrap();
        assert_eq!(sinais.len(), 2);
    }

    /// Task 8.3 — volume sintético para o critério de revisão do D-1
    /// ("revisão se uma consulta real passar de 200ms com doze meses de
    /// dados"). O blueprint não fixa uma contagem exata; 200 mil registros
    /// é a estimativa deste teste para "uso pesado, multi-provider,
    /// multi-cliente, 12 meses" — um consultor ativo com vários clientes
    /// gerando algumas centenas de chamadas por dia ao longo de um ano.
    /// Documentado aqui porque é premissa, não fato do blueprint.
    #[test]
    #[ignore = "lento (~alguns segundos de setup) — rodar com `cargo test -- --ignored`"]
    fn desempenho_com_volume_sintetico_de_doze_meses() {
        const N: i64 = 200_000;
        const CLIENTES: i64 = 20;
        const PROVIDERS: i64 = 5;
        const SEGUNDOS_EM_12_MESES: i64 = 365 * 86_400;

        let s = store();
        for c in 0..CLIENTES {
            s.upsert_client(&format!("cliente-{c}")).unwrap();
        }

        {
            let conn = s.conn();
            conn.execute(
                "INSERT INTO provider (id) VALUES ('p0'),('p1'),('p2'),('p3'),('p4')",
                params![],
            )
            .unwrap();

            conn.execute_batch("BEGIN").unwrap();
            {
                let mut stmt = conn
                    .prepare(
                        "INSERT INTO usage_record (
                            id, dedup_key, provider_id, model,
                            tokens_input, tokens_output,
                            custo_pago_micros, custo_equivalente_micros,
                            billing_mode, usage_source, cost_source,
                            client_id, attribution_status, occurred_at
                        ) VALUES (?1,?1,?2,?3,?4,?5,?6,?6,'api','provider','provider',?7,'attributed',?8)",
                    )
                    .unwrap();
                for i in 0..N {
                    let id = format!("synt-{i}");
                    let provider = format!("p{}", i % PROVIDERS);
                    let cliente = format!("cliente-{}", i % CLIENTES);
                    let occurred_at = i * (SEGUNDOS_EM_12_MESES / N);
                    stmt.execute(params![
                        id,
                        provider,
                        "modelo-sintetico",
                        1000i64,
                        200i64,
                        1_500_000i64,
                        cliente,
                        occurred_at
                    ])
                    .unwrap();
                }
            }
            conn.execute_batch("COMMIT").unwrap();
        }

        let periodo_completo = Periodo {
            desde: Instante(0),
            ate: Some(Instante(SEGUNDOS_EM_12_MESES)),
        };
        let periodo_um_mes = Periodo {
            desde: Instante(0),
            ate: Some(Instante(SEGUNDOS_EM_12_MESES / 12)),
        };

        let medir = |nome: &str, f: &dyn Fn() -> usize| -> std::time::Duration {
            let inicio = std::time::Instant::now();
            let n = f();
            let duracao = inicio.elapsed();
            println!("{nome}: {n} registros em {duracao:?}");
            duracao
        };

        let exigir_200ms = |nome: &str, duracao: std::time::Duration| {
            assert!(
                duracao.as_millis() < 200,
                "{nome} passou de 200ms (D-1): {duracao:?}"
            );
        };

        // Casos que o uso normal exercita e que precisam ficar sob 200ms de
        // verdade: consulta por cliente (sempre escopada), e a janela mensal
        // que --period produz na prática -- não o histórico inteiro de uma
        // vez, que é o caso raro tratado abaixo.
        exigir_200ms(
            "consumo_do_cliente (1 de 20 clientes, 12 meses)",
            medir("consumo_do_cliente (1 de 20 clientes, 12 meses)", &|| {
                s.consumo_do_cliente("cliente-0", periodo_completo)
                    .unwrap()
                    .len()
            }),
        );
        exigir_200ms(
            "consumo_no_periodo (todos os clientes, 1 mês)",
            medir("consumo_no_periodo (todos os clientes, 1 mês)", &|| {
                s.consumo_no_periodo(periodo_um_mes).unwrap().len()
            }),
        );
        exigir_200ms(
            "nao_atribuidos (12 meses, ledger 100% atribuído)",
            medir("nao_atribuidos (12 meses, ledger 100% atribuído)", &|| {
                s.nao_atribuidos(periodo_completo).unwrap().len()
            }),
        );

        // Limite conhecido, documentado em vez de escondido (D-1, achado
        // real desta task): consumo_no_periodo() sem recorte, varrendo os
        // 12 meses inteiros de uma vez (o que `--by provider/model` sem
        // `--period` faz, e o que verificar_integridade() sempre faz),
        // passa de 200ms com 200 mil registros -- o gargalo é materializar
        // o UsageRecord inteiro (14 colunas, enums, várias String) por
        // linha em Rust, não a consulta SQL (COUNT(*) leva 2ms; um SELECT
        // enxuto de 4 colunas leva 40ms para as mesmas 200 mil linhas).
        // Índice em occurred_at (adicionado nesta task) ajuda pouco porque
        // o problema não é achar as linhas, é montar todas elas.
        //
        // Não corrigido agora porque a correção real é agregação do lado
        // do SQL (GROUP BY em vez de trazer tudo para o Rust), que é
        // trabalho novo de storage, desproporcional ao que esta change
        // pede. D-1 preexiste exatamente para este caso: revisitar se o
        // dado real confirmar que isso importa na prática.
        let duracao_sem_recorte = medir(
            "consumo_no_periodo (todos os clientes, 12 meses, SEM recorte -- limite conhecido)",
            &|| s.consumo_no_periodo(periodo_completo).unwrap().len(),
        );
        println!(
            "  ^ acima de 200ms é esperado aqui ({duracao_sem_recorte:?}) -- ver comentário no teste"
        );
    }

    // --- Identidade (context-and-identity-switching) ----------------------

    fn novo_perfil(id: &str, client_id: &str, project: Option<&str>) -> NovoPerfil {
        NovoPerfil {
            id: id.into(),
            client_id: client_id.into(),
            project: project.map(String::from),
            git_author_name: Some("Joao Costa".into()),
            git_author_email: Some("joao@xpto.com.br".into()),
            github_org: Some("xpto-org".into()),
            bindings: vec![ProviderBinding {
                provider_id: "codex".into(),
                config_home: "/tmp/xpto/codex".into(),
            }],
            created_at: Instante(1000),
        }
    }

    #[test]
    fn criar_perfil_e_le_de_volta_com_bindings() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_perfil(novo_perfil("prof1", "xpto", Some("checkout-api")))
            .unwrap();

        let perfil = s.perfil("prof1").unwrap().unwrap();
        assert_eq!(perfil.client_id, "xpto");
        assert_eq!(perfil.project.as_deref(), Some("checkout-api"));
        assert_eq!(perfil.bindings.len(), 1);
        assert_eq!(perfil.bindings[0].provider_id, "codex");
    }

    #[test]
    fn perfil_com_multiplos_providers_tem_caminhos_isolados_e_distintos() {
        // Spec provider-isolation, "Perfil com múltiplos providers
        // vinculados": cada provider tem seu próprio caminho, sem colisão.
        let s = store();
        s.upsert_client("xpto").unwrap();
        let mut perfil = novo_perfil("prof1", "xpto", None);
        perfil.bindings = vec![
            ProviderBinding {
                provider_id: "codex".into(),
                config_home: "/tmp/xpto/codex".into(),
            },
            ProviderBinding {
                provider_id: "claude".into(),
                config_home: "/tmp/xpto/claude".into(),
            },
        ];
        s.criar_perfil(perfil).unwrap();

        let de_volta = s.perfil("prof1").unwrap().unwrap();
        assert_eq!(de_volta.bindings.len(), 2);
        let caminhos: std::collections::HashSet<&str> = de_volta
            .bindings
            .iter()
            .map(|b| b.config_home.as_str())
            .collect();
        assert_eq!(
            caminhos.len(),
            2,
            "caminhos de provider distintos não colidem"
        );
    }

    #[test]
    fn perfil_inexistente_e_none() {
        let s = store();
        assert_eq!(s.perfil("fantasma").unwrap(), None);
    }

    #[test]
    fn perfis_do_cliente_lista_todos_com_bindings() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_perfil(novo_perfil("prof1", "xpto", Some("checkout-api")))
            .unwrap();
        s.criar_perfil(novo_perfil("prof2", "xpto", Some("billing-api")))
            .unwrap();

        let perfis = s.perfis_do_cliente("xpto").unwrap();
        assert_eq!(
            perfis.len(),
            2,
            "múltiplos projetos = ambiguidade pra quem chama decidir"
        );
        assert!(perfis.iter().all(|p| !p.bindings.is_empty()));
    }

    #[test]
    fn conectar_e_desconectar_contexto() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_perfil(novo_perfil("prof1", "xpto", None)).unwrap();

        assert_eq!(s.contexto_ativo().unwrap(), None);

        s.conectar(ContextoAtivo {
            client_id: "xpto".into(),
            project: None,
            identity_profile_id: "prof1".into(),
            connected_at: Instante(5000),
        })
        .unwrap();

        let ativo = s.contexto_ativo().unwrap().unwrap();
        assert_eq!(ativo.client_id, "xpto");
        assert_eq!(ativo.identity_profile_id, "prof1");

        s.desconectar().unwrap();
        assert_eq!(s.contexto_ativo().unwrap(), None);
    }

    #[test]
    fn desconectar_sem_contexto_ativo_e_no_op() {
        let s = store();
        s.desconectar().unwrap(); // não deve falhar
        assert_eq!(s.contexto_ativo().unwrap(), None);
    }

    #[test]
    fn conectar_substitui_contexto_anterior() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();
        s.criar_perfil(novo_perfil("prof-xpto", "xpto", None))
            .unwrap();
        s.criar_perfil(novo_perfil("prof-acme", "acme", None))
            .unwrap();

        s.conectar(ContextoAtivo {
            client_id: "xpto".into(),
            project: None,
            identity_profile_id: "prof-xpto".into(),
            connected_at: Instante(1),
        })
        .unwrap();
        s.conectar(ContextoAtivo {
            client_id: "acme".into(),
            project: None,
            identity_profile_id: "prof-acme".into(),
            connected_at: Instante(2),
        })
        .unwrap();

        let ativo = s.contexto_ativo().unwrap().unwrap();
        assert_eq!(
            ativo.client_id, "acme",
            "troca de contexto substitui o anterior, não acumula (singleton)"
        );
    }

    fn nova_credencial(id: &str, class: ClasseSecret) -> NovaCredencialMetadados {
        NovaCredencialMetadados {
            id: id.into(),
            label: "AWS produção".into(),
            keychain_service: "brian".into(),
            keychain_account: format!("xpto/{id}"),
            class,
            created_at: Instante(1000),
            expires_at: None,
            rotation_policy: Some("90d".into()),
        }
    }

    #[test]
    fn registrar_credencial_grava_so_metadados() {
        let s = store();
        let cred = s
            .registrar_credencial(nova_credencial("c1", ClasseSecret::Critical))
            .unwrap();
        assert_eq!(cred.class, ClasseSecret::Critical);
        assert_eq!(cred.last_used_at, None);

        let de_volta = s.credencial("c1").unwrap().unwrap();
        assert_eq!(de_volta, cred);
    }

    #[test]
    fn atualizar_ultimo_uso_credencial_registra_instante() {
        let s = store();
        s.registrar_credencial(nova_credencial("c1", ClasseSecret::Low))
            .unwrap();
        s.atualizar_ultimo_uso_credencial("c1", Instante(5000))
            .unwrap();

        let cred = s.credencial("c1").unwrap().unwrap();
        assert_eq!(cred.last_used_at, Some(Instante(5000)));
    }

    #[test]
    fn atualizar_ultimo_uso_de_credencial_inexistente_e_notfound() {
        let s = store();
        let err = s
            .atualizar_ultimo_uso_credencial("fantasma", Instante(1))
            .unwrap_err();
        assert!(matches!(err, StorageError::NotFound(_)));
    }

    #[test]
    fn listar_credenciais_traz_todas() {
        let s = store();
        s.registrar_credencial(nova_credencial("c1", ClasseSecret::Low))
            .unwrap();
        s.registrar_credencial(nova_credencial("c2", ClasseSecret::High))
            .unwrap();
        assert_eq!(s.listar_credenciais().unwrap().len(), 2);
    }

    // --- Notas de memória (continuity-pack-handoff) ------------------------

    fn nova_nota(id: &str, client_id: &str, categoria: CategoriaNota) -> NovaNota {
        NovaNota {
            id: id.into(),
            client_id: client_id.into(),
            project: Some("checkout-api".into()),
            categoria,
            texto: "texto de teste".into(),
            rationale: None,
            created_at: Instante(1000),
        }
    }

    #[test]
    fn registrar_nota_e_le_de_volta() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.registrar_nota(nova_nota("n1", "xpto", CategoriaNota::Nota))
            .unwrap();

        let notas = s.notas_do_contexto("xpto", Some("checkout-api")).unwrap();
        assert_eq!(notas.len(), 1);
        assert_eq!(notas[0].texto, "texto de teste");
    }

    #[test]
    fn registrar_decisao_sem_rationale_e_invalid() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let err = s
            .registrar_nota(nova_nota("n1", "xpto", CategoriaNota::Decisao))
            .unwrap_err();
        assert!(matches!(err, StorageError::Invalid(_)));
    }

    #[test]
    fn registrar_decisao_com_rationale_funciona() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let mut nota = nova_nota("n1", "xpto", CategoriaNota::Decisao);
        nota.rationale = Some("porque sim".into());
        let gravada = s.registrar_nota(nota).unwrap();
        assert_eq!(gravada.rationale.as_deref(), Some("porque sim"));
    }

    #[test]
    fn marcar_superseded_de_nota_inexistente_e_notfound() {
        let s = store();
        let erro = s.marcar_superseded("fantasma", "n2").unwrap_err();
        assert!(matches!(erro, StorageError::NotFound(_)));
    }

    #[test]
    fn marcar_superseded_preserva_texto_rationale_e_categoria_originais() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let mut nota = nova_nota("n1", "xpto", CategoriaNota::Decisao);
        nota.rationale = Some("motivo original".into());
        s.registrar_nota(nota).unwrap();
        s.registrar_nota(nova_nota("n2", "xpto", CategoriaNota::Nota))
            .unwrap();

        s.marcar_superseded("n1", "n2").unwrap();

        let notas = s.notas_do_contexto("xpto", Some("checkout-api")).unwrap();
        let n1 = notas.iter().find(|n| n.id == "n1").unwrap();
        assert_eq!(n1.texto, "texto de teste");
        assert_eq!(n1.rationale.as_deref(), Some("motivo original"));
        assert_eq!(n1.categoria, CategoriaNota::Decisao);
        assert_eq!(n1.superseded_by.as_deref(), Some("n2"));
    }

    #[test]
    fn notas_de_um_cliente_nao_vazam_para_outro() {
        // Spec memory-notes, "Isolamento entre Contexts por construção".
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("acme").unwrap();
        s.registrar_nota(nova_nota("n1", "xpto", CategoriaNota::Nota))
            .unwrap();
        s.registrar_nota(nova_nota("n2", "acme", CategoriaNota::Nota))
            .unwrap();

        let de_xpto = s.notas_do_contexto("xpto", Some("checkout-api")).unwrap();
        assert_eq!(de_xpto.len(), 1);
        assert_eq!(de_xpto[0].client_id, "xpto");
    }

    #[test]
    fn notas_escopadas_por_project_none_nao_misturam_com_project_setado() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.registrar_nota(nova_nota("n1", "xpto", CategoriaNota::Nota))
            .unwrap(); // project = Some("checkout-api")
        let mut sem_projeto = nova_nota("n2", "xpto", CategoriaNota::Nota);
        sem_projeto.project = None;
        s.registrar_nota(sem_projeto).unwrap();

        let com_projeto = s.notas_do_contexto("xpto", Some("checkout-api")).unwrap();
        assert_eq!(com_projeto.len(), 1);
        let sem_projeto = s.notas_do_contexto("xpto", None).unwrap();
        assert_eq!(sem_projeto.len(), 1);
    }

    #[test]
    fn duas_notas_coexistem_append_only() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        let mut n1 = nova_nota("n1", "xpto", CategoriaNota::Analise);
        n1.texto = "primeira análise".into();
        s.registrar_nota(n1).unwrap();
        let mut n2 = nova_nota("n2", "xpto", CategoriaNota::Analise);
        n2.texto = "análise corrigida".into();
        s.registrar_nota(n2).unwrap();

        let notas = s.notas_do_contexto("xpto", Some("checkout-api")).unwrap();
        assert_eq!(notas.len(), 2, "ambas permanecem, nenhuma é sobrescrita");
    }

    // --- Run (isolated-tracked-run) -----------------------------------------

    fn novo_run(id: &str) -> NovoRun {
        NovoRun {
            id: id.into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            base_commit: "abc123".into(),
            worktree_path: format!("/tmp/brian-worktrees/{id}"),
            branch: format!("brian/{id}"),
            provider_id: "codex".into(),
            started_at: Instante(1000),
        }
    }

    #[test]
    fn criar_run_e_le_de_volta() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(novo_run("run1")).unwrap();

        let run = s.run("run1").unwrap().unwrap();
        assert_eq!(run.status, StatusRun::EmExecucao);
        assert_eq!(run.pid, None, "pid ausente até o processo existir");
        assert_eq!(run.finished_at, None);
    }

    #[test]
    fn run_inexistente_e_none() {
        let s = store();
        assert_eq!(s.run("fantasma").unwrap(), None);
    }

    #[test]
    fn definir_pid_run_atualiza_o_registro() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(novo_run("run1")).unwrap();
        s.definir_pid_run("run1", 4242).unwrap();

        let run = s.run("run1").unwrap().unwrap();
        assert_eq!(run.pid, Some(4242));
    }

    #[test]
    fn definir_pid_de_run_inexistente_e_notfound() {
        let s = store();
        let err = s.definir_pid_run("fantasma", 1).unwrap_err();
        assert!(matches!(err, StorageError::NotFound(_)));
    }

    #[test]
    fn definir_worktree_run_atualiza_o_registro() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(novo_run("run1")).unwrap();
        s.definir_worktree_run("run1", "/tmp/worktrees/run1", "brian/run_run1")
            .unwrap();

        let run = s.run("run1").unwrap().unwrap();
        assert_eq!(run.worktree_path, "/tmp/worktrees/run1");
        assert_eq!(run.branch, "brian/run_run1");
    }

    #[test]
    fn definir_worktree_de_run_inexistente_e_notfound() {
        let s = store();
        let err = s
            .definir_worktree_run("fantasma", "/tmp/x", "b")
            .unwrap_err();
        assert!(matches!(err, StorageError::NotFound(_)));
    }

    #[test]
    fn atualizar_status_run_registra_conclusao() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(novo_run("run1")).unwrap();
        s.atualizar_status_run(
            "run1",
            StatusRun::Concluido,
            Some(Instante(2000)),
            Some(Money(1_500_000)),
        )
        .unwrap();

        let run = s.run("run1").unwrap().unwrap();
        assert_eq!(run.status, StatusRun::Concluido);
        assert_eq!(run.finished_at, Some(Instante(2000)));
        assert_eq!(run.custo_equivalente, Some(Money(1_500_000)));
    }

    #[test]
    fn runs_em_execucao_lista_so_os_nao_finalizados() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(novo_run("run1")).unwrap();
        s.criar_run(novo_run("run2")).unwrap();
        s.atualizar_status_run("run2", StatusRun::Concluido, Some(Instante(2000)), None)
            .unwrap();

        let em_execucao = s.runs_em_execucao().unwrap();
        assert_eq!(em_execucao.len(), 1);
        assert_eq!(em_execucao[0].id, "run1");
    }

    #[test]
    fn runs_abandonados_lista_so_os_abandonados() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(novo_run("run1")).unwrap();
        s.criar_run(novo_run("run2")).unwrap();
        s.atualizar_status_run("run2", StatusRun::Abandonado, Some(Instante(2000)), None)
            .unwrap();

        let abandonados = s.runs_abandonados().unwrap();
        assert_eq!(abandonados.len(), 1);
        assert_eq!(abandonados[0].id, "run2");
    }

    #[test]
    fn runs_finalizados_do_cliente_filtra_por_cliente_e_status() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.upsert_client("outro-cliente").unwrap();

        s.criar_run(novo_run("run-concluido")).unwrap();
        s.atualizar_status_run(
            "run-concluido",
            StatusRun::Concluido,
            Some(Instante(2000)),
            None,
        )
        .unwrap();

        s.criar_run(novo_run("run-falho")).unwrap();
        s.atualizar_status_run("run-falho", StatusRun::Falhou, Some(Instante(2000)), None)
            .unwrap();

        // Em execução (não finalizado) -- não deve aparecer.
        s.criar_run(novo_run("run-em-execucao")).unwrap();

        // De outro cliente -- não deve aparecer mesmo finalizado.
        s.criar_run(NovoRun {
            id: "run-outro-cliente".into(),
            client_id: "outro-cliente".into(),
            project: None,
            base_commit: "abc".into(),
            worktree_path: "/tmp/x".into(),
            branch: "brian/run-outro-cliente".into(),
            provider_id: "codex".into(),
            started_at: Instante(1000),
        })
        .unwrap();
        s.atualizar_status_run(
            "run-outro-cliente",
            StatusRun::Concluido,
            Some(Instante(2000)),
            None,
        )
        .unwrap();

        let finalizados = s.runs_finalizados_do_cliente("xpto").unwrap();
        let ids: Vec<_> = finalizados.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"run-concluido"));
        assert!(ids.contains(&"run-falho"));
    }

    #[test]
    fn registrar_e_listar_eventos_do_run() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(novo_run("run1")).unwrap();
        s.registrar_evento_run(NovoEvento {
            id: "e1".into(),
            run_id: "run1".into(),
            tipo: "worktree.create".into(),
            detalhe: None,
            ocorrido_em: Instante(1000),
        })
        .unwrap();
        s.registrar_evento_run(NovoEvento {
            id: "e2".into(),
            run_id: "run1".into(),
            tipo: "provider.execute".into(),
            detalhe: Some("codex".into()),
            ocorrido_em: Instante(1001),
        })
        .unwrap();

        let eventos = s.eventos_do_run("run1").unwrap();
        assert_eq!(eventos.len(), 2);
        assert_eq!(eventos[0].tipo, "worktree.create");
    }

    fn novo_workflow_run(id: &str) -> NovoWorkflowRun {
        NovoWorkflowRun {
            id: id.into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            workflow_id: "fast".into(),
            workflow_version: 1,
            definicao_json: "{}".into(),
            tarefa: "tarefa de teste".into(),
            current_phase: "implement".into(),
            started_at: Instante(1000),
        }
    }

    #[test]
    fn criar_workflow_run_e_le_de_volta() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_workflow_run(novo_workflow_run("wf1")).unwrap();

        let wf = s.workflow_run("wf1").unwrap().unwrap();
        assert_eq!(wf.status, StatusWorkflowRun::Running);
        assert_eq!(wf.current_phase, "implement");
        assert_eq!(wf.total_phases, 0);
        assert_eq!(wf.workflow_version, 1);
    }

    #[test]
    fn workflow_run_inexistente_e_none() {
        let s = store();
        assert_eq!(s.workflow_run("fantasma").unwrap(), None);
    }

    #[test]
    fn atualizar_workflow_run_registra_transicao() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_workflow_run(novo_workflow_run("wf1")).unwrap();

        s.atualizar_workflow_run("wf1", "verify", StatusWorkflowRun::Running, None, 1, None)
            .unwrap();

        let wf = s.workflow_run("wf1").unwrap().unwrap();
        assert_eq!(wf.current_phase, "verify");
        assert_eq!(wf.total_phases, 1);
    }

    #[test]
    fn atualizar_workflow_run_registra_pausa_com_motivo() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_workflow_run(novo_workflow_run("wf1")).unwrap();

        s.atualizar_workflow_run(
            "wf1",
            "plan_review",
            StatusWorkflowRun::Paused,
            Some("aguardando aprovação"),
            1,
            None,
        )
        .unwrap();

        let wf = s.workflow_run("wf1").unwrap().unwrap();
        assert_eq!(wf.status, StatusWorkflowRun::Paused);
        assert_eq!(wf.pause_reason.as_deref(), Some("aguardando aprovação"));
    }

    #[test]
    fn registrar_e_concluir_entrada_de_fase() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_workflow_run(novo_workflow_run("wf1")).unwrap();
        s.criar_run(novo_run("run1")).unwrap();

        let entrada = s
            .registrar_entrada_fase(NovaEntradaDeFase {
                id: "entrada1".into(),
                workflow_run_id: "wf1".into(),
                phase_id: "implement".into(),
                run_id: Some("run1".into()),
                entrada_numero: 1,
                started_at: Instante(1000),
            })
            .unwrap();
        assert_eq!(entrada.outcome, None);

        s.concluir_entrada_fase("entrada1", "success", Instante(1010))
            .unwrap();

        let entradas = s.entradas_do_workflow_run("wf1").unwrap();
        assert_eq!(entradas.len(), 1);
        assert_eq!(entradas[0].outcome.as_deref(), Some("success"));
        assert_eq!(entradas[0].ended_at, Some(Instante(1010)));
    }

    #[test]
    fn entrada_de_fase_terminal_sem_run_associado() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_workflow_run(novo_workflow_run("wf1")).unwrap();

        s.registrar_entrada_fase(NovaEntradaDeFase {
            id: "entrada-terminal".into(),
            workflow_run_id: "wf1".into(),
            phase_id: "done".into(),
            run_id: None,
            entrada_numero: 1,
            started_at: Instante(2000),
        })
        .unwrap();

        let entradas = s.entradas_do_workflow_run("wf1").unwrap();
        assert_eq!(entradas[0].run_id, None);
    }

    fn nova_comparacao(id: &str) -> NovaComparacao {
        NovaComparacao {
            id: id.into(),
            client_id: "xpto".into(),
            project: Some("checkout-api".into()),
            tarefa: "tarefa de teste".into(),
            started_at: Instante(1000),
        }
    }

    #[test]
    fn criar_comparacao_e_le_de_volta() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_comparacao(nova_comparacao("cmp1")).unwrap();

        let cmp = s.comparacao("cmp1").unwrap().unwrap();
        assert_eq!(cmp.vencedor_provider_id, None);
        assert_eq!(cmp.tarefa, "tarefa de teste");
    }

    #[test]
    fn comparacao_inexistente_e_none() {
        let s = store();
        assert_eq!(s.comparacao("fantasma").unwrap(), None);
    }

    #[test]
    fn registrar_e_listar_candidatos_de_comparacao() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_comparacao(nova_comparacao("cmp1")).unwrap();
        s.criar_run(novo_run("run1")).unwrap();

        s.registrar_candidato_comparacao(NovoCandidatoComparacao {
            id: "cand1".into(),
            comparacao_id: "cmp1".into(),
            provider_id: "codex".into(),
            run_id: Some("run1".into()),
        })
        .unwrap();

        let candidatos = s.candidatos_da_comparacao("cmp1").unwrap();
        assert_eq!(candidatos.len(), 1);
        assert_eq!(candidatos[0].provider_id, "codex");
        assert_eq!(candidatos[0].run_id.as_deref(), Some("run1"));
    }

    #[test]
    fn definir_vencedor_comparacao_atualiza_o_registro() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_comparacao(nova_comparacao("cmp1")).unwrap();

        s.definir_vencedor_comparacao("cmp1", "codex", Instante(2000))
            .unwrap();

        let cmp = s.comparacao("cmp1").unwrap().unwrap();
        assert_eq!(cmp.vencedor_provider_id.as_deref(), Some("codex"));
        assert_eq!(cmp.finished_at, Some(Instante(2000)));
    }

    #[test]
    fn definir_vencedor_de_comparacao_inexistente_e_notfound() {
        let s = store();
        let err = s
            .definir_vencedor_comparacao("fantasma", "codex", Instante(0))
            .unwrap_err();
        assert!(matches!(err, StorageError::NotFound(_)));
    }

    #[test]
    fn registrar_execucao_experimento_e_le_de_volta() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(novo_run("run1")).unwrap();

        let execucao = s
            .registrar_execucao_experimento(NovaExecucaoExperimento {
                id: "exp1".into(),
                case_id: "caso-1".into(),
                braco: "a".into(),
                run_id: "run1".into(),
                started_at: Instante(1000),
            })
            .unwrap();
        assert_eq!(execucao.braco, "a");

        let todas = s.execucoes_do_experimento(None).unwrap();
        assert_eq!(todas.len(), 1);
        assert_eq!(todas[0].case_id, "caso-1");
    }

    #[test]
    fn execucoes_do_experimento_filtra_por_braco() {
        let s = store();
        s.upsert_client("xpto").unwrap();
        s.criar_run(novo_run("run1")).unwrap();
        s.criar_run(novo_run("run2")).unwrap();

        s.registrar_execucao_experimento(NovaExecucaoExperimento {
            id: "exp1".into(),
            case_id: "caso-1".into(),
            braco: "a".into(),
            run_id: "run1".into(),
            started_at: Instante(1000),
        })
        .unwrap();
        s.registrar_execucao_experimento(NovaExecucaoExperimento {
            id: "exp2".into(),
            case_id: "caso-1".into(),
            braco: "b".into(),
            run_id: "run2".into(),
            started_at: Instante(1001),
        })
        .unwrap();

        let do_braco_a = s.execucoes_do_experimento(Some("a")).unwrap();
        assert_eq!(do_braco_a.len(), 1);
        assert_eq!(do_braco_a[0].id, "exp1");
    }
}
