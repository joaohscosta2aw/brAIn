## Why

`routing/historical-scoring` decide entre providers a partir de histórico —
mas com `n` pequeno (que é o caso real hoje), estatística não é o mecanismo
certo (blueprint §38.1/§38.4). O blueprint propõe comparação pareada
explícita como "o mecanismo útil enquanto `n` é pequeno... útil desde o
primeiro dia": rodar a mesma tarefa em mais de um provider, mostrar os
resultados lado a lado, e deixar o operador escolher — a escolha em si vira
dado de qualidade melhor que observação passiva.

## What Changes

- `brian run "<tarefa>" --compare <provider1>,<provider2>,...` roda a mesma
  tarefa em cada provider da lista, cada um em seu próprio worktree isolado
  (D-7, reaproveita `execucao::iniciar_run` sem alteração), sequencialmente.
- Cada provider da lista SHALL passar pela mesma validação de
  `PROVIDERS_EXECUCAO_VERIFICADA` que qualquer run — comparar com um
  provider não verificado falha com erro explícito nomeando esse provider,
  não silenciosamente pula ou finge suporte.
- Resultado apresentado lado a lado: provider, status, worktree, resumo do
  gate (quando `--gate` também for passado).
- `brian compare choose <comparacao_id> --winner <provider_id>` registra a
  escolha do operador — nunca automática, sempre uma ação explícita
  separada da execução.

## Capabilities

### New Capabilities
- `evaluation/paired-comparison`: roda a mesma tarefa em múltiplos
  providers e registra a escolha humana entre os resultados.

## Impact

- `src/storage/migrations/0007_comparacao.sql` (novo): `comparacao`,
  `comparacao_candidato` (liga cada candidato ao `run` real que o executou).
- `src/comparacao.rs` (novo): orquestra N chamadas de
  `execucao::iniciar_run` (uma por provider), persiste a comparação e seus
  candidatos.
- `src/comandos.rs`, `src/main.rs`: `--compare` em `brian run`;
  `brian compare choose`.
- Sem mudança em `execucao.rs` — cada candidato é um run normal.

## Não-objetivos

- **Sem execução paralela real dos candidatos**: hoje só `codex` tem
  execução verificada — não há dois providers reais pra rodar em paralelo
  ainda, então paralelizar seria otimização sem beneficiário. Candidatos
  rodam sequencialmente; paralelizar fica para quando houver 2+ providers
  verificados de verdade e o tempo de espera se tornar um problema real.
- **Sem diff visual lado a lado** (blueprint menciona "os dois diffs"):
  Brian não tem UI (D-2, CLI first) — a saída é texto, listando status e
  worktree de cada candidato; o operador inspeciona os diffs reais nos
  worktrees preservados (mesma disciplina de "worktree nunca é removido em
  silêncio").
- **Sem uso automático da escolha no scoring** (`routing/historical-scoring`
  já lê `run.status`, não vencedor de comparação): a escolha registrada
  aqui é dado para consulta futura, não é consumida automaticamente por
  nenhum mecanismo desta change — ligar isso ao scoring é uma decisão
  separada, fora de escopo.

## Conformidade — checklist §16

- **M1-M6 / OP-1..OP-8**: atende OP-2/OP-4 diretamente — dado de escolha
  humana é evidência de qualidade melhor que histórico passivo, blueprint
  explícito sobre isso.
- **D-16/D-17**: não toca ledger nem Continuity Pack diretamente — cada
  candidato é um run normal, já entra no ledger como qualquer outro.
- **D-10**: não viola — compara resultados de um mesmo run lógico entre
  providers, não orquestra inner loop de nenhum deles.
- **H-1**: não depende do Context Governor.
- **Versão alvo**: v0.3 nominalmente (blueprint §84), implementada agora
  fora da ordem original por decisão do autor.
