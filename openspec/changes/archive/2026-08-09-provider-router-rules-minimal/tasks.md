## 1. Regras e decisão

- [x] 1.1 `src/router.rs` (novo): struct `Regra` (`when.client`,
      `when.project`, `then.provider`, todos os campos de `when` opcionais) +
      `RegrasDeRoteamento` (lista de `Regra` + `default.provider`).
- [x] 1.2 `carregar_regras(caminho) -> RegrasDeRoteamento` via `serde_json`,
      mesmo padrão de `carregar_caso` em `eval.rs`. Sem arquivo: erro claro,
      não valor chumbado (design.md).
- [x] 1.3 `decidir_provider(regras, client_id, project) -> &str` — primeira
      regra cujo `when` casa (campo ausente = curinga) vence; nenhuma casando
      usa `default.provider` (spec: "Regra decide o provider quando não há
      override", "Nenhuma regra casa usa o default").
- [x] 1.4 Testes: primeira regra que casa decide, entre várias candidatas;
      regra com `when` parcial (só `client`, só `project`) casa como curinga
      no campo ausente; nenhuma regra casando usa `default`; arquivo sem
      `default` falha ao carregar.

## 2. Integração com `brian run`

- [x] 2.1 `src/comandos.rs`: `Comando::Run.provider` vira `Option<String>`.
- [x] 2.2 `executar_run`: quando `provider` é `None`, carrega regras (caminho
      padrão `routing/rules.json`) e chama `decidir_provider` com
      `client_id`/`project` do contexto ativo antes de chamar
      `execucao::iniciar_run` — override explícito nunca consulta regra
      nenhuma (spec: "Override explícito sempre vence a regra").
- [x] 2.3 Provider decidido (por regra ou explícito) continua validado só
      dentro de `execucao::iniciar_run` (`PROVIDERS_EXECUCAO_VERIFICADA`) —
      nenhuma validação duplicada em `router.rs` (spec: "Provider decidido por
      regra ainda precisa ser válido").
- [x] 2.4 Testes: `executar_run` sem `--provider` usa o resultado de
      `decidir_provider`; `executar_run` com `--provider` explícito nunca
      chama `decidir_provider`.

## 3. `--explain-only`

- [x] 3.1 `Comando::Run` ganha `#[arg(long)] explain_only: bool`.
- [x] 3.2 Com `--explain-only`: decide o provider (regra ou override) e
      formata a explicação (provider + regra responsável ou `"default"`), sem
      chamar `execucao::iniciar_run` (spec: "Explain-only mostra a decisão sem
      efeito colateral").
- [x] 3.3 Testes: `--explain-only` não persiste nenhum run nem cria worktree
      (verificação via `store.runs_em_execucao()` continuando vazio depois da
      chamada).

## 4. Verificação

- [x] 4.1 Cobertura de cada cenário do spec desta change (auditoria manual,
      mesmo processo das changes anteriores).
- [x] 4.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 4.3 Teste manual: `brian run --explain-only` com regras reais mostra a
      decisão correta; `brian run` sem `--provider` e sem regras configuradas
      falha com erro claro pedindo `--provider` ou arquivo de regras (não
      precisa de `codex` real — esta change não muda o caminho de execução em
      si, só a resolução do provider antes dele).
- [x] 4.4 `openspec validate --strict` limpo antes de considerar a change
      pronta para archive.
