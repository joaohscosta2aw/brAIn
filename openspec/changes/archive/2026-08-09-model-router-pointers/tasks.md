## 1. Dados e resolução

- [x] 1.1 `src/model_router.rs` (novo): structs `Pointer` (`primary`/
      `fallback`, cada um `{provider, tier}`) e `ModelosPorProvider` (mapa
      provider → tier → nome concreto, com `resolved_at`/`resolution_source`).
- [x] 1.2 `carregar_pointers(caminho) -> HashMap<String, Pointer>` e
      `carregar_modelos(dir) -> HashMap<String, ModelosPorProvider>` via
      `serde_json`, mesmo padrão de `router::carregar_regras`.
- [x] 1.3 `resolver_pointer(pointers, modelos, disponiveis: &[&str], nome) ->
      Result<(String, String, Option<Aviso>), ErroModelRouter>` — devolve
      `(provider, modelo_concreto, aviso_degradacao)`. `disponiveis` é a
      lista de providers com execução verificada (injetada, não hardcoded,
      pra ser testável sem depender de `execucao::PROVIDERS_EXECUCAO_VERIFICADA`
      direto na função pura).
- [x] 1.4 Ordem de resolução: `primary` disponível com tier configurado →
      usa; `primary` indisponível → tenta `fallback` (spec: "Primary
      indisponível usa o fallback"); nenhum dos dois disponível → erro
      nomeando o pointer (spec: "Fallback também indisponível recusa o run
      explicitamente").
- [x] 1.5 Tier ausente no provider resolvido → degrada para o tier mais
      próximo (`strong → balanced → cheap` ou inverso) com aviso; nenhum tier
      configurado nesse provider → mesmo erro de "fallback indisponível"
      (spec: "Tier ausente degrada com aviso, nunca em silêncio").
- [x] 1.6 Testes: primary resolve direto; primary indisponível cai pro
      fallback; nenhum dos dois disponível falha nomeando o pointer; tier
      ausente degrada com aviso; provider sem nenhum tier falha como
      "indisponível".

## 2. Integração com `brian run`

- [x] 2.1 `src/comandos.rs`: `Comando::Run` ganha `#[arg(long)]
      model_pointer: Option<String>`.
- [x] 2.2 `executar_run`: `--model` explícito sempre vence, nunca consulta
      pointer (spec: "Override explícito de modelo vence o ponteiro") —
      mesma estrutura de `resolver_provider` já existente para
      `routing/provider-rules`.
- [x] 2.3 Sem `--model` e sem `--model-pointer`: comportamento idêntico ao
      atual (nenhum modelo explícito, provider decide o próprio default) —
      zero regressão para quem não usa pointers.
- [x] 2.4 Aviso de degradação de tier vai para a mesma saída de erro/aviso já
      usada por `brian run` (stderr) — sem subsistema de trace novo
      (design.md).
- [x] 2.5 Testes: `--model-pointer` sem `--model` resolve via
      `model_router`; `--model` presente nunca chama `resolver_pointer`
      (mesmo padrão estrutural do teste equivalente em
      `provider-router-rules-minimal`).

## 3. Verificação

- [x] 3.1 Cobertura de cada cenário do spec desta change (auditoria manual,
      mesmo processo das changes anteriores).
- [x] 3.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 3.3 Teste manual: `models/pointers.json` + `providers/codex/models.json`
      reais, `brian run --model-pointer coding --explain-only` mostra a
      resolução correta; pointer cujo primary/fallback não incluem `codex`
      falha com erro explícito nomeando o pointer.
- [x] 3.4 `openspec validate --strict` limpo antes de considerar a change
      pronta para archive.
