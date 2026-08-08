## Purpose

Registro fiel de todo consumo de IA observado, com procedência explícita de cada
número. O ledger é a unidade de verdade sobre o que foi gasto; tudo que o Brian
afirma sobre custo e capacidade deriva dele.

## ADDED Requirements

### Requirement: Registro de consumo por chamada de provider

O sistema SHALL registrar cada chamada de provider observada como um `usage_record`
contendo, no mínimo: provider, modelo, tokens de entrada, tokens de cache, tokens de
saída, tokens de reasoning, custo, e o instante de ocorrência (`occurred_at`).

Tokens não reportados por um provider SHALL ser registrados como ausentes, nunca
como zero — ausência de dado e consumo zero são fatos distintos.

#### Scenario: Provider reporta consumo completo
- **WHEN** uma chamada de provider é observada com tokens de entrada, cache, saída e reasoning
- **THEN** um `usage_record` é criado com todos os campos preenchidos e `occurred_at` do momento da chamada

#### Scenario: Provider não reporta uma categoria de token
- **GIVEN** um provider que não expõe tokens de reasoning
- **WHEN** o consumo é registrado
- **THEN** o campo de reasoning é registrado como ausente
- **AND** não é registrado como zero

#### Scenario: Registro sem instante de ocorrência é rejeitado
- **WHEN** um consumo é apresentado sem `occurred_at` determinável
- **THEN** o sistema rejeita o registro com erro explícito
- **AND** nenhuma linha parcial é gravada no ledger

### Requirement: Procedência rotulada obrigatória

Todo `usage_record` SHALL carregar `usage_source` (`provider` | `brian_measured` |
`estimated`) e `cost_source` (`provider` | `catalog` | `unknown`).

Nenhum registro pode existir sem ambos os rótulos. O sistema SHALL NOT inferir um
rótulo mais forte do que a evidência disponível sustenta.

#### Scenario: Consumo lido diretamente do provider
- **WHEN** tokens e custo vêm da resposta do próprio provider
- **THEN** `usage_source` é `provider` e `cost_source` é `provider`

#### Scenario: Custo derivado de catálogo de preço
- **GIVEN** um provider que reporta tokens mas não reporta custo
- **WHEN** existe entrada de catálogo para o modelo usado
- **THEN** o custo é calculado a partir do catálogo
- **AND** `cost_source` é `catalog`

#### Scenario: Modelo ausente do catálogo de preço
- **GIVEN** um provider que reporta tokens mas não reporta custo
- **WHEN** não existe entrada de catálogo para o modelo usado
- **THEN** `cost_source` é `unknown`
- **AND** o custo não é registrado como zero

### Requirement: Precedência da fonte de custo

Quando houver mais de uma fonte de custo disponível para o mesmo consumo, o valor
reportado pelo provider SHALL prevalecer sobre o valor calculado por catálogo (D-6).

#### Scenario: Provider e catálogo divergem
- **GIVEN** um consumo com custo reportado pelo provider e entrada de catálogo para o mesmo modelo
- **WHEN** o registro é gravado
- **THEN** o custo persistido é o do provider
- **AND** `cost_source` é `provider`

#### Scenario: Custo do provider chega depois do registro
- **GIVEN** um `usage_record` gravado com `cost_source` = `catalog`
- **WHEN** o provider posteriormente reporta o custo real da mesma chamada
- **THEN** o custo é atualizado para o valor do provider
- **AND** `cost_source` passa a `provider`
- **AND** a substituição é auditável

### Requirement: Importação idempotente

A importação de consumo SHALL ser re-executável sobre janelas já importadas sem
criar registros duplicados.

#### Scenario: Reimportação da mesma janela
- **GIVEN** uma janela de tempo já importada
- **WHEN** a importação é executada novamente sobre a mesma janela
- **THEN** nenhum registro duplicado é criado
- **AND** o total de consumo da janela permanece inalterado

#### Scenario: Importação sobre janela parcialmente coberta
- **GIVEN** uma janela cujo início já foi importado
- **WHEN** a importação cobre essa janela e um período posterior ainda não importado
- **THEN** apenas o consumo ainda não registrado é adicionado

#### Scenario: Fonte de importação indisponível
- **WHEN** a fonte de uso de um provider está inacessível
- **THEN** o sistema reporta a falha identificando o provider afetado
- **AND** o consumo dos demais providers é importado normalmente
- **AND** o ledger não é deixado em estado parcial silencioso

### Requirement: Integridade do ledger

O sistema SHALL garantir, para todo `usage_record`, que: existe `occurred_at`;
existem `usage_source` e `cost_source`; e existe cliente atribuído ou marcação
explícita de não-atribuído.

Violação de qualquer uma destas invariantes SHALL ser tratada como defeito de
release, não como estado tolerável.

#### Scenario: Verificação de integridade do ledger
- **WHEN** a integridade do ledger é verificada
- **THEN** o sistema reporta qualquer registro que viole uma das invariantes
- **AND** identifica qual invariante foi violada e em quais registros

### Requirement: Agregação preserva procedência

Ao somar ou agregar consumo de múltiplos registros, o sistema SHALL NOT apresentar
um total que misture fontes de custo distintas sem informar essa mistura.

#### Scenario: Total agrega custos de fontes distintas
- **GIVEN** registros com `cost_source` `provider` e outros com `catalog` no mesmo período
- **WHEN** o custo total do período é apresentado
- **THEN** a composição por fonte é informada junto ao total

#### Scenario: Total inclui custo desconhecido
- **GIVEN** registros com `cost_source` = `unknown` no período consultado
- **WHEN** o custo total é apresentado
- **THEN** o total identifica explicitamente a parcela cujo custo é desconhecido
- **AND** a parcela desconhecida não é somada como zero
