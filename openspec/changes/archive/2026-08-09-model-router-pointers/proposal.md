## Why

`brian run --model` hoje aceita só um nome de modelo concreto (`gpt-5.4`), decidido
pelo operador. O blueprint (§12) separa provider e modelo como decisões
distintas: ponteiros semânticos (`coding`, `reasoning`, `quick`, ...) mapeiam
para `{provider, tier}`, e o tier resolve para o nome concreto do provider já
decidido (pelo `routing/provider-rules`, já implementado). Sem isso, cada
config de workflow/skill que precisar "usar o modelo forte" ou "usar o modelo
barato" tem que hardcodar um nome de modelo específico, que quebra a cada
release de provider.

## What Changes

- `models/pointers.json` (dado): ponteiro semântico → `{primary: {provider,
  tier}, fallback: {provider, tier}}`. Os 7 pointers do blueprint §12.1
  (`reasoning`, `coding`, `quick`, `compact`, `review`, `long-context`,
  `research`).
- `providers/<provider>/models.json` (dado, editado manualmente pelo
  operador): `tier` → nome concreto de modelo, com `resolved_at` e
  `resolution_source` (sempre `"manual"` nesta change — ver Não-objetivos).
- `brian run --model-pointer <nome>` resolve pointer → provider (respeitando
  o que `routing/provider-rules` já decidiu, quando aplicável) → tier → nome
  concreto. `--model` continua existindo para override direto de nome
  concreto (vence o pointer, mesma disciplina de override de
  `routing/provider-rules`).
- Regra de fallback (blueprint §12.4): primary indisponível → fallback;
  fallback também indisponível → erro explícito, run não inicia; tier ausente
  no provider resolvido → degrada para o tier mais próximo disponível, com
  aviso — nunca escolhe silenciosamente um modelo mais fraco sem avisar.
- "Indisponível" reaproveita a mesma fonte de verdade que já existe:
  `execucao::PROVIDERS_EXECUCAO_VERIFICADA`.

## Capabilities

### New Capabilities
- `routing/model-pointers`: resolve ponteiro semântico de modelo em nome
  concreto, com fallback explícito e degradação avisada.

## Impact

- `src/model_router.rs` (novo): carregamento de `pointers.json` e
  `providers/*/models.json`, resolução de pointer → modelo concreto.
- `src/comandos.rs`, `src/main.rs`: `--model-pointer` em `brian run`,
  precedência sobre `--model` documentada (override explícito de nome
  concreto sempre vence, mesma disciplina de `--provider`).
- `src/execucao.rs`: nenhuma mudança de contrato — `iniciar_run` continua
  recebendo `model: Option<&str>` já resolvido, mesma separação de
  responsabilidade usada para `routing/provider-rules`.
- Nova dependência: nenhuma — reaproveita `serde_json`.

## Não-objetivos

- **Sem `brian providers models <provider>` (auto-detecção)**: o blueprint
  propõe um comando que popula `models.json` sozinho a partir de uma consulta
  ao provider. Não existe hoje nenhuma fonte real disso — `codex --help` não
  expõe lista de modelos por tier, e inventar uma lista sem fonte violaria a
  mesma disciplina de honestidade de `PROVIDERS_EXECUCAO_VERIFICADA`/
  `Brian-Model: unknown` já estabelecida no projeto. `models.json` é editado
  manualmente pelo operador nesta change; `resolution_source` sempre
  `"manual"`.
- **Sem UI/trace de degradação** (blueprint: "visível na UI"): Brian não tem
  UI ainda (CLI first, D-2). Degradação é avisada via stderr/evento de run,
  não um subsistema de trace novo.

## Conformidade — checklist §16

- **M1-M6 / OP-1..OP-8**: atende OP-3 (nomes de modelo trocam sem reescrever
  lógica) e OP-7 (modelo certo pra tarefa, não o mais caro por hábito).
- **D-16/D-17**: não toca ledger nem Continuity Pack.
- **D-10**: não viola — resolve o modelo de um único run, não orquestra nada
  entre providers.
- **H-1**: não depende do Context Governor.
- **Versão alvo**: v0.3 (complementa `routing/provider-rules`, pré-requisito
  conceitual de Router Fase 2/3 — a próxima change desta sequência).
