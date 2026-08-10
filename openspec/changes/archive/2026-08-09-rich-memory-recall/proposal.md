## Why

Blueprint §35: "Brian não injeta toda a memória no prompt". Hoje isso já é
verdade sem querer, no pior sentido: `brian run` não injeta *nenhuma*
memória no prompt do provider. As notas registradas (`brian memory
note`/`decide`) só aparecem no Continuity Pack (artefato para o operador
ler no handoff manual) — um agente rodando via `brian run` nunca vê o que
já foi decidido ou tentado no Context antes dele, mesmo quando isso existe
e está gravado.

## What Changes

- `src/memoria.rs` (novo): seleciona um subconjunto orçado das notas do
  Context ativo — prioriza `decisao` (sempre incluída quando existe,
  mesmo peso de "always_include_types" do blueprint), depois as demais por
  recência, corta por `max_items` e por orçamento de caracteres (proxy de
  tokens, mesma convenção já usada em `continuidade`/`context_governor`).
- `brian run` passa a anexar esse recall ao texto da tarefa antes de
  invocar o provider, quando o Context ativo tem notas — silencioso
  quando não há nenhuma (nenhuma mudança de comportamento pro caso comum
  de hoje).
- `brian memory recall` (novo subcomando) — mostra exatamente o que seria
  injetado, para o operador auditar antes/depois de um run.

## Capabilities

### New Capabilities
- `continuity/memory-recall`: seleção orçada e ranqueada de notas para
  injeção automática no prompt de um run — distinto do Continuity Pack
  (que continua completo e só avisa, nunca corta; ver Não-objetivos).

## Impact

- `src/memoria.rs` (novo): `OrcamentoRecall`, `selecionar_para_recall`
  (função pura), `formatar_recall`, `montar_recall` (busca via `Store`).
- `src/comandos.rs`, `src/main.rs`: `executar_run` passa a montar o
  recall e anexá-lo à tarefa antes de `execucao::iniciar_run`;
  `ComandoMemory::Recall` novo, `executar_memory_recall`.
- Sem mudança em `continuidade.rs`/`execucao.rs`: Continuity Pack e o
  motor de execução continuam exatamente como são hoje.

## Não-objetivos

- **Sem símbolos tocados, sem similaridade semântica** (blueprint §35.1
  pede os dois como critério de ranking; §35.3 admite que mesmo o v0.4 só
  chega a FTS5, embeddings ficam para v1.0+): Brian não tem índice de
  símbolos nem embeddings. Ranking desta v1 é só categoria (`decisao`
  sempre) + recência — mais simples até que o próprio v0.4 do blueprint,
  documentado como divergência deliberada, não lacuna escondida.
- **Sem confiança (`confidence`)**: `NotaDeMemoria` não tem esse campo
  hoje; adicionar um multiplicador de confiança fabricado (sempre 1.0)
  não teria efeito real no ranking — YAGNI até existir um jeito honesto de
  calcular confiança.
- **Sem classes epistêmicas do blueprint §36.2** (`fact`/`hypothesis`/
  `observation`/`incident`/`lesson`) nem governança de memória (§36:
  estados suggested/active/superseded/rejected, aprovação humana): §36 é
  uma seção separada do blueprint (Governança), fora do escopo desta
  change. `CategoriaNota` existente (`memory-notes`, já validada) não
  muda.
- **Continuity Pack não muda.** `pack` (`continuity/pack`) já tem o
  requisito travado "nenhuma nota é omitida... o conteúdo continua
  completo, não é cortado sem aviso" — isso é o artefato de handoff
  *humano*, propositalmente completo. Esta change cria um ponto de
  injeção *diferente*: o que um provider vê automaticamente durante um
  run, que precisa ser orçado por definição (é prompt, não documento).
  Os dois convivem sem conflito porque servem leitores diferentes.
- **Só `brian run` nesta v1** — `brian workflow run`/`eval`/`compare`/
  `experiment` continuam sem recall automático. Extensão para essas
  superfícies fica para quando houver demanda real (mesmo padrão de
  escopo mínimo já aplicado a cada change anterior desta sessão).

## Conformidade — checklist §16

- **D-14**: não adiciona edição/remoção de nota — `selecionar_para_recall`
  só lê o que `Store::notas_do_contexto` já devolve.
- **D-9**: nenhum SQL novo; reaproveita `Store::notas_do_contexto`.
- Isolamento cliente/projeto: herdado de `notas_do_contexto`, já travado
  por `memory-notes`.
- **Versão alvo**: blueprint §35 nominalmente v0.4; implementada agora
  fora da ordem original, item (7) da lista combinada com o autor.
