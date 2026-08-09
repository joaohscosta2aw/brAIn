## Purpose

Divide a capacidade consumida de um plano de assinatura entre os clientes atendidos
por ele, proporcional ao consumo de cada um — showback em fração, não em dólar, já
que nenhuma fonte automática desta change expõe o preço do plano.

## Requirements

### Requirement: Rateio proporcional da capacidade do plano entre clientes

Para um plano com `billing_mode` = `subscription`, o sistema SHALL calcular, por
janela do plano, a fração de consumo de cada cliente sobre o total atribuído na
janela.

O rateio SHALL ser expresso como fração (percentual do total atribuído), nunca em
valor monetário — o preço do plano não é detectável automaticamente por nenhuma das
fontes desta change (Claude, Codex, Gemini), e este projeto não introduz declaração
manual de custo.

#### Scenario: Dois clientes dividem o plano
- **GIVEN** um plano de assinatura consumido por dois clientes na mesma janela
- **WHEN** o rateio do plano é consultado
- **THEN** cada cliente recebe uma fração proporcional aos tokens que consumiu na
  janela
- **AND** a soma das frações dos clientes atribuídos não excede 100%

#### Scenario: Cliente único consome todo o plano
- **GIVEN** um plano de assinatura cujo único consumo atribuído na janela pertence a
  um cliente
- **WHEN** o rateio é consultado
- **THEN** esse cliente recebe 100% da fração atribuível

### Requirement: Consumo não atribuído não é rateado entre clientes

Consumo `unattributed` no período SHALL ser excluído do rateio entre clientes e
apresentado como parcela separada.

#### Scenario: Parte do consumo do período é não atribuída
- **GIVEN** uma janela de plano com consumo atribuído a clientes e consumo
  `unattributed`
- **WHEN** o rateio é consultado
- **THEN** a fração correspondente ao consumo não atribuído é mostrada à parte
- **AND** essa fração não é dividida entre os clientes atribuídos

### Requirement: Rateio se aplica apenas a planos de assinatura

O sistema SHALL NOT calcular rateio de plano para consumo com `billing_mode` = `api`,
já que nesse modo o custo já é direto por chamada (cost-attribution existente).

#### Scenario: Consulta de rateio para plano de API
- **WHEN** o operador consulta o rateio de um plano com `billing_mode` = `api`
- **THEN** o sistema informa que rateio não se aplica a esse modo de cobrança

### Requirement: Rateio é identificado como fração de capacidade, não custo

A fração alocada a um cliente por rateio de plano SHALL ser sempre identificada como
showback de capacidade derivado do consumo, nunca apresentada como valor monetário
nem como custo pago por aquele cliente especificamente.

#### Scenario: Apresentação do rateio
- **WHEN** a fração alocada de um cliente é apresentada
- **THEN** ela é identificada como percentual do plano consumido, distinta do custo
  pago por chamada do mesmo cliente (cost-attribution)
