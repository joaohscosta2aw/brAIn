//! Adapters de provider (grupo 6).
//!
//! **Risco aceito conscientemente (design.md, R-4):** leitura de arquivo de
//! sessão, sem verificação prévia dos termos de uso de cada provider. Mitigação
//! que permanece: cada adapter lê **somente campos de uso** (tokens, custo,
//! modelo, identificador de chamada, instante) — nunca prompt, resposta, ou
//! qualquer conteúdo de conversa.

pub mod claude;
pub mod codex;
pub mod copilot;
pub mod gemini;
pub mod grok;
pub mod qwen;
pub(crate) mod tempo;

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

/// Cobertura declarada dos providers ativos nesta versão da
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
            status: StatusCobertura::Verificado,
        },
        CoberturaProvider {
            provider_id: "gemini",
            // Verificado, mas com a ressalva mais forte de toda a cobertura:
            // este adapter não produz ledger no sentido pleno. Oito fontes
            // locais investigadas, nenhuma com uso/custo por chamada (ver
            // histórico do design.md) -- confirmado por issues públicas do
            // repositório oficial que a contabilização é só no servidor
            // (v1internal:retrieveUserQuota, issue #387), sem exportação
            // local (issue #366) e sem subcomando dedicado (issue #543).
            // Decisão consciente do autor: usar `agy --print "/usage"` como
            // sinal de presença -- sem tokens, sem custo, sem atribuição a
            // cliente (a cota é por conta, não por projeto). Ver
            // src/adapters/gemini.rs e design.md, seção "Gemini: sinal sem
            // rastreabilidade".
            status: StatusCobertura::Verificado,
        },
        CoberturaProvider {
            provider_id: "grok",
            status: StatusCobertura::Verificado,
        },
        CoberturaProvider {
            provider_id: "github-copilot",
            status: StatusCobertura::Verificado,
        },
        CoberturaProvider {
            provider_id: "qwen-deepseek",
            status: StatusCobertura::Verificado,
        },
        CoberturaProvider {
            provider_id: "qwen-zai",
            status: StatusCobertura::Verificado,
        },
        CoberturaProvider {
            provider_id: "qwen-kimi",
            status: StatusCobertura::NaoTentado {
                motivo: "Mesma fonte local que qwen-deepseek e qwen-zai (adapter já pronto, \
                         só falta filtrar model.starts_with(\"kimi\")), mas configurado e \
                         bloqueado por saldo do provider — zero chamadas bem-sucedidas \
                         registradas para verificar contra dado real. Não é limitação técnica \
                         da fonte nem falta de investigação"
                    .into(),
            },
        },
        CoberturaProvider {
            provider_id: "zcode",
            status: StatusCobertura::SemFonteUtilizavel {
                motivo: "Sem CLI — só existe app nativo Mac (Electron). \
                         Inspecionado: ~/Library/Application Support/ZCode/ \
                         contém apenas armazenamento interno do motor Chromium \
                         (LevelDB, cache, blob storage), sem log de uso \
                         documentado. Extrair dali seria engenharia reversa de \
                         formato interno não estável, categoria de risco \
                         diferente e maior do que ler um JSONL documentado — \
                         não tentado por essa razão, não por falta de esforço"
                    .into(),
            },
        },
    ]
}
