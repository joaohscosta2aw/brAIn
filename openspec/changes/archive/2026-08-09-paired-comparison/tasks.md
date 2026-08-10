## 1. Schema

- [x] 1.1 Migração `0007_comparacao.sql`: `comparacao` (id, client_id,
      project, tarefa, vencedor_provider_id nullable, started_at,
      finished_at).
- [x] 1.2 Migração: `comparacao_candidato` (id, comparacao_id, provider_id,
      run_id).
- [x] 1.3 Confirmar migração puramente aditiva: `cargo test` das changes
      anteriores continua passando sem alteração.

## 2. Domínio e Store

- [x] 2.1 `src/domain.rs`: `ComparacaoRegistrada`, `CandidatoComparacao`.
- [x] 2.2 `src/storage/mod.rs`/`sqlite.rs`: `criar_comparacao`,
      `comparacao`, `registrar_candidato_comparacao`,
      `candidatos_da_comparacao`, `definir_vencedor_comparacao` — mesmo
      padrão de `criar_workflow_run`/`registrar_entrada_fase`.
- [x] 2.3 Testes: CRUD básico; `definir_vencedor_comparacao` falha claro
      para comparação inexistente.

## 3. Execução da comparação

- [x] 3.1 `src/comparacao.rs` (novo): `rodar_comparacao(store, contexto,
      repo, cwd, providers: &[&str], tarefa, gate, agora) ->
      Result<ComparacaoRegistrada, String>`.
- [x] 3.2 Valida todos os `providers` contra
      `execucao::PROVIDERS_EXECUCAO_VERIFICADA` antes de rodar qualquer
      candidato — falha nomeando o(s) inválido(s), nada roda (spec:
      "Candidato inválido falha a comparação inteira, sem pular
      silenciosamente").
- [x] 3.3 Persiste `comparacao` antes de qualquer candidato rodar (D-12).
- [x] 3.4 Roda cada candidato sequencialmente via
      `execucao::iniciar_run` (sem alteração), registra
      `comparacao_candidato` ligando provider ao run real (spec: "Dois
      providers geram dois runs isolados").
- [x] 3.5 Testes: lista com provider inválido falha antes de qualquer run
      real (mesmo truque de `resolucao_de_commit_base_invalida...` —
      verificável sem `codex`); candidato válido único produz 1 run
      persistido ligado à comparação.

## 4. Escolha do vencedor

- [x] 4.1 `escolher_vencedor(store, comparacao_id, provider_id) ->
      Result<(), String>` — só grava `vencedor_provider_id`, nunca
      reexecuta nada (spec: "Escolha do vencedor é sempre uma ação
      explícita separada").
- [x] 4.2 Testes: escolher vencedor de comparação inexistente falha claro;
      escolher vencedor válido persiste corretamente; comparação recém-
      criada (sem escolha) não tem vencedor (spec: "Comparação termina sem
      vencedor definido").

## 5. Superfície CLI

- [x] 5.1 `brian run "<tarefa>" --compare <p1>,<p2>,...` (mutuamente
      exclusivo com `--provider`/`--model-pointer`/`--scored` — comparação
      decide o conjunto de providers explicitamente, não delega a
      regra/score) — `src/comandos.rs`, `src/main.rs`.
- [x] 5.2 `brian compare choose <comparacao_id> --winner <provider_id>`.
- [x] 5.3 Saída de `--compare`: lista cada candidato com provider, status,
      worktree.

## 6. Verificação

- [x] 6.1 Cobertura de cada cenário do spec desta change (auditoria manual,
      mesmo processo das changes anteriores).
- [x] 6.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 6.3 Teste manual: `brian run --compare codex --explain-only`... não
      há `--explain-only` para compare (non-goal); teste manual real
      supervisionado com `codex` real (único provider verificado hoje) —
      confirmar que 1 candidato válido roda e persiste corretamente, e que
      pedir `--compare codex,claude` falha nomeando `claude` sem rodar
      nada (isso sim sem custo).
- [x] 6.4 `openspec validate --strict` limpo antes de considerar a change
      pronta para archive.
