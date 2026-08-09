## Purpose

Roteamento adaptativo (D-8/D-13) sem harness de eval é aposta, não dado — este
harness responde, de forma repetível e barata de auditar, se um caso passa e com
que taxa, sobre a mesma infraestrutura de run/gate já existente.

## Requirements

### Requirement: Caso de eval é dado, não código

Um caso de eval SHALL ser definido como dado (fixture, tarefa, provider, gate),
carregado de um diretório, sem exigir alteração de código para adicionar ou
mudar um caso.

#### Scenario: Novo caso sem tocar código
- **GIVEN** um novo arquivo de caso no diretório de eval
- **WHEN** o operador roda `brian eval run`
- **THEN** o novo caso é executado sem nenhuma alteração em código

### Requirement: Cada tentativa é um run rastreado normalmente

Uma execução de caso de eval SHALL produzir runs reais através da mesma
infraestrutura de `execution/isolated-run` — worktree isolado, persistência,
gate — não um caminho de execução paralelo e não auditável.

#### Scenario: Tentativa de eval aparece como run
- **GIVEN** um caso de eval executado
- **WHEN** a execução termina
- **THEN** existe um run rastreado no banco para essa tentativa, no mesmo
  formato de qualquer outro run

### Requirement: Runs de eval nunca são atribuídos a cliente real

Runs produzidos por casos de eval SHALL usar um contexto sintético dedicado,
nunca o contexto ativo do operador — para não poluir atribuição ou custo real
(D-16).

#### Scenario: Eval não aparece na atribuição de um cliente real
- **GIVEN** um operador com um contexto de cliente real ativo
- **WHEN** ele roda `brian eval run`
- **THEN** os runs produzidos são atribuídos ao contexto sintético de eval, não
  ao cliente ativo

### Requirement: Caso roda N vezes e reporta taxa, não booleano

Um caso de eval SHALL rodar múltiplas vezes (variância do blueprint §112.4) e
reportar a fração de tentativas que passaram, não um resultado único.

#### Scenario: Taxa de sucesso reportada
- **GIVEN** um caso executado múltiplas vezes
- **WHEN** o relatório é gerado
- **THEN** ele mostra quantas tentativas passaram sobre o total

### Requirement: Taxa intermediária é marcada instável, não passa nem falha

Uma taxa de sucesso entre 0.34 e 0.66 SHALL ser classificada como instável — nem
aprovada nem reprovada — em vez de arredondada para um dos dois lados.

#### Scenario: Caso com taxa intermediária
- **GIVEN** um caso cuja taxa de sucesso está entre 0.34 e 0.66
- **WHEN** o relatório classifica o caso
- **THEN** ele aparece como instável, distinto de aprovado e de reprovado

### Requirement: Tentativa passa só por critério determinístico

Uma tentativa de caso de eval SHALL ser considerada aprovada exclusivamente pelo
status final do run (`Concluido`) — nunca por avaliação assistida por LLM nesta
capability.

#### Scenario: Run concluído conta como aprovado
- **GIVEN** uma tentativa cujo run termina `Concluido`
- **WHEN** o resultado da tentativa é computado
- **THEN** ela conta como aprovada

#### Scenario: Run falho conta como reprovado
- **GIVEN** uma tentativa cujo run termina `Falhou`
- **WHEN** o resultado da tentativa é computado
- **THEN** ela conta como reprovada
