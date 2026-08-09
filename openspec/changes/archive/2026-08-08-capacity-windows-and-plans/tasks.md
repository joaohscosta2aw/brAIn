## 1. Esquema

- [x] 1.1 Migração `0002_capacidade.sql`: tabela `provider_plan` (provider_id,
      billing_mode, plan_label, ativo_desde, ativo_ate) — vigência no mesmo padrão de
      `price_catalog`. Sem coluna de baseline (fora de escopo, design.md).
- [x] 1.2 Migração: tabela `quota_signal` (provider_id, bucket_id, grupo,
      remaining_percent, reset_at, observed_at), chave `(provider_id, bucket_id)`.
- [x] 1.3 Índices necessários para consulta por `provider_id` + vigência/instante em
      ambas as tabelas.
- [x] 1.4 Confirmar que a migração é puramente aditiva: `cargo test` da change
      anterior continua passando sem alteração.

## 2. Tipos de domínio

- [x] 2.1 `TipoJanela` (dia, semana, mês, ciclo do plano) em `src/domain.rs`.
- [x] 2.2 `Plano` (billing_mode, identificador do plano, origem `provider`).
- [x] 2.3 `JanelaDeCapacidade` (consumido, capacidade opcional, % opcional, restante
      opcional, fonte rotulada por campo — `provider` ou `brian_measured` —, burn,
      eta de esgotamento opcional).
- [x] 2.4 Nenhum campo de capacidade desconhecida vira zero — mesma disciplina de
      `Option<T>` da change anterior (`domain.rs::Tokens`).

## 3. Detecção de plano e quota — Claude

- [x] 3.1 `src/adapters/claude.rs`: novo método `consultar_plano()` que invoca
      `claude auth status` (JSON), extrai `subscriptionType`.
- [x] 3.2 Falha da consulta (CLI ausente, não autenticado, JSON inesperado) não
      derruba a coleta normal de `ColetorDeUso` nem os demais providers.
- [x] 3.3 Persistir via `Store` em `provider_plan` (fecha vigência anterior se o
      plano mudou, spec "Plano detectado muda").
- [x] 3.4 Teste com fixture de saída de `claude auth status` (sem invocar o binário
      real).

## 4. Detecção de plano e quota — Codex

- [x] 4.1 `src/adapters/codex.rs` (ou módulo novo `codex_appserver.rs`): cliente
      JSON-RPC mínimo sobre stdio — abre `codex app-server --stdio`, envia
      `initialize`, envia `account/read` e `account/rateLimits/read`, correlaciona
      respostas por `id`, encerra o processo (design.md: "cliente mínimo, não SDK
      completo").
- [x] 4.2 `stdin` controlado e timeout explícito no lado do Rust, mesmo padrão do
      `GeminiAdapter` (design.md, risco aceito).
- [x] 4.3 Extrair `planType` de `account/read` e `usedPercent`/`windowDurationMins`/
      `resetsAt` de `account/rateLimits/read` (janelas `primary`/`secondary`).
- [x] 4.4 Persistir plano em `provider_plan` e janelas em `quota_signal`.
- [x] 4.5 Falha (processo não inicia, handshake falha, timeout) não derruba a coleta
      normal nem os demais providers.
- [x] 4.6 Teste com fixture de mensagens JSON-RPC (sem invocar o binário real).

## 5. Sinal de quota do Gemini (fidelidade)

- [x] 5.1 `GeminiAdapter`: novo método `consultar_quota()` reaproveitando
      `extrair_sinais`, preservando `remaining_fraction` (convertido para
      `remaining_percent`) e `reset_time` — hoje descartados, só o sinal de
      presença vai para o ledger.
- [x] 5.2 Persistir via `Store` em `quota_signal`, upsert por `(provider_id,
      bucket_id)`.
- [x] 5.3 Reaproveitar `aguardar_com_timeout` e `stdin` nulo (mesmas proteções do
      incidente registrado na change anterior).
- [x] 5.4 Comportamento existente de `GeminiAdapter` como `ColetorDeUso` (sinal no
      ledger) permanece inalterado.

## 6. Cálculo de janela de capacidade

- [x] 6.1 `src/capacidade.rs`: função pura que recebe consumo do ledger + plano
      vigente + sinal de quota mais recente (quando houver) e retorna
      `JanelaDeCapacidade` com fonte por campo.
- [x] 6.2 Sem sinal de quota: capacidade/%/restante retornam ausentes, nunca zero
      (spec "Janela sem capacidade conhecida").
- [x] 6.3 Burn rate a partir do consumo medido nas últimas N horas do ledger.
- [x] 6.4 Projeção de esgotamento apenas quando capacidade e burn são ambos
      conhecidos; rotulada como projeção linear simples.
- [x] 6.5 Janela histórica que cruza troca de plano usa o plano vigente em cada
      trecho (spec "Janela histórica usa o plano vigente à época").
- [x] 6.6 Testes de unidade cobrindo os cenários acima com valores sintéticos.

## 7. Rateio de custo de plano (showback)

- [x] 7.1 `src/capacidade.rs` (ou módulo próprio): função pura que soma tokens
      atribuídos por cliente no mês corrente do plano e aloca fração do custo do
      plano proporcional a cada cliente.
- [x] 7.2 Consumo `unattributed` excluído do rateio entre clientes e reportado à
      parte.
- [x] 7.3 Rateio recusado (não calculado) para `billing_mode = api`.
- [x] 7.4 Toda apresentação do valor alocado identifica sua natureza de showback,
      distinta de custo pago.
- [x] 7.5 Testes: dois clientes dividem, cliente único consome tudo, parcela não
      atribuída à parte, plano API recusa rateio.

## 8. Providers sem fonte (Grok, Copilot, Qwen)

- [x] 8.1 Registro explícito (mesmo padrão de `StatusCobertura` da change anterior)
      de que esses três providers não têm fonte de plano/quota nesta change, com o
      motivo investigado (design.md: `--help`, código-fonte, docs oficiais checados).
- [x] 8.2 `brian capacity` e `brian plans list` listam esses providers como "sem
      fonte", nunca os omitem.

## 9. Superfície CLI

- [x] 9.1 `brian capacity` — todos os providers com plano detectado: plano, janela,
      %, restante, reset, burn; providers sem fonte aparecem marcados como tal.
- [x] 9.2 `brian capacity --provider <id>` — mesmo detalhe para um provider.
- [x] 9.3 `brian plans list` — leitura simples do plano vigente por provider
      (nenhum `brian plans set`, não há o que declarar).
- [x] 9.4 Critério de UX do D-16 (§13.7): resultado em modo síncrono, sem chamada de
      rede além da leitura local do ledger/plano (as fontes de quota já foram
      importadas antes, não são buscadas ao vivo na consulta de capacidade).

## 10. Verificação

- [x] 10.1 Cobertura de cada cenário dos três specs desta change (auditoria manual,
      mesmo processo da task 8.1 da change anterior).
- [x] 10.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos verdes.
- [x] 10.3 Testar com dado real da máquina do operador: `claude auth status` e
      `codex app-server` reais (não só fixtures) para confirmar o parsing contra a
      saída de verdade.
- [x] 10.4 `openspec validate --strict` limpo antes de considerar a change pronta
      para archive.
