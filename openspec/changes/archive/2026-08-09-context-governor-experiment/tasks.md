## 1. Schema

- [x] 1.1 Migração `0008_experimento.sql`: `experimento_execucao` (id,
      case_id, braco, run_id, started_at).
- [x] 1.2 Confirmar migração puramente aditiva: `cargo test` das changes
      anteriores continua passando sem alteração.

## 2. Store

- [x] 2.1 `src/storage/mod.rs`/`sqlite.rs`: `registrar_execucao_experimento`,
      `execucoes_do_experimento` (filtra por braço opcionalmente) — mesmo
      padrão de `registrar_candidato_comparacao`/`candidatos_da_comparacao`.
- [x] 2.2 Testes: CRUD básico; filtro por braço retorna só as execuções
      desse braço.

## 3. Montagem do pacote curado

- [x] 3.1 `src/context_governor.rs` (novo): `extrair_palavras_chave(tarefa)
      -> Vec<String>` — tokeniza, remove stopwords comuns, filtra por
      tamanho mínimo (função pura, sem I/O).
- [x] 3.2 `buscar_arquivos_relevantes(repo, palavras) -> Vec<String>` — roda
      `grep -rln` por palavra, deduplica resultados.
- [x] 3.3 `diff_recente(repo) -> Option<String>` — `git log -3 -p`, ou
      `None` se o repo não tiver histórico suficiente.
- [x] 3.4 `montar_pacote_curado(repo, tarefa, notas: &[NotaDeMemoria],
      orcamento_caracteres) -> String` — combina arquivos + diff + notas,
      corta no orçamento (spec: "Pacote curado é montado sem grafo de
      código real").
- [x] 3.5 Testes: palavras-chave ignora stopwords e palavras curtas; busca
      de arquivos encontra e deduplica; pacote respeita o orçamento de
      caracteres.

## 4. Formatação por braço e execução

- [x] 4.1 `formatar_tarefa_por_braco(tarefa, pacote: Option<&str>, braco:
      Braco) -> String` — spec: "Braço A não recebe pacote curado", "Braços
      B e C recebem o mesmo pacote com instruções diferentes".
- [x] 4.2 `rodar_execucao_experimento(store, contexto_sintetico, repo,
      case_id, braco, tarefa_base, agora) -> Result<RunRegistrado, String>`
      — monta pacote (só para B/C), formata tarefa, chama
      `execucao::iniciar_run`, registra `experimento_execucao`.
- [x] 4.3 Contexto sintético `client_id = "h1-experiment"` — nunca o
      contexto ativo do operador (mesma disciplina de `evaluation/eval-
      harness`).
- [x] 4.4 Testes: braço A não contém o texto do pacote na tarefa formatada;
      braços B e C contêm o mesmo pacote, com instruções de texto
      diferentes entre si; execução registra `experimento_execucao`
      corretamente.

## 5. Relatório

- [x] 5.1 `calcular_resultado_por_braco(execucoes: &[(Braco,
      RunRegistrado)]) -> Vec<ResultadoBraco>` — taxa de sucesso, duração
      média, `n` por braço, reaproveitando a mesma lógica de
      `router::calcular_scores` adaptada (spec: "n aparece junto a cada
      taxa e duração reportadas").
- [x] 5.2 `formatar_relatorio_h1(resultados) -> String` — inclui,
      incondicionalmente, a nota de limitação de métrica (spec: "Relatório
      nunca esconde que custo em USD não é medido").
- [x] 5.3 Testes: relatório sempre contém a nota de limitação, mesmo com
      resultados vazios; `n` aparece para cada braço.

## 6. Superfície CLI e dados do experimento

- [x] 6.1 `brian experiment run-h1 --case <id> --arm a|b|c` — carrega o
      case de `experiments/h1-tasks.json`, roda a execução.
- [x] 6.2 `brian experiment report-h1` — mostra o relatório agregado.
- [x] 6.3 `experiments/h1-tasks.json` (novo, no repo): 10 tarefas
      sintéticas, cada uma com `id`, `description`, `tipo`
      (`bug_pequeno`/`feature_media`/`refactor`), `fixture_repo` (caminho
      a ser preparado manualmente).

## 7. Verificação

- [x] 7.1 Cobertura de cada cenário do spec desta change (auditoria manual,
      mesmo processo das changes anteriores).
- [x] 7.2 `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`, `./scripts/verificar-invariantes.sh` — todos
      verdes.
- [x] 7.3 Teste manual supervisionado: preparar 1 fixture real, rodar os 3
      braços (A/B/C) para 1 case com `codex` real, confirmar que as 3
      tarefas formatadas são de fato diferentes entre si e que
      `experiment report-h1` reporta `n=1` por braço com a nota de
      limitação presente.
- [x] 7.4 `openspec validate --strict` limpo antes de considerar a change
      pronta para archive.

## 8. Execução do experimento completo (fora do gate normal)

- [ ] 8.1 Preparar os 10 fixtures de `experiments/h1-tasks.json`
      (trabalho manual, não código).
- [ ] 8.2 Rodar os 3 braços para as 10 tarefas (30 execuções reais de
      `codex`) — ação separada, supervisionada, com confirmação explícita
      do autor antes de cada lote (custo real substancial).
- [ ] 8.3 `brian experiment report-h1` final — resultado do experimento,
      com a decisão (confirma/descarta H-1) registrada em
      `docs/DECISIONS.md` junto com o `n` real usado.
