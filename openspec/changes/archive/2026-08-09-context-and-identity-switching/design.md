## Context

v0.0 (`client-cost-attribution`, `capacity-windows-and-plans`) já existe: `Store`
trait em `src/storage/mod.rs` (D-9: todo SQL fica ali), `SqliteStore` único banco
(D-1), CLI via `clap` (`src/comandos.rs`). Nenhum daemon, nenhum `run` ainda — tudo
é invocação síncrona de `brian <comando>`.

Essa restrição molda o problema central desta change: `brian connect` no blueprint
troca a identidade que o **terminal do operador** usa quando ele roda `claude`/
`codex` diretamente, fora do controle do Brian. Sem daemon e sem hook de shell, o
Brian não pode reescrever o ambiente de um processo já em execução (o shell
interativo do operador). A solução Unix padrão para isso é a mesma do `direnv`/
`aws-vault`: o comando imprime instruções de exportação de variável de ambiente que
o operador aplica ao próprio shell.

## Goals / Non-Goals

**Goals:**

- Meta central do blueprint: "se `connect` não economizar tempo real de troca de
  contexto, nada mais no documento importa" — este design prioriza isso.
- Nenhum efeito colateral irreversível: desconectar sempre restaura o estado padrão.
- Vault nunca persiste valor de secret; toda a superfície de storage é testável sem
  tocar no Keychain real.

**Non-Goals:**

- Reescrever o ambiente de um shell já aberto sem ação do operador — sem daemon,
  isso exigiria um hook de shell (`.bashrc`/`.zshrc`) que este design não instala
  automaticamente. `brian connect` imprime o que precisa ser avaliado; o operador
  decide aplicar.
- Continuity Pack / handoff de memória — `continuity-pack-handoff` (próxima change).
- Backends de secret além do Keychain — v1.0+.
- Bloqueio automático de operação por política (deploy produção com aprovação) —
  v0.2, quando `run` existir para ter o que bloquear.

## Decisions

### `brian connect` imprime exports; não abre subshell nem reescreve `.bashrc`

Duas alternativas descartadas antes desta:

*Subshell dedicado* (`brian connect xpto` abre um novo shell com o ambiente já
setado, `exit` para sair). Rejeitada: quebra o fluxo de quem já está num shell com
outras ferramentas abertas (tmux, IDE integrado); nenhuma ferramenta Unix madura
(`aws-vault`, `direnv`) faz isso como caminho principal.

*Hook de shell instalado* (Brian escreve em `.zshrc`/`.bashrc` para interceptar
`cd`). Rejeitada: modifica arquivo de configuração do operador sem pedido explícito,
e o comportamento passa a depender de qual shell/hook está instalado — superfície
maior que o problema exige.

Escolhida: `brian connect xpto` grava o contexto ativo no banco (fonte de verdade
para o próprio Brian) e imprime `export VAR=valor` por linha para
`eval "$(brian connect xpto)"` — mesmo padrão de `aws-vault exec`. Reversível:
`brian disconnect` imprime os `unset` correspondentes.

### Contexto ativo persistido no SQLite, não em arquivo solto

`active_context` como tabela de linha única (`PRIMARY KEY` fixo, tipo singleton) em
vez de `~/.brian/context.json` separado. Razão: D-1 já estabelece um único banco
como fonte de verdade; um segundo arquivo de estado duplicaria a pergunta "qual é a
verdade" que D-1 existe para evitar. Comandos que o Brian mesmo invoca (`import`,
`capacity`) leem o contexto ativo do banco e aplicam a identidade ao processo filho
que eles próprios lançam — funciona independentemente de o operador ter avaliado o
`export` no shell ou não.

### Identidade Git via variável de ambiente, não `git config --local`

`GIT_AUTHOR_NAME`/`GIT_AUTHOR_EMAIL`/`GIT_COMMITTER_NAME`/`GIT_COMMITTER_EMAIL` em
vez de escrever no `.git/config` do repositório.

*Razão:* `git config --local` muta um arquivo do repositório do operador — efeito
que sobrevive ao `disconnect` e pode vazar para outro branch/worktree. Variável de
ambiente é escopada à sessão do shell (ou ao processo filho que o Brian lança),
reversível por construção, e é o mecanismo que o próprio Git documenta para esse
exato caso de uso.

### Vault: `security-framework` (bindings oficiais da Apple), trait própria testável sem Keychain real

`src/vault.rs` define uma trait `Vault` com um backend `KeychainVault` (usa
`security_framework::passwords` e `access_control::SecAccessControl`) e um backend
`VaultFalso` só para teste (`HashMap` em memória, mesmo padrão de
`storage::sqlite::SqliteStore` vs. um fake em `:memory:`). Nenhum teste automatizado
grava no Keychain real — `cargo test` nunca dispara um prompt de Touch ID.

*Gate biométrico:* `SecAccessControl::create_with_flags` com a flag
`kSecAccessControlBiometryAny` do `Security.framework` — exige Touch ID válido,
falha (não cai para senha) quando biometria está indisponível, consistente com o
spec ("SHALL NOT cair para autenticação mais fraca em silêncio"). **Valor exato da
flag a confirmar contra o SDK instalado na máquina antes de codar** (tarefa
dedicada) — não adivinhar o bit e testar contra o Keychain real do operador sem essa
confirmação.

*Alternativa considerada:* invocar o CLI `security` via subprocess
(`security add-generic-password -w <valor>`). Rejeitada: o valor do secret passaria
por argv/stdin do processo, visível via `ps`/histórico de shell — exatamente o que a
FFI direta evita.

### `credential_ref`: referência + metadados, nunca o valor

```text
credential_ref(id, label, keychain_service, keychain_account, class,
                created_at, expires_at, last_used_at, rotation_policy)
```

Nenhuma coluna de valor. `class` (`low|medium|high|critical`) decide, na leitura, se
`KeychainVault::resolve` passa `SecAccessControl` biométrico ou não.

### `isolation_verified` é dado declarado, não inferido

Mesma disciplina do `campos_disponiveis()` dos adapters de `ColetorDeUso`
(client-cost-attribution): a lista de providers com isolamento verificado é mantida
à mão, testada contra evidência real (duas identidades simultâneas funcionando),
nunca presumida por "o provider tem uma flag de config home, deve isolar direito".

## Risks / Trade-offs

- **`eval "$(brian connect ...)"` é um passo manual** — o operador pode esquecer de
  rodar o `eval` e achar que está no contexto certo quando não está. Mitigação:
  `brian whoami` sempre mostra a conta autenticada de cada provider, não só "há
  contexto ativo" — se o `eval` não rodou, o provider mostra a conta pessoal, visível
  na consulta.
- **Flag de `SecAccessControl` errada trava o Vault de forma confusa** — mitigado
  adiando a confirmação do valor exato para uma tarefa dedicada com verificação
  manual antes de qualquer código depender dela.
- **Nenhum provider real testado nesta máquina tem `isolation_verified = true`
  ainda** — até a task de verificação rodar duas identidades simultâneas de verdade,
  a lista começa vazia; nenhum provider ganha isolamento paralelo por suposição.

## Achado real: gate biométrico exige binário assinado (task 7.4)

Verificação manual contra o Keychain real (não simulada, `KeychainVault` de
verdade) confirmou:

- Classe `low`/`medium` (sem `SecAccessControl`): grava e resolve normalmente.
- Classe `high`/`critical` (com `AccessControlOptions::BIOMETRY_ANY`): a
  **gravação** falha com `errSecMissingEntitlement` ("A required entitlement
  isn't present") num binário `cargo build`/`cargo test` comum — ad-hoc,
  sem assinatura de código nem entitlement `keychain-access-groups`.

Isto **não é um defeito no código**: o próprio blueprint já previa essa exigência
(§7.4, "Access Control Lists — binding ao binário assinado do Brian") antes desta
implementação existir. macOS exige que o processo que grava um item com controle de
acesso biométrico seja assinado com o entitlement correto — um binário de
desenvolvimento não assinado não satisfaz isso, independentemente de o código Rust
estar certo.

**Consequência prática desta change:** `armazenar()`/`resolver()` para classes
`high`/`critical` funcionam corretamente contra `VaultFalso` (toda a lógica de
decisão de classe, e o valor da flag confirmado contra o SDK real — task 3.1), mas
só funcionam contra o Keychain real depois que o binário `brian` for assinado com um
certificado de desenvolvedor Apple e o entitlement `keychain-access-groups`. Isso é
trabalho de empacotamento/distribuição, fora do escopo de código desta change — e
seria descoberto tarde demais se não fosse verificado manualmente agora.

*Decisão do autor:* não bloquear esta change por isso. O mecanismo está correto e
testado no nível que este código controla; assinatura de binário é pré-requisito de
distribuição, não de implementação, e se aplica a qualquer funcionalidade futura que
peça Touch ID — registrado aqui para não ser redescoberto do zero na primeira vez que
alguém tentar usar uma credencial `critical` de verdade.

## Migration Plan

Migração aditiva `0003_identidade.sql`: `identity_profile`, `active_context`,
`credential_ref`. Nenhuma coluna de `usage_record`/`provider_plan`/`quota_signal`
muda. Reversão: apagar `~/.brian/brian.db` remove o estado do Brian; nenhuma
credencial real é apagada do Keychain por isso — o Vault só perde a referência,
não o item, e o operador pode limpar o item do Keychain manualmente se quiser.
