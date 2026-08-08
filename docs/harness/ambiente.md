# Ambiente

O que precisa existir na máquina antes de executar qualquer task, e quais são os
comandos canônicos do projeto.

Enquanto não houver código, este documento é o contrato que a task 1.1 deve
cumprir. Depois dela, ele descreve o que já existe.

## Pré-requisitos

| Requisito | Estado | Observação |
|---|---|---|
| Rust (stable) | **ausente** | Não instalado nesta máquina. Instale via `rustup` antes da task 1.1 |
| OpenSpec CLI | presente (1.8.0) | Necessário para descobrir e validar changes |
| git | presente | D-7 depende de worktrees |

A versão exata do Rust **não é fixada aqui de propósito**. Fixar um número que
não foi verificado é pior que não fixar. A task 1.1 registra a stable vigente no
momento da criação, em `rust-toolchain.toml`, e a partir daí ela é a verdade.

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
