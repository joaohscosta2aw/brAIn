## Context

`Store::notas_do_contexto(client_id, project)` já devolve as notas de um
Context, mais recente primeiro (usado hoje só por
`continuidade::montar_pacote`, que inclui todas sem cortar — correto para
o Continuity Pack, errado para injeção em prompt). Ver proposal.md para
motivação (blueprint §35) e por que o Pack não muda.

## Goals / Non-Goals

**Goals:**
- Seleção determinística, orçada, testável sem I/O.
- `decisao` nunca perde para uma nota mais recente de outra categoria.
- Caso comum (sem notas) é exatamente o comportamento de hoje.

**Non-Goals:** ver proposal.md.

## Decisions

**Ranking = categoria (`decisao` primeiro) + recência, nada mais.**
`Store::notas_do_contexto` já devolve em ordem de recência; a seleção só
particiona em duas listas (`decisao` / demais, cada uma já ordenada por
recência) e concatena `decisao` na frente. Sem símbolos, sem embeddings,
sem confiança (proposal.md, não-objetivos).

```rust
pub struct OrcamentoRecall {
    pub max_items: usize,
    pub max_caracteres: usize,
}

impl Default for OrcamentoRecall {
    fn default() -> Self {
        // blueprint §35.2: max_items 8, max_tokens 4000 -- caracteres como
        // proxy de tokens, mesma convenção de continuidade/context_governor.
        Self { max_items: 8, max_caracteres: 4000 }
    }
}
```

**Corte é guloso e simples**: percorre a lista ordenada (decisão primeiro,
depois demais por recência); para em `max_items` notas OU quando a
próxima nota estouraria `max_caracteres` acumulado — o que vier primeiro.
Uma nota de decisão gigante que sozinha já estoura o orçamento ainda
entra (spec: "decisão sobrevive ao corte") — o orçamento limita
quantidade, não trunca o texto de uma nota individual (mesma disciplina
do Continuity Pack: nunca corta conteúdo no meio, só limita o que entra).

**`formatar_recall`** — bloco de texto simples, agrupado só por ordem de
seleção (não por categoria como o Pack, porque aqui o objetivo é caber no
prompt, não ser um documento de leitura humana):

```text
Memória relevante deste Context:
- [decisao] <texto> (motivo: <rationale>)
- [analise] <texto>
- [proximo_passo] <texto>
```

**`montar_recall(store, contexto, orcamento) -> Result<String, ErroRecall>`**
— busca as notas, seleciona, formata; devolve string vazia (não `None`)
quando não há notas, para o chamador simplesmente checar `.is_empty()`
antes de anexar.

**`comandos::executar_run` anexa o recall à tarefa, não substitui.**
`tarefa_final = if recall.is_empty() { tarefa.to_string() } else {
format!("{tarefa}\n\n{recall}") }` — passado a `PedidoRun.tarefa` no
lugar do texto original. Acontece depois da checagem de orçamento
(`checar_orcamento`, budget-alerts) e antes de `execucao::iniciar_run` —
mesma camada de decisão de `router`/`model_router`/`budget`, motor de
execução continua sem saber de memória.

**`brian memory recall`** reaproveita exatamente `montar_recall` —
garante por construção que o que o operador vê é o que seria injetado
(spec: "Recall exibido é idêntico ao que seria injetado"), sem duas
implementações divergentes.

## Risks / Trade-offs

- **Ranking mais simples que o próprio v0.4 do blueprint** (sem FTS5, sem
  filtro por símbolo): aceito, declarado em não-objetivos — Brian não tem
  os dados que esses critérios exigiriam ainda.
- **Só `brian run`, não workflow/eval/compare**: um run dentro de um
  workflow não recebe recall automático nesta v1. Aceito — extensão fica
  para quando houver demanda real, mesmo padrão de escopo mínimo das
  changes anteriores.
