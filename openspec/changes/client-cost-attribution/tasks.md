## 1. Fundação

- [ ] 1.1 Criar o projeto Rust com o binário `brian` e estrutura de módulos separando `storage/` do restante (D-9)
- [ ] 1.2 Definir as traits de armazenamento que o núcleo consome, sem nenhuma dependência de SQLite fora de `storage/`
- [ ] 1.3 Implementar migrações versionadas com registro das migrações já aplicadas, idempotentes na reexecução
- [ ] 1.4 Criar o esquema inicial: `client`, `provider`, `usage_record`, `price_catalog`, `schema_migration` (referência: BRIAN-BLUEPRINT-V1.md §60)

## 2. Ledger

Depende de 1.

- [ ] 2.1 Implementar gravação de `usage_record` com provider, modelo, tokens de entrada/cache/saída/reasoning, custo e `occurred_at` em UTC
- [ ] 2.2 Distinguir ausente, zero e desconhecido na representação de tokens e custo — verificável por teste que falha se ausente virar zero
- [ ] 2.3 Tornar `usage_source` e `cost_source` obrigatórios: nenhum registro pode ser gravado sem ambos
- [ ] 2.4 Rejeitar registro sem `occurred_at` determinável, sem gravar linha parcial
- [ ] 2.5 Implementar a verificação de integridade do ledger cobrindo as quatro invariantes, reportando qual invariante falhou e em quais registros

## 3. Custo

Depende de 2.

- [ ] 3.1 Implementar cálculo de custo a partir do catálogo de preço quando o provider não reporta custo
- [ ] 3.2 Implementar a precedência D-6: custo do provider prevalece sobre catálogo
- [ ] 3.3 Marcar `cost_source = unknown` quando não há custo do provider nem entrada de catálogo, sem registrar zero
- [ ] 3.4 Implementar supersessão auditável de custo: quando o custo real chega depois, o valor e a fonte anteriores permanecem recuperáveis
- [ ] 3.5 Garantir que agregações informem a composição por fonte e destaquem a parcela de custo desconhecido

## 4. Atribuição

Depende de 2.

- [ ] 4.1 Implementar a cadeia de atribuição ao cliente, permitindo run e fase nulos em observe mode
- [ ] 4.2 Gravar consumo sem cliente determinável como `unattributed`, nunca descartando nem supondo dono
- [ ] 4.3 Expor o alarme de consumo não atribuído com tokens e custo, persistente enquanto houver registro órfão
- [ ] 4.4 Implementar atribuição manual a cliente existente, com registro auditável da origem humana
- [ ] 4.5 Recusar atribuição a cliente inexistente sem alterar o registro
- [ ] 4.6 Implementar reatribuição preservando a atribuição anterior de forma auditável
- [ ] 4.7 Expor leitura de consumo já escopada por cliente na camada de armazenamento, sem caminho que exija filtragem pelo chamador

## 5. Importação

Depende de 2 e 4.

- [ ] 5.1 Definir a trait de coleta em que cada adapter declara seu tier de integração (D-4) e quais campos fornece
- [ ] 5.2 Implementar deduplicação por identificador estável do provider quando disponível
- [ ] 5.3 Implementar o fallback de impressão digital (provider, modelo, instante, tokens, referência de sessão) e declarar tier degradado quando nenhum sinal estiver disponível
- [ ] 5.4 Garantir idempotência: reimportar janela já coberta não cria duplicata nem altera totais
- [ ] 5.5 Importar apenas o período ainda não coberto quando a janela é parcialmente conhecida
- [ ] 5.6 Isolar falha por provider: fonte indisponível reporta o provider afetado e não impede a importação dos demais

## 6. Adapters de provider

Depende de 5. Cada adapter é um incremento independente e verificável isoladamente.

- [ ] 6.1 Adapter Claude — identificar a fonte de uso, declarar tier e campos disponíveis
- [ ] 6.2 Adapter Codex — identificar a fonte de uso, declarar tier e campos disponíveis
- [ ] 6.3 Adapter Gemini — identificar a fonte de uso, declarar tier e campos disponíveis
- [ ] 6.4 Adapter Grok — identificar a fonte de uso, declarar tier e campos disponíveis
- [ ] 6.5 Adapter ZCode — identificar a fonte de uso, declarar tier e campos disponíveis
- [ ] 6.6 Expor a cobertura declarada por provider, de modo que provider sem adapter funcional apareça como lacuna explícita e não como ausência silenciosa de consumo

## 7. Comandos

Depende de 3, 4, 5.

- [ ] 7.1 `brian import` com recorte de período
- [ ] 7.2 `brian attribute` para atribuição manual
- [ ] 7.3 `brian costs` por cliente e por período, distinguindo cliente sem consumo de cliente inexistente
- [ ] 7.4 `brian costs --by provider` com soma coerente com o total do mesmo período
- [ ] 7.5 `brian costs --unattributed` listando cada registro órfão com provider, modelo, tokens, custo e instante
- [ ] 7.6 `brian costs --export` em formato tabular incluindo `usage_source` e `cost_source` por registro, com custo desconhecido marcado como tal

## 8. Verificação

Depende de todas as anteriores.

- [ ] 8.1 Cobrir cada cenário dos dois specs com teste automatizado, incluindo os unhappy paths
- [ ] 8.2 Verificar as invariantes de integridade contra um ledger populado, incluindo registros órfãos e de custo desconhecido
- [ ] 8.3 Medir as consultas de custo com volume sintético equivalente a doze meses e confirmar o limite de 200ms que D-1 estabelece como critério de revisão
- [ ] 8.4 Confirmar isolamento entre clientes por teste que falha se uma consulta escopada retornar registro de outro cliente ou não atribuído
- [ ] 8.5 Confirmar que `brian costs --unattributed` retorna vazio em um cenário de ledger íntegro — teste de integridade do D-16
