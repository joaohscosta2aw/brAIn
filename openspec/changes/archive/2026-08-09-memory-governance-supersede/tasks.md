## 1. Schema

- [x] 1.1 `src/storage/migrations/0009_memoria_supersede.sql`: `ALTER
      TABLE memory_note ADD COLUMN superseded_by TEXT REFERENCES
      memory_note(id)`.
- [x] 1.2 Confirmar migração aditiva: `cargo test` das changes anteriores
      continua passando sem alteração; `migrate()` idempotente sobre
      banco já existente.

## 2. Domínio e Store

- [x] 2.1 `src/domain.rs`: `NotaDeMemoria.superseded_by: Option<String>`.
- [x] 2.2 `src/storage/mod.rs`/`sqlite.rs`: `Store::marcar_superseded(nota_id,
      superseded_by) -> Result<()>` -- `NotFound` se `nota_id` não existir.
      `notas_do_contexto`/`row_to_nota` passam a ler `superseded_by`.
- [x] 2.3 Testes: marcar nota inexistente é `NotFound`; nota marcada
      mantém `texto`/`rationale`/`categoria` originais intactos; leitura
      via `notas_do_contexto` traz `superseded_by` corretamente.

## 3. `continuidade::supersede`

- [x] 3.1 `supersede(store, contexto, nova: NovaNota, supersedes_id) ->
      Result<NotaDeMemoria, ErroNota>` -- valida que `supersedes_id`
      pertence ao Context ativo antes de gravar qualquer coisa; grava a
      nota nova via `registrar_nota` reaproveitado; marca a anterior.
- [x] 3.2 `ErroNota::NotaDeOutroContext` (ou equivalente) para o caso de
      isolamento.
- [x] 3.3 Testes: supersede de nota do mesmo Context funciona
      end-to-end (nota nova gravada, anterior marcada); supersede de nota
      de outro Context é recusado, nenhuma nota nova é gravada; supersede
      de id inexistente é recusado com erro claro.

## 4. Recall exclui superseded

- [x] 4.1 `memoria::montar_recall`: filtra `superseded_by.is_none()`
      antes de chamar `selecionar_para_recall`.
- [x] 4.2 Teste: nota superseded não aparece no recall, só a que a
      substituiu.

## 5. Superfície CLI

- [x] 5.1 `--supersedes <id>` opcional em `ComandoMemory::Note`/`Decide`;
      `executar_memory_note`/`executar_memory_decide` chamam
      `continuidade::supersede` quando presente.
- [x] 5.2 Teste: `brian memory note --supersedes <id>` e `brian memory
      decide --supersedes <id>` end-to-end via store real.

## 6. Verificação

- [x] 6.1 Cobertura de cada cenário do spec desta change (auditoria
      manual).
- [x] 6.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 6.3 `openspec validate --strict` limpo antes do archive.
