## 1. Esquema

- [x] 1.1 Migração `0004_continuidade.sql`: `memory_note` (id, client_id, project,
      categoria, texto, rationale nullable, created_at).
- [x] 1.2 Índice por `(client_id, project, created_at)` — consulta sempre escopada
      ao Context.
- [x] 1.3 Confirmar migração puramente aditiva: `cargo test` das changes
      anteriores continua passando sem alteração.

## 2. Tipos de domínio

- [x] 2.1 `CategoriaNota` (`Objetivo`/`Decisao`/`Analise`/`TentativaFalha`/
      `ProximoPasso`/`Nota`) em `src/domain.rs`.
- [x] 2.2 `NotaDeMemoria` (client_id, project, categoria, texto, rationale
      opcional, created_at).
- [x] 2.3 `PactoDeContinuidade` (notas agrupadas por categoria, arquivos tocados,
      aviso de orçamento opcional).

## 3. Notas de memória (Store + orquestração)

- [x] 3.1 Extensão do `Store` trait: `registrar_nota`, `notas_do_contexto`.
- [x] 3.2 `src/continuidade.rs`: `registrar_nota(store, contexto_ativo, categoria,
      texto, rationale)` — erro explícito sem contexto ativo (spec memory-notes).
- [x] 3.3 Decisão sem `rationale` é recusada antes de chegar ao storage (spec:
      "Registrar decisão sem motivo").
- [x] 3.4 Isolamento por Context garantido pela consulta (`notas_do_contexto`
      nunca aceita filtro opcional — sempre client_id+project obrigatórios, mesmo
      padrão de `consumo_do_cliente`).
- [x] 3.5 Testes cobrindo os quatro requisitos do spec `memory-notes`.

## 4. Montagem do pack

- [x] 4.1 `src/continuidade.rs`: função pura que agrupa notas por categoria em
      `PactoDeContinuidade`.
- [x] 4.2 Arquivos tocados: função que invoca `git status --porcelain` no `cwd`
      informado, sem repositório = lista vazia, não erro.
- [x] 4.3 Aviso de orçamento: soma de caracteres do pack montado comparada a um
      limite de referência documentado; nunca trunca conteúdo.
- [x] 4.4 Testes cobrindo os quatro requisitos do spec `pack`, incluindo git real
      via fixture de diretório temporário (mesmo padrão de fixture usado nos
      adapters).

## 5. Handoff

- [x] 5.1 `src/continuidade.rs`: `handoff(store, contexto_ativo, cwd, provider) ->
      PactoDeContinuidade` — erro explícito sem contexto ativo.
- [x] 5.2 Formatação do pack para apresentação (texto estruturado, seções
      nomeadas, arquivos reais citados).
- [x] 5.3 Testes cobrindo os três requisitos do spec `handoff`.

## 6. Superfície CLI

- [x] 6.1 `brian memory note "<texto>"`.
- [x] 6.2 `brian memory decide "<texto>" --why "<motivo>"`.
- [x] 6.3 `brian continuity show` — pack do Context ativo, sem handoff.
- [x] 6.4 `brian handoff --to <provider>` — pack formatado para o provider de
      destino.

## 7. Verificação

- [x] 7.1 Cobertura de cada cenário dos três specs desta change (auditoria
      manual, mesmo processo das changes anteriores).
- [x] 7.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 7.3 Testar com dado real: notas reais registradas nesta sessão de trabalho,
      handoff real gerado a partir do estado real deste repositório.
- [x] 7.4 `openspec validate --strict` limpo antes de considerar a change pronta
      para archive.
