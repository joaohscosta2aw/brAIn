## Why

`isolated-tracked-run` (v0.2) marca um run como `Concluido` só a partir do exit code
do processo do provider. Isso viola OP-5 ("gates determinísticos como juiz barato")
e OP-8 ("default 1 fase de trabalho + gates"): o provider pode sair com código 0
tendo quebrado os testes, e o Brian registraria sucesso mesmo assim — sem "juiz"
nenhum além da autoavaliação do próprio provider. Fechar esse buraco é o que falta
de v0.2 (§9 do PREMISSAS-BASICAS.md: "Run + worktree + workflow curto + handoff no
run" — worktree/run/handoff já existem, workflow curto ainda não).

## What Changes

- Depois que o provider termina no worktree (`iniciar_run`, `src/execucao.rs`), o
  sistema roda um **gate determinístico** configurável (comando shell, ex.:
  `cargo test`) dentro do worktree.
- Um run só é marcado `Concluido` se o provider **e** o gate passarem. Provider
  verde + gate vermelho → `Falhou`, com a saída do gate registrada como motivo
  (mesmo tratamento de falha que já existe para o provider).
- Gate ausente/não configurado → comportamento atual preservado (status decidido só
  pelo provider) — não é regressão para quem já usa `brian run` sem gate.
- Novo evento (`gate.run`) na trilha de eventos do run, mesmo padrão de
  `provider.execute`/`provider.finished`.
- CLI: `brian run "<tarefa>" --provider <id> [--gate "<comando>"]`.

## Capabilities

### New Capabilities
- `execution/deterministic-gate`: gate determinístico pós-provider decide o status
  final do run — provider sozinho nunca é o juiz.

### Modified Capabilities
- `execution/isolated-run`: o requirement "Run bem-sucedido registra o resultado"
  passa a exigir também o gate, quando configurado.

## Impact

- `src/execucao.rs`: `iniciar_run` ganha um passo de gate após `executar_provider`,
  antes de decidir `status_final`.
- `src/comandos.rs`, `src/main.rs`: novo argumento `--gate` em `brian run`.
- Sem migração de schema nova — reaproveita `run_event` já existente
  (`isolated-tracked-run`).

## Não-objetivos (v0.2, não v0.3)

- **Sem workflow em YAML** (`fast.yaml`/`governed.yaml`, §15.3 do blueprint): esta
  change não introduz fases, `role`, `model_pointer`, nem transições declarativas —
  só um gate único pós-run. Workflow como dado versionado fica para quando houver
  mais de uma fase de fato para orquestrar.
- **Sem retry/fix automático**: gate vermelho marca o run como falho e para — não
  invoca o provider de novo para corrigir (evita gastar de novo sem decisão
  explícita, mesma disciplina de "nunca duplica custo" de `orphan-recovery`).
- **Sem security gates** (semgrep/osv/secrets) nem `requires_approval`: o operador
  escolhe o comando do gate; nenhuma varredura é embutida nesta change.
- **Sem seleção automática de workflow por risco/path**: não há classificador de
  risco nem `policy_set` — fora de escopo até existir mais de um workflow.

## Conformidade — checklist §16

- **M1-M6 / OP-1..OP-8**: atende diretamente OP-5 (gate como juiz barato) e OP-8
  (1 fase + gate, sem ida-e-volta automática). Não viola nenhum outro princípio.
- **D-16/D-17**: não toca ledger nem Continuity Pack — só o ciclo de vida do run
  (D-12, já coberto por `isolated-tracked-run`).
- **D-10**: gate roda depois que o provider termina seu trabalho — não orquestra o
  inner loop do provider, só julga o resultado (outer loop).
- **H-1**: não depende do Context Governor.
- **Versão alvo**: v0.2 (fecha o item "workflow curto" que ficou pendente).
