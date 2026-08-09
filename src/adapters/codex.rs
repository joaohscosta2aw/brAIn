//! Adapter Codex (tier `session_files`, D-4).
//!
//! Lê os arquivos de sessão em `~/.codex/sessions/AAAA/MM/DD/rollout-*.jsonl`.
//! Diferente do Claude e do Grok: as sessões do Codex são organizadas por
//! **data**, não por projeto — o filtro por `cwd` acontece lendo o próprio
//! conteúdo do arquivo (`session_meta.payload.cwd`), não o nome do diretório.
//!
//! Modelo e uso vêm em eventos separados dentro da mesma sessão:
//! `turn_context` traz o modelo (e carrega `developer_instructions`, um
//! campo de conteúdo que este adapter nunca lê); `token_count` traz os
//! tokens do turno em `info.last_token_usage`. Nenhum dos dois expõe custo em
//! dólar — Codex autenticado via ChatGPT é assinatura, não API por token.
//!
//! Risco de ToS aceito conscientemente (design.md, R-4).

use super::tempo::parse_timestamp_iso8601;
use crate::capacidade::ColetorDeCapacidade;
use crate::domain::{
    BillingMode, Instante, PlanoDetectado, SinalDeQuotaColetado, Tokens, UsageSource,
};
use crate::importacao::{ColetorDeUso, ConsumoColetado, ErroColeta, TierIntegracao};
use crate::storage::Periodo;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
struct UsoExtraido {
    session_id: String,
    turn_id: String,
    model: String,
    occurred_at: Instante,
    tokens: Tokens,
}

/// Estado acumulado ao varrer uma sessão linha a linha: o `turn_context` mais
/// recente é o que qualifica os `token_count` seguintes, até o próximo
/// `turn_context`.
#[derive(Default)]
struct EstadoSessao {
    session_id: Option<String>,
    turn_id: Option<String>,
    model: Option<String>,
}

/// Processa uma linha, atualizando o estado e devolvendo um uso quando a
/// linha for um `token_count` com contexto suficiente já visto.
///
/// Nunca lê `base_instructions` nem `developer_instructions` — o estado só
/// guarda `id`/`cwd`/`turn_id`/`model`, os únicos campos extraídos das linhas
/// `session_meta` e `turn_context`.
fn processar_linha(estado: &mut EstadoSessao, linha: &str) -> Option<UsoExtraido> {
    let v: serde_json::Value = serde_json::from_str(linha).ok()?;
    let tipo_evento = v["type"].as_str()?;
    let payload = &v["payload"];

    match tipo_evento {
        "session_meta" => {
            estado.session_id = payload["id"].as_str().map(String::from);
            None
        }
        "event_msg" => match payload["type"].as_str()? {
            "turn_context" => {
                estado.turn_id = payload["turn_id"].as_str().map(String::from);
                estado.model = payload["model"].as_str().map(String::from);
                None
            }
            "token_count" => {
                let session_id = estado.session_id.clone()?;
                let turn_id = estado.turn_id.clone()?;
                let model = estado.model.clone()?;
                let occurred_at = Instante(parse_timestamp_iso8601(v["timestamp"].as_str()?)?);

                let u = &payload["info"]["last_token_usage"];
                let campo_u64 = |nome: &str| u.get(nome).and_then(|x| x.as_u64());

                Some(UsoExtraido {
                    session_id,
                    turn_id,
                    model,
                    occurred_at,
                    tokens: Tokens {
                        input: campo_u64("input_tokens"),
                        cache: campo_u64("cached_input_tokens"),
                        output: campo_u64("output_tokens"),
                        reasoning: campo_u64("reasoning_output_tokens"),
                    },
                })
            }
            _ => None,
        },
        _ => None,
    }
}

/// `cwd` de uma sessão — lido só da linha `session_meta` (primeira do
/// arquivo), sem precisar ler o arquivo inteiro para decidir se ele é
/// relevante ao projeto pedido.
fn cwd_da_sessao(primeira_linha: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(primeira_linha).ok()?;
    if v["type"].as_str()? != "session_meta" {
        return None;
    }
    v["payload"]["cwd"].as_str().map(String::from)
}

pub struct CodexAdapter {
    raiz_sessions: PathBuf,
    cwd: PathBuf,
    /// Injetável para teste — em produção é sempre `codex`.
    executavel_appserver: String,
}

impl CodexAdapter {
    pub fn new(cwd: PathBuf) -> Self {
        let raiz_sessions = dirs_home().join(".codex").join("sessions");
        Self {
            raiz_sessions,
            cwd,
            executavel_appserver: "codex".to_string(),
        }
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `codex app-server` já responde rápido a estas duas chamadas (é leitura de
/// estado de conta, não geração de modelo) — mesma lógica de prazo curto do
/// timeout do Gemini (design.md: "risco aceito, mesmo padrão do incidente
/// Gemini").
const TIMEOUT_APP_SERVER: Duration = Duration::from_secs(5);

fn erro(motivo: impl Into<String>) -> ErroColeta {
    ErroColeta {
        motivo: motivo.into(),
    }
}

fn enviar(stdin: &mut ChildStdin, msg: &serde_json::Value) -> Result<(), ErroColeta> {
    let linha = format!("{msg}\n");
    stdin
        .write_all(linha.as_bytes())
        .map_err(|e| erro(format!("erro escrevendo para app-server: {e}")))?;
    stdin
        .flush()
        .map_err(|e| erro(format!("erro fazendo flush para app-server: {e}")))
}

/// Lê linhas até achar a resposta com `id` esperado, ignorando notificações
/// e respostas de outras chamadas que cheguem intercaladas (confirmado
/// contra o processo real: `remoteControl/status/changed` chega sem pedir).
fn ler_resposta(
    leitor: &mut BufReader<ChildStdout>,
    id_esperado: i64,
) -> Result<serde_json::Value, ErroColeta> {
    loop {
        let mut linha = String::new();
        let n = leitor
            .read_line(&mut linha)
            .map_err(|e| erro(format!("erro lendo app-server: {e}")))?;
        if n == 0 {
            return Err(erro("app-server encerrou antes de responder"));
        }
        let v: serde_json::Value = serde_json::from_str(linha.trim())
            .map_err(|e| erro(format!("resposta inválida do app-server: {e}")))?;
        if v.get("id").and_then(|x| x.as_i64()) == Some(id_esperado) {
            return Ok(v);
        }
        // notificação (sem id) ou resposta de outra chamada — ignora e segue lendo.
    }
}

/// Um bucket de janela extraído de `account/rateLimits/read` — `primary` e
/// `secondary` do payload, cada um com o formato que vira `SinalDeQuotaColetado`.
fn extrair_janelas(rate_limits: &serde_json::Value) -> Vec<SinalDeQuotaColetado> {
    let mut janelas = Vec::new();
    for bucket_id in ["primary", "secondary"] {
        let janela = &rate_limits[bucket_id];
        let Some(used_percent) = janela["usedPercent"].as_f64() else {
            continue; // ausente/null — este bucket não existe para o plano atual
        };
        let reset_at = janela["resetsAt"].as_i64().map(Instante);
        janelas.push(SinalDeQuotaColetado {
            bucket_id: bucket_id.to_string(),
            grupo: "rate_limits".to_string(),
            remaining_percent: 100.0 - used_percent,
            reset_at,
        });
    }
    janelas
}

/// Handshake `initialize` + as duas chamadas de conta, tudo síncrono sobre o
/// mesmo par stdin/stdout do processo já aberto por quem chama.
fn conversar(
    mut stdin: ChildStdin,
    stdout: ChildStdout,
) -> Result<(PlanoDetectado, Vec<SinalDeQuotaColetado>), ErroColeta> {
    let mut leitor = BufReader::new(stdout);

    enviar(
        &mut stdin,
        &serde_json::json!({
            "method": "initialize",
            "id": 0,
            "params": {"clientInfo": {"name": "brian", "title": "Brian", "version": "0.0.0"}}
        }),
    )?;
    ler_resposta(&mut leitor, 0)?;

    enviar(
        &mut stdin,
        &serde_json::json!({"method": "account/read", "id": 1, "params": {"refreshToken": false}}),
    )?;
    let resposta_conta = ler_resposta(&mut leitor, 1)?;

    enviar(
        &mut stdin,
        &serde_json::json!({"method": "account/rateLimits/read", "id": 2}),
    )?;
    let resposta_limites = ler_resposta(&mut leitor, 2)?;

    let plan_label = resposta_conta["result"]["account"]["planType"]
        .as_str()
        .map(String::from);
    let billing_mode = if plan_label.is_some() {
        BillingMode::Subscription
    } else {
        BillingMode::Api
    };
    let account_email = resposta_conta["result"]["account"]["email"]
        .as_str()
        .map(String::from);

    let janelas = extrair_janelas(&resposta_limites["result"]["rateLimits"]);

    Ok((
        PlanoDetectado {
            billing_mode,
            plan_label,
            account_email,
        },
        janelas,
    ))
}

impl ColetorDeCapacidade for CodexAdapter {
    fn provider_id(&self) -> &str {
        "codex"
    }

    /// Cliente JSON-RPC mínimo (design.md: "não um SDK completo") — abre o
    /// processo, faz o handshake, lê as duas respostas, encerra. Sem manter
    /// conexão viva entre importações, sem assinar notificações.
    fn consultar(&self) -> Result<(PlanoDetectado, Vec<SinalDeQuotaColetado>), ErroColeta> {
        let mut filho = Command::new(&self.executavel_appserver)
            .args(["app-server", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                erro(format!(
                    "não foi possível executar `{} app-server`: {e}",
                    self.executavel_appserver
                ))
            })?;

        let stdin = filho
            .stdin
            .take()
            .ok_or_else(|| erro("sem stdin do app-server"))?;
        let stdout = filho
            .stdout
            .take()
            .ok_or_else(|| erro("sem stdout do app-server"))?;

        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(conversar(stdin, stdout));
        });

        let resultado = rx.recv_timeout(TIMEOUT_APP_SERVER).unwrap_or_else(|_| {
            Err(erro(format!(
                "`{} app-server` não respondeu em {}s",
                self.executavel_appserver,
                TIMEOUT_APP_SERVER.as_secs()
            )))
        });

        let _ = filho.kill();
        let _ = filho.wait();

        resultado
    }
}

/// Lista recursivamente os `.jsonl` sob `raiz` (estrutura AAAA/MM/DD/*.jsonl,
/// mas não presumimos a profundidade — só filtramos pela extensão).
fn listar_jsonl_recursivo(raiz: &Path, saida: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !raiz.is_dir() {
        return Ok(());
    }
    for entrada in std::fs::read_dir(raiz)? {
        let entrada = entrada?;
        let caminho = entrada.path();
        if caminho.is_dir() {
            listar_jsonl_recursivo(&caminho, saida)?;
        } else if caminho.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            saida.push(caminho);
        }
    }
    Ok(())
}

impl ColetorDeUso for CodexAdapter {
    fn provider_id(&self) -> &str {
        "codex"
    }

    fn tier(&self) -> TierIntegracao {
        TierIntegracao::SessionFiles
    }

    fn campos_disponiveis(&self) -> &[&'static str] {
        &["tokens", "model", "occurred_at", "identificador_estavel"]
        // Sem "custo": Codex via login ChatGPT é assinatura, sem custo por
        // chamada exposto no arquivo de sessão.
    }

    fn coletar(&self, periodo: Periodo) -> Result<Vec<ConsumoColetado>, ErroColeta> {
        let mut arquivos = Vec::new();
        listar_jsonl_recursivo(&self.raiz_sessions, &mut arquivos).map_err(|e| ErroColeta {
            motivo: format!("erro listando {}: {e}", self.raiz_sessions.display()),
        })?;

        let cwd_alvo = self.cwd.to_string_lossy().to_string();
        let mut resultado = Vec::new();

        for caminho in arquivos {
            let conteudo = std::fs::read_to_string(&caminho).map_err(|e| ErroColeta {
                motivo: format!("não foi possível ler {}: {e}", caminho.display()),
            })?;

            let Some(primeira_linha) = conteudo.lines().next() else {
                continue;
            };
            if cwd_da_sessao(primeira_linha).as_deref() != Some(cwd_alvo.as_str()) {
                continue; // sessão de outro projeto — nem processamos o resto
            }

            let mut estado = EstadoSessao::default();
            for linha in conteudo.lines() {
                let Some(uso) = processar_linha(&mut estado, linha) else {
                    continue;
                };

                if uso.occurred_at < periodo.desde {
                    continue;
                }
                if let Some(ate) = periodo.ate
                    && uso.occurred_at >= ate
                {
                    continue;
                }

                resultado.push(ConsumoColetado {
                    identificador_estavel: Some(format!("{}:{}", uso.session_id, uso.turn_id)),
                    provider_id: "codex".to_string(),
                    model: uso.model,
                    tokens: uso.tokens,
                    custo_pago: None,
                    billing_mode: BillingMode::Subscription,
                    usage_source: UsageSource::Provider,
                    session_ref: Some(uso.session_id.clone()),
                    occurred_at: uso.occurred_at,
                    client_id: None,
                });
            }
        }

        Ok(resultado)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/codex_sessao.jsonl");

    #[test]
    fn processa_sessao_completa_e_extrai_dois_usos() {
        let mut estado = EstadoSessao::default();
        let usos: Vec<_> = FIXTURE
            .lines()
            .filter_map(|l| processar_linha(&mut estado, l))
            .collect();

        assert_eq!(usos.len(), 2);
        assert_eq!(usos[0].model, "gpt-5.4");
        assert_eq!(usos[0].turn_id, "turn-fixture-0001");
        assert_eq!(usos[0].tokens.input, Some(13375));
        assert_eq!(usos[0].tokens.reasoning, Some(34));

        // Segundo token_count usa last_token_usage (delta), não
        // total_token_usage (acumulado) -- são valores diferentes na fixture,
        // de propósito, para este teste distinguir os dois.
        assert_eq!(usos[1].turn_id, "turn-fixture-0002");
        assert_eq!(usos[1].tokens.input, Some(4625));
    }

    #[test]
    fn nunca_extrai_instructions() {
        let mut estado = EstadoSessao::default();
        for l in FIXTURE.lines() {
            processar_linha(&mut estado, l);
        }
        // Prova indireta: nenhum campo de UsoExtraido ou EstadoSessao tem
        // onde "SEGREDO" caberia -- os dois structs só carregam id/cwd
        // (descartado após uso)/turn_id/model/tokens/timestamp.
        let repr = format!(
            "{:?} {:?} {:?}",
            estado.session_id, estado.turn_id, estado.model
        );
        assert!(!repr.contains("SEGREDO"));
    }

    #[test]
    fn cwd_da_sessao_le_so_a_primeira_linha() {
        let primeira = FIXTURE.lines().next().unwrap();
        assert_eq!(cwd_da_sessao(primeira).as_deref(), Some("/projeto/teste"));
    }

    fn diretorio_temporario_unico() -> PathBuf {
        crate::testutil::dir_temporario_unico("adapter-codex")
    }

    #[test]
    fn coletar_ponta_a_ponta_filtra_por_cwd() {
        let raiz = diretorio_temporario_unico();
        let dia = raiz.join("2026").join("07").join("06");
        std::fs::create_dir_all(&dia).unwrap();
        std::fs::write(dia.join("rollout-fixture.jsonl"), FIXTURE).unwrap();

        // Sessão de outro projeto no mesmo diretório de data — deve ser
        // ignorada inteiramente.
        let outra = FIXTURE.replace("/projeto/teste", "/outro/projeto");
        std::fs::write(dia.join("rollout-outro.jsonl"), outra).unwrap();

        let adapter = CodexAdapter {
            raiz_sessions: raiz.clone(),
            cwd: PathBuf::from("/projeto/teste"),
            executavel_appserver: "codex".to_string(),
        };

        let itens = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();

        assert_eq!(itens.len(), 2, "só a sessão do projeto certo, 2 usos nela");
        assert!(itens.iter().all(|i| i.provider_id == "codex"));

        std::fs::remove_dir_all(&raiz).ok();
    }

    #[test]
    fn coletar_de_raiz_inexistente_devolve_vazio() {
        let adapter = CodexAdapter {
            raiz_sessions: PathBuf::from("/caminho/inexistente/de/proposito"),
            cwd: PathBuf::from("/qualquer"),
            executavel_appserver: "codex".to_string(),
        };
        let r = adapter
            .coletar(Periodo {
                desde: Instante(0),
                ate: None,
            })
            .unwrap();
        assert!(r.is_empty());
    }

    fn caminho_temporario_unico(nome: &str) -> PathBuf {
        crate::testutil::dir_temporario_unico(&format!("fake-codex-{nome}"))
    }

    /// Fake `codex app-server`: lê linhas JSON-RPC do stdin e responde por
    /// `id`, incluindo uma notificação sem `id` intercalada antes da
    /// resposta de `account/read` — exatamente o que o processo real fez
    /// (`remoteControl/status/changed`), para provar que `ler_resposta`
    /// não se engana com ela.
    fn script_app_server_que_responde() -> PathBuf {
        let caminho = caminho_temporario_unico("responde");
        std::fs::write(
            &caminho,
            r#"#!/bin/sh
while IFS= read -r line; do
  case "$line" in
    *'"id":0'*) echo '{"id":0,"result":{"userAgent":"test"}}' ;;
    *'"id":1'*)
      echo '{"method":"remoteControl/status/changed","params":{"status":"disabled"}}'
      echo '{"id":1,"result":{"account":{"type":"chatgpt","email":"x@y.com","planType":"team"},"requiresOpenaiAuth":true}}'
      ;;
    *'"id":2'*)
      echo '{"id":2,"result":{"rateLimits":{"primary":{"usedPercent":25,"windowDurationMins":10080,"resetsAt":1786844408},"secondary":null}}}'
      ;;
  esac
done
"#,
        )
        .unwrap();
        std::fs::set_permissions(
            &caminho,
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .unwrap();
        caminho
    }

    fn script_que_dorme(segundos: u64) -> PathBuf {
        let caminho = caminho_temporario_unico("dorme");
        std::fs::write(&caminho, format!("#!/bin/sh\nsleep {segundos}\n")).unwrap();
        std::fs::set_permissions(
            &caminho,
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .unwrap();
        caminho
    }

    #[test]
    fn extrair_janelas_le_primary_e_ignora_secondary_nulo() {
        let rate_limits = serde_json::json!({
            "primary": {"usedPercent": 25, "windowDurationMins": 10080, "resetsAt": 1_786_844_408i64},
            "secondary": null,
        });
        let janelas = extrair_janelas(&rate_limits);
        assert_eq!(janelas.len(), 1);
        assert_eq!(janelas[0].bucket_id, "primary");
        assert_eq!(janelas[0].remaining_percent, 75.0);
        assert_eq!(janelas[0].reset_at, Some(Instante(1_786_844_408)));
    }

    #[test]
    fn extrair_janelas_le_ambos_quando_presentes() {
        let rate_limits = serde_json::json!({
            "primary": {"usedPercent": 25, "resetsAt": 1},
            "secondary": {"usedPercent": 60, "resetsAt": 2},
        });
        assert_eq!(extrair_janelas(&rate_limits).len(), 2);
    }

    #[test]
    fn consultar_ponta_a_ponta_extrai_plano_e_janelas() {
        let script = script_app_server_que_responde();
        let adapter = CodexAdapter {
            raiz_sessions: PathBuf::from("/nao/usado"),
            cwd: PathBuf::from("/nao/usado"),
            executavel_appserver: script.to_string_lossy().to_string(),
        };

        let (plano, janelas) = adapter.consultar().unwrap();
        assert_eq!(plano.billing_mode, BillingMode::Subscription);
        assert_eq!(plano.plan_label.as_deref(), Some("team"));
        assert_eq!(plano.account_email.as_deref(), Some("x@y.com"));
        assert_eq!(janelas.len(), 1);
        assert_eq!(janelas[0].bucket_id, "primary");
        assert_eq!(janelas[0].remaining_percent, 75.0);

        std::fs::remove_file(&script).ok();
    }

    #[test]
    fn consultar_com_processo_que_nao_responde_falha_apos_timeout_nao_trava() {
        let script = script_que_dorme(30); // bem maior que TIMEOUT_APP_SERVER
        let adapter = CodexAdapter {
            raiz_sessions: PathBuf::from("/nao/usado"),
            cwd: PathBuf::from("/nao/usado"),
            executavel_appserver: script.to_string_lossy().to_string(),
        };

        let inicio = std::time::Instant::now();
        let resultado = adapter.consultar();
        let duracao = inicio.elapsed();

        assert!(resultado.is_err(), "deve falhar, não travar esperando");
        assert!(
            duracao < Duration::from_secs(10),
            "deve respeitar o timeout de {}s, levou {duracao:?}",
            TIMEOUT_APP_SERVER.as_secs()
        );

        std::fs::remove_file(&script).ok();
    }

    #[test]
    fn consultar_com_executavel_ausente_falha_sem_travar() {
        let adapter = CodexAdapter {
            raiz_sessions: PathBuf::from("/nao/usado"),
            cwd: PathBuf::from("/nao/usado"),
            executavel_appserver: "/caminho/que/nao/existe/de/proposito".to_string(),
        };
        assert!(adapter.consultar().is_err());
    }
}
