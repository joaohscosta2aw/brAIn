## Why

`client-cost-attribution` (v0.0, D-16) registra cada chamada e atribui custo, mas não
responde à pergunta central de assinatura: "quanto me resta esta semana?". Sem plano
declarado nem janela computada, o operador só vê tokens acumulados sem denominador —
não sabe se está a 10% ou 95% da capacidade paga, nem quando ela reseta. D-16 exige
essa resposta em <5s (BRIAN-BLUEPRINT-V1.md §13.7); hoje ela não existe.

## What Changes

- **[Δ] Escopo revisado pelo autor em 2026-08-08**, depois de investigar o que cada
  CLI realmente expõe (documentação oficial de cada projeto, não suposição):
  apenas providers com fonte headless de plano/quota entram nesta change. Sem
  declaração manual de baseline — se um provider não tem fonte, ele fica fora,
  documentado como tal, e o operador decide se quer editar isso manualmente por fora
  do produto.

| Provider | Fonte de plano/quota | Nível |
|---|---|---|
| `claude` | `claude auth status` (CLI, JSON) — `subscriptionType` | 1 (provider) |
| `codex` | `codex app-server` (JSON-RPC/stdio) — `account/read` (planType) e `account/rateLimits/read` (janelas com %, duração, reset) | 1 (provider) |
| `gemini` | `agy --print "/usage"` — já usado na change anterior, agora também alimenta janela | 1 (provider) |
| `grok` | nenhuma encontrada (checado `--help`, código-fonte, docs oficiais) | fora do escopo |
| `github-copilot` | nenhuma encontrada (`/usage` é interativo e por sessão, não por conta) | fora do escopo |
| `qwen-*` | nenhuma encontrada | fora do escopo |

- Cálculo de janela de capacidade a partir do ledger existente para os três
  providers com fonte: consumido, %, restante, burn rate, tempo até reset — sempre
  com `source` rotulado (nível 1 provider > nível 2 medido pelo Brian).
- Alocação proporcional do custo de plano de assinatura entre clientes (showback),
  usando a fração de consumo de cada cliente na janela do plano — item explicitamente
  deferido da change anterior.
- Superfície CLI: `brian capacity`, `brian plans list` (somente leitura — nada de
  `plans set`, não há baseline para declarar).

## Capabilities

### New Capabilities
- `capacity/plan-catalog`: declaração de plano por provider/identidade (billing_mode,
  custo, janela primária, baseline de capacidade quando quota não é reportada).
- `capacity/capacity-windows`: cálculo de consumo/capacidade/%/restante/burn/reset por
  janela, com hierarquia de verdade rotulada (provider > medido > baseline).
- `capacity/plan-cost-allocation`: rateio do custo do plano de assinatura entre
  clientes proporcional ao consumo de cada um na janela do plano.

### Modified Capabilities
(nenhuma — `usage-ledger` e `cost-attribution` permanecem como estão; esta change lê
o ledger existente sem alterar seu contrato.)

## Impact

- Novo módulo `src/capacidade.rs` (cálculo de janela) e `src/adapters/gemini.rs`
  passa a alimentar `capacity_snapshot`, não mais descartado como sem fonte útil.
- Novas tabelas: `provider_plan`, `capacity_snapshot` — extensão aditiva do schema
  existente, sem migração destrutiva (design.md da change anterior já reservou isso).
- Novos subcomandos CLI em `src/comandos.rs`.
- Nenhum provider adicional; nenhuma orquestração, run ou bloqueio hard de execução
  (v0.2). Alertas de limite continuam informativos, não bloqueantes.

## Conformidade (PREMISSAS-BASICAS.md §16)

- M1-M6, OP-1..OP-8: sim — protege capacidade paga (M2), responde em <5s (OP-... /
  §13.7), não introduz coding agent nem router.
- Toca D-16: sim, é a lei que esta change implementa. Não toca D-17.
- Não viola D-10: opera onde há N providers × M clientes × T tempo (janela = tempo).
- Não depende de H-1 (Context Governor): nenhuma dependência de redução de contexto.
- Versão alvo: v0.0 (ainda D-16; nenhum run, worktree ou workflow entra aqui).

## Não-objetivos

- Circuit breaker (§13.4) e políticas de otimização automáticas (§13.8) — dependem de
  routing/retries, que não existem antes do v0.2.
- Bloqueio hard de execução por budget — não há `run` para bloquear ainda (v0.2).
- UI nativa — CLI apenas, como todo o v0.0.
- Qualquer plano para providers além dos já cobertos em `client-cost-attribution`.
