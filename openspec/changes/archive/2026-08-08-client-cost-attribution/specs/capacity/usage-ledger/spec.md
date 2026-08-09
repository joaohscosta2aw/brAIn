## Purpose

Registro fiel de todo consumo de IA observado, com procedência explícita de cada
número. O ledger é a unidade de verdade sobre o que foi gasto; tudo que o Brian
afirma sobre custo e capacidade deriva dele.

## ADDED Requirements

### Requirement: Registro de consumo por chamada de provider

O sistema SHALL registrar cada chamada de provider observada como um `usage_record`
contendo, no mínimo: provider, modelo, tokens de entrada, tokens de cache, tokens de
saída, tokens de reasoning, modo de cobrança (`billing_mode`) e o instante de
ocorrência (`occurred_at`).

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

### Requirement: Custo real e custo equivalente coexistem

O sistema SHALL manter, para cada `usage_record`, dois valores monetários distintos
e independentes:

- **custo reportado pelo provider** — o que efetivamente se paga por aquela chamada,
  quando o provider o informa;
- **custo equivalente em API** — o que aqueles tokens custariam a preço de tabela,
  calculado sempre que houver entrada de catálogo para o modelo.

Um SHALL NOT substituir o outro. A ausência de um SHALL NOT impedir o registro do
outro. O custo equivalente SHALL ser calculável independentemente do `billing_mode`.

#### Scenario: Consumo por API
- **GIVEN** consumo com `billing_mode` = `api`
- **WHEN** o provider reporta o custo da chamada e existe catálogo para o modelo
- **THEN** o custo reportado e o custo equivalente são ambos registrados
- **AND** permanecem distinguíveis um do outro

#### Scenario: Consumo por assinatura
- **GIVEN** consumo com `billing_mode` = `subscription`, em que não há custo por chamada
- **WHEN** existe entrada de catálogo para o modelo usado
- **THEN** o custo equivalente em API é registrado
- **AND** o custo reportado pelo provider permanece ausente
- **AND** o registro não é marcado como de custo desconhecido

#### Scenario: Modelo ausente do catálogo de preço
- **WHEN** não existe entrada de catálogo para o modelo usado e o provider não reporta custo
- **THEN** `cost_source` é `unknown`
- **AND** nenhum dos dois valores é registrado como zero

### Requirement: Procedência e confiança rotuladas

Todo `usage_record` SHALL carregar `usage_source` (`provider` | `brian_measured` |
`estimated`), `cost_source` (`provider` | `catalog` | `unknown`) e uma indicação de
confiança do custo.

O sistema SHALL NOT atribuir um rótulo mais forte do que a evidência disponível
sustenta.

#### Scenario: Tokens lidos diretamente do provider
- **WHEN** as contagens de token vêm da resposta do próprio provider
- **THEN** `usage_source` é `provider`

#### Scenario: Tokens medidos pelo Brian
- **GIVEN** um provider que não reporta contagens de token
- **WHEN** o Brian deriva as contagens do que observou
- **THEN** `usage_source` é `brian_measured`
- **AND** a confiança do custo reflete essa origem

### Requirement: Precedência do custo reportado sobre o catálogo

Quando o provider reporta o custo de uma chamada, esse valor SHALL prevalecer como
custo real sobre qualquer valor derivado de catálogo (D-6).

Essa precedência SHALL NOT apagar o custo equivalente, que serve a outro propósito.

#### Scenario: Provider e catálogo divergem
- **GIVEN** um consumo com custo reportado pelo provider e entrada de catálogo para o mesmo modelo
- **WHEN** o registro é gravado
- **THEN** o custo real persistido é o do provider e `cost_source` é `provider`
- **AND** o custo equivalente derivado do catálogo permanece registrado à parte

#### Scenario: Custo do provider chega depois do registro
- **GIVEN** um `usage_record` cujo custo real ainda não era conhecido
- **WHEN** o provider posteriormente reporta o custo real da mesma chamada
- **THEN** o custo real é preenchido e `cost_source` passa a `provider`
- **AND** o valor anterior e sua fonte permanecem recuperáveis

### Requirement: Custo equivalente nunca é apresentado como custo real

O sistema SHALL NOT apresentar o custo equivalente em API como se fosse valor
efetivamente pago, em nenhuma consulta, agregação ou exportação.

Onde os dois aparecem juntos, cada valor SHALL ser identificável quanto à sua
natureza.

#### Scenario: Apresentação de consumo por assinatura
- **GIVEN** consumo por assinatura com custo equivalente registrado
- **WHEN** o custo desse consumo é apresentado
- **THEN** o valor é identificado como equivalente em API
- **AND** não é apresentado como valor pago ao provider

#### Scenario: Período mistura assinatura e API
- **GIVEN** um período com consumo por assinatura e consumo por API
- **WHEN** o custo total do período é apresentado
- **THEN** o valor efetivamente pago e o valor equivalente são apresentados como grandezas distintas
- **AND** não são somados em um único número indiferenciado

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
