## Purpose

Estabelece de quem é cada token gasto. Liga o consumo registrado no ledger a um
cliente, torna ruidoso todo consumo sem dono, e responde quanto custou o trabalho
de cada cliente para faturamento e decisão humana.

## ADDED Requirements

### Requirement: Cadeia de atribuição até o cliente

Todo `usage_record` SHALL estar ligado a um cliente, ou marcado explicitamente com
`attribution_status` = `unattributed`.

Em modo observe, os elos intermediários da cadeia (run, fase, change) MAY ser nulos.
O cliente SHALL NOT ser nulo sem a marcação explícita de não-atribuído.

#### Scenario: Consumo com cliente determinável
- **WHEN** o consumo observado pode ser ligado a um cliente
- **THEN** o registro recebe o cliente
- **AND** `attribution_status` é `attributed`

#### Scenario: Consumo sem cliente determinável
- **WHEN** o consumo observado não pode ser ligado a nenhum cliente
- **THEN** o registro é gravado com `attribution_status` = `unattributed`
- **AND** o consumo não é descartado nem atribuído a um cliente por suposição

#### Scenario: Elos intermediários ausentes em observe mode
- **GIVEN** consumo observado fora de um run orquestrado
- **WHEN** o registro é gravado com cliente determinado
- **THEN** run e fase permanecem nulos
- **AND** o registro é considerado atribuído

### Requirement: Consumo não atribuído é alarme visível

Consumo com `attribution_status` = `unattributed` SHALL ser tratado como alarme
persistente, nunca como estado normal silencioso.

O alarme SHALL permanecer visível enquanto existir qualquer registro não atribuído
no período corrente.

#### Scenario: Existe consumo não atribuído
- **GIVEN** ao menos um registro `unattributed` no período corrente
- **WHEN** o estado do sistema é consultado
- **THEN** o consumo não atribuído é sinalizado com sua quantidade de tokens e custo

#### Scenario: Consulta dedicada de não atribuídos
- **WHEN** o operador consulta o consumo não atribuído
- **THEN** o sistema lista cada registro sem dono com provider, modelo, tokens, custo e instante

#### Scenario: Ledger íntegro não tem consumo sem dono
- **GIVEN** um período em que todo consumo foi atribuído
- **WHEN** o consumo não atribuído é consultado
- **THEN** o resultado é vazio
- **AND** nenhum alarme de atribuição é sinalizado

### Requirement: Atribuição manual de consumo órfão

O sistema SHALL permitir que o operador atribua um registro não atribuído a um
cliente existente.

Toda atribuição manual SHALL ser auditável, preservando que o valor foi definido
por intervenção humana e não por observação.

#### Scenario: Atribuir registro órfão a um cliente
- **GIVEN** um registro com `attribution_status` = `unattributed`
- **WHEN** o operador o atribui a um cliente existente
- **THEN** o registro passa a `attributed` com aquele cliente
- **AND** a origem manual da atribuição fica registrada

#### Scenario: Atribuir a cliente inexistente
- **WHEN** o operador tenta atribuir um registro a um cliente que não existe
- **THEN** o sistema recusa a operação com erro explícito
- **AND** o registro permanece não atribuído

#### Scenario: Reatribuir registro já atribuído
- **GIVEN** um registro já atribuído a um cliente
- **WHEN** o operador o reatribui a outro cliente
- **THEN** a nova atribuição vale
- **AND** a atribuição anterior permanece auditável

### Requirement: Consulta de custo por cliente

O sistema SHALL responder quanto foi consumido e gasto por cliente, com recorte por
período e desdobramento por provider.

#### Scenario: Custo de um cliente no período
- **WHEN** o operador consulta o custo de um cliente em um período
- **THEN** o sistema retorna tokens e custo do cliente restritos àquele período

#### Scenario: Desdobramento por provider
- **WHEN** o operador consulta o custo de um cliente desdobrado por provider
- **THEN** o sistema retorna tokens e custo por provider
- **AND** a soma das linhas corresponde ao total do cliente no mesmo período

#### Scenario: Cliente sem consumo no período
- **WHEN** o operador consulta um cliente existente sem consumo no período
- **THEN** o sistema retorna resultado vazio com custo zero
- **AND** distingue esse caso de cliente inexistente

### Requirement: Isolamento de consumo entre clientes

Uma consulta restrita a um cliente SHALL NOT retornar consumo pertencente a outro
cliente, nem consumo não atribuído.

O isolamento SHALL ser garantido pela construção da consulta, não por filtragem
posterior do resultado.

#### Scenario: Consulta de cliente não vaza consumo alheio
- **GIVEN** consumo registrado para dois clientes distintos no mesmo período
- **WHEN** o operador consulta o custo de um deles
- **THEN** apenas o consumo daquele cliente é retornado

#### Scenario: Consulta de cliente não inclui não atribuídos
- **GIVEN** consumo não atribuído no mesmo período de um cliente com consumo
- **WHEN** o operador consulta o custo daquele cliente
- **THEN** o consumo não atribuído não é incluído no resultado

### Requirement: Exportação de custo

O sistema SHALL exportar o consumo atribuído em formato tabular para uso externo,
preservando a procedência de cada valor.

#### Scenario: Exportar custo de um período
- **WHEN** o operador exporta o custo de um período
- **THEN** o arquivo contém cliente, provider, modelo, tokens, custo e instante por registro
- **AND** contém `usage_source` e `cost_source` de cada registro

#### Scenario: Exportação com custo desconhecido
- **GIVEN** registros com custo desconhecido no período exportado
- **WHEN** a exportação é gerada
- **THEN** esses registros aparecem com custo marcado como desconhecido
- **AND** não aparecem com custo zero
