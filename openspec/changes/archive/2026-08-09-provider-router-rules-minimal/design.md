## Context

`brian run` (`isolated-tracked-run` + `run-fast-path-gates`, já arquivadas)
exige `--provider` sempre. `execucao::iniciar_run` já valida o provider contra
`PROVIDERS_EXECUCAO_VERIFICADA` e recusa antes de qualquer persistência quando
inválido — essa validação não muda; o router só decide *qual* string chega até
ali quando o operador não informa uma. Ver proposal.md para motivação
(blueprint §11, D-8 Fase 1) e não-objetivos.

## Goals / Non-Goals

**Goals:**
- `--provider` opcional em `brian run`; regra decide quando ausente.
- Override explícito sempre vence, sem exceção.
- `--explain-only` para inspecionar a decisão sem efeito colateral.

**Non-Goals:**
- Model pointers, classificação de task_type/risk/complexity, evidência
  histórica no scoring, log de decisão estruturado, constraints de
  deny/allowlist — todos non-goals explícitos do proposal.md (Fase 2/3 do
  blueprint §11, ou infraestrutura que ainda não existe).

## Decisions

**Regras em JSON, não YAML** — mesma razão de `eval-harness-minimal`:
`serde_json` já é dependência; YAML exigiria uma nova só para um arquivo de
poucos campos, sem D-decisão travando o formato.

**Sinais disponíveis: só `client` e `project`.** São os únicos que o Brian
calcula de verdade hoje (vêm do `ContextoAtivo`, já resolvido antes de
`iniciar_run` ser chamado). `task_type`/`risk`/`complexity` exigiriam um
classificador de tarefa que não existe — inventar um valor fake para "ter mais
sinal" seria pior que não ter regra nenhuma (viola a disciplina de honestidade
já estabelecida em `PROVIDERS_EXECUCAO_VERIFICADA`/`Brian-Model: unknown`).

**Formato da regra**: `{"when": {"client": "...", "project": "..."}, "then":
{"provider": "..."}}` — ambas as chaves de `when` são opcionais; ausente = não
participa do casamento (curinga). `default: {"provider": "..."}` obrigatório
no arquivo — sem `default`, "nenhuma regra casou" não tem resposta.

**Avaliação em `src/router.rs`, chamada de `comandos::executar_run` antes de
`execucao::iniciar_run`.** `execucao.rs` não muda: continua recebendo
`provider_id: &str` já resolvido. O router é uma camada fina acima, não uma
mudança de contrato do motor de execução — mesma separação de
`capacidade.rs`/`identidade.rs` (lógica pura, sem SQL, testável sem banco).

**`--explain-only` roda a decisão e imprime, sem chamar `iniciar_run`.** Não é
um modo separado do run — é o mesmo caminho de decisão, só que a CLI para antes
do passo com efeito colateral (spec: "nenhum worktree é criado e nenhum
provider é invocado").

**Arquivo de regras ausente → comportamento como se não houvesse nenhuma
regra configurada e nenhum default: erro claro pedindo `--provider` explícito
ou o arquivo de regras**, não um valor chumbado no binário. Rota alternativa
considerada (hardcode de um provider default no código) rejeitada: geraria uma
segunda fonte de verdade sobre "qual é o provider padrão", competindo com o
arquivo.

## Risks / Trade-offs

- **Regra mal escrita aponta para provider inválido** → já coberto pela
  validação existente em `iniciar_run` (spec: "Provider decidido por regra
  ainda precisa ser válido") — falha explícita, não substituição silenciosa.
- **Só dois sinais (`client`/`project`) é pouco poder de regra** → aceito
  conscientemente; mais sinais exigem um classificador que ainda não existe
  (non-goal explícito).
