## Purpose

Chargeback é a metade do §44 que Showback (`brian costs`) não cobre:
quanto faturar, não só quanto foi consumido. Política comercial pertence
à organização (blueprint §44.1) — Brian calcula, nunca decide preço.

## Requirements

### Requirement: Chargeback exige markup configurado explicitamente

`brian billing chargeback` SHALL recusar calcular um valor faturável para
um cliente sem `markup` configurado em `billing/clients.json` — SHALL NOT
assumir markup 1.0 (sem markup) como padrão silencioso.

#### Scenario: Cliente sem markup configurado
- **GIVEN** um cliente sem entrada em `billing/clients.json`
- **WHEN** o operador roda `brian billing chargeback --client <id>`
- **THEN** o comando falha com um erro claro sobre markup ausente, não
  calcula um valor com markup fabricado

### Requirement: Relatório mostra custo interno e valor faturável lado a lado

O sistema SHALL sempre mostrar o custo interno apurado (`equivalente`) e o
valor faturável resultante (após markup e piso mensal) juntos — SHALL NOT
mostrar só o valor final sem o custo que o origina.

#### Scenario: Relatório de chargeback mostra os dois valores
- **GIVEN** um cliente com markup configurado e consumo no período
- **WHEN** `brian billing chargeback --client <id>` roda
- **THEN** o relatório mostra o custo interno e o valor faturável (custo ×
  markup, ou o piso mensal se for maior)

### Requirement: Piso mensal nunca é escondido quando aplicado

Quando o valor calculado (custo × markup) fica abaixo de
`minimum_monthly_usd`, o sistema SHALL usar o piso e SHALL sinalizar
explicitamente que o piso foi aplicado.

#### Scenario: Consumo baixo aciona o piso mensal
- **GIVEN** um cliente cujo custo × markup fica abaixo do
  `minimum_monthly_usd` configurado
- **WHEN** o relatório de chargeback é gerado
- **THEN** o valor faturável é o piso mensal, e o relatório indica
  explicitamente que o piso foi aplicado
