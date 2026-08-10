## Context

`Store::consumo_do_cliente` + `comandos::agregar` já dão o custo interno
(`pago`/`equivalente` separados, nunca somados — invariante verificada).
Chargeback só precisa de um número a mais: markup, e um cálculo por cima
do `equivalente` já existente. Ver proposal.md para não-objetivos
(sem alocação de assinatura, sem infra, sem PDF).

## Decisions

**`billing/clients.json`, mesma convenção JSON de `budgets/clients.json`.**

```json
{
  "clients": {
    "xpto": {
      "markup": 1.6,
      "minimum_monthly_usd": 2000,
      "includes_infrastructure": true
    }
  }
}
```

**`src/billing.rs`:**
- `ConfigBillingCliente { markup: f64, minimum_monthly: Option<Money>, includes_infrastructure: bool }`
- `carregar_billing(caminho) -> Result<HashMap<String, ConfigBillingCliente>, ErroBilling>`
  — arquivo ausente = mapa vazio (mesmo padrão de `budget::carregar_budgets`),
  erro só se existir e for JSON inválido.
- `struct RelatorioChargeback { client_id, custo_interno: Option<Money>, markup: f64, valor_faturavel: Money, piso_aplicado: bool, includes_infrastructure: bool }`
- `calcular_chargeback(client_id, config: Option<&ConfigBillingCliente>, custo_interno: Option<Money>) -> Result<RelatorioChargeback, ErroBilling>`
  — `Err(ErroBilling::SemMarkupConfigurado)` se `config` for `None` (spec:
  "Chargeback exige markup configurado explicitamente"). Com config:
  `marcado_up = custo_interno.unwrap_or(Money::ZERO) × markup`;
  `valor_faturavel = max(marcado_up, minimum_monthly)`;
  `piso_aplicado = minimum_monthly.is_some() && valor_faturavel > marcado_up`.
- `formatar_chargeback(&RelatorioChargeback) -> String`.

**Multiplicação de `Money` por `f64`**: reaproveita `Money::em_unidades()`
+ `Money::de_unidades()` (já existentes, já testados contra `NaN`/overflow)
— `Money::de_unidades(custo.em_unidades() * markup)`. Nenhuma aritmética
de ponto flutuante nova é inventada.

**`brian billing chargeback --client <id> [--period AAAA-MM] [--export csv]`**
reaproveita exatamente `periodo_do_mes`/`periodo_aberto` e
`consumo_do_cliente` já usados por `executar_costs` — mesma fonte de
verdade de `brian costs --client`, nunca uma segunda consulta divergente.

## Risks / Trade-offs

- **Markup aplicado só sobre `equivalente`, nunca sobre `pago`**: dinheiro
  já pago diretamente ao provider (fora de assinatura) não é remarcado —
  decisão de escopo consciente (proposal.md), evita inventar uma regra de
  negócio que o blueprint não especifica claramente para esse caso.
