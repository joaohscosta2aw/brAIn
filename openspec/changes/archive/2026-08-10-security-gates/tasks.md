## 1. `src/security.rs`: as três ferramentas

- [x] 1.1 `Achado`, `ErroSecurity`.
- [x] 1.2 `rodar_gitleaks(dir)` -- arquivo temporário único, roda
      `gitleaks detect --source <dir> --no-git -f json -r <tmp>`, lê e
      deleta o arquivo, parseia o array.
- [x] 1.3 `rodar_semgrep(dir)` -- `semgrep scan --config auto --json
      --quiet <dir>`, parseia `results` do stdout.
- [x] 1.4 `rodar_osv_scanner(dir)` -- `osv-scanner scan source --format
      json -r <dir>`, stdout vazio vira lista vazia (não erro).
- [x] 1.5 Testes contra fixtures reais criadas na hora (git temporário
      com segredo/vulnerabilidade conhecida via `testutil`): gitleaks
      encontra o achado esperado; semgrep encontra o achado esperado;
      osv-scanner encontra o CVE esperado numa dependência antiga
      conhecida; os três devolvem lista vazia num diretório limpo.
- [x] 1.6 `security/gitleaks-rust.toml` (achado real do teste manual
      4.3): regra padrão `generic-api-key` não detecta `pub const X:
      &str = "..."` (sintaxe Rust) -- regra `rust-const-secret` própria,
      embutida via `include_str!`, `useDefault = true` para manter as
      regras padrão. Testado contra o próprio código-fonte do Brian sem
      falso positivo.

## 2. Secret scan obrigatório em `brian run`

- [x] 2.1 `execucao::iniciar_run`: chama `security::rodar_gitleaks` no
      worktree, incondicionalmente, depois do gate (ou ausência dele).
- [x] 2.2 `decidir_status_final` ganha `achados_secretos: &[Achado]` --
      não vazio força `Falhou`, mesmo com provider e gate bem-sucedidos.
- [x] 2.3 Evento `security.secrets.scan` sempre registrado; falha da
      própria ferramenta (não instalada) vira evento
      `security.secrets.failed`, não derruba o run.
- [x] 2.4 Testes: run com segredo no worktree é `Falhou` mesmo com
      provider e gate OK; run sem segredo não é afetado; secret scan
      roda mesmo quando o provider falha.

## 3. `brian security scan`

- [x] 3.1 `comandos::executar_security_scan(path, sast, dependencies)` --
      chama `rodar_semgrep`/`rodar_osv_scanner` conforme as flags,
      formata achados (ferramenta, arquivo, linha, severidade,
      mensagem).
- [x] 3.2 `ComandoSecurity::Scan`, dispatch em `main.rs`.
- [x] 3.3 Teste: saída lista os achados de um fixture com vulnerabilidade
      conhecida.

## 4. Verificação

- [x] 4.1 Cobertura de cada cenário do spec desta change (auditoria
      manual).
- [x] 4.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 4.3 Teste manual supervisionado: `brian run` real contra um
      fixture com uma credencial hardcoded proposital, confirmar que o
      run reprova (`Falhou`) mesmo com gate configurado passando.
- [x] 4.4 `openspec validate --strict` limpo antes do archive.
