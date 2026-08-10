## Why

Blueprint §45 (D-16): "budgets de cliente... existem no v0.0 (alertas + ledger)"
— a definição e o monitoramento de orçamento são "minuto 0", não uma
feature adiada. Hoje Brian atribui e reporta custo (`brian costs`), mas
não tem nenhuma noção de limite: um cliente pode consumir qualquer valor
sem aviso. Esta change fecha essa lacuna com o menor escopo honesto.

## What Changes

- `budgets/clients.json` (config, JSON — mesma convenção de
  `routing/rules.json`): orçamento mensal opcional por cliente
  (`monthly_usd_equivalent`, `monthly_tokens`, `alert_at_percent`).
  Cliente sem entrada = sem orçamento configurado = nunca bloqueado nem
  alertado (opt-in, não um limite silencioso por omissão).
- `brian budget check --client <id>` — gasto do mês corrente (reaproveita
  `Store::consumo_do_cliente` + `comandos::agregar`, já usados por `brian
  costs`) comparado ao orçamento configurado; reporta % usado e quais
  limiares de `alert_at_percent` já foram cruzados.
- **Limite duro em `brian run`**: se o gasto do mês já atingiu ou excedeu
  `monthly_usd_equivalent` (ou `monthly_tokens`), o run é recusado *antes*
  de qualquer persistência ou chamada de provider — mesmo ponto de recusa
  já usado por "provider não suportado" em `execucao::iniciar_run`, só que
  na camada de `comandos::executar_run` (onde o provider já é resolvido
  antes de chamar `iniciar_run`).

## Capabilities

### New Capabilities
- `capacity/budget-alerts`: orçamento mensal por cliente, alertas suaves e
  limite duro que recusa novos runs, ambos definidos por config editável
  (`budgets/clients.json`), nunca por estado oculto.

## Impact

- `src/budget.rs` (novo): carrega `budgets/clients.json`, calcula status
  de orçamento (gasto, %, limiares cruzados, excedido), função pura de
  decisão testável sem I/O.
- `src/comandos.rs`, `src/main.rs`: `brian budget check --client <id>`;
  `executar_run` ganha a checagem de limite duro antes de resolver
  provider/chamar `iniciar_run`.
- Sem mudança em `execucao.rs`: o motor de execução continua sem saber de
  orçamento, mesma disciplina de `router`/`model_router` (decisão fica na
  camada de comando, não no motor).

## Não-objetivos

- **Sem hierarquia completa de orçamento** (blueprint §45: organização /
  cliente / projeto / change / run / fase / provider·plan, com precedência
  "menor limite vence"): Brian não tem os conceitos de "change" nem
  "organização" no seu modelo de dados hoje. Esta change cobre só o nível
  que já existe de fato: cliente, mensal.
- **Sem orçamento de capacidade de plano/assinatura** (`providers.claude.budgets.week_used_percent_soft/hard`
  do blueprint): isso já é coberto por `capacity/capacity-windows-and-plans`
  (janela, %, restante, alerta por provider) — sobreporia, não somaria.
- **Sem `brian budget override`**: o blueprint pede override auditado com
  motivo textual — infraestrutura de auditoria própria. Nesta v1, ajustar
  o limite é editar `budgets/clients.json` diretamente (rastreável por git,
  se o arquivo for versionado) — mais simples e honesto sobre o que existe
  hoje. Override formal fica para quando houver demanda real.
- **Sem pausar runs em andamento**: `iniciar_run` bloqueia até o provider
  terminar (não há processo em segundo plano para pausar); a recusa
  acontece só na entrada de um novo run, nunca no meio de um já iniciado.

## Conformidade — checklist §16

- **D-16**: esta change é exatamente a definição/monitoramento de budget
  que D-16 pede como "lei do produto" desde o v0.0.
- **D-9**: `Store::consumo_do_cliente` já existe; nenhum SQL novo fora de
  `storage/`.
- Reaproveita `comandos::agregar` (já usado por `brian costs`), sem
  duplicar lógica de soma de custo.
- **Versão alvo**: blueprint §45 nominalmente v0.0-v0.2; implementada
  agora fora da ordem original, item (6) da lista combinada com o autor.
