# BrIAn — Contexto do Projeto

Plano de controle de engenharia de IA. **Não é um coding agent.**
Governa capacidade paga, custo e continuidade de raciocínio entre múltiplos
providers (Claude, Codex, Gemini, Grok, ZCode) e múltiplos clientes.

## Estado atual

Fase de especificação. Nenhum código ainda — o repositório contém blueprint,
premissas e decisões. Não implemente features do produto sem uma change
OpenSpec aprovada.

## Fonte da verdade (nesta ordem)

1. `docs/PREMISSAS-BASICAS.md` — lei do repo (missão, princípios OP-1..8, fronteiras)
2. `docs/DECISIONS.md` — decisões travadas D-1..D-17
3. `BRIAN-BLUEPRINT-V1.md` — blueprint canônico (v1.0)
4. `openspec/` — comportamento aprovado, quando existir

`BRIAN-BLUEPRINT.md` é o v0.1-draft **histórico**. Serve para entender *por que*
certas decisões foram tomadas. Não use para implementar.

## Duas leis do produto

- **D-16 — Zero token perdido.** Ledger, janelas, %, burn e atribuição desde o minuto 0.
- **D-17 — Continuidade multi-LLM.** Continuity Pack permite trocar de provider sem reexplicar o trabalho.

Ambas sem reversão. Nada precede D-16.

## Fronteira do produto (D-10)

Brian atua onde há **N providers × M clientes × T tempo**.
Se um provider único numa sessão única já resolve, não é problema do Brian.
Brian não orquestra o inner loop de um provider.

## Ordem de construção (obrigatória)

`v0.0` capacidade/custo → `v0.1` continuidade → `v0.2` run/worktree →
`v0.3` router/UI → `v0.4+` memória rica.

Nunca workflow, UI ou learning antes de D-16 verde.

## Invariantes

- Custo reportado pelo provider tem precedência sobre catálogo de preço (D-6)
- Fonte do dado sempre rotulada: `provider` | `brian_measured` | `estimated` | `unknown` — nunca misturadas sem rótulo
- `unattributed` é alarme, nunca estado normal
- Memória é append-only; correção cria registro que supersede (D-14)
- Memória cross-client negada por construção, não por filtro
- Secrets nunca em texto claro — nem em log, nem em UI, nem para agentes
- SQL apenas em `storage/`, atrás de traits (D-9)
- Estado do run persistido antes de qualquer efeito colateral externo (D-12)
- Runtime chama-se "Brian Core". O termo "Brain" é proibido (D-11)

## Onde encontrar

| Assunto | Fonte |
|---|---|
| Comportamento aprovado | `openspec/specs/` |
| Change ativa | `openspec/changes/` — descubra com `openspec list` |
| Missão, princípios, fronteiras | `docs/PREMISSAS-BASICAS.md` |
| Decisões travadas | `docs/DECISIONS.md` |
| O que posso decidir sozinho; o que persistir | `docs/harness/autonomia-e-memoria.md` |
| Vou implementar | `docs/harness/protocolo-implementacao.md` |
| Vou revisar | `docs/harness/protocolo-revisao.md` |
| Arquitetura, subsistemas, schema SQLite | `BRIAN-BLUEPRINT-V1.md` |
| Glossário do domínio | `BRIAN-BLUEPRINT-V1.md` §96 |
| Política de ferramentas e autoridade | `Prompts/ToolingAndContextPolicy,md` |

## Hipótese, não premissa

**H-1 — Context Governor.** Reduzir custo ≥30% via contexto pré-montado.
Isolada por construção: nada pode depender dela até o experimento confirmar.

## Quando perguntar (RED)

Confirme com humano antes de: alterar comportamento especificado, tocar qualquer
decisão D-1..D-17, mexer no **caminho do dinheiro** (cálculo de custo, atribuição,
base de faturamento), contrato de CLI, esquema com dados reais, segurança ou
secrets, nova dependência crítica, enviar dado para fora da máquina, ou mudar
requisito do blueprint.

Classificação completa em `docs/harness/autonomia-e-memoria.md`.

## Ferramentas

Grafo estrutural do repositório disponível via MCP `code-review-graph`
(útil apenas quando houver código). Hierarquia de autoridade quando ferramentas
divergem: `Prompts/ToolingAndContextPolicy,md`.
