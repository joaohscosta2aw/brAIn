## Context

`src/execucao.rs::iniciar_run` (`isolated-tracked-run` + `run-fast-path-gates`,
já arquivadas) já resolve worktree isolado, persistência D-12, execução do
provider e gate determinístico. Um caso de eval é, mecanicamente, o mesmo
`iniciar_run` chamado N vezes contra um repositório de fixture fixo, com
`--gate` como critério de aprovação — só falta o "N vezes + relatório de taxa"
por cima disso. Ver proposal.md para a motivação (D-8/D-13) e não-objetivos.

## Goals / Non-Goals

**Goals:**
- Caso de eval como dado (JSON), executável sem tocar código.
- Taxa de sucesso por caso, com banda de instabilidade (blueprint §112.4).
- Cada tentativa vira um `run` real e auditável — zero caminho de execução
  paralelo.

**Non-Goals:**
- Grading assistido por LLM, AST pattern, revisão humana amostrada (§112.2/
  §112.5) — fora de escopo (proposal.md).
- `max_cost_usd`/`max_turns` como critério — Brian não mede custo de run ainda.
- Comparação entre providers/baselines (`eval compare`).

## Decisions

**Casos de eval em JSON, não YAML.** O exemplo do blueprint (§112.2) usa YAML,
mas isso não é uma decisão travada (D-1..D-17 não cobre formato de caso de
eval) — só ilustração. JSON reaproveita `serde_json`, já dependência do
projeto; YAML exigiria uma dependência nova para uma feature deliberadamente
pequena. Alternativa (YAML) rejeitada só por custo de dependência, não por
mérito técnico — se o volume de casos crescer a ponto de JSON incomodar (sem
comentário, vírgula final chata), revisitar.

**Contexto sintético de eval (`client_id = "eval"`), garantido via
`upsert_client` antes de cada execução**, nunca o contexto ativo do operador —
runs de eval não são trabalho de cliente real e não podem aparecer em
`brian costs` de ninguém (D-16). `project` do contexto sintético é o id do
caso, para poder filtrar runs de um caso específico no banco sem tabela nova.

**`iniciar_run` ganha `base_commit: Option<&str>` dentro de `PedidoRun`** (não
um novo argumento posicional — já resolvido para `too_many_arguments` em
`run-fast-path-gates`). `None` preserva o comportamento atual (resolve `HEAD`);
`Some` fixa o commit — necessário porque um caso de eval precisa rodar sempre
contra o mesmo ponto de partida, não o `HEAD` corrente do repositório de
fixture (que pode ter avançado entre execuções).

**Sem tabela nova para resultado de eval.** Cada tentativa já persiste como
`run` normal (client_id="eval", project=case_id) — a taxa de sucesso é
computada em memória a partir dos N resultados retornados por `iniciar_run` na
própria invocação de `brian eval run`, e o relatório só imprime; consultar
depois é `SELECT ... FROM run WHERE client_id='eval' AND project=<case_id>`,
sem harness dedicado de storage. Alternativa (tabela `eval_result`) rejeitada:
duplicaria o que `run` já guarda (status, custo, timestamps) sem necessidade —
D-8 (n≥30 por célula, para o router futuro) já é servível por essa mesma
consulta.

**N fixo em 3**, sem flag de configuração — blueprint §112.4 define N=3 como
padrão e N=5 só para comparação entre configurações (fora de escopo aqui,
`eval compare` é non-goal). Adicionar uma flag de N configurável antes de
existir uma segunda necessidade real de N diferente é generalização
antecipada.

**Classificação por taxa**: `> 0.66` → Passou, `< 0.34` → Falhou, banda
`[0.34, 0.66]` → Instável — limites exatos do blueprint §112.4.

## Risks / Trade-offs

- **JSON sem comentário** dificulta documentar um caso complexo inline —
  aceito; descrição do caso já é um campo (`description`).
- **Fixture repo precisa existir localmente** (mesmo requisito de `--gate`/
  `codex exec` hoje) — sem fetch remoto automático nesta change.
- **N=3 é caro** (blueprint: suíte de 24 casos com N=3 ≈ $120) — mesma
  advertência do blueprint: rodar antes de release/mudança de prompt, não em
  todo commit; este harness não impõe cadência, só a mecânica.
