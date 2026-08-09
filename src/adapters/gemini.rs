//! Adapter Gemini via Antigravity (`agy`) — tier `headless_json` (D-4).
//!
//! **Único adapter desta change que invoca o provider ao vivo** em vez de
//! ler arquivo local — `agy --print "/usage"` é uma consulta ao servidor,
//! não uma leitura de histórico (design.md explica por quê: uso do Gemini
//! via Antigravity é rastreado só no servidor, sem log local acessível).
//!
//! Sinal **sem rastreabilidade por chamada**, decisão consciente do autor
//! (design.md): sem tokens (`/usage` só dá percentual, não conta absoluta),
//! sem custo, sem atribuição a cliente (a cota é por conta, não por
//! projeto). Existe só para não deixar consumo real de Gemini invisível no
//! relatório. `usage_source = Estimated` sinaliza essa natureza distinta
//! para quem consumir o ledger depois.
//!
//! Risco de ToS aceito conscientemente (design.md, R-4) — aqui nem se
//! aplica da mesma forma que os demais: não há arquivo de sessão sendo lido,
//! só uma consulta de cota que o próprio `/usage` interativo também faz.

use super::tempo::parse_timestamp_iso8601;
use crate::capacidade::ColetorDeCapacidade;
use crate::domain::{
    BillingMode, Instante, PlanoDetectado, SinalDeQuotaColetado, Tokens, UsageSource,
};
use crate::importacao::{ColetorDeUso, ConsumoColetado, ErroColeta, TierIntegracao};
use crate::storage::Periodo;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

/// `agy` tem seu próprio timeout interno de 60s esperando autenticação
/// quando não há credencial em cache — independente do `stdin` do processo
/// pai. Sem limite aqui, `brian import` ficaria parado até 1 minuto por
/// chamada toda vez que o Gemini não estiver autenticado. 5s é suficiente
/// para uma resposta real de `/usage` (é uma consulta de cota, não uma
/// geração de modelo) e curto o bastante para não travar o import inteiro.
const TIMEOUT_AGY: Duration = Duration::from_secs(5);

pub struct GeminiAdapter {
    /// Injetável para teste — em produção é sempre `agy`.
    executavel: String,
}

impl GeminiAdapter {
    pub fn new() -> Self {
        Self {
            executavel: "agy".to_string(),
        }
    }

    /// Invoca `agy --print "/usage"` e devolve a saída bruta — compartilhado
    /// entre `coletar` (sinal de presença no ledger) e `consultar` (sinal de
    /// quota fiel, capacity-windows-and-plans). `stdin` nulo sempre e
    /// timeout explícito: mesmas duas proteções do incidente registrado no
    /// design.md, necessárias em toda chamada a este binário, não só na
    /// primeira que as motivou.
    fn executar_agy(&self) -> Result<String, ErroColeta> {
        let filho = Command::new(&self.executavel)
            .args(["--print", "/usage", "--output-format", "json"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ErroColeta {
                motivo: format!("não foi possível executar `{}`: {e}", self.executavel),
            })?;

        let saida = aguardar_com_timeout(filho, TIMEOUT_AGY)?;

        if !saida.status.success() {
            return Err(ErroColeta {
                motivo: format!(
                    "`{}` retornou erro: {}",
                    self.executavel,
                    String::from_utf8_lossy(&saida.stderr)
                ),
            });
        }

        Ok(String::from_utf8_lossy(&saida.stdout).into_owned())
    }
}

/// Espera o processo terminar até `timeout`; mata e devolve erro se estourar.
/// Sem dependência nova: só `try_wait()` num laço com espera curta, que é o
/// que uma crate de "wait com timeout" faria por baixo dos panos mesmo.
fn aguardar_com_timeout(
    mut filho: Child,
    timeout: Duration,
) -> Result<std::process::Output, ErroColeta> {
    let inicio = Instant::now();
    loop {
        match filho.try_wait() {
            Ok(Some(_)) => {
                return filho.wait_with_output().map_err(|e| ErroColeta {
                    motivo: format!("erro coletando saída: {e}"),
                });
            }
            Ok(None) => {
                if inicio.elapsed() >= timeout {
                    let _ = filho.kill();
                    let _ = filho.wait();
                    return Err(ErroColeta {
                        motivo: format!(
                            "`agy` não respondeu em {}s — provavelmente aguardando autenticação",
                            timeout.as_secs()
                        ),
                    });
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(ErroColeta {
                    motivo: format!("erro aguardando `agy`: {e}"),
                });
            }
        }
    }
}

impl Default for GeminiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SinalDeConsumo {
    bucket_id: String,
    grupo: String,
    reset_time: String,
    /// Preservado para `consultar` (task 5.1) — `coletar` não usa este
    /// campo, só bucket_id/grupo/reset_time para o sinal de presença.
    remaining_fraction: f64,
}

/// Extrai um sinal por bucket com `remaining_fraction < 1.0` — evidência
/// real de consumo, não suposição. Buckets em 100% não geram sinal.
fn extrair_sinais(saida_json: &str) -> Vec<SinalDeConsumo> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(saida_json) else {
        return Vec::new();
    };
    let Some(groups) = v["command"]["data"]["groups"].as_array() else {
        return Vec::new();
    };

    let mut sinais = Vec::new();
    for group in groups {
        let Some(nome_grupo) = group["name"].as_str() else {
            continue;
        };
        let Some(buckets) = group["buckets"].as_array() else {
            continue;
        };
        for bucket in buckets {
            let fracao = bucket["remaining_fraction"].as_f64().unwrap_or(1.0);
            if fracao >= 1.0 {
                continue; // sem evidência de consumo nesta janela
            }
            let (Some(id), Some(reset_time)) =
                (bucket["id"].as_str(), bucket["reset_time"].as_str())
            else {
                continue;
            };
            sinais.push(SinalDeConsumo {
                bucket_id: id.to_string(),
                grupo: nome_grupo.to_string(),
                reset_time: reset_time.to_string(),
                remaining_fraction: fracao,
            });
        }
    }
    sinais
}

impl ColetorDeUso for GeminiAdapter {
    fn provider_id(&self) -> &str {
        "gemini"
    }

    fn tier(&self) -> TierIntegracao {
        TierIntegracao::HeadlessJson
    }

    fn campos_disponiveis(&self) -> &[&'static str] {
        &[] // Nenhum campo de uso real -- sinal de presença, não de conteúdo.
    }

    fn coletar(&self, periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta> {
        let stdout = self.executar_agy()?;
        let agora = Instante::agora();

        if agora < periodo.desde || periodo.ate.is_some_and(|ate| agora >= ate) {
            return Ok(Vec::new()); // observação de agora cai fora do período pedido
        }

        let sinais = extrair_sinais(&stdout);
        Ok(sinais
            .into_iter()
            .map(|s| ConsumoColetado {
                identificador_estavel: Some(format!(
                    "gemini-quota:{}:{}",
                    s.bucket_id, s.reset_time
                )),
                provider_id: "gemini".to_string(),
                model: format!("agregado:{}", s.grupo),
                tokens: Tokens::default(), // tudo ausente -- ver módulo doc
                custo_pago: None,
                billing_mode: BillingMode::Subscription,
                usage_source: UsageSource::Estimated,
                session_ref: None,
                occurred_at: agora,
                // Nunca atribuído: /usage é por conta, não por projeto —
                // não há como saber que este workspace causou o consumo.
                client_id: None,
            })
            .collect())
    }
}

impl ColetorDeCapacidade for GeminiAdapter {
    fn provider_id(&self) -> &str {
        "gemini"
    }

    /// Mesma extração de `coletar` (`extrair_sinais`, task 5.1: reaproveitar,
    /// não escrever nova), mas aqui `remaining_fraction` e `reset_time` são
    /// preservados como sinal de quota — nada vira registro de ledger. `agy`
    /// não expõe plano por nome; o Gemini via Antigravity é sempre
    /// assinatura (design.md).
    fn consultar(&self) -> Result<(PlanoDetectado, Vec<SinalDeQuotaColetado>), ErroColeta> {
        let stdout = self.executar_agy()?;
        let sinais = extrair_sinais(&stdout);

        let janelas = sinais
            .into_iter()
            .map(|s| SinalDeQuotaColetado {
                bucket_id: s.bucket_id,
                grupo: s.grupo,
                remaining_percent: s.remaining_fraction * 100.0,
                reset_at: parse_timestamp_iso8601(&s.reset_time).map(Instante),
            })
            .collect();

        Ok((
            PlanoDetectado {
                billing_mode: BillingMode::Subscription,
                plan_label: None,
                account_email: None,
            },
            janelas,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
        "command": {
            "name": "usage",
            "data": {
                "groups": [
                    {
                        "name": "Gemini Models",
                        "buckets": [
                            {"id": "gemini-weekly", "window": "weekly", "remaining_fraction": 0.92, "reset_time": "2026-08-11T22:51:04Z"},
                            {"id": "gemini-5h", "window": "5h", "remaining_fraction": 0.98, "reset_time": "2026-08-08T22:29:01Z"}
                        ]
                    },
                    {
                        "name": "Claude and GPT models",
                        "buckets": [
                            {"id": "3p-weekly", "window": "weekly", "remaining_fraction": 1.0, "reset_time": "2026-08-15T19:33:22Z"},
                            {"id": "3p-5h", "window": "5h", "remaining_fraction": 1.0, "reset_time": "2026-08-09T00:33:22Z"}
                        ]
                    }
                ]
            }
        }
    }"#;

    #[test]
    fn extrai_sinal_so_de_buckets_com_consumo_real() {
        let sinais = extrair_sinais(FIXTURE);
        assert_eq!(
            sinais.len(),
            2,
            "só os 2 buckets de Gemini têm fração < 1.0"
        );
        assert!(sinais.iter().all(|s| s.grupo == "Gemini Models"));
        assert!(sinais.iter().any(|s| s.bucket_id == "gemini-weekly"));
        assert!(sinais.iter().any(|s| s.bucket_id == "gemini-5h"));
    }

    #[test]
    fn bucket_em_100_por_cento_nao_gera_sinal() {
        let sinais = extrair_sinais(FIXTURE);
        assert!(!sinais.iter().any(|s| s.bucket_id.starts_with("3p")));
    }

    #[test]
    fn json_invalido_devolve_vazio_sem_panico() {
        assert!(extrair_sinais("isto não é json").is_empty());
        assert!(extrair_sinais("{}").is_empty());
    }

    #[test]
    fn dedup_key_e_estavel_por_janela_nao_por_execucao() {
        // Duas extrações da mesma saída (simulando duas chamadas de import
        // na mesma janela) devem produzir o mesmo par bucket_id+reset_time --
        // é isso que torna reimportar a mesma janela idempotente.
        let a = extrair_sinais(FIXTURE);
        let b = extrair_sinais(FIXTURE);
        assert_eq!(a, b);
    }

    #[test]
    fn campos_disponiveis_e_vazio_declarado() {
        let adapter = GeminiAdapter::new();
        assert!(
            adapter.campos_disponiveis().is_empty(),
            "sinal de presença não declara nenhum campo de conteúdo real"
        );
    }

    #[test]
    fn tier_e_headless_json_nao_session_files() {
        let adapter = GeminiAdapter::new();
        assert_eq!(adapter.tier(), TierIntegracao::HeadlessJson);
    }

    /// Cria um executável de mentira que só dorme — nunca toca no `agy`
    /// real nem em qualquer serviço de autenticação de verdade.
    fn script_que_dorme(segundos: u64) -> std::path::PathBuf {
        let caminho = crate::testutil::dir_temporario_unico("fake-agy-dorme");
        std::fs::write(&caminho, format!("#!/bin/sh\nsleep {segundos}\n")).unwrap();
        std::fs::set_permissions(
            &caminho,
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .unwrap();
        caminho
    }

    fn script_que_responde(saida_json: &str) -> std::path::PathBuf {
        let caminho = crate::testutil::dir_temporario_unico("fake-agy-responde");
        std::fs::write(
            &caminho,
            format!("#!/bin/sh\ncat <<'EOF'\n{saida_json}\nEOF\n"),
        )
        .unwrap();
        std::fs::set_permissions(
            &caminho,
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .unwrap();
        caminho
    }

    #[test]
    fn executavel_que_nao_responde_falha_apos_timeout_nao_trava() {
        let script = script_que_dorme(30); // bem maior que TIMEOUT_AGY
        let adapter = GeminiAdapter {
            executavel: script.to_string_lossy().to_string(),
        };

        let inicio = Instant::now();
        let resultado = adapter.coletar(Periodo {
            desde: Instante(0),
            ate: None,
        });
        let duracao = inicio.elapsed();

        assert!(resultado.is_err(), "deve falhar, não travar esperando");
        assert!(
            duracao < Duration::from_secs(10),
            "deve respeitar o timeout de {}s, levou {duracao:?}",
            TIMEOUT_AGY.as_secs()
        );

        std::fs::remove_file(&script).ok();
    }

    #[test]
    fn executavel_que_responde_rapido_funciona_normalmente() {
        let script = script_que_responde(FIXTURE);
        let adapter = GeminiAdapter {
            executavel: script.to_string_lossy().to_string(),
        };

        let itens = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();

        assert_eq!(itens.len(), 2);

        std::fs::remove_file(&script).ok();
    }

    #[test]
    fn consultar_preserva_percentual_e_reset_descartados_pelo_ledger() {
        let script = script_que_responde(FIXTURE);
        let adapter = GeminiAdapter {
            executavel: script.to_string_lossy().to_string(),
        };

        let (plano, sinais) = adapter.consultar().unwrap();
        assert_eq!(plano.billing_mode, BillingMode::Subscription);
        assert_eq!(plano.plan_label, None, "agy não expõe plano por nome");

        assert_eq!(sinais.len(), 2, "mesmos 2 buckets que coletar() vê");
        let semanal = sinais
            .iter()
            .find(|s| s.bucket_id == "gemini-weekly")
            .unwrap();
        assert_eq!(semanal.remaining_percent, 92.0);
        assert!(
            semanal.reset_at.is_some(),
            "reset_time deve ser parseado para Instante"
        );
    }

    #[test]
    fn consultar_com_processo_que_nao_responde_falha_apos_timeout() {
        let script = script_que_dorme(30);
        let adapter = GeminiAdapter {
            executavel: script.to_string_lossy().to_string(),
        };

        let inicio = Instant::now();
        let resultado = adapter.consultar();
        assert!(resultado.is_err());
        assert!(inicio.elapsed() < Duration::from_secs(10));

        std::fs::remove_file(&script).ok();
    }
}
