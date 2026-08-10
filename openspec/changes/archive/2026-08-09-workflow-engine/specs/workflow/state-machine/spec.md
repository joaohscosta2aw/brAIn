## Purpose

Dez fases fixas em código viraram if/else que ninguém consegue mudar sem
recompilar. Uma máquina de estados sobre dado versionado (D-3) resolve isso
sem virar acoplamento com o que decide o conteúdo de cada fase.

## ADDED Requirements

### Requirement: Workflow é definido como dado, não código

Um workflow SHALL ser carregado de um arquivo de definição (fases, transições,
limites) — adicionar ou mudar um workflow SHALL NOT exigir alteração de
código.

#### Scenario: Novo workflow sem tocar código
- **GIVEN** um novo arquivo de definição de workflow
- **WHEN** o operador roda `brian workflow run <id> "<tarefa>"` com esse id
- **THEN** o workflow roda sem nenhuma alteração em código

### Requirement: Transição é determinística e nunca chama provider diretamente

O motor de transição SHALL decidir a próxima fase só a partir do outcome da
fase atual (sucesso/falha) e da tabela de transições do workflow — SHALL NOT
invocar provider nenhum diretamente nem interpretar conteúdo de resposta de
LLM.

#### Scenario: Transição usa só a tabela declarada
- **GIVEN** uma fase que termina com sucesso
- **WHEN** o motor decide a próxima fase
- **THEN** a fase seguinte é exatamente a declarada em `on_success` para a
  fase atual, sem lógica adicional

### Requirement: Fase repetida demais escala para humano

Quando uma fase é reentrada mais vezes que seu `max_entries`, o sistema SHALL
transicionar para a fase de fallback declarada (`on_max_entries`) em vez de
continuar reentrando a mesma fase indefinidamente.

#### Scenario: Loop de correção escala após o limite
- **GIVEN** uma fase de correção com `max_entries: 2` já reentrada 2 vezes
- **WHEN** ela falha de novo
- **THEN** o workflow transiciona para a fase de fallback, não tenta a
  correção pela terceira vez

### Requirement: Limites do workflow encerram o run

Quando o total de fases executadas atinge `max_total_phases`, ou o tempo
decorrido atinge `max_wall_seconds`, o sistema SHALL encerrar o
`workflow_run` mesmo que a fase atual não seja terminal.

#### Scenario: Limite de fases encerra o workflow
- **GIVEN** um `workflow_run` que atingiu `max_total_phases`
- **WHEN** o motor tentaria decidir a próxima fase
- **THEN** o `workflow_run` é encerrado por limite, não avança para outra
  fase

### Requirement: Versão do workflow é congelada no início do run

O `workflow_version` usado por um `workflow_run` SHALL ser fixado no
momento em que o run começa — alterações posteriores no arquivo de
definição SHALL NOT afetar um run já em andamento nem a leitura de um run
já finalizado.

#### Scenario: Edição do arquivo não afeta run em andamento
- **GIVEN** um `workflow_run` em andamento na versão 1 de um workflow
- **WHEN** o arquivo de definição é editado para a versão 2 antes do run
  terminar
- **THEN** o `workflow_run` continua sendo interpretado pela versão 1
