## Why

Blueprint §85 (v0.4) lista "Gemini e Grok como providers". Hoje
`execucao::PROVIDERS_EXECUCAO_VERIFICADA` só tem `codex` — `adapters::grok`
e `adapters::gemini` existem, mas só como coletores de consumo (leitura de
sessão/`/usage`), nunca invocados para *executar* uma tarefa. Verificação
manual real (não suposição): `grok -p "<tarefa>" --cwd <dir>
--permission-mode bypassPermissions` roda de forma genuinamente
não-interativa, edita arquivos, sai sem travar (~12s numa tarefa
pequena). `agy`/Gemini (Antigravity) não tem equivalente — é sessão
interativa, sem modo de execução scriptável — confirmado ao ler o próprio
adapter (`adapters/gemini.rs`: "`agy --print "/usage"` é uma consulta ao
servidor", nunca execução de tarefa).

## What Changes

- `execucao::PROVIDERS_EXECUCAO_VERIFICADA` ganha `"grok"`.
- `executar_provider` (hoje só sabe invocar `codex`) passa a despachar por
  `provider_id` — `grok` invocado via `grok --cwd <worktree> -p <tarefa>
  --permission-mode bypassPermissions [-m <model>]`.
- Gemini **não** entra — documentado como verificado e recusado
  (Não-objetivos), não como lacuna esquecida.

## Capabilities

### Modified Capabilities
- `execution/isolated-run`: mais um provider verificado, mesmo contrato.

## Impact

- `src/execucao.rs`: `PROVIDERS_EXECUCAO_VERIFICADA`, `executar_provider`
  refatorado para despachar por provider em vez de assumir `codex`
  implicitamente.

## Não-objetivos

- **Sem Gemini/Antigravity como execução verificada**: `agy` não tem modo
  de execução de tarefa não-interativo — é uma sessão de agente, não um
  CLI scriptável tipo `codex exec`/`grok -p`. Confirmado lendo o próprio
  adapter, não testado ao vivo (não há o que testar: a capability não
  existe). Continua coletor de consumo apenas.
- **Sem commit automático em nome do Grok**: `codex exec` cria commit
  próprio (por isso `execucao.rs` tem
  `aplicar_trailers_se_houver_commit_novo`); `grok -p` edita arquivos mas
  **não commita**. Isso já é um caminho existente e tratado sem erro
  (`aplicar_trailers_se_houver_commit_novo` já faz no-op quando
  `commit_depois == commit_antes`) — só significa que um run via Grok
  nunca ganha trailer de proveniência automático. Documentado, não
  escondido; Brian não passa a fazer `git commit` por nenhum provider.
- **Sem mudança de sandbox/permissão além de `bypassPermissions`**: é o
  modo que provou não travar em teste real; ajuste fino de perfil de
  sandbox do Grok fica para quando houver necessidade real observada.

## Conformidade — checklist §16

- **Honestidade de capability**: verificação manual real antes de declarar
  provider suportado, mesma disciplina já usada para `codex`
  (`PROVIDERS_EXECUCAO_VERIFICADA` só cresce com fonte confirmada).
- **Versão alvo**: blueprint §85 (v0.4) nominalmente pede Gemini E Grok —
  esta change entrega só a metade real (Grok); Gemini fica documentado
  como recusado, não pendente por engano.
