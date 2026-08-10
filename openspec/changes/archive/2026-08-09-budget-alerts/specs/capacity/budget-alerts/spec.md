## Purpose

Budget de cliente é lei do produto desde o v0.0 (D-16, blueprint §45): um
cliente sem limite configurado nunca é surpreendido por um bloqueio, e um
cliente com limite configurado nunca ultrapassa o limite duro sem que
Brian tenha recusado a próxima chamada.

## ADDED Requirements

### Requirement: Cliente sem orçamento configurado nunca é bloqueado nem alertado

O sistema SHALL tratar a ausência de entrada em `budgets/clients.json`
como "sem orçamento definido" — SHALL NOT aplicar limite implícito nem
gerar alerta para esse cliente.

#### Scenario: Cliente ausente do config roda sem checagem de limite
- **GIVEN** um cliente sem entrada em `budgets/clients.json`
- **WHEN** `brian run` é executado para esse cliente
- **THEN** nenhuma checagem de orçamento bloqueia ou atrasa o run

### Requirement: Alerta suave reporta limiares cruzados sem bloquear

Quando o gasto do mês corrente de um cliente cruza um valor em
`alert_at_percent`, `brian budget check` SHALL reportar explicitamente
quais limiares já foram cruzados — SHALL NOT impedir a consulta nem
qualquer run em andamento.

#### Scenario: Gasto acima de um limiar aparece no relatório
- **GIVEN** um cliente com `alert_at_percent: [50, 80, 95]` e gasto mensal
  em 82% do `monthly_usd_equivalent`
- **WHEN** `brian budget check --client <id>` roda
- **THEN** o relatório mostra os limiares 50 e 80 como cruzados, e 95 como
  não cruzado

### Requirement: Limite duro recusa novo run antes de qualquer efeito colateral

Quando o gasto do mês corrente de um cliente já atingiu ou excedeu
`monthly_usd_equivalent` ou `monthly_tokens`, `brian run` SHALL recusar
iniciar um novo run para esse cliente — SHALL NOT criar worktree, SHALL
NOT persistir registro de run, SHALL NOT invocar nenhum provider.

#### Scenario: Run recusado com orçamento já excedido
- **GIVEN** um cliente cujo gasto do mês já é >= `monthly_usd_equivalent`
  configurado
- **WHEN** o operador roda `brian run "qualquer tarefa"`
- **THEN** o comando falha com um erro claro sobre o orçamento excedido, e
  nenhum run é persistido

### Requirement: Relatório de orçamento nunca esconde o gasto real por trás do limite

`brian budget check` SHALL sempre mostrar o gasto apurado e o limite
configurado lado a lado, nunca só um veredito binário.

#### Scenario: Relatório mostra gasto e limite juntos
- **GIVEN** um cliente com orçamento configurado
- **WHEN** `brian budget check --client <id>` roda
- **THEN** o relatório mostra o valor gasto e o valor limite, não só
  "dentro"/"fora" do orçamento
