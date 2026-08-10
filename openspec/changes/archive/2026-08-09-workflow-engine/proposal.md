## Why

`brian run` hoje executa uma única fase (um provider, um gate). Tarefas
maiores precisam de mais de uma fase — implementar, verificar, corrigir se o
gate falhar, escalar pra humano se corrigir de novo não resolver — sem que
isso vire lógica if/else hardcoded no core (D-3: workflow é dado versionado).
Blueprint §15 define o Workflow Engine como máquina de estados determinística
sobre fases; cada fase, no vocabulário já existente do Brian, é só mais um
`brian run`.

## What Changes

- `workflows/<id>.json` (dado): fases com `role`, `gates`, `on_success`,
  `on_failure`, `max_entries`, `terminal`. `role` resolve para um
  `model_pointer` default (`builder→coding`, `planner→reasoning`,
  `reviewer→review`), a menos que a fase informe um `model_pointer`
  explícito — reaproveita `routing/model-pointers`, já implementada.
- `gates` de uma fase são concatenados num único comando (`&&`) e passados
  para o `--gate` já existente de `execucao::iniciar_run` — zero mudança no
  motor de execução.
- Cada fase não-terminal vira um `run` real via `execucao::iniciar_run`
  (mesmo worktree isolado, D-7/D-12 já garantidos). O Workflow Engine nunca
  chama provider diretamente — delega 100% para o que já existe.
- Nova tabela `workflow_run` (estado da máquina: `current_phase`,
  `total_phases`, `status`, `workflow_id`, `workflow_version` — congelado no
  início, blueprint §15.6/§113) e `workflow_phase_entry` (histórico de
  fases, cada entrada ligada ao `run` real que a executou).
- `brian workflow run <workflow_id> "<tarefa>"` roda a máquina até bater
  fase terminal, respeitando `max_entries` por fase e `limits` do workflow
  (`max_total_phases`, `max_wall_seconds`).
- Fase com `requires_approval: true` pausa o `workflow_run`
  (`status=paused`) — `brian workflow approve <workflow_run_id>` retoma.
- `brian workflow show <workflow_run_id>` mostra fase atual e histórico.

## Capabilities

### New Capabilities
- `workflow/state-machine`: máquina de estados determinística sobre fases —
  única autoridade de transição, nunca chama LLM diretamente (blueprint
  §15.5, fronteira Workflow×Reasoning).
- `workflow/phase-execution`: cada fase não-terminal executa como um `run`
  real via `execucao::iniciar_run`, reaproveitando gate e model pointer já
  existentes.
- `workflow/human-approval`: fase com `requires_approval` pausa o workflow
  até aprovação explícita do operador.

## Impact

- `src/storage/migrations/0006_workflow.sql` (novo): `workflow_run`,
  `workflow_phase_entry`.
- `src/workflow.rs` (novo): carregamento de definição, máquina de
  transição, execução de fase.
- `src/storage/mod.rs`, `src/storage/sqlite.rs`: novos métodos de
  persistência de `workflow_run`/`workflow_phase_entry`.
- `src/comandos.rs`, `src/main.rs`: `brian workflow run|approve|show`.
- Sem mudança em `execucao.rs` — o Workflow Engine é uma camada acima,
  mesma disciplina de `router.rs`/`model_router.rs`.

## Não-objetivos

- **Sem `governed.yaml`/gates de segurança nomeados** (`semgrep`, `osv`,
  `secrets`, `ocr`): Brian não tem nenhum scanner de segurança integrado —
  `gates` desta change são só comandos shell livres, mesma disciplina de
  `run-fast-path-gates`. Um workflow que "precisa" de security gate usa um
  comando shell que rode a ferramenta real, se o operador tiver uma.
- **Sem seleção automática por risco/`trigger`/`policy_set`**: não existe
  classificador de risco nem política de cliente no Brian — seleção de
  workflow é só `--workflow <id>` explícito, com `"fast"` como padrão
  quando omitido (blueprint §15.4, item 1 e 4; itens 2-3 ficam de fora).
- **Sem `max_cost_usd` nos limites**: Brian não calcula custo de run ainda
  (mesmo non-goal já registrado em `eval-harness-minimal`/
  `provider-router-scoring`) — só `max_total_phases` e `max_wall_seconds`.
- **Sem `requires_spec`**: não há verificação de OpenSpec integrada ao
  workflow nesta change.
- **Sem Reasoning Engine** (planner/evaluator/replanner, blueprint §16):
  fases com `role: planner` viram um `run` normal com `model_pointer:
  reasoning` — não existe um subsistema de raciocínio separado que produza
  propostas estruturadas. A fronteira declarada (§15.5) é respeitada porque
  não há nada do lado "Reasoning" que a violaria.

## Conformidade — checklist §16

- **M1-M6 / OP-1..OP-8**: atende diretamente OP-7/OP-8 (workflow curto por
  padrão, "fast" tem 3 fases não 10) e D-3 (workflow é dado, não código).
- **D-16/D-17**: cada fase é um `run` normal — já entra no ledger/histórico
  como qualquer outro.
- **D-7/D-12**: preservados por construção — cada fase usa
  `execucao::iniciar_run` sem alteração.
- **D-10**: workflow orquestra fases do MESMO run lógico ao longo do tempo,
  não múltiplos providers numa sessão simultânea — dentro da definição de
  produto (N providers × M clientes × T tempo).
- **H-1**: não depende do Context Governor.
- **Versão alvo**: v0.2 nominalmente pendente (workflow curto era o único
  item de v0.2 não fechado por `run-fast-path-gates`) — implementada agora
  fora da ordem original por decisão do autor.
