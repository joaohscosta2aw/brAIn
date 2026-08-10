## Context

`memory_note` já existe (migração 0004, continuity-pack-handoff) e
`continuidade::registrar_nota`/`Store::notas_do_contexto` já são a
fronteira toda. Esta é a primeira vez que uma coluna é adicionada a uma
tabela já existente via `ALTER TABLE` (todas as extensões anteriores
adicionaram a coluna dentro da própria migração de criação, porque a
tabela nascia na mesma change). Ver proposal.md para não-objetivos
(sem papéis, sem suggested/rejected, sem classes epistêmicas).

## Decisions

**Migração 0009: `ALTER TABLE memory_note ADD COLUMN superseded_by TEXT
REFERENCES memory_note(id)`.** Nullable, sem default além de `NULL` —
toda nota existente continua válida sem migração de dados.

**`Store::marcar_superseded(&self, nota_id: &str, superseded_by: &str) ->
Result<()>`** — `UPDATE memory_note SET superseded_by = ?1 WHERE id =
?2`, `NotFound` se `nota_id` não existir. Não valida isolamento de
Context aqui (isso é responsabilidade de `continuidade::supersede`, que
já tem o Context ativo em mãos — `Store` não tem contexto, só client_id
explícito, mesma separação de responsabilidade já usada em todo o
resto do módulo).

**`continuidade::supersede(store, contexto, nova_nota: NovaNota,
supersedes_id: &str) -> Result<NotaDeMemoria, ErroNota>`**:
1. Busca `notas_do_contexto` do Context ativo, confirma que
   `supersedes_id` está entre elas (spec: "Supersede de nota de outro
   Context é recusado") — `ErroNota::NotaDeOutroContext` se não estiver.
2. Chama `registrar_nota` (reaproveitado sem mudança) para gravar a nota
   nova.
3. Chama `store.marcar_superseded(supersedes_id, &nova.id)`.

Passo 1 acontece **antes** do passo 2 -- se a nota referenciada não
pertence ao Context, nada é gravado (nem a nota nova), mesma disciplina
de "validar tudo antes de qualquer efeito colateral" já usada em
`comparacao::rodar_comparacao`.

**CLI**: `--supersedes <id>` opcional em `ComandoMemory::Note`/`Decide` —
quando presente, `executar_memory_note`/`executar_memory_decide` chamam
`continuidade::supersede` em vez de `registrar_nota` diretamente.

**`memoria::montar_recall` filtra `superseded_by.is_none()`** antes de
chamar `selecionar_para_recall` — um filtro a mais na função que já busca
as notas via `Store::notas_do_contexto`, `selecionar_para_recall`
continua pura e não precisa saber de supersede.

## Risks / Trade-offs

- **Sem papel/aprovação real** (não-objetivo): `--supersedes` é uma ação
  humana direta, sem revisão de terceiro — aceito, é exatamente o nível
  de governança que já existe hoje (`brian memory note` também não passa
  por aprovação).
