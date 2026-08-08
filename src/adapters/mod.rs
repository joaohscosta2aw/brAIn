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
pub mod grok;
mod tempo;

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
            status: StatusCobertura::SemFonteUtilizavel {
                motivo: "Diferente do ZCode: a CLI (`agy`, Antigravity) existe, é real, \
                         e a autenticação funciona — headless JSON confirmado com \
                         chamada real e tokens reais. O que falta é uma superfície de \
                         histórico consultável. Investigado: history.jsonl (histórico \
                         de prompt digitado, não uso), o SQLite em \
                         ~/.gemini/antigravity-cli/conversations/*.db (colunas `data` \
                         são blob protobuf sem schema — busca do token count real da \
                         chamada de teste como varint no blob não encontrou nada), e \
                         transcript.jsonl em brain/<id>/.system_generated/logs/ (é \
                         transcrição de conversa — conteúdo, não uso, fora do escopo \
                         por construção). Sem o .proto oficial do Antigravity, decodificar \
                         os blobs de forma confiável não é viável nesta investigação"
                    .into(),
            },
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
