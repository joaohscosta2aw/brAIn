## 1. Commit base explícito em `iniciar_run`

- [x] 1.1 `src/execucao.rs`: `PedidoRun` ganha campo `base_commit: Option<&str>`.
      `None` preserva o comportamento atual (`commit_atual(repo)` = `HEAD`).
- [x] 1.2 `Some(c)` usa `c` como base do worktree, sem chamar `commit_atual`.
- [x] 1.3 Testes: sem override usa `HEAD` (regressão dos testes existentes);
      com override usa o commit explícito, mesmo que `HEAD` tenha avançado
      desde então.

## 2. Definição e carregamento de casos

- [x] 2.1 `src/eval.rs` (novo): struct `CasoEval` (id, description, fixture
      repo path, base_commit, tarefa, provider, model opcional, gate) +
      `carregar_caso(caminho) -> CasoEval` via `serde_json`.
- [x] 2.2 `carregar_casos_do_diretorio(dir) -> Vec<CasoEval>` — lê todo `.json`
      do diretório (spec: "Caso de eval é dado, não código").
- [x] 2.3 Testes: caso válido carrega os campos corretos; JSON malformado
      falha com erro claro, não pânico.

## 3. Execução do caso e contexto sintético

- [x] 3.1 `src/eval.rs`: contexto sintético (`client_id = "eval"`,
      `project = Some(caso.id)`) — nunca o contexto ativo do operador (spec:
      "Runs de eval nunca são atribuídos a cliente real").
- [x] 3.2 `store.upsert_client("eval")` garantido antes de qualquer run de
      eval (FK de `run.client_id`).
- [x] 3.3 `rodar_caso(store, caso, agora) -> Vec<RunRegistrado>` — chama
      `execucao::iniciar_run` 3 vezes (N fixo, design.md) com o `base_commit`
      do caso; cada tentativa é um run real (spec: "Cada tentativa é um run
      rastreado normalmente").
- [x] 3.4 Testes: `rodar_caso` produz exatamente 3 runs persistidos, todos com
      `client_id = "eval"` e `project = Some(<id do caso>)`.

## 4. Classificação e relatório

- [x] 4.1 `taxa_de_sucesso(runs: &[RunRegistrado]) -> f64` — fração com
      `status == Concluido` (spec: "Tentativa passa só por critério
      determinístico").
- [x] 4.2 `classificar(taxa: f64) -> Veredito` (`Passou`/`Falhou`/`Instavel`) —
      `> 0.66` Passou, `< 0.34` Falhou, banda intermediária Instável (spec:
      "Taxa intermediária é marcada instável, não passa nem falha").
- [x] 4.3 Testes: `taxa_de_sucesso` com mistura de `Concluido`/`Falhou`;
      `classificar` nos limites exatos (0.34, 0.66) e fora deles.
- [x] 4.4 Formatação do relatório (caso, taxa, veredito) — texto simples, mesmo
      padrão de `brian costs`/`brian capacity`.

## 5. Superfície CLI

- [x] 5.1 `brian eval run [--case <id>] [--dir <caminho>]` (`--dir` padrão
      `evals/cases`) — `src/comandos.rs`, `src/main.rs`.

## 6. Verificação

- [x] 6.1 Cobertura de cada cenário dos dois specs desta change (auditoria
      manual, mesmo processo das changes anteriores).
- [x] 6.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 6.3 Teste manual supervisionado com `codex` real: um caso de eval
      completo contra um repositório de teste descartável (não este
      repositório) — confirmar 3 runs reais, taxa correta, veredito correto, e
      que os runs aparecem com `client_id = "eval"` (não vazam para nenhum
      cliente real).
- [x] 6.4 `openspec validate --strict` limpo antes de considerar a change
      pronta para archive.
