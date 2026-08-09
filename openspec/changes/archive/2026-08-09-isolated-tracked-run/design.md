## Context

D-7 (worktree obrigatório) e D-12 (persistir antes de efeito colateral) já são
leis travadas — este design não as rediscute, só as implementa. `identity/
context-switching` já resolve qual cliente/projeto está ativo; `capacity/
cost-attribution` já sabe processar consumo assim que ele existe no ledger — esta
change só precisa gerar esse consumo de forma rastreável, não reinventar como ele
é lido depois.

**[Δ] Escopo confirmado pelo autor em 2026-08-09**, depois de o autor apontar que
esta é a primeira change que executa (spawna um provider que escreve código e gera
commits, gastando dinheiro real, sem supervisão passo a passo). Reduzido ao mínimo
que satisfaz o critério de aceitação central do blueprint (§107.3): worktree, run
persistido, `recover` de órfão, trailers de proveniência. Telemetria completa,
limites de concorrência configuráveis, gates e router ficam para depois — ver
Non-Goals do proposal.md.

## Goals / Non-Goals

**Goals:**

- Critério central: `brian run` cria worktree, persiste antes de invocar o
  provider, nunca escreve na árvore principal.
- Um run interrompido por SIGKILL é recuperável sem duplicar custo.
- Commit gerado é auditável só pelo Git, sem precisar do banco do Brian.

**Non-Goals:**

- Telemetria OTel completa (§39) — um log de eventos local simples entra agora.
- Limites de concorrência configuráveis, alocação de porta/banco por run (§109.4,
  §109.5).
- Gates determinísticos, router, workflow `governed`, `--compare`.
- `brian worktree gc` automático.
- Suporte a múltiplos providers executando tarefas nesta change — só um, ver
  decisão abaixo.

## Decisions

### Só `codex` executa tarefas nesta change — não é preferência, é o único
### verificado com modo não-interativo seguro

`codex exec -s workspace-write -C <worktree>` roda sem prompt de aprovação e sem
precisar de nenhuma flag "dangerously" — o sandbox `workspace-write` é aplicado
pelo próprio Codex (não uma promessa do Brian), restringindo escrita de arquivo ao
workspace. Nenhuma outra flag de bypass é necessária para rodar de forma
totalmente não-interativa.

`claude -p` (headless) não tem um modo equivalente: a documentação da própria
Anthropic recomenda `--dangerously-skip-permissions`/
`--allow-dangerously-skip-permissions` para uso não-interativo, mas explicitamente
qualifica isso como seguro apenas em "sandboxes sem acesso à rede" — o que o
worktree do Brian **não é** (D-7 isola *filesystem* via `git worktree`, não rede,
processo, nem recursos). Usar essa flag aqui seria alegar uma garantia de
segurança que o Brian não entrega de verdade.

*Decisão:* `brian run` aceita `--provider codex` nesta change. Outros providers
ficam com execução não-interativa **não verificada como segura** — mesmo padrão de
honestidade de `PROVIDERS_ISOLAMENTO_VERIFICADO` (`context-and-identity-switching`)
e `providers_sem_fonte` (`capacity-windows-and-plans`): a lista começa pequena e
cresce quando alguém verifica de verdade, nunca por suposição.

*Alternativa considerada:* aceitar `--dangerously-skip-permissions` para Claude
mesmo assim, documentando o risco. Rejeitada: o próprio nome da flag e a
documentação da Anthropic descrevem exatamente a lacuna que estaríamos
escondendo — não é uma decisão de engenharia, é aceitar um risco de segurança sem
necessidade, quando existe uma alternativa (Codex) que não exige isso.

### Log de eventos local, não OTel completo

`run_event(run_id, tipo, detalhe, ocorrido_em)` — uma tabela simples, não o
subsistema de spans do §39. Suficiente para reconstruir "o que aconteceu neste
run" (criação de worktree, início de execução, fim, erro) sem construir um
pipeline de telemetria inteiro para um único tipo de evento por enquanto.

*Alternativa considerada:* implementar os spans do §39 (`worktree.create`,
`provider.execute`, etc.) com atributos OTel completos. Rejeitada por
desproporção: o valor de spans ricos aparece com volume real de runs e múltiplas
fases — esta change tem uma fase só. Migrar para spans reais quando `workflow`
multi-fase existir é extensão aditiva, não retrabalho.

### Worktree em `~/.brian/worktrees/run_<id>`, branch `brian/run_<id>`

Caminho fixo fora do repositório do usuário (blueprint §109.2). `run_<id>` é o
mesmo id do registro de run — sem gerador de nome separado.

### `recover` nunca reexecuta — só finaliza contabilidade

Consistente com blueprint §109.3 ("worktree preservado para inspeção... marcado
`released_at = null`, `status = abandoned`"). Reexecutar automaticamente
duplicaria custo se o run já tivesse gasto tokens antes de morrer — a única forma
seguramente correta de "não duplicar custo" é nunca gastar de novo sem decisão
humana explícita. O operador decide se roda `brian run` de novo (um run novo,
custo novo, não um "resume").

### Detecção de processo vivo via `kill -0`

`std::process::id()` do processo do provider é gravado no registro do run.
`recover` verifica vivacidade enviando sinal `0` (não mata, só testa
existência/permissão) — mecanismo POSIX padrão, sem dependência nova. Falso
positivo (PID reciclado por outro processo depois que o original morreu) é um
risco pequeno e aceito: a janela entre a morte do processo e a checagem de
`recover` é normalmente curta, e o pior caso é um run realmente morto continuar
marcado como "em execução" até a próxima checagem — não um dado incorreto
persistido.

### Trailers só quando há commit real

`git log -1 --format=%H` no worktree antes/depois da execução do provider detecta
se um commit novo foi criado. Se não, nenhum commit é forjado — spec: "Run sem
alteração não força commit vazio". Se sim, `git commit --amend` acrescenta os
trailers ao commit que o provider já criou (preserva a mensagem original do
provider, só adiciona os trailers ao rodapé).

## Risks / Trade-offs

- **Só um provider executa tarefas nesta change** — reduz utilidade imediata, mas
  evita alegar isolamento que não existe. Mitigação: lista de providers
  executáveis é extensível, mesma disciplina das changes anteriores.
- **Sandbox `workspace-write` do Codex não é auditado por este design** — o Brian
  confia na garantia que o próprio Codex declara, não a reimplementa. Se essa
  garantia falhar, é uma falha do provider, não do Brian — mas o worktree
  descartável (D-7) ainda limita o dano a algo que não é a árvore principal do
  usuário.
- **PID reciclado entre a morte do processo e a checagem de `recover`** — risco
  pequeno, aceito conscientemente (ver decisão acima).
- **`git commit --amend` reescreve o hash do commit que o provider criou** — o
  provider não sabe disso previamente; aceito porque é o único jeito de anexar
  trailers sem um segundo commit vazio "só de metadado", que seria pior para
  quem for ler o histórico depois.

## Migration Plan

Migração aditiva `0005_execucao.sql`: `run`, `run_event`. Nenhuma coluna de tabela
existente muda. Reversão: apagar `~/.brian/brian.db` remove o rastreamento; os
worktrees em `~/.brian/worktrees/` sobrevivem à reversão e podem ser removidos
manualmente pelo operador (`git worktree remove`).
