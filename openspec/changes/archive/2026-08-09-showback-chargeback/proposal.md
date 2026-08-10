## Why

Blueprint §44: Showback (o que o cliente consumiu) já existe de fato via
`brian costs --client` — o que falta é Chargeback (o que deve ser
faturado): markup configurável por cliente, aplicado sobre o custo
interno já apurado, com piso mensal opcional.

## What Changes

- `billing/clients.json` (config, JSON — mesma convenção de
  `budgets/clients.json`/`routing/rules.json`): `markup` (multiplicador),
  `minimum_monthly_usd` (piso opcional), `includes_infrastructure`
  (rótulo informativo — ver Não-objetivos).
- `brian billing chargeback --client <id> [--period AAAA-MM]
  [--export csv]` — aplica o markup ao custo interno (`equivalente`, via
  `Store::consumo_do_cliente` + `comandos::agregar`, mesma fonte de
  `brian costs`) e ao piso mensal, mostra os dois lados (custo interno e
  valor faturável) sempre juntos.

## Capabilities

### New Capabilities
- `capacity/chargeback`: markup configurável por cliente aplicado ao
  custo interno já atribuído — nunca decide preço, só calcula.

## Impact

- `src/billing.rs` (novo): `ConfigBillingCliente`, `carregar_billing`,
  `calcular_chargeback` (função pura), `formatar_chargeback`.
- `src/comandos.rs`, `src/main.rs`: `brian billing chargeback`.
- Sem mudança em `storage/`: reaproveita `consumo_do_cliente` já
  existente, nenhuma tabela nova.

## Não-objetivos

- **Sem alocação de assinatura nem custo de infraestrutura do Brian**
  (blueprint mostra "Alocação de assinatura" e "Infraestrutura Brian"
  como linhas do custo interno): Brian não mede nenhum dos dois hoje.
  Chargeback desta v1 aplica markup só sobre o `equivalente` já apurado
  (mesma fonte de `brian costs`) — `includes_infrastructure` no config é
  só um rótulo informativo copiado pro relatório, não afeta o cálculo.
- **Sem PDF**: o próprio blueprint (§44.1) escalona PDF para v0.3. Esta
  change fica em CSV/terminal (mesmo padrão de `brian costs --export`).
- **Sem integração de faturamento** (blueprint §44.1: v1.0+): fora de
  escopo, sem sistema de faturamento pra integrar ainda.
- **Cliente sem `markup` configurado não tem chargeback calculável** —
  `brian billing chargeback` recusa com erro claro nesse caso (nunca
  assume markup 1.0 silenciosamente, que fabricaria "sem markup" como se
  fosse uma configuração real e não uma ausência).

## Conformidade — checklist §16

- Reaproveita `Store::consumo_do_cliente`/`comandos::agregar` — nenhum SQL
  novo (D-9).
- Nunca soma `pago` e `equivalente` (invariante já verificada por
  `scripts/verificar-invariantes.sh`) — chargeback aplica markup só sobre
  `equivalente`, mostrado separado.
- **Versão alvo**: blueprint §44 nominalmente v0.0-v0.3; implementada
  agora fora da ordem original.
