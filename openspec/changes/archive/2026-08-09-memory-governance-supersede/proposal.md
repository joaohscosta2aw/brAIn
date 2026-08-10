## Why

Blueprint §36: "Agentes sugerem. Brian decide o que se torna durável." e
"memória nunca é editada nem deletada. Uma decisão revista cria um novo
registro que supersede o anterior." Hoje `brian memory note`/`decide`
grava e nunca mais liga uma nota à correção que a substitui — uma decisão
errada e a decisão que a corrigiu ficam lado a lado, sem ordem, e
`rich-memory-recall` (change anterior) pode injetar as duas no prompt de
um agente ao mesmo tempo, contradizendo-se.

## What Changes

- `memory_note` ganha `superseded_by` (nullable, aditivo) — nunca editado
  depois de gravado, só preenchido uma vez quando uma nota nova supersede
  a anterior.
- `brian memory note`/`brian memory decide` ganham `--supersedes <id>`
  opcional: registra a nota nova normalmente e, na mesma operação, marca a
  nota anterior como superseded por ela — a nota anterior nunca é
  reescrita, só ganha o ponteiro.
- `memoria::montar_recall` (continuity/memory-recall) passa a excluir
  notas já superseded da seleção — uma decisão substituída nunca mais
  entra no prompt de um agente junto com a que a substituiu.

## Capabilities

### New Capabilities
- `continuity/memory-supersede`: cadeia de substituição append-only entre
  notas — a parte de §36 que já é real no modelo de dados do Brian hoje.

## Impact

- `src/storage/migrations/0009_memoria_supersede.sql` (novo):
  `ALTER TABLE memory_note ADD COLUMN superseded_by TEXT REFERENCES
  memory_note(id)`.
- `src/domain.rs`: `NotaDeMemoria.superseded_by: Option<String>`.
- `src/storage/mod.rs`/`sqlite.rs`: `Store::marcar_superseded`.
- `src/continuidade.rs`: `supersede(...)` reaproveitando
  `registrar_nota` para a parte de gravação.
- `src/memoria.rs`: `selecionar_para_recall` passa a receber só notas não
  superseded (filtradas antes de chegar na seleção).
- `src/comandos.rs`, `src/main.rs`: `--supersedes` em
  `ComandoMemory::Note`/`Decide`.

## Não-objetivos

- **Sem papéis Builder/Reviewer/Brian Core/Usuário nem
  `memory.suggest`/`memory.approve`/`memory.commit`** (blueprint §36,
  tabela de permissões): Brian não tem nenhum fluxo hoje em que um agente
  grava nota autonomamente — toda nota vem de `brian memory note`/`decide`,
  chamado pelo operador. Um sistema de papéis para um fluxo que não existe
  seria abstração sem uso real (YAGNI).
- **Sem estados `suggested`/`rejected`**: consequência direta do
  não-objetivo acima — sem sugestão de agente, não há o que aprovar ou
  rejeitar. Só `superseded` é real, porque `brian memory note/decide` já
  é a ação humana direta que blueprint chama de "commit".
- **Sem classes epistêmicas §36.2** (`fact`/`hypothesis`/`observation`/
  `incident`/`lesson`): `CategoriaNota` existente (memory-notes, já
  travada) não muda. Renomear/estender categorias é uma decisão
  independente, fora desta change.
- **Continuity Pack não muda** — continua mostrando todas as notas,
  inclusive as já superseded (spec pack: "nenhuma nota é omitida", travado
  antes desta change). Só o recall automático de `brian run`
  (`continuity/memory-recall`) exclui superseded — o Pack é para o
  operador ler o histórico completo, o recall é para o que entra no
  prompt de um agente.

## Conformidade — checklist §16

- **D-14**: reforça, não enfraquece — nota superseded nunca é editada, só
  ganha um ponteiro; o texto original permanece intacto e recuperável.
- **D-9**: `ALTER TABLE` fica em `storage/migrations/`, único lugar com
  SQL.
- **Versão alvo**: blueprint §36 nominalmente v0.4; implementada agora
  fora da ordem original, no escopo real do que o Brian já faz hoje.
