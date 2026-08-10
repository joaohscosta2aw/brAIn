## Purpose

Blueprint §35: um agente rodando via `brian run` deve ver o que já foi
decidido e tentado no Context antes dele — orçado, nunca a memória
inteira despejada no prompt.

## Requirements

### Requirement: Recall é orçado por itens e por caracteres

O sistema SHALL selecionar no máximo `max_items` notas para o recall,
cortando também por um orçamento de caracteres — SHALL NOT injetar todas
as notas do Context sem limite.

#### Scenario: Context com mais notas que o orçamento permite
- **GIVEN** um Context com mais notas do que `max_items`
- **WHEN** o recall é montado
- **THEN** o resultado tem no máximo `max_items` notas

#### Scenario: Notas que excedem o orçamento de caracteres são cortadas
- **GIVEN** notas cujo texto somado excede o orçamento de caracteres
- **WHEN** o recall é montado
- **THEN** o resultado fica dentro do orçamento, mesmo que isso signifique
  menos notas que `max_items`

### Requirement: Notas de decisão são sempre priorizadas quando existem

O sistema SHALL incluir notas de categoria `decisao` antes de qualquer
outra categoria na seleção, enquanto houver espaço no orçamento.

#### Scenario: Decisão sobrevive ao corte quando outras notas não sobrevivem
- **GIVEN** um Context com uma nota de `decisao` antiga e várias notas de
  outras categorias mais recentes, mais notas do que o orçamento permite
- **WHEN** o recall é montado
- **THEN** a nota de `decisao` está no resultado, mesmo sendo mais antiga

### Requirement: Recall vazio quando o Context não tem notas

O sistema SHALL produzir um recall vazio (sem seção anexada à tarefa)
quando o Context ativo não tem nenhuma nota registrada — SHALL NOT alterar
a tarefa enviada ao provider nesse caso.

#### Scenario: Context sem notas não altera a tarefa do run
- **GIVEN** um Context ativo sem nenhuma nota registrada
- **WHEN** `brian run` é executado
- **THEN** a tarefa enviada ao provider é exatamente a informada pelo
  operador, sem nenhuma seção de memória anexada

### Requirement: Operador pode auditar o recall antes de rodar

`brian memory recall` SHALL mostrar exatamente o conteúdo que seria
anexado a um `brian run` no momento da consulta, para o mesmo Context
ativo.

#### Scenario: Recall exibido é idêntico ao que seria injetado
- **GIVEN** um Context ativo com notas
- **WHEN** o operador roda `brian memory recall` e, em seguida, `brian
  run`
- **THEN** o conteúdo mostrado por `brian memory recall` é o mesmo
  anexado à tarefa do run
