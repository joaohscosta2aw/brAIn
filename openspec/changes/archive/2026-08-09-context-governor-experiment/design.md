## Context

`execucao::iniciar_run` já executa uma tarefa (string) isolada por
worktree. O experimento de H-1 não precisa de nenhuma mudança nesse motor
— só precisa gerar strings de tarefa diferentes por braço e agrupar os
resultados. Ver proposal.md para motivação (blueprint §18) e não-objetivos
(sem grafo de código real, sem medição de custo, N=10 não 30).

## Goals / Non-Goals

**Goals:**
- Pacote curado honesto (só fontes que existem: grep, git diff, memória).
- Três braços comparáveis, resultado com `n` e limitações sempre visíveis.

**Non-Goals:** ver proposal.md.

## Decisions

**"Busca simbólica" = `grep -rn` por palavras-chave extraídas da tarefa.**
Tokeniza a descrição da tarefa (palavras com 4+ letras, sem stopwords
comuns em português/inglês), roda `grep -rln <palavra> <repo>` para cada
uma, deduplica os arquivos encontrados, corta em um orçamento de
caracteres fixo (proxy simples de "orçamento de tokens" — Brian não tem
tokenizer real). Alternativa considerada (embeddings/busca semântica):
rejeitada — exigiria infraestrutura nova (índice vetorial) para uma
capability cujo próprio propósito é decidir se vale a pena existir.

**"Diff recente" = `git log -3 -p` no repo do case, antes da tarefa
rodar.** Aproximação de "o que mudou recentemente que pode ser relevante"
— não é o diff da própria tarefa (que ainda não aconteceu).

**"Memória" reaproveita `continuidade::notas_do_contexto` — sem
subsistema novo.** Notas do `client_id`/`project` ativo (o experimento usa
um contexto sintético dedicado, mesma disciplina de `evaluation/eval-
harness`: `client_id = "h1-experiment"`).

**Formatação por braço:**
```text
A: "<tarefa>"
B: "<tarefa>\n\nContexto (use SOMENTE isto, não explore o resto do
    repositório):\n<pacote>"
C: "<tarefa>\n\nContexto (ponto de partida — explore mais se
    precisar):\n<pacote>"
```

**Nova tabela `experimento_execucao`** (case_id, braço, run_id,
started_at) — liga cada execução real a um `run` já existente, mesmo
padrão de `comparacao_candidato`/`workflow_phase_entry`.

**`brian experiment report-h1` reaproveita a fórmula de
`router::calcular_scores`/`melhor_por_score`**, mas agrupando por braço
(A/B/C) em vez de provider — mesma disciplina de honestidade (`n` sempre
visível, duração `None` quando ausente).

**Relatório sempre inclui a nota de limitação de métrica primária.**
Hardcoded no formatador, não uma opção — spec: "Relatório nunca esconde
que custo em USD não é medido" é uma garantia, não um comportamento
configurável que alguém possa desligar.

**10 tarefas sintéticas em `experiments/h1-tasks.json`**, cada uma com
`tipo` (`bug_pequeno`/`feature_media`/`refactor`, vocabulário do blueprint
§18.3) e um `fixture_repo` (caminho de repositório de teste a ser
preparado manualmente pelo operador antes de rodar — esta change não gera
repositórios fixture automaticamente, non-goal implícito: preparar 10
fixtures realistas é trabalho manual, não código).

## Risks / Trade-offs

- **Duração como proxy de custo é imperfeita** (uma tarefa pode ser rápida
  e cara, ou lenta e barata) — aceito e declarado explicitamente em todo
  relatório, nunca escondido.
- **N=10 é pequeno demais para o critério estatístico do blueprint** (§18.3
  pede detectar diferença de 30 pontos com confiança — N=10 por braço não
  sustenta isso com rigor) — aceito conscientemente; o relatório declara
  `n` sempre, permitindo ao operador julgar a força da evidência.
- **`grep` por palavra-chave é uma aproximação grosseira de busca
  relevante** (falsos positivos/negativos comuns) — aceito, é exatamente o
  tipo de limitação que o experimento existe para revelar, não para
  esconder.
