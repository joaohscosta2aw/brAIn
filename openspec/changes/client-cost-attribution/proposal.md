## Why

Hoje o consumo de IA acontece sem dono: tokens são gastos em múltiplos providers,
para múltiplos clientes, sem registro de quem consumiu o quê. Não há como faturar
um cliente de consultoria com base em consumo real, nem detectar capacidade sendo
queimada sem retorno.

Esta é a primeira lei do produto (**D-16 — zero token perdido**) e nada precede ela.
Sem ledger íntegro, toda capacidade construída depois — janelas, budget, roteamento,
comparação de providers — repousa sobre dado não confiável.

**Versão alvo: v0.0.**

## What Changes

- Novo ledger `usage_record` como unidade de verdade do consumo: tokens de entrada,
  cache, saída e reasoning, por chamada de provider.
- Toda linha do ledger carrega **procedência rotulada**: `usage_source`
  (`provider` | `brian_measured` | `estimated`), `cost_source`
  (`provider` | `catalog` | `unknown`) e `billing_mode`. Fontes nunca se misturam
  sem rótulo.
- **Dois valores monetários coexistem** por registro (BRIAN-BLUEPRINT-V1.md §42):
  o custo efetivamente pago ao provider e o custo equivalente em API. O primeiro é
  base de custo; o segundo é base de faturamento — pode-se cobrar do cliente a preço
  de token mesmo operando sob assinatura. Um nunca substitui nem é apresentado como
  o outro.
- Custo reportado pelo provider tem precedência sobre catálogo de preço como valor
  **pago** (**D-6**), sem apagar o equivalente, que serve a outro propósito.
- Cadeia de atribuição até o cliente. Em observe mode, `run` e `phase` podem ser
  nulos; **cliente não pode**, exceto com `attribution_status = unattributed` explícito.
- `unattributed` vira alarme observável e permanente até zerar — nunca estado normal
  silencioso.
- Importação de consumo a partir das fontes disponíveis de cada provider, idempotente
  e re-executável sobre janelas já importadas.
- Atribuição manual de registros órfãos a um cliente.
- Consultas de custo por cliente, por provider, por período, e exportação para decisão
  humana e faturamento.

Sem breaking changes: não existe comportamento anterior.

## Non-Goals

Escopo deliberadamente fora desta change:

- **Janelas de capacidade, planos, % consumido, burn rate e budget** → change
  `capacity-windows-and-plans`. Consequência: a **alocação proporcional do custo do
  plano** entre clientes (assinatura de R$ 1.000 → 42% para um cliente → R$ 420)
  fica fora daqui, pois exige o custo do plano declarado. O custo equivalente em API
  **não** depende disso e está no escopo desta change.
- **Análise de margem e a decisão "assinatura ou API"**, que comparam o pago com o
  faturável. Dependem da alocação por plano; chegam com a change seguinte. Esta
  change entrega os dois números que essa análise vai consumir.
- **Bloqueio hard por limite.** No v0.0 as políticas alertam e registram; bloqueio
  exige budget, que é a próxima change.
- **Orquestração, run, worktree, workflow** → v0.2. Aqui o modo é *observe*: contar e
  atribuir consumo sem dirigir o agente.
- **Continuidade entre LLMs (D-17)** → v0.1.
- **UI, daemon, roteamento, comparação de providers** → v0.3+.
- **Qualquer forma de otimização de contexto (H-1).** Hipótese isolada; nada aqui
  depende dela.

## Conformidade (PREMISSAS-BASICAS.md §16)

- **M1–M6:** respeitados. M3 preservado — nada aqui transforma Brian em coding agent;
  o modo é observe. M4 confirmado: o problema só existe porque há N providers × M
  clientes × T tempo.
- **OP-1..OP-8:** OP-1 (eficiência na utilização) é o alvo direto; o ledger é o
  instrumento que torna desperdício visível. OP-4 (inteligência retroalimentada)
  fica habilitado, não exercido.
- **D-16:** é a razão de existir desta change.
- **D-17:** não tocado.
- **D-10:** não violado. Atribuição multi-cliente e multi-provider é justamente o
  território exclusivo do Brian; um provider único numa sessão única não resolve isso.
- **H-1:** não há dependência.

## Capabilities

### New Capabilities

- `capacity/usage-ledger`: captura fiel do consumo. O que é um `usage_record`, quais
  campos são obrigatórios, como a procedência é rotulada, precedência entre fontes de
  custo, idempotência de importação e as invariantes de integridade do ledger.
- `capacity/cost-attribution`: propriedade do consumo. Cadeia de atribuição até o
  cliente, tratamento e visibilidade de `unattributed`, atribuição manual, e consultas
  e exportação de custo por cliente, provider e período.

### Modified Capabilities

Nenhuma — não existem specs anteriores.

## Impact

- **Novo:** binário CLI `brian` em Rust; camada `storage/` com SQLite atrás de traits
  (D-1, D-9); coletores de uso por provider seguindo a hierarquia de integração
  headless JSON → session files → PTY (D-4).
- **Comandos introduzidos:** `brian import`, `brian attribute`, `brian costs`
  (incluindo `--unattributed`, `--by`, `--period`, `--export`).
- **Schema:** tabelas `client`, `provider`, `usage_record`, `price_catalog`,
  `schema_migration`. Referência: BRIAN-BLUEPRINT-V1.md §60.
- **Sem dependências externas de rede** além das fontes de uso dos próprios providers.
- **Critério de release:** violar qualquer invariante de integridade do ledger
  (§13.9 do blueprint) é bug de release, não débito técnico.
