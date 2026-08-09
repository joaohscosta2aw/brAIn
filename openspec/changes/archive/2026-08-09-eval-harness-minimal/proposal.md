## Why

D-8 trava o router adaptativo atrás de "harness de eval funcionando" e D-13 trava
qualquer roteamento adaptativo até esse harness existir (§112 do blueprint).
Sem ele, nenhuma decisão futura de routing tem base — é aposta, não dado. Este
change constrói o mínimo que responde "esse caso passa, de forma repetível?",
sem o qual v0.3 (router) nunca pode começar de forma honesta.

## What Changes

- Casos de eval como arquivos de dados (`evals/cases/*.json`): fixture (repo +
  commit base), tarefa, provider, gate — reaproveita exatamente o vocabulário já
  existente de `brian run --gate` (`run-fast-path-gates`).
- `brian eval run [--case <id>] [--dir <caminho>]`: roda cada caso N=3 vezes
  (variância do blueprint §112.4) via `execucao::iniciar_run`, cada tentativa
  vira um run rastreado normalmente (mesma tabela `run`/`run_event`).
- Grading só automático: tentativa passa se e só se o run terminar `Concluido`
  (provider + gate, já resolvido pela infraestrutura de `run-fast-path-gates`).
  Sem grading assistido por LLM, sem AST pattern.
- Relatório de taxa de sucesso por caso: `PASSOU`/`FALHOU`/`INSTÁVEL` (taxa entre
  0.34 e 0.66 — banda exata do blueprint §112.4).
- `execucao::iniciar_run` ganha capacidade de fixar um `base_commit` explícito
  (hoje sempre resolve `HEAD` do repo) — sem isso, um caso de eval não é
  reprodutível entre execuções.
- Runs de eval usam um contexto sintético dedicado (`client_id = "eval"`), nunca
  o contexto ativo do operador — não polui atribuição/custo real (D-16).

## Capabilities

### New Capabilities
- `evaluation/eval-harness`: define e executa casos de eval, reporta taxa de
  sucesso com classificação de instabilidade.

### Modified Capabilities
- `execution/isolated-run`: `base_commit` do run passa a ser opcionalmente
  explícito (antes, sempre `HEAD` do repo) — mudança aditiva, comportamento
  padrão (sem override) é idêntico ao atual.

## Impact

- `src/eval.rs` (novo): carregamento de casos, execução N vezes, classificação.
- `src/execucao.rs`: `PedidoRun` ganha campo `base_commit: Option<&str>`.
- `src/comandos.rs`, `src/main.rs`: `brian eval run`.
- Nova dependência: nenhuma — casos de eval em JSON reaproveitam `serde_json`,
  já presente (ver design.md para a alternativa YAML descartada).
- Sem migração de schema: cada tentativa é um `run` normal; nenhuma tabela nova.

## Não-objetivos (mínimo, não o harness completo do blueprint §112)

- **Sem grading assistido por LLM** (`must_contain_pattern`/AST) nem
  `human_review_sample`: só `must_pass` via gate determinístico.
- **Sem `max_cost_usd`/`max_turns` como critério de reprovação**: Brian ainda
  não calcula custo de run (`Brian-Cost-USD: unknown`, `run-fast-path-gates`) —
  não dá para fazer cumprir um teto que não se mede.
- **Sem comparação entre providers** (`--provider a,b`) nem
  `brian eval compare --baseline`: um caso roda contra um provider por vez.
- **Sem calibração de Evaluator** (§112.5): não existe Evaluator/Reasoning
  Engine no Brian ainda — nada para calibrar.
- **Sem router**: este change só produz o dado que o router (v0.3, D-8) vai
  precisar depois; não decide nada sozinho.

## Conformidade — checklist §16

- **M1-M6 / OP-1..OP-8**: atende OP-2 (tuning contínuo precisa de medição
  confiável) e OP-4 (inteligência retroalimentada) — sem harness, não há como
  medir se um ajuste melhorou ou piorou.
- **D-16/D-17**: não toca ledger de custo real nem Continuity Pack — runs de
  eval usam client sintético isolado, nunca atribuídos a cliente real.
- **D-10**: não viola — cada caso ainda é um único provider fazendo um único
  run (mesma disciplina de `isolated-tracked-run`); o harness é infraestrutura
  de medição ao longo do tempo (T), não orquestração de múltiplos providers
  numa sessão.
- **H-1**: não depende do Context Governor.
- **Versão alvo**: pré-requisito de v0.3 (D-13) — não é v0.3 em si; nenhum
  subsistema de router/UI é implementado aqui.
