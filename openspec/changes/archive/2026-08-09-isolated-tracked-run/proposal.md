## Why

D-16 (v0.0) e D-17 (v0.1) estão verdes: nenhum token some, nenhuma troca de LLM
apaga a cabeça do trabalho. Mas o Brian ainda não faz o produto acontecer — ele só
observa. `isolated-tracked-run` é o terceiro OpenSpec candidato do blueprint
(§107.3) e o primeiro que executa: `brian run` de fato spawna um provider para
trabalhar, isolado por worktree (D-7), com o run persistido antes de qualquer
efeito colateral (D-12).

**Escopo revisado pelo autor em 2026-08-09**, confirmado explicitamente: só o
mínimo que satisfaz o critério de aceitação central entra agora — worktree, run
rastreado, `recover` de órfão. Telemetria OTel completa (§39, todos os spans),
limites de concorrência configuráveis, alocação de porta/banco por run, gates
determinísticos, router e workflow `governed` ficam para depois. Justificativa
completa em design.md.

## What Changes

- `brian run "<tarefa>" --provider <id>`: cria worktree dedicado a partir do commit
  base (D-7), persiste o run **antes** de invocar o provider (D-12), invoca o
  provider de forma não-interativa dentro do worktree, registra o resultado.
- Registro de eventos do run (log estruturado local — não OTel completo, ver
  design.md) suficiente para reconstruir o que aconteceu.
- Commit gerado (quando o provider produz um) carrega trailers de proveniência
  (`Brian-Run`, `Brian-Client`, `Brian-Context`, `Brian-Provider`, `Brian-Model`,
  `Brian-Cost-USD`) — blueprint §78.1.
- `brian recover`: detecta run marcado como em execução cujo processo morreu
  (SIGKILL não é capturável) e finaliza a contabilidade sem gerar novo custo —
  nunca re-executa a tarefa automaticamente.
- `brian worktree list`: visibilidade dos worktrees ativos/abandonados.
- Worktree promovido a branch dedicada ao final (sucesso) ou preservado para
  inspeção (falha/cancelamento) — nunca removido silenciosamente.

## Capabilities

### New Capabilities
- `execution/isolated-run`: ciclo de vida do run — worktree, persistência antes de
  efeito colateral, invocação do provider, resultado.
- `execution/orphan-recovery`: detecção de run órfão (processo morto) e
  finalização sem duplicar custo.
- `execution/provenance-trail`: trailers de proveniência no commit gerado.

### Modified Capabilities
(nenhuma — reaproveita `identity/context-switching` para resolver o contexto do
run e `capacity/cost-attribution` para o custo já capturado por import; esta
change não redefine atribuição de custo, só gera o consumo que ela já sabe
processar.)

## Impact

- Novo módulo `src/execucao.rs`.
- Novas tabelas: `run`, `run_event` — aditivas.
- Nova dependência: nenhuma — worktree via `git worktree` (subprocess, já usado
  por `continuidade.rs`), execução de provider via `Command` (já usado pelos
  adapters de `client-cost-attribution`/`capacity-windows-and-plans`).
- `brian run`, `brian recover`, `brian worktree list`.
- Providers com execução não-interativa confirmada nesta máquina: `codex exec`
  (`--sandbox workspace-write --ask-for-approval never`) e `claude -p`. Detalhe e
  trade-off de permissão em design.md.

## Conformidade (PREMISSAS-BASICAS.md §16)

- M1-M6, OP-1..OP-8: sim — é o primeiro subsistema que efetivamente poupa trabalho
  manual (M1), com direção clara e caminho curto (M5/OP-6).
- Toca D-16 ou D-17: usa os dois (consumo do run entra no ledger existente D-16;
  não introduz novo mecanismo de continuidade, D-17 já resolvido). Toca
  diretamente **D-7** (worktree obrigatório) e **D-12** (persistir antes de efeito
  colateral).
- Não viola D-10: opera por Context (cliente × tempo).
- Não depende de H-1: nenhuma dependência de Context Governor.
- Versão alvo: v0.2 — primeira change depois de D-16 (v0.0) e D-17 mínimo (v0.1)
  verdes, ordem sagrada do blueprint (§104: "só então worktree, run, workflow").

## Não-objetivos

- Telemetria OTel completa com todos os spans do blueprint §39 — um log de eventos
  local simples entra agora; o subsistema de tracing completo é trabalho futuro
  proporcional a quando houver volume real de runs para justificá-lo.
- Limites de concorrência configuráveis, alocação de porta/banco por run (§109.4,
  §109.5) — três runs concorrentes não colidem por construção (worktrees
  distintos, SQLite trata concorrência entre processos nativamente); limitar
  quantos rodam ao mesmo tempo é politica de recurso, não correção.
- Gates determinísticos (tests/lint/security antes de nova fase), router,
  workflow `governed`, `--compare` entre providers — exigem múltiplas fases e
  política, que esta change não introduz. Workflow desta change é uma fase única
  (R2: "default de workflow curto").
- `brian worktree gc` automático — o operador remove manualmente por ora; a
  visibilidade (`worktree list`) já torna isso possível sem automação.
