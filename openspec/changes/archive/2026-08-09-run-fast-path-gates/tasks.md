## 1. Gate no ciclo de vida do run

- [x] 1.1 `src/execucao.rs`: função `rodar_gate(worktree, comando) -> ResultadoGate`
      (`sh -c <comando>` com `current_dir` no worktree, mesmo padrão de
      `executar_provider`) — sucesso/falha + resumo de saída (3 primeiras linhas).
- [x] 1.2 `iniciar_run` ganha parâmetro `gate: Option<&str>`. Quando `Some` e o
      provider já teve sucesso: roda o gate antes de decidir `status_final`
      (spec deterministic-gate: "Provider e gate bem-sucedidos concluem o run").
- [x] 1.3 Gate reprovado com provider verde: `status_final = Falhou`, motivo =
      saída do gate (spec: "Gate reprovado marca o run como falho mesmo com
      provider verde").
- [x] 1.4 Provider já falho: gate não roda (spec: "Provider já falho não chega a
      rodar o gate").
- [x] 1.5 Sem `gate` (`None`): comportamento idêntico ao pré-existente — status
      decidido só pelo provider (spec: "Ausência de gate preserva o
      comportamento anterior").
- [x] 1.6 Trailers de proveniência continuam aplicados a partir do sucesso do
      provider, não do resultado do gate (design.md, decisão já tomada — não
      reabrir).
- [x] 1.7 Novo evento `gate.run` registrado (sucesso/falha) via
      `registrar_evento_run`, mesmo padrão de `provider.execute`/
      `provider.finished`.
- [x] 1.8 Gate reprovado nunca invoca o provider de novo — run finaliza, uma
      nova tentativa exige novo `brian run` explícito (spec: "Gate reprovado
      nunca reexecuta o provider automaticamente").

## 2. Testes

- [x] 2.1 `rodar_gate`: comando que passa (exit 0) e comando que falha (exit
      != 0), com repositório de teste real (mesmo padrão de fixture já usado em
      `execucao.rs`).
- [x] 2.2 `iniciar_run` com gate configurado e provider real não é testável sem
      `codex` — cobrir a lógica de decisão isoladamente: extrair a combinação
      "resultado do provider + resultado do gate → status final" numa função
      pura testável sem processo nenhum.
- [x] 2.3 Teste: gate ausente preserva status decidido só pelo provider.
- [x] 2.4 Teste: gate reprovado nunca aciona um novo processo de provider
      (verificação estrutural do código, não só comportamental — ex.: nenhuma
      chamada a `executar_provider` depois da falha do gate).

## 3. Superfície CLI

- [x] 3.1 `brian run "<tarefa>" --provider <id> [--gate "<comando>"]` —
      `src/comandos.rs`, `src/main.rs`, mesmo padrão de `--model` já existente.

## 4. Verificação

- [x] 4.1 Cobertura de cada cenário dos dois specs desta change (auditoria
      manual, mesmo processo das changes anteriores).
- [x] 4.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 4.3 Teste manual supervisionado com `codex` real, num repositório de teste
      descartável (não este repositório): um run com `--gate "true"` (passa) e
      um run com `--gate "false"` (reprova) — confirmar status final, evento
      `gate.run`, e que o commit do provider (quando existir) carrega trailers
      mesmo no caso reprovado.
- [x] 4.4 `openspec validate --strict` limpo antes de considerar a change pronta
      para archive.
