## 1. Config e cálculo (sem I/O na lógica)

- [x] 1.1 `src/billing.rs` (novo): `ConfigBillingCliente`,
      `carregar_billing(caminho)` — arquivo ausente = mapa vazio.
- [x] 1.2 `RelatorioChargeback`, `calcular_chargeback(client_id, config,
      custo_interno)` — função pura: erro se sem markup; markup sobre
      `equivalente`; piso mensal quando configurado.
- [x] 1.3 `formatar_chargeback` — sempre mostra custo interno e valor
      faturável juntos; sinaliza piso quando aplicado.
- [x] 1.4 Testes: sem markup configurado é erro claro; markup aplicado
      corretamente sobre custo interno; piso mensal ativa quando
      custo×markup fica abaixo dele e sinaliza isso; sem piso configurado
      nunca aciona piso; cliente sem consumo no período (custo interno
      ausente) não quebra o cálculo.

## 2. Superfície CLI

- [x] 2.1 `comandos::executar_billing_chargeback(store, billing_path,
      client, period, export)` — busca `consumo_do_cliente` +
      `comandos::agregar` (mesma fonte de `brian costs`), aplica
      `calcular_chargeback`, formata.
- [x] 2.2 `ComandoBilling::Chargeback`, `Comando::Billing`, dispatch em
      `main.rs`.
- [x] 2.3 Testes: cliente inexistente é erro claro; saída inclui custo
      interno e valor faturável.

## 3. Verificação

- [x] 3.1 Cobertura de cada cenário do spec desta change (auditoria
      manual).
- [x] 3.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes (inclui a checagem "custos pago e equivalente não são
      somados").
- [x] 3.3 `openspec validate --strict` limpo antes do archive.
