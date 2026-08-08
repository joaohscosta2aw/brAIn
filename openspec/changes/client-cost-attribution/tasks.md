## 1. Fundação

- [x] 1.1 Criar o projeto Rust com o binário `brian` e estrutura de módulos separando `storage/` do restante (D-9), cumprindo o contrato de `docs/harness/ambiente.md`: `Cargo.toml`, `rust-toolchain.toml` fixando a stable vigente, e `cargo build`/`test`/`clippy`/`fmt --check` executando num clone limpo
- [x] 1.2 Definir as traits de armazenamento que o núcleo consome, sem nenhuma dependência de SQLite fora de `storage/`
- [x] 1.3 Implementar migrações versionadas com registro das migrações já aplicadas, idempotentes na reexecução
- [x] 1.4 Criar o esquema inicial: `client`, `provider`, `usage_record`, `price_catalog`, `schema_migration` (referência: BRIAN-BLUEPRINT-V1.md §60)

## 2. Ledger

Depende de 1.

- [x] 2.1 Implementar gravação de `usage_record` com provider, modelo, tokens de entrada/cache/saída/reasoning, `billing_mode`, os dois campos de custo e `occurred_at` em UTC
- [x] 2.2 Distinguir ausente, zero e desconhecido na representação de tokens e custo — verificável por teste que falha se ausente virar zero
- [x] 2.3 Tornar `usage_source` e `cost_source` obrigatórios: nenhum registro pode ser gravado sem ambos (campos não-opcionais em `NovoConsumo` — o compilador impede a omissão, não apenas um `NOT NULL` no banco)
- [x] 2.4 Rejeitar registro sem `occurred_at` determinável, sem gravar linha parcial (mesmo mecanismo: `occurred_at: Instante` é obrigatório no tipo)
- [x] 2.5 Implementar a verificação de integridade do ledger cobrindo as quatro invariantes, reportando qual invariante falhou e em quais registros

## 3. Custo

Depende de 2.

- [x] 3.1 Implementar o catálogo de preço versionado por vigência, de modo que o equivalente de um consumo passado permaneça reproduzível após atualização de preços
- [x] 3.2 Calcular o custo equivalente em API a partir de tokens e catálogo, para qualquer `billing_mode`, inclusive assinatura
- [x] 3.3 Registrar o custo pago quando o provider o informa, mantendo-o em campo distinto do equivalente
- [x] 3.4 Implementar a precedência D-6 sobre o valor pago, sem apagar o equivalente
- [x] 3.5 Marcar `cost_source = unknown` apenas quando não há custo do provider nem entrada de catálogo, sem registrar zero em nenhum dos dois campos
- [x] 3.6 Implementar supersessão auditável: quando o custo pago chega depois, o valor e a fonte anteriores permanecem recuperáveis
- [x] 3.7 Garantir que nenhuma apresentação exiba o equivalente como valor pago — verificável por teste que falha se os dois forem somados num único número
- [x] 3.8 Garantir que agregações informem a composição por fonte e destaquem a parcela sem catálogo, que é receita não faturável (`comandos::formatar_agrupado` expõe `registros_custo_desconhecido` por grupo, testado em `agregar_conta_custo_desconhecido_separadamente`)

## 4. Atribuição

Depende de 2.

- [x] 4.1 Implementar a cadeia de atribuição ao cliente, permitindo run e fase nulos em observe mode (v0.0 não tem conceito de run/fase — não-objetivo explícito da proposal; nenhuma coluna especulativa foi criada para v0.2. A atribuição não depende deles, satisfazendo o requisito sem nada a fazer)
- [x] 4.2 Gravar consumo sem cliente determinável como `unattributed`, nunca descartando nem supondo dono
- [x] 4.3 Expor o alarme de consumo não atribuído com tokens e custo, persistente enquanto houver registro órfão (consulta disponível em `Store::nao_atribuidos`; exibição como banner é do grupo 7)
- [x] 4.4 Implementar atribuição manual a cliente existente, com registro auditável da origem humana
- [x] 4.5 Recusar atribuição a cliente inexistente sem alterar o registro
- [x] 4.6 Implementar reatribuição preservando a atribuição anterior de forma auditável
- [x] 4.7 Expor leitura de consumo já escopada por cliente na camada de armazenamento, sem caminho que exija filtragem pelo chamador

## 5. Importação

Depende de 2 e 4.

- [x] 5.1 Definir a trait de coleta em que cada adapter declara seu tier de integração (D-4) e quais campos fornece
- [x] 5.2 Implementar deduplicação por identificador estável do provider quando disponível
- [x] 5.3 Implementar o fallback de impressão digital (provider, modelo, instante, tokens, referência de sessão) e declarar tier degradado quando nenhum sinal estiver disponível (tier degradado é o adapter reportar `campos_disponiveis()` vazio — task 6.7 usa isso)
- [x] 5.4 Garantir idempotência: reimportar janela já coberta não cria duplicata nem altera totais
- [x] 5.5 Importar apenas o período ainda não coberto quando a janela é parcialmente conhecida
- [x] 5.6 Isolar falha por provider: fonte indisponível reporta o provider afetado e não impede a importação dos demais

## 6. Adapters de provider

Depende de 5. Cada adapter é um incremento independente e verificável isoladamente.

**Critério de pronto, válido para 6.2 a 6.10.** Um adapter só pode ser marcado
concluído quando existir uma amostra real capturada da fonte daquele provider,
guardada como fixture no repositório, e um teste que falhe se o adapter não
extrair dela exatamente os campos que declara fornecer. Declarar tier não é
concluir a task — sem fixture, o adapter fica em `não tentado`.

Escopo de providers revisado em 2026-08-08 (design.md) — substitui os cinco
nomes originais do blueprint pelo que existe e responde de fato nesta
máquina.

- [x] 6.1 Definir o formato de fixture de amostra e o teste genérico que confronta os campos declarados por um adapter contra o que ele realmente extrai da amostra (padrão: parsing puro separado de I/O, testável sem tocar disco — `src/adapters/claude.rs::parse_linha`)
- [x] 6.2 Adapter Claude — capturar amostra real, declarar tier e campos, provar extração contra a fixture (tier `session_files`, risco de R-4 aceito conscientemente — ver design.md)
- [x] 6.3 Adapter Codex — capturar amostra real, declarar tier e campos, provar extração contra a fixture (tier `session_files`; fixture real versionada em `src/adapters/fixtures/codex_sessao.jsonl`, extração coberta por testes do módulo `src/adapters/codex.rs`)
- [x] 6.4 Adapter Gemini — sem histórico por chamada (8 fontes investigadas, uso rastreado só no servidor — issues #387/#366/#543 do repositório oficial). Decisão consciente do autor: não desprezar o consumo mesmo assim. `src/adapters/gemini.rs` invoca `agy --print "/usage"` ao vivo (tier `headless_json`, único adapter desta change que não lê arquivo local) e emite um sinal de presença por janela de cota com evidência real de consumo (`remaining_fraction < 1.0`) — sem tokens, sem custo, sem atribuição a cliente (cota é por conta, não por projeto). Dedup por `reset_time` da janela, não por execução de import — reimportar a mesma janela é idempotente. Ver design.md, seção "Gemini: sinal sem rastreabilidade"
- [x] 6.5 Adapter Grok — capturar amostra real, declarar tier e campos, provar extração contra a fixture (tier `session_files`; formato mais favorável que o Claude: usage/custo em eventos `turn_completed` totalmente separados de conteúdo de mensagem — filtrar por tipo de evento já evita ler conteúdo, estruturalmente)
- [ ] 6.6 Adapter ZCode — `sem fonte utilizável`: app nativo inspecionado, sem log de uso estável/documentado para importação retroativa; permanece explicitamente fora de conclusão, não como ausência silenciosa
- [x] 6.9 Adapter GitHub Copilot CLI (`provider_id: github-copilot`) — investigar fonte de histórico local, capturar amostra real, declarar tier e campos, provar extração contra a fixture (`~/.copilot/session-store.db`, SQLite com schema nomeado — tabela `assistant_usage_events` já existe para isso, sem protobuf nem engenharia reversa; validado também contra o banco real desta máquina, não só a fixture: 3 eventos reais para este projeto)
- [x] 6.10 Adapter Qwen (`provider_id: qwen-deepseek` / `qwen-zai` / `qwen-kimi`) — uma CLI, três backends por `--model`. Fonte mais limpa de todas: `~/.qwen/usage_record.jsonl` (resumo por sessão, com `project` para filtro) + `~/.qwen/usage/token-usage-AAAA-MM.jsonl` (registro por chamada, `schemaVersion`, `id` estável, tokens). `qwen-deepseek` e `qwen-zai` verificados com fixture e validados contra dados reais desta máquina (26 e 4 eventos reais, respectivamente). `qwen-kimi` fica `não tentado`: configurado, bloqueado por saldo do provider — zero chamadas bem-sucedidas para verificar, não é falha de investigação
- [x] 6.7 Distinguir três estados por provider na cobertura declarada: `verificado` (fixture existe e o teste passa), `sem fonte utilizável` (fonte inspecionada e comprovadamente insuficiente, com o motivo registrado) e `não tentado`. Um provider nunca pode aparecer como ausência silenciosa de consumo (`src/adapters/mod.rs::StatusCobertura`, `cobertura_v0_0()`)
- [x] 6.8 Remover das amostras qualquer conteúdo de prompt, credencial ou dado de cliente antes de versioná-las — a fixture prova formato, não guarda trabalho real (IDs fictícios, testado que `content` nunca aparece no registro extraído)

## 7. Comandos

Depende de 3, 4, 5.

- [x] 7.1 `brian import` com recorte de período (`--desde`/`--ate` ISO-8601). Testado com binário real: todos os 8 coletores rodam, falha isolada por provider não impede os demais (task 5.6 exercitada em produção, não só em teste)
- [x] 7.2 `brian attribute` para atribuição manual. Testado com binário real contra cliente inexistente: recusa com erro claro, exit code 1
- [x] 7.3 `brian costs --client` por cliente e por período, apresentando custo pago e custo equivalente como grandezas distintas, e distinguindo cliente sem consumo de cliente inexistente
- [x] 7.4 `brian costs --by provider` com soma coerente com o total do mesmo período
- [x] 7.5 `brian costs --by model` expondo custo equivalente por token, permitindo comparar modelos de providers distintos numa base comum
- [x] 7.6 `brian costs --unattributed` listando cada registro órfão com provider, modelo, tokens, custo e instante
- [x] 7.7 `brian costs --export` em formato tabular (CSV) com colunas separadas para custo pago e custo equivalente, mais `billing_mode`, `usage_source` e `cost_source` por registro. Testado com binário real, arquivo CSV gerado e conferido

## 8. Verificação

Depende de todas as anteriores.

- [ ] 8.1 Cobrir cada cenário dos dois specs com teste automatizado, incluindo os unhappy paths
- [ ] 8.2 Verificar as invariantes de integridade contra um ledger populado, incluindo registros órfãos e de custo desconhecido
- [ ] 8.3 Medir as consultas de custo com volume sintético equivalente a doze meses e confirmar o limite de 200ms que D-1 estabelece como critério de revisão
- [ ] 8.4 Confirmar isolamento entre clientes por teste que falha se uma consulta escopada retornar registro de outro cliente ou não atribuído
- [ ] 8.5 Confirmar que `brian costs --unattributed` retorna vazio em um cenário de ledger íntegro — teste de integridade do D-16
- [ ] 8.6 Confirmar que consumo por assinatura produz custo equivalente e custo pago inexistente, e que nenhuma consulta apresenta o equivalente como valor pago
- [ ] 8.7 Confirmar que `./scripts/verificar-invariantes.sh` passa e que ele falha quando uma violação de D-9 ou de soma de custos é plantada deliberadamente
