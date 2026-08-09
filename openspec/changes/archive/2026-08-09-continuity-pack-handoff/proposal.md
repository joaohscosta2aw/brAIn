## Why

Context e identidade (change anterior) resolvem metade da missão: trocar de cliente
sem misturar credencial. A outra metade — trocar de **LLM** sem recomeçar a
conversa, a análise e as decisões — ainda não existe. D-17 é lei do produto,
co-primária com D-16: "perder contexto na troca de LLM é bug de produto"
(BRIAN-BLUEPRINT-V1.md §2.10). Sem isso, multi-provider é teatro — o operador paga
de novo o mesmo raciocínio a cada troca.

## What Changes

- **Continuity Pack**: artefato versionado por Context (cliente/projeto, já existente
  desde `context-and-identity-switching`) com objetivo, critérios de sucesso,
  decisões, análise, arquivos tocados, tentativas que falharam, próximos passos e
  ponteiros de evidência.
- **Notas de memória**: `brian memory note "..."` e `brian memory decide "..." --why
  "..."` — registro manual, append-only, com proveniência (D-14), que alimenta o
  pack.
- **Handoff**: `brian handoff --to <provider>` monta o pack atual e imprime uma
  versão pronta para o próximo worker receber (sem `run`, que é v0.2 — aqui é o
  operador que leva o pack ao próximo provider, mesmo padrão de `connect` imprimir
  `export` em vez de agir por trás do operador).
- **Arquivos tocados detectados, não digitados**: `git diff`/`git log` do
  repositório do context ativo alimenta a seção `touched` do pack automaticamente —
  critério de aceitação do blueprint exige que o pack "cite arquivos/símbolos reais
  do trabalho" (§34.0), não invenção nem digitação manual.

## Capabilities

### New Capabilities
- `continuity/memory-notes`: notas e decisões manuais, append-only, escopadas ao
  Context ativo (spec: nunca cruzam cliente — D-14 + §37.1).
- `continuity/pack`: montagem do Continuity Pack a partir de notas + git diff do
  projeto — denso, orçado, nunca o transcript bruto.
- `continuity/handoff`: comando que materializa o pack para o próximo provider.

### Modified Capabilities
(nenhuma — reaproveita `active_context`/`identity_profile` de
`context-and-identity-switching` sem alterar seus requisitos.)

## Impact

- Novo módulo `src/continuidade.rs`.
- Novas tabelas: `memory_note`, `continuity_pack_snapshot` (ou equivalente —
  detalhado em design.md) — aditivas.
- `brian memory note|decide`, `brian handoff --to <provider>`, `brian continuity
  show`.
- Nenhuma dependência nova: leitura de `git diff`/`git log` via subprocess `git`
  (já presente no ambiente, D-7 já depende de worktrees Git).

## Conformidade (PREMISSAS-BASICAS.md §16)

- M1-M6, OP-1..OP-8: sim — é o segundo ganho mínimo da missão (M1/M5), impede
  reexplicar (poupa tempo real).
- Toca D-16 ou D-17: **D-17**, diretamente — é a lei que esta change implementa
  (mínimo).
- Não viola D-10: opera por Context (cliente × tempo), dentro da fronteira.
- Não depende de H-1: nenhuma dependência de Context Governor.
- Versão alvo: v0.1 (D-17 mínimo — não o Memory Engine completo do v0.4).

## Não-objetivos

- Memory Engine rico (retrieval, embedding, episodic/incident completo) — v0.4+.
- Namespace compartilhado entre clientes / promoção de memória (§37.2) — exige
  fluxo de auditoria e anonimização que esta change não constrói; nenhuma memória
  cruza cliente aqui, ponto final, sem exceção configurável.
- Injeção automática no processo do próximo provider (`brian run` fazendo o handoff
  sozinho) — v0.2, quando `run` existir. Aqui o pack é impresso/materializado, o
  operador decide como levá-lo ao próximo worker.
- Graph RAG, embedding cluster — explicitamente fora do D-17 mínimo (blueprint
  §2.10, "O que não é D-17").
