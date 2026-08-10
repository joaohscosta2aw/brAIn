## Purpose

Uma fase de workflow não é um conceito novo de execução — é o mesmo `run`
isolado que `execution/isolated-run` já garante, só que encadeado. Reusar em
vez de duplicar é o que mantém D-7/D-12 válidos para workflow também.

## Requirements

### Requirement: Fase não-terminal executa como um run real

Uma fase não-terminal SHALL executar através da mesma infraestrutura de
`execution/isolated-run` — worktree isolado, persistência antes de efeito
colateral, gate — não um caminho de execução paralelo.

#### Scenario: Fase aparece como run rastreado
- **GIVEN** uma fase não-terminal de um workflow em execução
- **WHEN** a fase termina
- **THEN** existe um run real associado a essa entrada de fase, no mesmo
  formato de qualquer outro run

### Requirement: Role da fase resolve para um model pointer

Quando uma fase declara `role` sem `model_pointer` explícito, o sistema
SHALL resolver um `model_pointer` default a partir do `role` (`builder`
→ `coding`, `planner` → `reasoning`, `reviewer` → `review`).

#### Scenario: Fase sem model_pointer usa o default do role
- **GIVEN** uma fase com `role: builder` e sem `model_pointer`
- **WHEN** a fase executa
- **THEN** o `model_pointer` resolvido é `coding`

#### Scenario: Model_pointer explícito da fase vence o default do role
- **GIVEN** uma fase com `role: builder` e `model_pointer: quick` explícito
- **WHEN** a fase executa
- **THEN** o `model_pointer` usado é `quick`, não o default de `builder`

### Requirement: Gates da fase reaproveitam o gate determinístico existente

Os `gates` de uma fase SHALL ser aplicados através do mesmo mecanismo de
`execution/deterministic-gate` — a fase só é bem-sucedida se o provider e
todos os gates da fase passarem.

#### Scenario: Fase com múltiplos gates só passa se todos passarem
- **GIVEN** uma fase com dois gates, um deles falhando
- **WHEN** a fase executa
- **THEN** a fase é marcada como falha, mesmo que o provider tenha tido
  sucesso
