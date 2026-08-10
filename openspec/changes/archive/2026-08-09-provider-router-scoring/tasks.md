## 1. Consulta de histórico

- [x] 1.1 `src/storage/mod.rs`: novo método `runs_finalizados_do_cliente(client_id)
      -> Result<Vec<RunRegistrado>>` no trait `Store` — runs com status
      `Concluido` ou `Falhou` desse cliente, mesmo padrão de
      `runs_em_execucao`/`runs_abandonados`.
- [x] 1.2 `src/storage/sqlite.rs`: implementação — reaproveita
      `runs_por_status` existente com um `OR` de dois status, ou duas
      chamadas concatenadas (decidir na implementação pelo que for mais
      simples sem duplicar SQL).
- [x] 1.3 Testes: runs de dois clientes diferentes — consulta só retorna os
      do cliente pedido; runs em execução (não finalizados) não aparecem.

## 2. Cálculo de score

- [x] 2.1 `src/router.rs`: `Score { provider: String, taxa_sucesso: f64, n:
      u32, duracao_media_segundos: Option<f64> }`.
- [x] 2.2 `calcular_scores(runs: &[RunRegistrado], candidatos: &[&str]) ->
      Vec<Score>` — função pura, agrupa por provider, calcula taxa de
      sucesso e duração média; candidato sem nenhum run no histórico entra
      com `n=0`, `taxa_sucesso=0.0`, `duracao_media_segundos=None` (spec:
      "Provider sem histórico nenhum não é penalizado silenciosamente" —
      aparece na lista, não é omitido).
- [x] 2.3 `melhor_por_score(scores: &[Score]) -> &Score` — ranking
      `(taxa_sucesso desc, duracao_media asc, provider_id asc)` (design.md).
- [x] 2.4 Testes: taxa de sucesso decide entre dois candidatos com histórico
      distinto; empate de taxa desempata por duração; candidato sem
      histórico nenhum aparece no ranking com `n=0`, não excluído; cálculo
      usa só os runs do `client_id` pedido (spec: "Score é calculado sobre
      runs reais do cliente").

## 3. Integração com `brian run` e CLI de auditoria

- [x] 3.1 `src/comandos.rs`: `Comando::Run` ganha `#[arg(long)] scored:
      bool`. Sem essa flag, `resolver_provider` funciona exatamente como
      hoje (spec: "Sem --scored usa só regra") — zero mudança de
      comportamento pra quem não usa a flag.
- [x] 3.2 Com `--scored`: candidatos = providers aprovados pela regra Fase 1
      quando ela decidir mais de um (ou todos os `disponiveis`, se a regra
      não restringir) — calcula scores via `calcular_scores` sobre
      `runs_finalizados_do_cliente`, decide via `melhor_por_score`.
- [x] 3.3 `--explain-only` com `--scored` mostra `(provider, taxa_sucesso, n,
      duracao_media)` de cada candidato, não só o vencedor — spec:
      "Decisão por score nunca esconde o tamanho da base".
- [x] 3.4 Novo subcomando `brian router score --provider <id>` (ou lista
      todos os candidatos se `--provider` omitido) — mesma função de
      cálculo, sem decidir nada, só relatório (design.md).
- [x] 3.5 Testes: `--scored` sem histórico nenhum ainda decide (via `n=0`
      para todos, desempate por nome) sem travar nem exigir dado que não
      existe; `--scored` com histórico real prefere o provider com taxa
      maior.

## 4. Verificação

- [x] 4.1 Cobertura de cada cenário do spec desta change (auditoria manual,
      mesmo processo das changes anteriores).
- [x] 4.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 4.3 Teste manual: `brian router score` num contexto com runs reais
      já persistidos de sessões anteriores (não precisa gastar `codex` de
      novo, o histórico já existe no banco) — confirmar números plausíveis
      e `n` correto.
- [x] 4.4 `openspec validate --strict` limpo antes de considerar a change
      pronta para archive.
