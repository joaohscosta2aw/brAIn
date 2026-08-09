## Why

O v0.0 já sabe quanto cada provider custou; ainda não sabe **como um usuário troca de
cliente sem misturar credencial, identidade Git e config de provider**. O erro mais
caro do produto (BRIAN-BLUEPRINT-V1.md §6.3) é rodar trabalho de um cliente
autenticado na conta de outro. `brian connect` é descrito no blueprint como "o teste
de valor mais rápido" do produto inteiro — sem ele, multi-cliente é teatro.

## What Changes

- `brian connect <client>[/<projeto>]` / `brian disconnect` / `brian whoami` /
  `brian context list|show|init`: troca simultânea de diretório de trabalho,
  identidade de provider (env var por adapter), identidade Git, e reserva do
  namespace de memória (só o ponteiro — o conteúdo do Continuity Pack é a próxima
  change, `continuity-pack-handoff`).
- Perfil de identidade (`identity_profile`): cliente, bindings de provider
  (`config_home` por provider), identidade Git (nome/email/chave de assinatura),
  organização GitHub.
- Isolamento de provider por variável de ambiente injetada no processo filho
  (`CODEX_HOME`, etc.) — cada adapter declara `isolation_verified` explicitamente;
  sem isso, Brian recusa oferecer identidade paralela para aquele provider em vez de
  fingir que isola.
- **Brian Vault**: referências de credencial (`keychain://...`) resolvidas contra o
  macOS Keychain no momento do uso, nunca persistidas como valor. Classes de secret
  (LOW/MEDIUM/HIGH/CRITICAL), metadados (criado em, expira em, última vez usado),
  exigência de Touch ID para classes HIGH/CRITICAL.

## Capabilities

### New Capabilities
- `identity/context-switching`: `connect`/`disconnect`/`whoami`/`context` — o "cd
  semântico" que troca diretório, identidade e reserva o namespace de memória.
- `identity/provider-isolation`: perfis de identidade, bindings de provider por
  variável de ambiente, declaração explícita de `isolation_verified`.
- `identity/vault`: armazenamento de referência de credencial no Keychain, classes de
  secret, metadados de rotação/uso, exigência de Touch ID por classe.

### Modified Capabilities
(nenhuma — `usage-ledger`, `cost-attribution`, `capacity-windows`, `plan-catalog` e
`plan-cost-allocation` permanecem como estão; esta change não altera atribuição de
custo, só adiciona de qual identidade o consumo partiu.)

## Impact

- Novo módulo `src/identidade.rs` (perfis, contexto ativo) e `src/vault.rs`
  (referências de credencial via Keychain).
- Nova dependência: `security-framework` (bindings oficiais da Apple para
  Security.framework — Keychain e `SecAccessControl` com biometria). Justificada:
  é o binding oficial do fornecedor, não uma lib de terceiros; a alternativa seria
  invocar o CLI `security` via subprocess passando segredo por argv/stdin, pior
  prática de segurança que a FFI direta.
- Novas tabelas: `identity_profile`, `credential_ref` — aditivas, sem alteração de
  `usage_record`/`provider_plan`.
- `src/main.rs` passa a resolver o contexto ativo antes de despachar comandos que
  dependem de identidade.
- Nenhuma chamada a Keychain acontece em `cargo test` — a fronteira de storage do
  Vault é uma trait, testada contra um backend falso em memória (mesmo padrão de
  `Store`/`SqliteStore`); só verificação manual explícita toca o Keychain real.

## Conformidade (PREMISSAS-BASICAS.md §16)

- M1-M6, OP-1..OP-8: sim — protege o erro mais caro do produto (rodar no cliente
  errado), reduz troca de contexto manual (M1/M5).
- Toca D-16 ou D-17: não diretamente. D-17 (continuidade multi-LLM) é a change
  seguinte; esta prepara o namespace, não a memória em si.
- Não viola D-10: opera exatamente na fronteira N providers × M clientes.
- Não depende de H-1: nenhuma dependência de Context Governor.
- Versão alvo: v0.1 (ordem: D-16 v0.0 já verde → D-17 mínimo v0.1).

## Não-objetivos

- Continuity Pack / handoff de memória entre LLMs — `continuity-pack-handoff`
  (próxima change).
- Run, worktree, workflow — v0.2.
- Backends de secret além do macOS Keychain (HashiCorp Vault, AWS/Azure/GCP Secrets
  Manager) — v1.0+ por declaração explícita do blueprint (§7.1).
- Política de aprovação automatizada por operação (§7.3 completo: deploy produção
  com Touch ID + aprovação) — aqui entra só a exigência de Touch ID por classe de
  secret na leitura da credencial; fluxo de aprovação de operação é do v0.2 (run).
