## Why

`brian run` hoje exige `--provider` sempre — não existe nenhuma decisão, só
digitação repetida do mesmo valor. Blueprint §11 define o Router em três fases
(D-8); a Fase 1 (regras explícitas + override manual) não depende de `n≥30`
(isso é só Fase 2) e já é possível com os sinais que o Brian tem hoje (cliente,
projeto, disponibilidade de provider). `eval-harness-minimal` deixou o
mecanismo para medir taxa de sucesso; este change deixa o operador não precisar
mais decidir manualmente toda vez que o padrão já responde.

## What Changes

- `--provider` em `brian run` vira opcional. Quando omitido, o Brian decide via
  regras; quando presente, o operador venceu (override explícito sempre tem
  prioridade sobre regra — blueprint §11.5).
- `routing/rules.json` (dado, não código): lista de regras `when → then`
  avaliadas em ordem, primeira que casa vence, mais um `default`. Sinais
  disponíveis nesta change: `client`, `project` — os únicos que o Brian
  realmente calcula hoje (nada de `task_type`/`risk`/`complexity`, que
  exigiriam um classificador que não existe).
- Provider escolhido pela regra é validado contra
  `PROVIDERS_EXECUCAO_VERIFICADA` antes de rodar — regra que aponta para um
  provider sem execução verificada falha com erro explícito, nunca troca
  silenciosamente para outro.
- `brian run --explain-only`: mostra qual provider seria escolhido e por qual
  regra, sem criar worktree nem invocar provider nenhum.

## Capabilities

### New Capabilities
- `routing/provider-rules`: decide o provider de um run a partir de regras
  quando o operador não especifica um explicitamente.

## Impact

- `src/router.rs` (novo): carregamento de `routing/rules.json`, avaliação de
  regras, decisão.
- `src/comandos.rs`, `src/main.rs`: `--provider` de `brian run` vira
  `Option<String>`; novo `--explain-only`.
- `src/execucao.rs`: nenhuma mudança de contrato — `iniciar_run` continua
  recebendo um `provider_id: &str` já resolvido; a decisão acontece antes, na
  camada de comando.
- Nova dependência: nenhuma — regras em JSON reaproveitam `serde_json`, mesmo
  padrão de `eval-harness-minimal`.
- Sem migração de schema.

## Não-objetivos (Fase 1 mínima, não o Router completo do blueprint §11)

- **Sem model pointers** (`coding`/`reasoning`/`quick`/`review`): não existem
  no Brian ainda — regra só escolhe `provider`, não `model_pointer`.
- **Sem classificação de `task_type`/`risk`/`complexity`**: ninguém calcula
  isso hoje; regra só casa em `client`/`project`.
- **Sem evidência histórica no scoring** (Fase 2, D-8): `n≥30` por célula não
  é avaliado nem perto de existir ainda — não é essa a barreira desta change,
  mas construir a Fase 2 antes da hora seria confiança injustificada (D-8).
- **Sem log de decisão auditável estruturado** (blueprint §11.4, JSON com
  `considered`/`historical_context`): `--explain-only` cobre a necessidade
  imediata de transparência sem um subsistema de auditoria novo.
- **Sem `constraints` (deny/allowlist por cliente)**: regra só decide o
  provider default; vetar um provider para um cliente específico fica para
  quando houver necessidade real documentada.

## Conformidade — checklist §16

- **M1-M6 / OP-1..OP-8**: atende OP-1 (menos digitação repetida = menos
  cerimônia) e OP-6 (direcionamento claro: regra explícita, não “route
  inteligente” opaco).
- **D-16/D-17**: não toca ledger nem Continuity Pack.
- **D-8**: respeitado por construção — Fase 1 apenas, sem termo histórico no
  scoring, `n≥30` nem entra na conta.
- **D-10**: não viola — continua um provider por run; a regra só decide
  *qual* provider antes de `iniciar_run` já existente rodar.
- **H-1**: não depende do Context Governor.
- **Versão alvo**: v0.3 (D-8, Fase 1).
