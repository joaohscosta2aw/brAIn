## 1. Despacho por provider

- [x] 1.1 `src/execucao.rs`: refatora `executar_provider` para despachar
      por `provider_id` (`executar_codex`/`executar_grok`), extraindo o
      corpo atual do codex para `executar_codex` sem mudar seu
      comportamento.
- [x] 1.2 `executar_grok(worktree, tarefa, model)`: monta `grok --cwd
      <worktree> -p <tarefa> --permission-mode bypassPermissions [-m
      <model>]`, `stdin` nulo, mesmo `ResultadoProvider` de saída.
- [x] 1.3 `PROVIDERS_EXECUCAO_VERIFICADA` ganha `"grok"`.
- [x] 1.4 Testes: `executar_codex` extraído continua com os testes
      existentes passando sem alteração; comando montado para grok tem os
      argumentos corretos (teste de construção de args, sem depender do
      binário real instalado -- mesmo padrão dos testes de timeout de
      outros adapters).

## 2. Verificação

- [x] 2.1 Cobertura de cada cenário do spec desta change (auditoria
      manual).
- [x] 2.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 2.3 Teste manual supervisionado adicional: `brian run --provider
      grok` de ponta a ponta contra um fixture real, confirmar run
      `Concluido`, confirmar ausência de trailer de proveniência
      (esperada, documentada) — já rodei uma verificação equivalente fora
      do Brian (proposal.md); este passo confirma o caminho completo via
      `brian run`.
- [x] 2.4 `openspec validate --strict` limpo antes do archive.
