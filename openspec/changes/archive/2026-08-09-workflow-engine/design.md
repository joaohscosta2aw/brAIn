## Context

`execucao::iniciar_run` (D-7/D-12, já arquivada) já executa uma fase
isolada com gate opcional. `router::model_router` já resolve pointer
semântico em modelo concreto. Este design encadeia múltiplos
`iniciar_run` sob uma máquina de estados persistida — não reimplementa
nada do motor de execução.

Ver proposal.md para motivação (D-3) e não-objetivos (sem `governed.yaml`
com security gates nomeados, sem trigger automático, sem `max_cost_usd`).

## Goals / Non-Goals

**Goals:**
- Workflow como dado (JSON), fases encadeadas via `execucao::iniciar_run`
  sem duplicar lógica de execução.
- Fronteira Workflow×Reasoning respeitada por construção: a função de
  transição nunca chama provider, só olha `PhaseOutcome`.
- `workflow_version` congelado no início do run.

**Non-Goals:** ver proposal.md.

## Decisions

**Workflow em JSON, não YAML** — mesmo padrão já estabelecido em
`routing/rules.json`, `models/pointers.json`, `evals/cases/*.json`.
`serde_json`, sem dependência nova.

**Duas tabelas novas: `workflow_run` e `workflow_phase_entry`.**
`workflow_run` guarda o estado da máquina (`current_phase`, `status`,
`workflow_id`, `workflow_version`, `total_phases`, `client_id`, `project`,
`started_at`, `finished_at`). `workflow_phase_entry` guarda o histórico
(`workflow_run_id`, `phase_id`, `run_id` — FK para a tabela `run` já
existente, quando a fase efetivamente executou — `outcome`,
`entrada_numero`, `started_at`, `ended_at`). Isso dá `phase_history`
(blueprint §15.6) de graça via `SELECT ... WHERE workflow_run_id = ?`.

**Toda fase não-terminal do JSON precisa de `role` ou `provider`.**
`role` resolve o `model_pointer` default (design table abaixo); a fase
ainda usa a resolução normal de provider (`routing/provider-rules`, com
`--scored` opcional propagado do `brian workflow run`) — o workflow não
inventa uma segunda forma de escolher provider.

```text
role       model_pointer default
builder    coding
planner    reasoning
reviewer   review
```

**`gates` da fase = lista de comandos concatenados com `&&`.** Uma fase com
`["cargo test", "cargo clippy"]` vira `--gate "cargo test && cargo clippy"`
em `iniciar_run` — reaproveita 100% o mecanismo existente de
`execution/deterministic-gate`, sem um "gate composto" novo.

**Outcome da fase = status do run.** `StatusRun::Concluido` →
`PhaseOutcome::Success`; `StatusRun::Falhou` → `PhaseOutcome::Failure`. Sem
terceiro estado — `Abandonado`/`EmExecucao` não deveriam aparecer aqui
porque `iniciar_run` é síncrono (mesma garantia de `isolated-tracked-run`).

**Função de transição pura, testável sem processo nenhum** — mesma
disciplina de `router::decidir`/`model_router::resolver_pointer`:

```rust
fn avancar(def: &WorkflowDef, wf_run: &WorkflowRunState, outcome: PhaseOutcome)
    -> Transicao
```

recebe só dados já em memória, decide `Transicao::ProximaFase(id)` |
`Transicao::Pausar(motivo)` | `Transicao::Encerrar(motivo)` — nenhuma
chamada de I/O dentro dela (blueprint §15.5: "Nenhuma chamada de LLM nesta
função. Se aparecer uma, a fronteira foi violada").

**Limites verificados antes de decidir a próxima fase**, não depois —
`max_total_phases`/`max_wall_seconds` checados no topo de `avancar`, mesma
ordem do pseudocódigo do blueprint §15.5.

**`requires_approval` pausa incondicionalmente em sucesso**, nunca em
falha (falha já vai para `on_failure` normalmente — pausar numa fase que
falhou seria confuso, "aprovar o quê").

**Workflow "fast" embutido como arquivo padrão** (`workflows/fast.json`,
criado por esta change como exemplo real, não hardcoded no binário) —
`--workflow` omitido carrega esse arquivo por convenção de nome, mesmo
padrão de `routing/rules.json` ter que existir no `cwd`.

**`max_entries` pertence à própria fase, mede reentradas nela mesma.** O
pseudocódigo do blueprint (§15.5) é ambíguo — lê `max_entries` da fase de
origem mas aplica à contagem da fase de destino, o que quebraria o próprio
exemplo YAML dele (`verify.max_entries: 1` impediria qualquer segunda
passagem por `verify` depois de um `fix`, mesmo sendo esse o propósito do
loop). Resolução adotada: `max_entries`/`on_max_entries` são atributos da
fase que está prestes a ser **entrada** — antes de transicionar para uma
fase, checa quantas vezes ela já foi entrada; se bateria o limite, desvia
para `on_max_entries` dela em vez de entrar. `workflows/fast.json` desta
change só declara `max_entries` em `fix` (não em `verify`), evitando a
ambiguidade do exemplo original.

## Risks / Trade-offs

- **`workflow_version` sempre = `1`** nesta change — não há mecanismo de
  detectar mudança de conteúdo do arquivo e incrementar versão sozinho;
  "congelado no início do run" significa "o valor declarado no JSON no
  momento em que o run começa", não um hash de conteúdo. Se o operador
  editar `fast.json` sem bumpar `version`, um run em andamento na prática
  pode ler o arquivo mudado se `avancar` for chamado de novo antes do fim —
  aceito, mitigação real (detectar mudança automaticamente) fica para
  quando houver um caso concreto de dano.
- **Sem retry automático de fase pausada** (`requires_approval`) — se o
  operador nunca aprova, o `workflow_run` fica pausado indefinidamente, sem
  timeout. Aceito — mesma disciplina de "nunca reexecuta sozinho" de
  `orphan-recovery`.
