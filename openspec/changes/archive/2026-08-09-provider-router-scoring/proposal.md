## Why

`routing/provider-rules` (já arquivada) decide o provider só por regra
explícita — Fase 1 do blueprint §11.1. Fases 2/3 (evidência histórica como
desempate, depois como scoring pleno) foram travadas por D-8 atrás de
`n≥30` runs reais por célula. O autor decidiu conscientemente reverter essa
trava agora (`docs/DECISIONS.md`, nota de 2026-08-09) e quer o mecanismo de
scoring funcionando já, mesmo sem o volume de dados que o justificaria
estatisticamente.

## What Changes

- Quando mais de uma regra de `routing/rules.json` poderia decidir (ou
  quando o operador pede explicitamente scoring em vez de "primeira regra
  que casa"), o provider é escolhido por um score calculado sobre o
  histórico real de runs desse `client_id` — taxa de sucesso e duração
  média, por provider.
- `brian run --scored` ativa o modo scoring; sem essa flag, comportamento
  idêntico ao atual (primeira regra que casa, spec já existente de
  `routing/provider-rules` preservada).
- Todo run decidido por score é marcado com `n` (quantidade de runs
  históricos que alimentaram o score) — nunca esconde que a base é pequena.
- `brian router score --provider <id>` mostra o score calculado por
  provider, para auditoria manual sem precisar rodar um `brian run`.

## Capabilities

### New Capabilities
- `routing/historical-scoring`: calcula e aplica um score baseado em
  histórico real de runs para desempate/decisão de provider.

## Impact

- `src/router.rs`: nova função de scoring, reaproveitando `decidir`/
  `RegrasDeRoteamento` já existentes — score só entra quando `--scored` pede.
- `src/storage/mod.rs`, `src/storage/sqlite.rs`: nova consulta
  `runs_finalizados_do_cliente(client_id) -> Vec<RunRegistrado>` (`Concluido`
  ou `Falhou`, já persistidos por `execucao::iniciar_run`) — sem tabela nova,
  sem migração.
- `src/comandos.rs`, `src/main.rs`: `--scored` em `brian run`;
  `brian router score --provider <id>`.

## Não-objetivos

- **Sem os "seis termos" do v0.1-draft**: o blueprint atual (§11.1) descreve
  essa fórmula como removida por confiança injustificada e não documenta os
  termos — não existe especificação canônica pra seguir. Esta change define
  sua própria fórmula (design.md), usando só os sinais que o Brian realmente
  calcula hoje: taxa de sucesso e duração. Custo (`custo_equivalente`) e
  retries ficam fora — Brian não calcula nenhum dos dois para um run ainda.
- **Sem `constraints` (deny/allowlist por cliente)**: mesmo non-goal já
  registrado em `routing/provider-rules`.
- **Sem UI/Brian Inspector** (blueprint §66, "mostra 94% de sucesso e deixa
  implícito que decidiu algo, quando não decidiu"): sem UI, `brian router
  score` é a forma de auditar manualmente.

## Conformidade — checklist §16

- **M1-M6 / OP-1..OP-8**: atende OP-2/OP-4 (tuning contínuo, inteligência
  retroalimentada) — é exatamente o propósito de existir.
- **D-16/D-17**: não toca ledger nem Continuity Pack.
- **D-8**: **revertida conscientemente** (nota de 2026-08-09 em
  `docs/DECISIONS.md`) — esta change é a materialização dessa reversão.
  Registrado explicitamente: o `n` de cada decisão nunca é escondido.
- **D-10**: não viola — score decide *qual* provider, um por run, mesma
  disciplina de `routing/provider-rules`.
- **H-1**: não depende do Context Governor.
- **Versão alvo**: v0.4 nominalmente (blueprint), implementada agora por
  decisão consciente do autor, fora da ordem sagrada de v0.0→v0.3.
