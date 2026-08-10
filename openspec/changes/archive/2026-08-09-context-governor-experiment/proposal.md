## Why

H-1 (Context Governor) é uma hipótese, não um pilar (D-5): "pré-montar um
pacote de contexto mínimo reduz o custo total de um change bem-sucedido em
≥30%". O blueprint (§18.2) lista razões concretas para ela poder estar
errada (prompt caching inverte a economia; busca agêntica supera retrieval
pré-computado; o turno 1 é fração pequena do custo total). A única forma
honesta de decidir é rodar o experimento formal (§18.3) — nunca assumir.

## What Changes

- `ContextPackage` (blueprint §18.5): `Repository` (padrão, agente busca
  sozinho) vs `Curated` (pacote pré-montado: busca simbólica via `grep`,
  diff recente, notas de memória já existentes via `continuidade.rs`,
  deduplicado e truncado por orçamento de caracteres).
- Três braços de experimento: **A** (baseline, tarefa crua), **B**
  (governor, pacote + instrução "use só isto"), **C** (híbrido, pacote +
  instrução "ponto de partida, pode explorar mais").
- `brian experiment run-h1 --case <id> --arm a|b|c` roda uma tarefa em um
  braço — cada execução é um `run` real via `execucao::iniciar_run`
  inalterado, com a tarefa formatada conforme o braço.
- `brian experiment report-h1` calcula, por braço: taxa de sucesso e
  duração média — mesma lógica de `routing/historical-scoring`, agrupada
  por braço em vez de provider.
- 10 tarefas sintéticas (`experiments/h1-tasks.json`), rotuladas
  explicitamente como sintéticas — não são "30 changes reais do histórico
  do usuário" que o blueprint pede (não existem ainda: o autor não tem
  histórico de uso real do Brian). População reduzida de 30 para 10 por
  decisão consciente do autor.

## Capabilities

### New Capabilities
- `context/governor-experiment`: monta pacotes de contexto curados e mede,
  por braço, taxa de sucesso e duração — o experimento formal de H-1, não
  o Governor como feature de produto.

## Impact

- `src/storage/migrations/0008_experimento.sql` (novo):
  `experimento_execucao` (liga task_id + braço ao `run` real).
- `src/context_governor.rs` (novo): monta `ContextPackage::Curated` (busca
  simbólica simples, diff recente, notas de memória), formata a tarefa por
  braço.
- `src/comandos.rs`, `src/main.rs`: `brian experiment run-h1`,
  `brian experiment report-h1`.
- `experiments/h1-tasks.json` (novo, no repo): as 10 tarefas sintéticas.
- Sem mudança em `execucao.rs` — cada execução do experimento é um `run`
  normal, só a `tarefa` (string) muda conforme o braço.

## Não-objetivos

- **Sem grafo de código real** (blueprint §21-25: Graphify,
  code-review-graph, ast-grep): nenhuma dessas ferramentas existe no Brian
  ainda. "Busca simbólica" nesta change é `grep` por palavras-chave da
  tarefa — aproximação honesta, não um grafo de código.
- **Sem medição de custo em USD**: Brian não calcula custo de run
  (`Brian-Cost-USD: unknown`, non-goal já registrado em
  `run-fast-path-gates`). A métrica primária do blueprint (§18.3, "custo
  total em USD por change bem-sucedida") **não é mensurável** nesta
  implementação — usa-se **duração (tempo de parede)** como proxy
  mensurável, com essa limitação declarada explicitamente em todo relatório
  produzido, nunca escondida.
- **População de 10, não 30**: decisão consciente do autor — sem histórico
  real de 30 changes, a alternativa seria inventar 30 tarefas sintéticas
  (mais ruído, não mais sinal) ou não rodar o experimento. Com N=10 por
  braço, o critério estatístico do blueprint (diferença de 30 pontos
  detectável) fica mais fraco — declarado no relatório, não escondido.
- **Sem Curated de produção**: mesmo que H-1 seja confirmada, integrar
  `ContextPackage::Curated` ao caminho normal de `brian run`/`brian
  workflow run` é uma decisão e um trabalho separados, fora desta change
  (que só existe para produzir o dado que decide se vale a pena).

## Conformidade — checklist §16

- **M1-M6 / OP-1..OP-8**: atende OP-2 (tuning contínuo com rigor: não
  assumir, medir) e a disciplina de honestidade já estabelecida em todo o
  projeto (n sempre visível, "unknown" nunca fabricado).
- **D-16/D-17**: não toca ledger nem Continuity Pack diretamente — cada
  execução é um run normal.
- **D-5**: esta change é exatamente o que D-5 pede — nada no caminho
  padrão depende do resultado; `ContextPackage::Repository` continua sendo
  tudo que `brian run` usa fora deste experimento.
- **D-10**: não viola — cada execução é um provider fazendo um run.
- **H-1**: esta change *é* o teste de H-1, não uma dependência dela.
- **Versão alvo**: Milestone 3 nominalmente (blueprint §100), implementada
  agora fora da ordem original por decisão do autor.
