## Context

`execucao::iniciar_run` já executa uma tarefa isolada por worktree (D-7).
Uma comparação é, mecanicamente, N chamadas independentes a essa mesma
função — nada nela precisa saber que está sendo comparada. Ver proposal.md
para motivação (blueprint §38.4) e não-objetivos (sem paralelismo real, sem
UI de diff, sem consumo automático no scoring).

## Goals / Non-Goals

**Goals:**
- `--compare` roda N candidatos, cada um um run real e isolado.
- Vencedor só registrado por ação explícita, nunca automática.

**Non-Goals:** ver proposal.md.

## Decisions

**Duas tabelas novas: `comparacao` e `comparacao_candidato`.**
`comparacao` (id, client_id, project, tarefa, started_at, vencedor_provider_id
nullable). `comparacao_candidato` (comparacao_id, provider_id, run_id) —
liga cada provider comparado ao `run` real que o executou, mesmo padrão de
`workflow_phase_entry` ligando fase a run.

**Candidatos rodam sequencialmente, cada um via `execucao::iniciar_run`
inalterado.** Provider inválido é detectado ANTES de rodar qualquer
candidato (valida a lista inteira contra `PROVIDERS_EXECUCAO_VERIFICADA`
primeiro) — spec: "Candidato inválido falha a comparação inteira, sem pular
silenciosamente". Alternativa considerada (validar um por um, rodando os
válidos e pulando o resto): rejeitada por poder produzir uma "comparação"
enganosa com menos candidatos do que o operador pediu, sem ele perceber.

**`comparacao` persistida antes de qualquer candidato rodar (D-12)** — mesma
disciplina de `run`/`workflow_run`.

**`brian compare choose` é o único jeito de registrar vencedor** — nenhum
código nesta change decide automaticamente, mesmo quando um candidato falha
e o outro conclui (o operador ainda decide, o Brian não assume "o que
funcionou venceu" — pode haver critério de qualidade além de status).

## Risks / Trade-offs

- **Só um provider verificado hoje** → comparação real (2+ candidatos
  válidos) não é possível ainda; o mecanismo existe e funciona
  estruturalmente, mas só produz valor prático quando mais providers
  ganharem execução verificada — mesma situação já aceita em
  `model-router-pointers`/`provider-router-scoring`.
- **Custo dobra (ou N-plica) por comparação** — aceito conscientemente,
  blueprint já assume isso ("gera dado de qualidade muito superior").
