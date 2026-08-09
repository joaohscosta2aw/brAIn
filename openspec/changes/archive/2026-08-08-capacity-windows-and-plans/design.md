## Context

`client-cost-attribution` já existe: `usage_record` no SQLite, `Store` trait em
`src/storage/mod.rs` (D-9: todo SQL fica ali), coletores por provider implementando
`ColetorDeUso`, e `GeminiAdapter` (`src/adapters/gemini.rs`) já invoca `agy --print
"/usage"` com timeout de 5s e `stdin` nulo — mas hoje descarta `remaining_fraction` e
`reset_time` num registro de ledger genérico (`model = "agregado:<grupo>"`), porque a
change anterior não tinha onde guardar percentual de janela. Ver proposal.md para a
motivação completa.

**[Δ] Escopo revisado pelo autor em 2026-08-08.** A primeira versão deste design
previa um plano *declarado pelo operador* (`brian plans set`) com baseline manual como
fallback para providers sem quota. Investigação na documentação oficial de cada
projeto (não suposição, não `--help` isolado) mostrou que três dos seis providers já
verificados na change anterior têm fonte própria de plano/quota, headless:

| Provider | Fonte | Nível |
|---|---|---|
| `claude` | `claude auth status` — CLI simples, JSON por padrão, campo `subscriptionType`. Documentado em code.claude.com/docs/en/cli-reference. | 1 |
| `codex` | `codex app-server` — servidor JSON-RPC 2.0 sobre stdio, mesmo mecanismo que a extensão oficial de VS Code usa. Método `account/read` retorna `planType`; `account/rateLimits/read` retorna janelas `primary`/`secondary` com `usedPercent`, `windowDurationMins`, `resetsAt`. Documentado e estável (não experimental) em `codex-rs/app-server/README.md`. | 1 |
| `gemini` | `agy --print "/usage"` — já em uso na change anterior. | 1 |

Os outros três **não têm fonte headless** — confirmado em três lugares para cada um
(`--help`, código-fonte do projeto quando aberto, documentação oficial):

| Provider | Onde foi checado | Resultado |
|---|---|---|
| `grok` | Lista completa de subcomandos em `xai-org/grok-build` (`crates/codegen/xai-grok-pager/src/app/cli.rs`), `--help`, guia oficial de autenticação | Nenhum comando de conta/plano/cota existe |
| `github-copilot` | `--help`, tópicos de ajuda oficiais `billing`/`limits` | `/usage` existe mas é interativo, por sessão — não por conta, não scriptável |
| `qwen-*` | `--help`, documentação oficial (`/auth`, `/model`, `/doctor`) | Só detecção reativa de erro de quota; nenhum endpoint de consulta |

Decisão do autor: **não construir declaração manual de baseline.** Providers sem
fonte ficam de fora desta change, documentados como tal (mesmo padrão de
`StatusCobertura` da change anterior — nunca desaparecem em silêncio). Se o operador
precisar de um número ali, resolve por fora do produto; não é escopo do v0.0.

Sem daemon, sem run, sem UI (mesma restrição da change anterior): tudo aqui é
consulta sob demanda via CLI.

## Goals / Non-Goals

**Goals:**

- Detecção automática de plano/quota para os três providers com fonte (Claude,
  Codex, Gemini), sem digitação do operador.
- Cálculo de janela (consumido, capacidade, %, restante, burn, reset) sob demanda,
  com fonte sempre rotulada (nível 1 provider / nível 2 medido pelo Brian).
- Sinal de quota do Gemini capturado com fidelidade (percentual e reset preservados),
  não mais reduzido a um registro de ledger genérico.
- Rateio de custo de plano de assinatura entre clientes, mensal.
- Providers sem fonte (Grok, Copilot, Qwen) continuam visíveis nas superfícies de
  capacidade, marcados como sem fonte — nunca omitidos.

**Non-Goals:**

- Declaração manual de plano/baseline pelo operador. Removido do escopo por decisão
  do autor (ver Context) — nada como `brian plans set` nesta change.
- Pré-cálculo/materialização de snapshot histórico (`capacity_snapshot` como tabela
  persistida, do blueprint) — calculado sob demanda a partir do ledger e do plano
  vigente, mesma decisão de "janelas derivadas na leitura" da change anterior. Revisar
  se um `capacity` real ultrapassar 200ms (mesmo critério do D-1).
- Circuit breaker (§13.4) e políticas automáticas de otimização (§13.8): dependem de
  routing, que não existe antes do v0.2.
- Rateio pro-rata para janelas diferentes do ciclo mensal do plano — v0.0 calcula o
  rateio apenas sobre a janela `calendar_month`.
- `identity_plan_binding` do blueprint: não há identidade multi-conta ainda
  (`context-and-identity-switching` é v0.1). Plano se liga a `provider_id`
  diretamente.

## Decisions

### Plano vinculado a `provider_id`, não a identidade

O blueprint liga plano a `identity_profile_id`. Essa entidade não existe no código
ainda. Ligar a `provider_id` é o subconjunto correto disponível hoje e não fecha a
porta para a extensão: quando identidade chegar (v0.1), a chave estrangeira troca de
`provider_id` para `identity_profile_id` numa migração aditiva, sem reescrever a
lógica de cálculo de janela.

### `provider_plan` com vigência própria, sem baseline

`provider_plan(provider_id, billing_mode, plan_label, ativo_desde, ativo_ate)`.
Sem coluna de baseline declarada (não existe mais nesta change) e sem tabela de
binding separada — só há um plano vigente por provider por vez (spec: "Vigência de
plano por provider"), mesma simplificação que `price_catalog` já usa (versionado por
vigência, não por tabela de junção).

*Alternativa considerada:* manter `baseline_json` para o caso de o operador querer
declarar algo manualmente no futuro. Rejeitada por YAGNI — adicionar a coluna quando
essa necessidade for real, não antes.

### `codex app-server`: cliente JSON-RPC mínimo, não um SDK completo

`codex app-server --stdio` fala JSON-RPC 2.0 line-delimited sobre stdin/stdout. Um
handshake (`initialize`) precede as chamadas normais. Este design usa um cliente
propositalmente mínimo: abre o processo, envia `initialize`, envia `account/read` e
`account/rateLimits/read`, lê as duas respostas por `id`, fecha o processo. Sem
manter conexão viva entre importações, sem assinar notificações (`account/updated`,
`account/rateLimits/updated`) — cada `brian import` abre e fecha sua própria sessão
curta.

*Razão:* o caso de uso é uma leitura pontual por import, não um cliente de app
interativo. Um cliente JSON-RPC genérico e assíncrono seria infraestrutura sem uso
real — a mesma disciplina que already levou a change anterior a rejeitar um cliente
HTTP genérico para o Gemini em favor de invocar o binário diretamente.

*Risco aceito, mesmo padrão do incidente Gemini da change anterior:* `codex
app-server` é um processo filho subject às mesmas proteções (`stdin` nulo — aqui não
esperamos OAuth interativo pois é leitura pós-login, mas o processo ainda pode
travar por outros motivos — e timeout explícito no lado do Rust).

### Capacidade calculada sob demanda, sem tabela de snapshot

`brian capacity` soma `usage_record` do provider na janela pedida, busca o plano
vigente e — para os três providers verificados — o sinal de quota mais recente na
nova tabela `quota_signal`. Nenhum resultado é persistido.

*Razão:* sem daemon, não há um processo de fundo gerando snapshots em intervalo
regular; persistir um snapshot só no momento da consulta duplicaria o que já está no
ledger sem ganho. Se o volume tornar a agregação lenta (mesmo critério do D-1, 200ms),
uma tabela de snapshot ou índice adicional resolve depois sem mudar o contrato da CLI.

### `quota_signal`: tabela nova, comum aos três providers, separada do `usage_record`

Percentual restante e reset não são consumo — são estado de cota. Guardá-los em
`usage_record` (como a change anterior fez para o Gemini, por não ter alternativa)
força um registro de "uso" fantasma. Aqui ganham tabela própria:
`quota_signal(provider_id, bucket_id, grupo, remaining_percent, reset_at,
observed_at)`, upsert por `(provider_id, bucket_id)` — sempre o sinal mais recente,
sem histórico de cada consulta. `bucket_id`/`grupo` cobrem o caso do Gemini (múltiplos
buckets por conta); Claude e Codex usam um bucket único por provider.

O comportamento existente de `GeminiAdapter` como `ColetorDeUso` (grava sinal de
presença no ledger) **não muda** — não há requisito de `usage-ledger` ou
`cost-attribution` sendo modificado por esta change. `GeminiAdapter` passa a também
alimentar `quota_signal`, via um método novo (`consultar_quota`), chamado pelo mesmo
comando de import.

*Alternativa considerada:* estender `usage_record` com colunas de quota nulas para os
demais providers. Rejeitada: mistura duas naturezas de dado na mesma tabela (D-9
trata schema como parte da fronteira de storage, mas a forma dos dados ainda deve
refletir o domínio — consumo e estado de cota são fatos diferentes).

### Burn rate a partir do ledger medido, não do sinal de quota

Burn (`tokens/hora`) usa sempre `usage_record` (medido), mesmo quando a janela tem
sinal de quota nível 1 disponível — nenhum dos três providers com sinal de quota
expõe tokens absolutos por janela (só percentual e, no caso do Codex, também não —
`RateLimitWindow` não traz contagem de tokens, só `usedPercent`). Projeção de
esgotamento usa `eta_exhaustion_at = null` quando não há capacidade em tokens
conhecida — consistente com "capacidade desconhecida não produz projeção inventada".

### Rateio mensal simples, sem prorateio

`plan_cost_allocation` soma tokens atribuídos por cliente dentro do mês corrente do
plano, divide pelo total atribuído (excluindo `unattributed`, spec explícito) e
multiplica pela fração do custo do plano. Sem prorateio por dias parciais do mês
(plano criado no meio do mês, por exemplo) — o mês é tratado como unidade inteira.

*Razão:* prorateio por dia introduz uma segunda unidade de tempo (dia) dentro de uma
janela mensal só para o caso raro de troca de plano no meio do mês, que a vigência do
plano já torna auditável mesmo sem prorateio automático. Adicionar quando um caso real
pedir.

## Risks / Trade-offs

- **`codex app-server` é uma superfície maior que uma chamada de CLI simples**
  (processo de longa duração em outros usos, protocolo com handshake) → mais
  código que `claude auth status`. Mitigação: cliente propositalmente mínimo (ver
  decisão acima), sessão curta por import, timeout explícito.
- **Plano detectado pode ficar desatualizado entre importações** (operador troca de
  plano fora de uma janela de import) → mesma primeira classe do incidente Gemini:
  mitigado por reconsultar a fonte a cada import, nunca cachear indefinidamente.
- **Três providers ficam sem número de capacidade nesta change** (Grok, Copilot,
  Qwen) → aceito conscientemente pelo autor; documentado nas specs como requisito
  explícito ("Provider sem fonte de plano/quota é excluído e documentado"), não como
  lacuna silenciosa.
- **Rateio mensal sem prorateio pode distorcer o primeiro/último mês de um plano** →
  aceito conscientemente (ver decisão acima); documentado, não escondido.

## Migration Plan

Migração aditiva `0002_capacidade.sql`: `provider_plan`, `quota_signal`. Nenhuma
coluna de `usage_record` muda. Reversível apagando `~/.brian/brian.db` (mesmo plano de
reversão da change anterior — nada é enviado ou publicado fora da máquina).
