## 1. Esquema

- [x] 1.1 Migração `0003_identidade.sql`: `identity_profile` (id, client_id, git
      author name/email, github org).
- [x] 1.2 Migração: `provider_binding` (identity_profile_id, provider_id,
      config_home) — um perfil pode ter vários providers vinculados.
- [x] 1.3 Migração: `active_context` como singleton (linha única, client_id,
      project, identity_profile_id, connected_at) — task 1.1 do design.md.
- [x] 1.4 Migração: `credential_ref` (id, label, keychain_service,
      keychain_account, class, created_at, expires_at, last_used_at,
      rotation_policy) — nenhuma coluna de valor.
- [x] 1.5 Confirmar migração puramente aditiva: `cargo test` das changes
      anteriores continua passando sem alteração.

## 2. Tipos de domínio

- [x] 2.1 `PerfilIdentidade`, `ProviderBinding`, `ContextoAtivo` em
      `src/domain.rs` (ou `src/identidade.rs`, decisão de organização de módulo).
- [x] 2.2 `ClasseSecret` (`Low`/`Medium`/`High`/`Critical`).
- [x] 2.3 `CredencialRegistrada` (referência + metadados, nunca valor).

## 3. Vault

- [x] 3.1 **Confirmar contra o SDK instalado** o valor exato da flag
      `kSecAccessControlBiometryAny` (ou equivalente) antes de codar qualquer
      coisa que dependa dela (design.md: "não adivinhar o bit").
- [x] 3.2 `src/vault.rs`: trait `Vault` — `registrar_referencia`,
      `resolver` (retorna o valor só em memória, escopado à chamada),
      `metadados` (não resolve o valor, só lê classe/expiração/último uso).
- [x] 3.3 `VaultFalso`: backend em memória (`HashMap`) para teste — nenhum teste
      automatizado toca o Keychain real.
- [x] 3.4 `KeychainVault`: backend real via `security-framework`
      (`passwords`, `access_control::SecAccessControl`).
- [x] 3.5 Classe `high`/`critical` exige `SecAccessControl` biométrico antes de
      liberar o valor; `low`/`medium` não exige (spec vault, "Classe de secret
      determina exigência de autenticação").
- [x] 3.6 Biometria indisponível em credencial `high`/`critical` recusa a
      resolução — sem fallback silencioso para outro método.
- [x] 3.7 `last_used_at` atualizado a cada resolução bem-sucedida.
- [x] 3.8 Consulta de credencial expirada sinaliza explicitamente, sem bloquear
      nem falhar em silêncio.
- [x] 3.9 Nenhum comando do Brian expõe o valor bruto de uma credencial (sem
      subcomando de "export secret").
- [x] 3.10 Testes contra `VaultFalso` cobrindo os cenários acima (exceto os que
      exigem Keychain real, cobertos na task 7.4).

## 4. Isolamento de provider (Identity Manager)

- [x] 4.1 `src/identidade.rs`: registro declarado de `isolation_verified` por
      provider — mesmo padrão de `adapters::cobertura_v0_0`, começa vazio até
      alguém testar duas identidades simultâneas de verdade (design.md).
- [x] 4.2 Função que monta o conjunto de variáveis de ambiente para um processo
      filho de provider, a partir do `ProviderBinding` do perfil ativo.
- [x] 4.3 Provider sem `isolation_verified` recusa identidade paralela com aviso
      explícito; demais providers do mesmo contexto continuam funcionando.
- [x] 4.4 Variável de ambiente nunca escrita no ambiente do processo do Brian —
      só passada ao `Command` do processo filho.
- [x] 4.5 Testes cobrindo os três cenários do spec `provider-isolation`.

## 5. Troca de contexto

- [x] 5.1 `src/identidade.rs`: `conectar(client, projeto) -> ContextoAtivo` —
      grava em `active_context`; erro explícito se cliente/projeto não existe ou
      se há ambiguidade de projeto (spec: "múltiplos projetos, sem especificar
      qual").
- [x] 5.2 `desconectar()` — limpa `active_context`; no-op sem contexto ativo.
- [x] 5.3 `contexto_ativo() -> Option<ContextoAtivo>` — leitura simples.
- [x] 5.4 Geração das linhas `export VAR=valor` (`connect`) e `unset VAR`
      (`disconnect`) a partir dos bindings do perfil — função pura, testável sem
      I/O (design.md: "imprime exports, não abre subshell").
- [x] 5.5 `whoami`: mostra cliente, projeto, perfil, identidade Git, org GitHub e,
      por provider, status de autenticação **com a conta autenticada** — não só
      status binário.
- [x] 5.6 `whoami` sem contexto ativo informa isso explicitamente, não mostra
      contexto vazio.
- [x] 5.7 Testes cobrindo os cenários do spec `context-switching`.

## 6. Superfície CLI

- [x] 6.1 `brian connect <client>[/<projeto>]`.
- [x] 6.2 `brian disconnect`.
- [x] 6.3 `brian whoami`.
- [x] 6.4 `brian context list` / `brian context show [id]`.
- [x] 6.5 `brian context init` — cria vínculo do diretório atual a
      cliente/projeto (equivalente ao `.brian/context.toml` do blueprint,
      adaptado ao que já existe: sem novo formato de arquivo se o banco já
      resolve, YAGNI).
- [x] 6.6 `brian vault list` — metadados de credenciais (label, classe,
      criado em, último uso, expiração), nunca o valor. Sinaliza credencial
      expirada explicitamente (spec vault: "Consulta de credencial expirada
      alerta, não bloqueia sem explicação") — achado do audit: sem isto, o
      requisito de alerta de expiração não tinha superfície nenhuma pra
      acontecer, igual ao gap achado em `plan-cost-allocation` na change
      anterior.

## 7. Verificação

- [x] 7.1 Cobertura de cada cenário dos três specs desta change (auditoria
      manual, mesmo processo das changes anteriores).
- [x] 7.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes. Confirma que nenhum teste automatizado toca o Keychain real.
- [x] 7.3 Auditoria de segurança dedicada: nenhum valor de secret aparece em
      log, mensagem de erro, saída de CLI ou teste (grep por padrões óbvios não
      basta — leitura manual dos caminhos que tocam `Vault::resolver`).
- [x] 7.4 Verificação manual contra o Keychain real (`cargo test -- --ignored`,
      supervisionada pelo operador): classe `low` gravou/resolveu normalmente.
      Classe `critical` falhou na gravação com `errSecMissingEntitlement` — achado
      real, não defeito de código: gate biométrico exige binário assinado (já
      previsto pelo blueprint §7.4). Documentado em design.md. Credencial de teste
      removida do Keychain real ao final (confirmado com `security
      find-generic-password`, item não encontrado).
- [x] 7.5 `openspec validate --strict` limpo antes de considerar a change pronta
      para archive.
