//! Adapter GitHub Copilot CLI (tier `session_files`, D-4).
//!
//! Lê `~/.copilot/session-store.db`, um SQLite mantido pelo próprio Copilot
//! CLI com schema nomeado — o mais limpo entre os providers investigados:
//! sem protobuf, sem engenharia reversa. A tabela `assistant_usage_events`
//! existe exatamente para isso.
//!
//! **Conexão somente-leitura, sempre.** Este é o banco vivo de outra
//! ferramenta; nunca escrevemos nele, mesmo que `rusqlite` permitisse.
//!
//! Risco de ToS aceito conscientemente (design.md, R-4).

use super::tempo::parse_timestamp_iso8601;
use crate::domain::{BillingMode, Instante, Tokens, UsageSource};
use crate::importacao::{ColetorDeUso, ConsumoColetado, ErroColeta, TierIntegracao};
use crate::storage::Periodo;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;

pub struct CopilotAdapter {
    db_path: PathBuf,
    cwd: PathBuf,
}

impl CopilotAdapter {
    pub fn new(cwd: PathBuf) -> Self {
        let db_path = dirs_home().join(".copilot").join("session-store.db");
        Self { db_path, cwd }
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

impl ColetorDeUso for CopilotAdapter {
    fn provider_id(&self) -> &str {
        "github-copilot"
    }

    fn tier(&self) -> TierIntegracao {
        TierIntegracao::SessionFiles
    }

    fn campos_disponiveis(&self) -> &[&'static str] {
        &["tokens", "model", "occurred_at", "identificador_estavel"]
        // Sem "custo": Copilot cobra em "premium requests"/nano-AIU, unidade
        // interna sem correspondência confirmada em dólar -- fora do escopo
        // de Custo::pago, que é especificamente moeda real.
    }

    fn coletar(&self, periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta> {
        if !self.db_path.is_file() {
            return Ok(Vec::new());
        }

        let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| ErroColeta {
                motivo: format!("não foi possível abrir {}: {e}", self.db_path.display()),
            })?;

        let mut stmt = conn
            .prepare(
                "SELECT e.session_id, e.turn_index, e.model, e.input_tokens, \
                        e.output_tokens, e.cache_read_tokens, e.cache_write_tokens, \
                        e.reasoning_tokens, e.created_at
                 FROM assistant_usage_events e
                 JOIN sessions s ON e.session_id = s.id
                 WHERE s.cwd = ?1",
            )
            .map_err(|e| ErroColeta {
                motivo: format!("erro preparando consulta: {e}"),
            })?;

        let cwd = self.cwd.to_string_lossy().to_string();
        let linhas = stmt
            .query_map([cwd], |row| {
                Ok((
                    row.get::<_, String>(0)?,      // session_id
                    row.get::<_, Option<i64>>(1)?, // turn_index
                    row.get::<_, String>(2)?,      // model
                    row.get::<_, Option<u64>>(3)?, // input_tokens
                    row.get::<_, Option<u64>>(4)?, // output_tokens
                    row.get::<_, Option<u64>>(5)?, // cache_read_tokens
                    row.get::<_, Option<u64>>(6)?, // cache_write_tokens
                    row.get::<_, Option<u64>>(7)?, // reasoning_tokens
                    row.get::<_, String>(8)?,      // created_at
                ))
            })
            .map_err(|e| ErroColeta {
                motivo: format!("erro consultando eventos de uso: {e}"),
            })?;

        let mut resultado = Vec::new();
        for linha in linhas {
            let (
                session_id,
                turn_index,
                model,
                input,
                output,
                cache_read,
                cache_write,
                reasoning,
                created_at,
            ) = linha.map_err(|e| ErroColeta {
                motivo: format!("erro lendo linha: {e}"),
            })?;

            let Some(occurred_at) = parse_timestamp_iso8601(&created_at).map(Instante) else {
                continue; // timestamp em formato inesperado -- pula, não inventa
            };

            if occurred_at < periodo.desde {
                continue;
            }
            if let Some(ate) = periodo.ate
                && occurred_at >= ate
            {
                continue;
            }

            let cache = match (cache_read, cache_write) {
                (None, None) => None,
                (a, b) => Some(a.unwrap_or(0) + b.unwrap_or(0)),
            };

            resultado.push(ConsumoColetado {
                identificador_estavel: Some(format!("{session_id}:{}", turn_index.unwrap_or(-1))),
                provider_id: "github-copilot".to_string(),
                model,
                tokens: Tokens {
                    input,
                    cache,
                    output,
                    reasoning,
                },
                custo_pago: None,
                billing_mode: BillingMode::Subscription,
                usage_source: UsageSource::Provider,
                session_ref: Some(session_id),
                occurred_at,
                client_id: None,
            });
        }

        Ok(resultado)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cria um `session-store.db` de teste com o mesmo schema do real
    /// (colunas relevantes), populado com dados sanitizados mas em formato
    /// e magnitude reais -- extraídos de uma chamada de teste desta sessão.
    fn criar_fixture(caminho: &std::path::Path) {
        let conn = Connection::open(caminho).unwrap();
        conn.execute_batch(
            "CREATE TABLE sessions (id TEXT PRIMARY KEY, cwd TEXT, repository TEXT);
             CREATE TABLE assistant_usage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                turn_index INTEGER,
                model TEXT NOT NULL,
                input_tokens INTEGER,
                output_tokens INTEGER,
                cache_read_tokens INTEGER,
                cache_write_tokens INTEGER,
                reasoning_tokens INTEGER,
                total_nano_aiu INTEGER,
                created_at TEXT
             );",
        )
        .unwrap();

        conn.execute(
            "INSERT INTO sessions (id, cwd, repository) VALUES ('sess-fixture-0001', '/projeto/teste', 'fixture/repo')",
            [],
        )
        .unwrap();
        // Sessão de outro projeto -- prova que o filtro por cwd exclui.
        conn.execute(
            "INSERT INTO sessions (id, cwd, repository) VALUES ('sess-fixture-outro', '/outro/projeto', 'fixture/outro')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO assistant_usage_events
                (session_id, turn_index, model, input_tokens, output_tokens, cache_read_tokens, cache_write_tokens, reasoning_tokens, total_nano_aiu, created_at)
             VALUES ('sess-fixture-0001', 0, 'claude-sonnet-5', 25401, 4, 0, 25399, 0, 6354150000, '2026-08-08T19:04:22.773Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO assistant_usage_events
                (session_id, turn_index, model, input_tokens, output_tokens, cache_read_tokens, cache_write_tokens, reasoning_tokens, total_nano_aiu, created_at)
             VALUES ('sess-fixture-outro', 0, 'claude-sonnet-5', 100, 10, 0, 0, 0, 1000000, '2026-08-08T19:05:00.000Z')",
            [],
        )
        .unwrap();
    }

    fn diretorio_temporario_unico() -> PathBuf {
        crate::testutil::dir_temporario_unico("adapter-copilot")
    }

    #[test]
    fn coletar_filtra_por_cwd_e_extrai_tokens_reais() {
        let dir = diretorio_temporario_unico();
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("session-store.db");
        criar_fixture(&db_path);

        let adapter = CopilotAdapter {
            db_path,
            cwd: PathBuf::from("/projeto/teste"),
        };

        let itens = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();

        assert_eq!(itens.len(), 1, "só o evento do projeto certo");
        let item = &itens[0];
        assert_eq!(item.model, "claude-sonnet-5");
        assert_eq!(item.tokens.input, Some(25401));
        assert_eq!(item.tokens.output, Some(4));
        assert_eq!(item.tokens.cache, Some(25399));
        assert_eq!(
            item.tokens.reasoning,
            Some(0),
            "reasoning explicitamente zero na fonte -- não é ausente"
        );
        assert_eq!(item.custo_pago, None, "sem custo em dólar exposto");
        assert_eq!(item.billing_mode, BillingMode::Subscription);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn coletar_de_banco_inexistente_devolve_vazio() {
        let adapter = CopilotAdapter {
            db_path: PathBuf::from("/caminho/inexistente/de/proposito.db"),
            cwd: PathBuf::from("/qualquer"),
        };
        let r = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn coletar_respeita_recorte_de_periodo() {
        let dir = diretorio_temporario_unico();
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("session-store.db");
        criar_fixture(&db_path);

        let adapter = CopilotAdapter {
            db_path,
            cwd: PathBuf::from("/projeto/teste"),
        };

        let ate = Instante(parse_timestamp_iso8601("2026-08-08T19:00:00.000Z").unwrap());
        let itens = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: Some(ate),
            })
            .unwrap();

        assert!(
            itens.is_empty(),
            "evento às 19:04 fica fora do corte às 19:00"
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
