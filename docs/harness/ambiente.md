# Ambiente

O que precisa existir na máquina antes de executar qualquer task, e quais são os
comandos canônicos do projeto.

Enquanto não houver código, este documento é o contrato que a task 1.1 deve
cumprir. Depois dela, ele descreve o que já existe.

## Pré-requisitos

| Requisito | Estado | Observação |
|---|---|---|
| Rust | presente (1.97.1) | Instalado via Homebrew, com `clippy` 0.1.97 e `rustfmt` 1.9.0 |
| OpenSpec CLI | presente (1.8.0) | Necessário para descobrir e validar changes |
| git | presente | D-7 depende de worktrees |

**Sobre fixar a versão.** O Rust aqui é gerenciado pelo Homebrew, que **não lê**
`rust-toolchain.toml` — esse arquivo é um recurso do `rustup`. A task 1.1 cria o
arquivo mesmo assim, porque o CI usa rustup e o respeita, e porque uma eventual
migração para rustup passa a funcionar sem retrabalho. Localmente, a verdade é a
versão que o Homebrew tiver instalado.

Consequência prática: uma divergência entre a versão local e a do CI é possível.
Se ela causar problema real, aí sim vale migrar para `rustup` — não antes.

## Comandos canônicos

Definidos aqui para que os protocolos possam citá-los em vez de descrevê-los.
Todos passam a existir com a task 1.1.

| Ação | Comando |
|---|---|
| Compilar | `cargo build` |
| Testar | `cargo test` |
| Lint | `cargo clippy -- -D warnings` |
| Formatar | `cargo fmt` |
| Verificar formatação sem alterar | `cargo fmt --check` |
| Validar a especificação | `openspec validate --strict` |
| Descobrir a change ativa | `openspec list` |
| Verificar invariantes do projeto | `./scripts/verificar-invariantes.sh` |

Nenhum outro comando é canônico. Se um protocolo ou uma task pedir "rode os
testes", é `cargo test` que se entende.

## Estado local

O banco vive em `~/.brian/brian.db`, coerente com `~/.brian/brian.sock` que o
blueprint define para IPC.

**Reset completo:** apagar `~/.brian/brian.db`. Não há efeito colateral externo
a desfazer — nada é enviado, publicado ou modificado fora da máquina. Este é o
plano de reversão que o `design.md` da change #1 declara.

## O que a task 1.1 deve produzir

Para que este documento deixe de ser contrato e vire descrição:

- `Cargo.toml` com o binário `brian`
- `rust-toolchain.toml` fixando a stable vigente
- `cargo build`, `cargo test`, `cargo clippy` e `cargo fmt --check` executando
  com sucesso num clone limpo
