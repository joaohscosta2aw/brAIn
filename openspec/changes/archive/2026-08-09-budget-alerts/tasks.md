## 1. Config e cálculo de status (sem I/O na lógica)

- [x] 1.1 `src/budget.rs` (novo): `BudgetCliente`, `carregar_budgets(caminho)`
      — navegação via `serde_json::Value`, sem derive (mesmo padrão de
      `router::carregar_regras`). Arquivo ausente → `Ok(HashMap::new())`.
- [x] 1.2 `StatusBudget`, `calcular_status(client_id, budget, gasto_usd,
      gasto_tokens)` — função pura: % de gasto, limiares de
      `alert_at_percent` cruzados, `limite_excedido`.
- [x] 1.3 Testes: arquivo ausente não é erro; JSON inválido é erro claro;
      cliente sem budget nunca tem `limite_excedido`/alertas; limiares
      cruzados corretos nos limites exatos (ex.: gasto == limite conta como
      cruzado); `limite_excedido` dispara por USD OU por tokens.

## 2. Superfície CLI: `brian budget check`

- [x] 2.1 `comandos::executar_budget_check(store, budgets_path, client_id)`
      — busca `consumo_do_cliente` do mês corrente, agrega com
      `comandos::agregar`, monta `StatusBudget`, formata relatório com
      gasto e limite lado a lado (nunca só um veredito).
- [x] 2.2 `ComandoBudget::Check { client }`, dispatch em `main.rs`.
- [x] 2.3 Testes: cliente inexistente é erro claro; cliente sem budget
      mostra "sem orçamento configurado"; relatório mostra gasto e limite.

## 3. Limite duro em `brian run`

- [x] 3.1 Em `comandos::executar_run`: antes de chamar
      `execucao::iniciar_run` (nenhum efeito colateral disparado ainda), se
      o contexto ativo tem `client_id`, carrega `budgets/clients.json` do
      cwd, calcula status do mês corrente; se `limite_excedido`, recusa com
      erro claro.
- [x] 3.2 Testes: cliente sem orçamento configurado roda run normalmente
      (nenhuma checagem bloqueia); cliente com orçamento excedido tem o
      run recusado antes de qualquer persistência (nenhum run gravado no
      store); cliente com orçamento configurado mas dentro do limite roda
      normalmente.

## 4. Verificação

- [x] 4.1 Cobertura de cada cenário do spec desta change (auditoria
      manual).
- [x] 4.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 4.3 `openspec validate --strict` limpo antes do archive.
