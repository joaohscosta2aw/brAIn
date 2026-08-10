## 1. Schema

- [x] 1.1 Migração `0006_workflow.sql`: `workflow_run` (id, client_id,
      project, workflow_id, workflow_version, current_phase, status,
      pause_reason, total_phases, started_at, finished_at).
- [x] 1.2 Migração: `workflow_phase_entry` (id, workflow_run_id, phase_id,
      run_id nullable — fases terminais sem execução real não têm run —
      outcome, entrada_numero, started_at, ended_at).
- [x] 1.3 Confirmar migração puramente aditiva: `cargo test` das changes
      anteriores continua passando sem alteração.

## 2. Domínio e Store

- [x] 2.1 `src/domain.rs`: `StatusWorkflowRun`
      (`Pending`/`Running`/`Paused`/`Completed`/`Failed`/`Cancelled`),
      `WorkflowRunRegistrado`, `EntradaDeFase`.
- [x] 2.2 `src/storage/mod.rs`/`sqlite.rs`: `criar_workflow_run`,
      `workflow_run`, `atualizar_workflow_run` (current_phase, status,
      pause_reason, total_phases, finished_at), `registrar_entrada_fase`,
      `entradas_do_workflow_run` — mesmo padrão de `criar_run`/
      `atualizar_status_run`/`registrar_evento_run` já existentes.
- [x] 2.3 Testes: CRUD básico de `workflow_run` e `workflow_phase_entry`.

## 3. Definição de workflow e transição pura

- [x] 3.1 `src/workflow.rs` (novo): `FaseDef` (`id`, `role` opcional,
      `model_pointer` opcional, `gates: Vec<String>`, `on_success`,
      `on_failure`, `max_entries` opcional, `on_max_entries` opcional,
      `terminal: bool`, `requires_approval: bool`), `WorkflowDef` (`id`,
      `version`, `fases: Vec<FaseDef>`, `max_total_phases`,
      `max_wall_seconds`).
- [x] 3.2 `carregar_workflow(caminho) -> WorkflowDef` via `serde_json`,
      mesmo padrão de `router::carregar_regras`.
- [x] 3.3 `resolver_model_pointer_do_role(role) -> Option<&str>` — tabela
      fixa `builder→coding, planner→reasoning, reviewer→review` (spec
      phase-execution: "Role da fase resolve para um model pointer").
- [x] 3.4 `avancar(def, estado_atual, outcome) -> Transicao` — função pura:
      checa limites primeiro (spec: "Limites do workflow encerram o run");
      decide `on_success`/`on_failure`; aplica `max_entries`/
      `on_max_entries` (spec: "Fase repetida demais escala para humano");
      pausa se a fase que terminou tem `requires_approval` (spec
      human-approval: "Fase com aprovação obrigatória pausa o workflow").
- [x] 3.5 Testes: transição simples sucesso/falha; `max_entries` escala
      para `on_max_entries`; `max_total_phases`/`max_wall_seconds`
      encerram antes de decidir próxima fase; `requires_approval` pausa em
      vez de avançar; função não faz I/O (verificação estrutural, mesmo
      padrão de `iniciar_run_chama_executar_provider_uma_unica_vez`).

## 4. Execução de fase e laço do workflow

- [x] 4.1 `executar_fase(store, contexto, repo, fase, provider_resolvido,
      agora) -> PhaseOutcome` — monta `--gate` concatenado (spec:
      "Gates da fase reaproveitam o gate determinístico existente"),
      chama `execucao::iniciar_run`, mapeia `StatusRun` para
      `PhaseOutcome` (design.md).
- [x] 4.2 `rodar_workflow(store, contexto, repo, def, tarefa, scored,
      agora) -> WorkflowRunRegistrado` — cria `workflow_run` (D-12: antes
      de qualquer fase rodar), laço: resolve provider da fase (regra ou
      score, spec: fase não inventa segunda forma de escolher provider),
      executa fase (task 4.1), registra `workflow_phase_entry`, chama
      `avancar`, aplica a transição, repete até fase terminal ou
      encerramento por limite.
- [x] 4.3 Fase terminal não executa `iniciar_run` — só finaliza o
      `workflow_run` com o status correspondente.
- [x] 4.4 Testes: workflow de 2 fases (`implement`→`verify`) com provider
      indisponível falha de forma determinística sem precisar de `codex`
      real (mesmo truque de `resolucao_de_commit_base_invalida...` — força
      erro cedo e verifica que o estado persistido é coerente); cada fase
      não-terminal gera um `run` real associado à `workflow_phase_entry`.

## 5. Aprovação humana

- [x] 5.1 `aprovar_workflow_run(store, workflow_run_id, agora)` — só age
      sobre `workflow_run` com `status = Paused`; transiciona para a fase
      declarada em `on_success` da fase que pausou e continua o laço (spec:
      "Aprovação explícita retoma o workflow pausado").
- [x] 5.2 Testes: aprovar workflow não-pausado falha com erro claro;
      aprovar workflow pausado avança corretamente.

## 6. Superfície CLI

- [x] 6.1 `brian workflow run <workflow_id> "<tarefa>" [--provider <id>]
      [--scored]` — `--workflow-id` omitido usa `workflows/fast.json`
      (design.md).
- [x] 6.2 `brian workflow approve <workflow_run_id>`.
- [x] 6.3 `brian workflow show <workflow_run_id>` — fase atual, status,
      histórico de fases.
- [x] 6.4 `workflows/fast.json` real, criado por esta change (3 fases:
      `implement`→`verify`→`fix`/`escalate`/`done`, mesmo desenho do
      blueprint §15.3, sem `max_cost_usd`).

## 7. Verificação

- [x] 7.1 Cobertura de cada cenário dos três specs desta change (auditoria
      manual, mesmo processo das changes anteriores).
- [x] 7.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 7.3 Teste manual supervisionado com `codex` real, num repositório de
      teste descartável (não este repositório): `brian workflow run fast
      "<tarefa real>"` completo até `done`, incluindo pelo menos uma
      reentrada de `fix` — confirmar `workflow_phase_entry` reflete a
      sequência real e cada fase tem um `run` de verdade associado.
- [x] 7.4 `openspec validate --strict` limpo antes de considerar a change
      pronta para archive.
