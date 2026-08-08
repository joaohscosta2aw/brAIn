//! Adapters de provider (grupo 6).
//!
//! **Risco aceito conscientemente (design.md, R-4):** leitura de arquivo de
//! sessão, sem verificação prévia dos termos de uso de cada provider. Mitigação
//! que permanece: cada adapter lê **somente campos de uso** (tokens, custo,
//! modelo, identificador de chamada, instante) — nunca prompt, resposta, ou
//! qualquer conteúdo de conversa.

pub mod claude;
pub mod grok;

/// Estado de cobertura de um provider — task 6.7. Três estados, nunca dois:
/// `NaoTentado` e `SemFonteUtilizavel` parecem a mesma coisa de longe, mas não
/// são. Um provider sem CLI instalado nesta máquina não teve sua fonte
/// *inspecionada* — é `NaoTentado`. Só vira `SemFonteUtilizavel` depois de
/// alguém efetivamente checar e a fonte se mostrar insuficiente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusCobertura {
    /// Fixture real capturada, teste passa contra ela.
    Verificado,
    /// Fonte inspecionada e comprovadamente insuficiente. `motivo` diz por quê.
    SemFonteUtilizavel { motivo: String },
    /// Ainda não verificado. `motivo` diz o que falta — nunca aparece como
    /// ausência silenciosa de consumo (spec: cobertura sempre visível).
    NaoTentado { motivo: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoberturaProvider {
    pub provider_id: &'static str,
    pub status: StatusCobertura,
}

/// Cobertura declarada dos cinco providers do blueprint, nesta versão da
/// change. Atualizado à mão conforme adapters são verificados — não há como
/// derivar isso automaticamente sem executar cada CLI.
pub fn cobertura_v0_0() -> Vec<CoberturaProvider> {
    vec![
        CoberturaProvider {
            provider_id: "claude",
            status: StatusCobertura::Verificado,
        },
        CoberturaProvider {
            provider_id: "codex",
            status: StatusCobertura::NaoTentado {
                motivo: "CLI oficial não instalado nesta máquina".into(),
            },
        },
        CoberturaProvider {
            provider_id: "gemini",
            status: StatusCobertura::NaoTentado {
                motivo: "CLI presente, mas sem autenticação configurada nesta máquina \
                         (variável de ambiente de API key ausente) — não é limitação \
                         técnica da fonte, é credencial do operador"
                    .into(),
            },
        },
        CoberturaProvider {
            provider_id: "grok",
            status: StatusCobertura::Verificado,
        },
        CoberturaProvider {
            provider_id: "zcode",
            status: StatusCobertura::NaoTentado {
                motivo: "CLI oficial não instalado nesta máquina".into(),
            },
        },
    ]
}
