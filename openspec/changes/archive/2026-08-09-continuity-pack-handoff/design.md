## Context

`context-and-identity-switching` já existe: `active_context` (singleton, client_id +
project + identity_profile_id), `Store` trait em `src/storage/mod.rs` (D-9). D-14
(memória append-only, correção supersede) e D-17 (continuidade multi-LLM, mínimo no
v0.1) são leis já travadas — este design não as rediscute, só as implementa.

Sem `run` ainda (v0.2): não há processo do Brian controlando o worker de IA, então
"injetar" o pack no próximo provider não pode ser automático. O operador é quem leva
o pack ao próximo worker — mesmo padrão já estabelecido por `brian connect` (imprime
`export` para o operador aplicar, não mexe no ambiente por trás dele).

## Goals / Non-Goals

**Goals:**

- Critério de aceitação do D-17 mínimo (blueprint §34.0): depois de um handoff, o
  operador não reexplica objetivo, decisões, nem o que já falhou.
- Arquivos citados no pack são reais, verificáveis contra o repositório.
- Isolamento por Context é garantido por construção (mesma disciplina de
  `consumo_do_cliente` em `client-cost-attribution`).

**Non-Goals:**

- Memory Engine completo (retrieval, embedding, episodic) — v0.4+.
- Namespace compartilhado entre clientes (`org:workwise/shared`, blueprint §37.2) —
  exige fluxo de anonimização e auditoria que esta change não constrói.
- Injeção automática no processo do próximo provider — depende de `run` (v0.2).
  Aqui o pack é impresso; o operador decide como entregá-lo.
- Contagem exata de tokens do pack — o "orçamento" desta change é um limite de
  caracteres, rotulado como estimativa, não uma contagem de tokens do provider.

## Decisions

### Context de memória é `(client_id, project)`, reaproveitado de `active_context`

Nenhuma tabela nova de "contexto" — `memory_note` referencia `client_id` e
`project` diretamente, os mesmos campos que `active_context` já usa. Uma nota exige
um Context ativo no momento do registro (spec: "Registrar nota sem contexto ativo").

*Alternativa considerada:* introduzir `context_id` próprio, gerado. Rejeitada:
duplicaria a identidade que `(client_id, project)` já fornece — mesma razão que
`provider_plan` não reinventou `identity_profile` na change anterior.

### Arquivos tocados vêm de `git diff`/`git status`, calculados contra o `cwd` da chamada

Sem coluna nova de "caminho do repositório": `brian memory note`/`brian handoff`
usam o diretório de trabalho de onde são chamados (mesmo padrão que
`ClaudeAdapter`/`CodexAdapter` já usam `cwd` para localizar sessões, desde
`client-cost-attribution`). `git status --porcelain` lista os arquivos alterados;
sem repositório Git no `cwd`, a seção fica vazia, não é erro (spec: "Repositório sem
alterações").

*Alternativa considerada:* gravar um `repo_path` no perfil de identidade. Rejeitada
por agora: adicionaria uma segunda fonte de verdade para "onde fica o código" que
pode divergir de onde o operador realmente está rodando o comando — o `cwd` já é a
fonte usada por todo o resto do sistema.

### Pack montado sob demanda, nunca persistido como snapshot

`brian continuity show`/`brian handoff` montam o pack a partir de `memory_note` +
`git diff` no momento da chamada — sem tabela de snapshot. Mesma decisão de
"calculado sob demanda" de `capacity-windows-and-plans` (janela de capacidade):
sem daemon, não há processo de fundo mantendo um snapshot atualizado: persistir um
duplicaria o que já está em `memory_note` mais o estado do Git, sem ganho.

### Orçamento em caracteres, rotulado como estimativa

Limite de referência simples (contagem de caracteres do texto montado), não
contagem de tokens real — nenhuma dependência de tokenizer por provider seria
proporcional a um aviso (o blueprint pede "orçamento (warn)", não bloqueio nem
precisão). Cruzar o limite gera aviso explícito; o pack nunca é truncado em
silêncio (spec: "Pack acima do orçamento").

### Decisão exige `--why` obrigatório, nota simples não

`memory decide` sem motivo é recusado (spec: "Registrar decisão sem motivo"). Uma
decisão sem porquê registrado é indistinguível de uma nota qualquer — perde
exatamente o valor que evita a pergunta "por que fizemos assim?" de novo depois.

## Risks / Trade-offs

- **`git diff`/`git status` no `cwd` errado** (operador roda o comando fora do
  repositório do Context) → seção de arquivos tocados vem vazia silenciosamente,
  não avisa que o `cwd` pode estar errado. Aceito por ora: mesmo risco que já existe
  em todos os adapters de `client-cost-attribution` que dependem de `cwd`; não é
  introduzido por esta change.
- **Limite de caracteres não reflete tokens reais** de forma precisa entre
  providers diferentes (tokenização varia) → aceito conscientemente (ver decisão
  acima); rotulado como estimativa em toda superfície.
- **Handoff impresso, não injetado** → o operador ainda faz um passo manual (copiar
  o pack pro próximo worker). Aceito: é o mesmo trade-off já validado em `connect`
  (`eval "$(...)"`), consistente até `run` existir.

## Migration Plan

Migração aditiva `0004_continuidade.sql`: `memory_note`. Nenhuma coluna de tabela
existente muda. Reversão: apagar `~/.brian/brian.db` (mesmo plano das changes
anteriores — nada é enviado ou publicado fora da máquina).
