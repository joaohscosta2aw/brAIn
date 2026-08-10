//! Security Gates (execution/security-gates, blueprint §30): secret scan
//! (`gitleaks`) obrigatório em todo `brian run`; SAST (`semgrep`) e
//! dependências vulneráveis (`osv-scanner`) sob demanda via `brian
//! security scan`. As três ferramentas são reais, instaladas e
//! verificadas contra fixtures antes desta capability existir (design.md)
//! -- nenhuma é simulada.

use std::path::Path;

/// Um achado de segurança -- `severidade` é o texto cru de cada
/// ferramenta, nunca normalizado entre elas (design.md: inventar uma
/// escala unificada fabricaria precisão que não existe).
#[derive(Debug, Clone, PartialEq)]
pub struct Achado {
    pub ferramenta: String,
    pub severidade: String,
    pub arquivo: String,
    pub linha: Option<u32>,
    pub mensagem: String,
}

#[derive(Debug)]
pub enum ErroSecurity {
    Processo(String, String),
    Json(String, String),
}

impl std::fmt::Display for ErroSecurity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Processo(ferramenta, m) => write!(f, "erro executando {ferramenta}: {m}"),
            Self::Json(ferramenta, m) => write!(f, "erro lendo saída de {ferramenta}: {m}"),
        }
    }
}

impl std::error::Error for ErroSecurity {}

fn caminho_temporario_unico(prefixo: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("brian-{prefixo}-{}-{nanos}", std::process::id()))
}

/// Regras padrão do gitleaks + `rust-const-secret` (`security/gitleaks-rust.toml`,
/// embutida no binário) -- a regra `generic-api-key` padrão não detecta
/// `pub const API_KEY: &str = "..."`, só `API_KEY = "..."` (verificado ao
/// vivo, design.md). `useDefault = true` mantém todas as regras padrão,
/// esta só soma.
const GITLEAKS_CONFIG: &str = include_str!("../security/gitleaks-rust.toml");

/// `gitleaks detect --source <dir> --no-git --config <config> -f json -r
/// <arquivo>` -- `-r /dev/stdout` não funciona (gitleaks recusa escrever
/// lá, verificado ao vivo), por isso escreve num arquivo temporário e lê
/// de volta. Exit code (0 limpo / 1 com achados) é ignorado -- só o
/// conteúdo do array importa.
pub fn rodar_gitleaks(dir: &Path) -> Result<Vec<Achado>, ErroSecurity> {
    let config = caminho_temporario_unico("gitleaks-config");
    std::fs::write(&config, GITLEAKS_CONFIG)
        .map_err(|e| ErroSecurity::Processo("gitleaks".to_string(), e.to_string()))?;

    let relatorio = caminho_temporario_unico("gitleaks-report");
    let saida = std::process::Command::new("gitleaks")
        .args([
            "detect",
            "--source",
            dir.to_str().unwrap_or_default(),
            "--no-git",
            "--config",
        ])
        .arg(&config)
        .args(["-f", "json", "-r"])
        .arg(&relatorio)
        .output()
        .map_err(|e| ErroSecurity::Processo("gitleaks".to_string(), e.to_string()));
    std::fs::remove_file(&config).ok();
    let saida = saida?;
    let _ = saida; // exit code não decide nada aqui, só o arquivo

    let texto = std::fs::read_to_string(&relatorio).unwrap_or_else(|_| "[]".to_string());
    std::fs::remove_file(&relatorio).ok();

    let v: serde_json::Value = serde_json::from_str(&texto)
        .map_err(|e| ErroSecurity::Json("gitleaks".to_string(), e.to_string()))?;

    Ok(v.as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| Achado {
                    ferramenta: "gitleaks".to_string(),
                    severidade: "secret".to_string(),
                    arquivo: item
                        .get("File")
                        .and_then(|x| x.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    linha: item
                        .get("StartLine")
                        .and_then(|x| x.as_u64())
                        .map(|n| n as u32),
                    mensagem: item
                        .get("Description")
                        .and_then(|x| x.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default())
}

/// `semgrep scan --config auto --json --quiet <dir>` -- exit code sempre
/// 0 nesta invocação (semgrep só falha por erro de execução, não por
/// achado), JSON no stdout.
pub fn rodar_semgrep(dir: &Path) -> Result<Vec<Achado>, ErroSecurity> {
    let saida = std::process::Command::new("semgrep")
        .args([
            "scan",
            "--config",
            "auto",
            "--json",
            "--quiet",
            dir.to_str().unwrap_or_default(),
        ])
        .output()
        .map_err(|e| ErroSecurity::Processo("semgrep".to_string(), e.to_string()))?;

    let texto = String::from_utf8_lossy(&saida.stdout);
    let v: serde_json::Value = serde_json::from_str(&texto)
        .map_err(|e| ErroSecurity::Json("semgrep".to_string(), e.to_string()))?;

    Ok(v.get("results")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| Achado {
                    ferramenta: "semgrep".to_string(),
                    severidade: item
                        .get("extra")
                        .and_then(|e| e.get("severity"))
                        .and_then(|x| x.as_str())
                        .unwrap_or("desconhecida")
                        .to_string(),
                    arquivo: item
                        .get("path")
                        .and_then(|x| x.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    linha: item
                        .get("start")
                        .and_then(|s| s.get("line"))
                        .and_then(|x| x.as_u64())
                        .map(|n| n as u32),
                    mensagem: item
                        .get("extra")
                        .and_then(|e| e.get("message"))
                        .and_then(|x| x.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default())
}

/// `osv-scanner scan source --format json -r <dir>` -- stdout vazio
/// significa "nenhum manifest de dependência encontrado" (verificado ao
/// vivo, exit code 128 nesse caso), tratado como zero achados, não erro.
pub fn rodar_osv_scanner(dir: &Path) -> Result<Vec<Achado>, ErroSecurity> {
    let saida = std::process::Command::new("osv-scanner")
        .args([
            "scan",
            "source",
            "--format",
            "json",
            "-r",
            dir.to_str().unwrap_or_default(),
        ])
        .output()
        .map_err(|e| ErroSecurity::Processo("osv-scanner".to_string(), e.to_string()))?;

    let texto = String::from_utf8_lossy(&saida.stdout);
    if texto.trim().is_empty() {
        return Ok(Vec::new());
    }
    let v: serde_json::Value = serde_json::from_str(&texto)
        .map_err(|e| ErroSecurity::Json("osv-scanner".to_string(), e.to_string()))?;

    let mut achados = Vec::new();
    for resultado in v
        .get("results")
        .and_then(|r| r.as_array())
        .into_iter()
        .flatten()
    {
        let arquivo = resultado
            .get("source")
            .and_then(|s| s.get("path"))
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string();
        for pacote in resultado
            .get("packages")
            .and_then(|p| p.as_array())
            .into_iter()
            .flatten()
        {
            let nome_pacote = pacote
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(|x| x.as_str())
                .unwrap_or("pacote desconhecido");
            for vuln in pacote
                .get("vulnerabilities")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
            {
                let id = vuln.get("id").and_then(|x| x.as_str()).unwrap_or("sem-id");
                let resumo = vuln.get("summary").and_then(|x| x.as_str()).unwrap_or("");
                achados.push(Achado {
                    ferramenta: "osv-scanner".to_string(),
                    severidade: id.to_string(),
                    arquivo: arquivo.clone(),
                    linha: None,
                    mensagem: format!("{nome_pacote}: {resumo}"),
                });
            }
        }
    }
    Ok(achados)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_git_temporario(prefixo: &str, arquivos: &[(&str, &str)]) -> std::path::PathBuf {
        let dir = crate::testutil::dir_temporario_unico(prefixo);
        std::fs::create_dir_all(&dir).unwrap();
        for (nome, conteudo) in arquivos {
            std::fs::write(dir.join(nome), conteudo).unwrap();
        }
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
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "inicial"]);
        dir
    }

    #[test]
    fn gitleaks_encontra_segredo_conhecido() {
        let dir = repo_git_temporario(
            "gitleaks-com-segredo",
            &[(
                "config.py",
                "API_KEY = \"sk-live-abcdef1234567890abcdef1234567890\"\n",
            )],
        );
        let achados = rodar_gitleaks(&dir).unwrap();
        assert!(!achados.is_empty());
        assert_eq!(achados[0].ferramenta, "gitleaks");
        assert!(achados[0].arquivo.ends_with("config.py"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn gitleaks_diretorio_limpo_nao_tem_achados() {
        let dir = repo_git_temporario("gitleaks-limpo", &[("app.py", "print('oi')\n")]);
        let achados = rodar_gitleaks(&dir).unwrap();
        assert!(achados.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn gitleaks_encontra_segredo_em_sintaxe_de_const_rust() {
        // Regra padrão do gitleaks não cobre isso -- exatamente a lacuna
        // encontrada no teste manual (security-gates/tasks.md 4.3),
        // fechada por security/gitleaks-rust.toml.
        let dir = repo_git_temporario(
            "gitleaks-const-rust",
            &[(
                "lib.rs",
                "pub const API_KEY: &str = \"sk-live-abcdef1234567890abcdef1234567890\";\n",
            )],
        );
        let achados = rodar_gitleaks(&dir).unwrap();
        assert!(
            !achados.is_empty(),
            "regra rust-const-secret deveria pegar isso"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn semgrep_encontra_vulnerabilidade_conhecida() {
        let dir = repo_git_temporario(
            "semgrep-com-vuln",
            &[(
                "app.py",
                "import subprocess\ndef f(x):\n    subprocess.run(x, shell=True)\n",
            )],
        );
        let achados = rodar_semgrep(&dir).unwrap();
        assert!(!achados.is_empty(), "semgrep deveria encontrar shell=True");
        assert_eq!(achados[0].ferramenta, "semgrep");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn semgrep_diretorio_limpo_nao_tem_achados() {
        let dir = repo_git_temporario("semgrep-limpo", &[("app.py", "print('oi')\n")]);
        let achados = rodar_semgrep(&dir).unwrap();
        assert!(achados.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn osv_scanner_encontra_cve_conhecido() {
        let dir = repo_git_temporario("osv-com-cve", &[("requirements.txt", "requests==2.6.0\n")]);
        let achados = rodar_osv_scanner(&dir).unwrap();
        assert!(
            !achados.is_empty(),
            "requests==2.6.0 tem CVE público conhecido"
        );
        assert_eq!(achados[0].ferramenta, "osv-scanner");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn osv_scanner_sem_manifest_nao_e_erro() {
        let dir = repo_git_temporario("osv-sem-manifest", &[("app.py", "print('oi')\n")]);
        let achados = rodar_osv_scanner(&dir).unwrap();
        assert!(achados.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }
}
