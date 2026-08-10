# Brian — Decisões travadas (D-1 … D-17)

> Resumo executável. Detalhe e critério longo: `BRIAN-BLUEPRINT-V1.md`.  
> Premissas de produto: `docs/PREMISSAS-BASICAS.md`.

**Convenção:** decisão travada **para de ser rediscutida** até o critério de reversão.

| ID | Decisão | Reversão |
|----|---------|----------|
| **D-1** | SQLite como banco operacional único, atrás de traits. | Query real de produção > 200 ms com 12 meses de dados. |
| **D-2** | Brian Core em Rust. CLI first. SwiftUI só no v0.3. | Só se CLI provar que o produto não se sustenta sem UI. |
| **D-3** | Workflow é YAML versionado; default curto (fast/direct). | 3 workflows distintos exigirem lógica que YAML não expressa. |
| **D-4** | Adapters: headless JSON → session files → PTY (PTY nunca 1ª escolha). | Provider relevante sem as duas primeiras opções por 2 releases. |
| **D-5** | Context Governor é hipótese (H-1), não pilar. Nada depende dele. | H-1 confirma ≥30% redução de $/change bem-sucedido. |
| **D-6** | Custo reportado pelo provider > price catalog (catalog rotulado). | Nenhum. |
| **D-7** | Todo run orquestrado em git worktree dedicado. | Nenhum (pré-requisito de concorrência). |
| **D-8** | ~~Router por regras + override até n≥30 por célula; depois evals.~~ **[Δ 2026-08-09]** Revertida antecipadamente por decisão consciente do autor — scoring histórico entra no código antes de haver `n≥30` reais por célula. Risco aceito e assumido: "confiança injustificada" nas próprias palavras do blueprint §11.1 até o volume real de uso chegar lá. | ~~Limiar + harness de eval funcionando.~~ Reversão consumida. |
| **D-9** | SQL só em `storage/` via traits. | Nenhum. |
| **D-10** | Brian não orquestra inner loop de um provider. Só N providers / M clientes. | Nenhum (definição de produto). |
| **D-11** | Runtime = “Brian Core”. Termo “Brain” proibido. | Nenhum. |
| **D-12** | Persistir estado do run antes de qualquer efeito colateral externo. | Nenhum. |
| **D-13** | ~~Roteamento adaptativo só depois do harness de eval.~~ **[Δ 2026-08-09]** Revertida antecipadamente junto com D-8 — mesma decisão consciente, mesmo risco assumido. | Reversão consumida. |
| **D-14** | Memória append-only com proveniência; correção supersede. | Nenhum. |
| **D-15** | MCP do Brian = único path de escrita de workflow *por agentes*. Core é writer. | Nenhum. |
| **D-16** | **Zero token perdido.** Usage Control no minuto 0: ledger, janelas, %, restante, burn, assinatura, atribuição, otimização de centavo. | Nenhum. Lei do produto. |
| **D-17** | **Continuidade multi-LLM.** Continuity Pack + memória do Brian; handoff sem reexplicar. | Nenhum. Lei do produto (mínimo no v0.1). |

### Nota — reversão antecipada de D-8/D-13 (2026-08-09)

O autor pediu explicitamente, ciente do trade-off, que Router Fase 2/3 e Learning
Engine entrassem em código antes de existir `n≥30` runs reais por célula —
confirmado duas vezes após eu apontar que isso contraria a trava original.
Consequência prática: qualquer scoring histórico/aprendizado que existir no
código a partir daqui não tem base estatística real ainda. Tratar como
"aposta instrumentada", não como fato — o próprio blueprint (§11.1) chama isso
de "confiança injustificada" enquanto o volume de uso real não chega lá.

## Hipóteses (não são leis)

| ID | Hipótese | Critério |
|----|----------|----------|
| **H-1** | Context Governor reduz custo total por change bem-sucedido ≥30%. | Experimento A/B/C; senão remove o módulo. |

### Nota — resultado do experimento H-1 (2026-08-09)

Rodado via `context-governor-experiment`: 9 tarefas sintéticas × 3 braços =
27 execuções reais de `codex` (`experiments/h1-tasks.json`, população
reduzida de 30 para 9 por decisão consciente já registrada na proposta da
change; N pequeno, resultado direcional, não conclusivo).

```text
braço A (baseline, sem pacote):                 taxa 100% (n=9)
braço B (pacote curado, uso exclusivo):         taxa  67% (n=9, 3 falhas de gate reais)
braço C (pacote curado, ponto de partida):      taxa 100% (n=9)
```

Custo em USD não foi medido (não-objetivo declarado); duração também não
ficou disponível como proxy nesta rodada (limitação pré-existente de
`execucao::iniciar_run`: `finished_at` usa o mesmo `Instante` passado para
`started_at`, não um timestamp reamostrado após a execução real — afeta
todo o sistema, não só o experimento, e fica fora do escopo desta nota).

**Leitura honesta, não uma conclusão de H-1:** as 3 falhas do braço B foram
todas falhas reais do gate (`cargo test` reprovado no worktree, não erro de
infraestrutura) — o braço que *obriga* o agente a usar só o pacote curado
(busca por grep + diff + memória) saiu pior que o baseline e que o híbrido
nesta amostra pequena. Isso é consistente com um dos riscos que o próprio
blueprint (§18.2) já listava como razão de H-1 poder estar errada: busca
agêntica livre pode superar retrieval pré-computado quando o retrieval é
uma aproximação grosseira (aqui, `grep` por palavra-chave, não um grafo de
código real). Com N=9 por braço, isso não é evidência estatística
suficiente para confirmar nem descartar H-1 — é sinal, não veredito. As 10
tarefas de `experiments/h1-tasks.json` já foram todas usadas (a 10ª,
h1-01, rodou como piloto manual antes desta rodada); aumentar a amostra
exige tarefas novas, não reexecutar as mesmas.

## Ordem sagrada de implementação

```text
D-16 (v0.0) → D-17 mínimo (v0.1) → run/worktree (v0.2) → resto
```

## Princípios operacionais (OP-1…OP-8) — ver `PREMISSAS-BASICAS.md`

| ID | Princípio |
|----|-----------|
| **OP-1** | Eficiência na utilização |
| **OP-2** | Tuning contínuo |
| **OP-3** | Tecnologia sempre atualizada |
| **OP-4** | Inteligência retroalimentada |
| **OP-5** | IA para resolver |
| **OP-6** | Direcionamento claro |
| **OP-7** | Trabalhos assertivos |
| **OP-8** | Poucas indas e vindas |

OpenSpec e código que violem OP-1…OP-8 ou D-16/D-17 são rejeitados por premissa.

## Nota — grafo de código real (§21-25) fica fora do Brian (2026-08-09)

O blueprint (§21-25) descreve um grafo de código real como fonte para
contexto agêntico. Esse grafo já existe neste ambiente — MCP
`code-review-graph`/skill Graphify, já documentado em `AGENTS.md` como
ferramenta disponível ao assistente — mas é uma ferramenta do *harness do
assistente*, não algo que o binário `brian` (Rust, CLI standalone) chama
programaticamente. Fazer Brian consumir isso exigiria Brian virar cliente
MCP, uma capability nova e não trivial, não uma integração pequena.

Decisão: não duplicar essa skill dentro do Brian nem fingir integração
que não existe. `context_governor.rs` (H-1) continua declarando
explicitamente que só usa `grep`/`git log`/memória, nunca um grafo de
código — isso permanece verdade. Se no futuro fizer sentido Brian
consumir o grafo real (ex.: para o Context Governor deixar de ser
aproximação grosseira), isso é uma proposta separada e maior, não um
item desta lista.
