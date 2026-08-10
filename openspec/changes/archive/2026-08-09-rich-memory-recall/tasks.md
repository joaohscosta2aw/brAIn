## 1. Seleção e formatação (sem I/O na lógica)

- [x] 1.1 `src/memoria.rs` (novo): `OrcamentoRecall` (com `Default`:
      `max_items: 8`, `max_caracteres: 4000`).
- [x] 1.2 `selecionar_para_recall(notas: Vec<NotaDeMemoria>, orcamento:
      &OrcamentoRecall) -> Vec<NotaDeMemoria>` — função pura: `decisao`
      primeiro (já ordenadas por recência), depois demais por recência;
      corta por `max_items` e por `max_caracteres` acumulado (guloso, para
      no primeiro limite atingido; uma nota individual gigante nunca é
      truncada, só limita o que mais entra depois dela).
- [x] 1.3 `formatar_recall(notas: &[NotaDeMemoria]) -> String` — bloco de
      texto simples, `- [categoria] texto (motivo: ...)` quando houver
      rationale.
- [x] 1.4 Testes: mais notas que `max_items` corta corretamente; notas que
      excedem `max_caracteres` cortam mesmo com `max_items` disponível;
      decisão antiga sobrevive ao corte com notas mais recentes de outras
      categorias; lista vazia produz string vazia.

## 2. Busca via Store e integração com `brian run`

- [x] 2.1 `montar_recall(store, contexto, orcamento) -> Result<String,
      ErroRecall>` — busca `notas_do_contexto`, seleciona, formata.
- [x] 2.2 Em `comandos::executar_run`: monta o recall do contexto ativo
      (depois de `checar_orcamento`, antes de `execucao::iniciar_run`);
      anexa à tarefa só se não vazio.
- [x] 2.3 Testes: Context sem notas não altera a tarefa enviada a
      `iniciar_run`; Context com notas anexa o recall corretamente;
      `--explain-only` não monta recall (não faz sentido sem invocar
      provider).

## 3. `brian memory recall`

- [x] 3.1 `comandos::executar_memory_recall(store, contexto, orcamento) ->
      Result<String, String>`; `ComandoMemory::Recall`; dispatch em
      `main.rs`.
- [x] 3.2 Teste: saída de `executar_memory_recall` é idêntica ao recall
      efetivamente anexado por `executar_run` para o mesmo Context e
      notas (prova a spec "Recall exibido é idêntico ao que seria
      injetado" por construção — mesma função reaproveitada nos dois
      lados, não duas implementações comparadas).

## 4. Verificação

- [x] 4.1 Cobertura de cada cenário do spec desta change (auditoria
      manual).
- [x] 4.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 4.3 `openspec validate --strict` limpo antes do archive.
