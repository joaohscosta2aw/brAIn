## Context

`src/execucao.rs::iniciar_run` (change `isolated-tracked-run`, já arquivada) já
decide `status_final` a partir só do resultado de `executar_provider`, aplica
trailers de proveniência quando esse status é `Concluido`, e persiste tudo via
`Store`. Este design encaixa um passo de gate nesse fluxo existente sem alterar
sua estrutura de worktree/persistência (D-7/D-12, já resolvidos).

Ver proposal.md para a motivação (OP-5/OP-8) e a lista de não-objetivos (sem YAML
de workflow, sem retry automático, sem security gates).

## Goals / Non-Goals

**Goals:**
- Gate determinístico opcional, configurável por invocação (`--gate "<comando>"`),
  decide se um run com provider bem-sucedido é `Concluido` ou `Falhou`.
- Zero regressão para quem não usa `--gate`.

**Non-Goals:**
- Não introduz `workflows/*.yaml`, fases, `role`, `model_pointer` (§15.3 do
  blueprint) — isso é v0.3+, quando houver mais de um passo de fato para
  orquestrar.
- Não faz retry/fix automático do provider quando o gate falha.
- Não valida nem sanitiza o comando do gate além do que o shell já faz — mesmo
  nível de confiança que `tarefa` (string do operador, já passada para o
  provider).

## Decisions

**Gate é um comando shell único, não uma lista de "gates" nomeados.**
`Command::new("sh").arg("-c").arg(<comando>)`, executado com `current_dir` no
worktree do run, mesmo padrão de `executar_provider`/`git()` em `execucao.rs`.
Exit code 0 = passa; qualquer outro = falha. Alternativa considerada: gates
nomeados fixos (`tests`, `lint`) como no blueprint (§15.3) — rejeitada porque
exigiria um registro de comandos por projeto/linguagem que esta change não
precisa; um comando shell livre cobre `cargo test`, `npm test`, `make check`, etc.
sem esse registro.

**Gate roda só se o provider já tiver sucesso.** Provider falho não chega a rodar
o gate (spec deterministic-gate: "Provider já falho não chega a rodar o gate") —
evita gastar tempo validando um worktree que o provider nem terminou de mexer, e
mantém uma causa de falha só (a do provider, já registrada).

**Trailers de proveniência continuam amarrados só ao resultado do provider, não
ao gate.** `aplicar_trailers_se_houver_commit_novo` já roda logo após
`executar_provider` retornar sucesso, antes do gate ser decidido. Isso não muda:
um commit real que o provider produziu carrega os trailers mesmo que o gate
reprove depois — é mais honesto para quem for inspecionar o worktree depois de
uma falha ("de quem é esse commit" continua respondível só com `git log`,
independente do gate ter passado). Alternativa considerada: só aplicar trailers
quando `status_final` (pós-gate) for `Concluido` — rejeitada por enfraquecer a
trilha de auditoria exatamente no caso (falha) em que mais se quer saber quem fez
o quê.

**Saída do gate é resumida do mesmo jeito que `resumo_stderr` do provider** (3
primeiras linhas de stderr, ou de stdout+stderr combinados se o gate não separar)
— reaproveita o padrão já existente, sem novo tipo de resultado.

## Risks / Trade-offs

- **`sh -c` com string do operador** → mesmo nível de confiança que `tarefa`
  (já passada ao provider); não é superfície nova de risco.
- **Gate lento trava `brian run` mais tempo** (execução síncrona, mesmo modelo do
  provider) → aceito; sem timeout nesta change, mesma ausência de timeout que
  `executar_provider` já tem hoje.
- **Comando de gate divergente entre runs do mesmo projeto** (typo, versão
  diferente) → aceito; é responsabilidade do operador até existir workflow em
  dado (v0.3+) que fixe isso por projeto.
