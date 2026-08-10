## Context

`Store::consumo_do_cliente(client_id, periodo)` já devolve os
`UsageRecord` do mês para um cliente (usado por `brian costs --client`), e
`comandos::agregar` já soma `pago`/`equivalente` sobre uma lista de
registros. Esta change não adiciona nenhuma tabela nova: orçamento é
config (arquivo), gasto é o que já existe no ledger. Ver proposal.md para
motivação (D-16, blueprint §45) e não-objetivos (sem hierarquia completa,
sem override auditado).

## Goals / Non-Goals

**Goals:**
- Config honesta: cliente sem entrada nunca é afetado.
- Alerta sempre mostra gasto e limite lado a lado, nunca só um veredito.
- Limite duro recusa *antes* de qualquer efeito colateral (D-12, mesma
  disciplina de "provider não suportado" em `iniciar_run`).

**Non-Goals:** ver proposal.md.

## Decisions

**`budgets/clients.json`, não YAML** — mesma convenção já estabelecida
(`routing/rules.json`, `models/pointers.json`): `serde_json::Value`, sem
derive, sem dependência nova.

```json
{
  "clients": {
    "xpto": {
      "monthly_usd_equivalent": 500,
      "monthly_tokens": 50000000,
      "alert_at_percent": [50, 80, 95]
    }
  }
}
```

Todos os três campos são opcionais — um cliente pode limitar só por USD,
só por tokens, ou nenhum dos dois (mas aí por que teria entrada? Permitido
mesmo assim, sem tratamento especial: `calcular_status` simplesmente não
teria limite para checar).

**Arquivo ausente = mapa vazio, não erro.** Orçamento é opt-in
(proposal.md) — um repositório sem `budgets/clients.json` continua
funcionando exatamente como hoje. Diferente de `routing/rules.json`
(`router::carregar_regras` falha se ausente porque roteamento é
obrigatório), aqui a ausência é o estado padrão esperado.

**`src/budget.rs` novo módulo:**
- `carregar_budgets(caminho: &Path) -> Result<HashMap<String, BudgetCliente>, ErroBudget>`
  — `Ok(HashMap::new())` se o arquivo não existir; erro só se existir e for
  JSON inválido.
- `struct BudgetCliente { monthly_usd_equivalent: Option<Money>, monthly_tokens: Option<u64>, alert_at_percent: Vec<u8> }`
- `struct StatusBudget { client_id: String, gasto_usd: Option<Money>, limite_usd: Option<Money>, gasto_tokens: u64, limite_tokens: Option<u64>, alertas_cruzados: Vec<u8>, limite_excedido: bool }`
- `fn calcular_status(client_id: &str, budget: Option<&BudgetCliente>, gasto_usd: Option<Money>, gasto_tokens: u64) -> StatusBudget`
  — função pura, sem I/O, testável isoladamente (mesmo padrão de
  `router::calcular_scores`/`workflow::avancar`). `limite_excedido` é
  `true` se `gasto_usd >= limite_usd` OU `gasto_tokens >= limite_tokens`
  (qualquer um dos dois configurados que estoure já basta).
- `alertas_cruzados`: subconjunto de `alert_at_percent` cujo limiar já foi
  ultrapassado pelo % de gasto USD (tokens não geram alerta separado nesta
  v1 — um limite já cobre o caso; YAGNI para um segundo eixo de alerta).

**Limite duro fica em `comandos::executar_run`, não em `execucao::iniciar_run`.**
`executar_run` já resolve o provider antes de chamar `iniciar_run`
(`resolver_provider`/`resolver_provider_por_score`) — a checagem de
orçamento entra no mesmo lugar, mesmo padrão: decide na camada de
comando, motor de execução continua sem saber de orçamento (mesma
separação já usada por `router`/`model_router`). Isso evita adicionar mais
um parâmetro a `iniciar_run`, que já é reaproveitado por eval/workflow/
comparacao/context_governor sem nenhum deles precisar saber de orçamento.

**`brian budget check --client <id>` reaproveita `Store::consumo_do_cliente`
+ `comandos::agregar`** — mesma fonte de dado de `brian costs --client`,
sem nova consulta.

## Risks / Trade-offs

- **Sem hierarquia de precedência** (blueprint §45.3): só nível cliente
  existe. Aceito — não há "change" no modelo de dados do Brian ainda;
  fingir a hierarquia sem os níveis reais seria pior que declarar a
  ausência.
- **Limite duro só é checado ao *iniciar* um run**, não durante: um run já
  em andamento não é interrompido, mesmo que ultrapasse o limite mid-run.
  Aceito e documentado (proposal.md) — `iniciar_run` é síncrono e
  bloqueante, não há processo em segundo plano para pausar.
