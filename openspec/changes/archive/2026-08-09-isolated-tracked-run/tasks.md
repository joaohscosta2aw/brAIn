## 1. Esquema

- [x] 1.1 Migração `0005_execucao.sql`: `run` (id, client_id, project, base_commit,
      worktree_path, branch, provider_id, pid, status, custo_equivalente_micros,
      started_at, finished_at).
- [x] 1.2 Migração: `run_event` (id, run_id, tipo, detalhe, ocorrido_em).
- [x] 1.3 Confirmar migração puramente aditiva: `cargo test` das changes
      anteriores continua passando sem alteração.

## 2. Tipos de domínio

- [x] 2.1 `StatusRun` (`EmExecucao`/`Concluido`/`Falhou`/`Abandonado`) em
      `src/domain.rs`.
- [x] 2.2 `RunRegistrado` (id, client_id, project, base_commit, worktree_path,
      branch, provider_id, pid opcional, status, custo equivalente opcional,
      started_at, finished_at opcional).
- [x] 2.3 `EventoDeRun` (run_id, tipo, detalhe, ocorrido_em).

## 3. Store: run + eventos

- [x] 3.1 Extensão do `Store` trait: `criar_run` (grava **antes** de qualquer
      efeito colateral — task 4.2 depende de a ordem estar certa), `run`,
      `atualizar_status_run`, `registrar_evento_run`, `eventos_do_run`,
      `runs_em_execucao` (para `recover` varrer).
- [x] 3.2 Testes cobrindo CRUD básico e a consulta `runs_em_execucao`.

## 4. Ciclo de vida do worktree e execução

- [x] 4.1 `src/execucao.rs`: `criar_worktree(repo, base_commit, run_id) ->
      PathBuf` — `git worktree add ~/.brian/worktrees/run_<id> -b
      brian/run_<id> <base_commit>`.
- [x] 4.2 `iniciar_run(store, contexto, provider, tarefa) -> RunRegistrado` —
      resolve commit base (HEAD do repo do context), cria registro do run **antes**
      de criar o worktree ou invocar o provider (D-12, spec: "Run persistido antes
      de qualquer efeito colateral").
- [x] 4.3 Falha ao criar o worktree ou invocar o provider marca o run como falho,
      nunca deixa um registro sem status (spec: "Falha ao invocar o provider ainda
      deixa rastro").
- [x] 4.4 `codex exec -s workspace-write -C <worktree> <tarefa>` — único provider
      executável nesta change (design.md); outros providers recusam com erro
      explícito ("execução não verificada como segura para este provider").
- [x] 4.5 PID do processo do provider gravado no registro do run assim que
      criado.
- [x] 4.6 Run bem-sucedido/falho atualiza status e `finished_at`; branch/worktree
      preservados em ambos os casos (spec: "Worktree nunca é removido em
      silêncio").
- [x] 4.7 Testes: worktree isolado criado; árvore principal nunca tocada; dois
      runs simultâneos não colidem (worktrees/branches distintos, verificado com
      diretórios de teste reais); run persistido antes do processo do provider
      existir.

## 5. Recuperação de órfão

- [x] 5.1 `src/execucao.rs`: `processo_vivo(pid) -> bool` via `kill -0` (nenhuma
      dependência nova — POSIX padrão).
- [x] 5.2 `runs_orfaos(store) -> Vec<RunRegistrado>` — runs em execução cujo PID
      não está mais vivo.
- [x] 5.3 `recuperar(store, run_id)` — marca o run como abandonado, preserva o
      worktree, **nunca** invoca o provider de novo (spec: "Finalização de órfão
      nunca duplica custo").
- [x] 5.4 Testes: processo morto detectado como órfão; processo vivo não
      detectado; recuperação não cria novo processo; consumo já registrado
      permanece intacto após recuperação.

## 6. Trilha de proveniência

- [x] 6.1 `src/execucao.rs`: detectar se o provider criou um commit novo
      (`git log -1 --format=%H` antes/depois).
- [x] 6.2 Quando há commit novo: `git commit --amend` acrescentando trailers
      `Brian-Run`/`Brian-Client`/`Brian-Context`/`Brian-Provider`/`Brian-Model`/
      `Brian-Cost-USD` à mensagem existente, sem alterar o conteúdo do commit.
- [x] 6.3 Sem commit novo: nenhum commit é forjado (spec: "Run sem alteração não
      força commit vazio").
- [x] 6.4 Testes com repositório Git real (mesmo padrão de fixture de
      `continuidade.rs`): commit com trailers; ausência de commit não gera um
      vazio; trailers legíveis só com `git log`, sem o banco do Brian.

## 7. Superfície CLI

- [x] 7.1 `brian run "<tarefa>" --provider <id>`.
- [x] 7.2 `brian recover` — lista órfãos e finaliza (confirmação do operador por
      run, ou `--all`).
- [x] 7.3 `brian worktree list` — worktrees ativos/abandonados com status do run
      associado.

## 8. Verificação

- [x] 8.1 Cobertura de cada cenário dos três specs desta change (auditoria
      manual, mesmo processo das changes anteriores).
- [x] 8.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 8.3 Teste manual supervisionado com `codex` real, num repositório de teste
      descartável (não este repositório) — confirmar worktree isolado, commit com
      trailers, e `brian recover` contra um processo morto de propósito
      (`kill -9`).
- [x] 8.4 `openspec validate --strict` limpo antes de considerar a change pronta
      para archive.
