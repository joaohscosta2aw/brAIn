## Purpose

Executa uma tarefa através de um provider, isolado por worktree Git dedicado (D-7),
com o run persistido antes de qualquer efeito colateral (D-12) — o primeiro
subsistema do Brian que efetivamente produz trabalho, não só observa.

## Requirements

### Requirement: Worktree dedicado a partir de um commit base

O sistema SHALL criar, para cada run, um `git worktree` isolado a partir de um
commit base explícito. O sistema SHALL NOT escrever na árvore de trabalho
principal do usuário.

#### Scenario: Run cria worktree isolado
- **WHEN** o operador inicia um run
- **THEN** um worktree dedicado é criado a partir do commit base
- **AND** o provider trabalha exclusivamente dentro desse worktree

#### Scenario: Árvore principal nunca é tocada
- **GIVEN** um run em andamento
- **WHEN** o provider produz qualquer alteração
- **THEN** a árvore de trabalho principal do usuário permanece inalterada

### Requirement: Run persistido antes de qualquer efeito colateral

O sistema SHALL gravar o registro do run — cliente, contexto, commit base, caminho
do worktree, status — antes de invocar o provider (D-12).

#### Scenario: Run persistido antes da invocação
- **WHEN** um run é iniciado
- **THEN** o registro do run existe no banco antes do processo do provider ser
  criado

#### Scenario: Falha ao invocar o provider ainda deixa rastro
- **GIVEN** um run cujo processo de provider falha ao iniciar
- **WHEN** a falha acontece
- **THEN** o run permanece registrado, com status refletindo a falha — nunca um
  buraco sem registro

### Requirement: Execução não-interativa do provider dentro do worktree

O sistema SHALL invocar o provider em modo não-interativo, com o diretório de
trabalho do processo igual ao worktree do run. O sistema SHALL despachar
para o comando correto de cada provider presente em
`PROVIDERS_EXECUCAO_VERIFICADA` — SHALL NOT assumir que existe um único
provider de execução possível.

#### Scenario: Grok executa via seu próprio comando não-interativo
- **GIVEN** um run com `provider_id = "grok"`
- **WHEN** o provider é invocado
- **THEN** o comando `grok` é chamado em modo não-interativo (`-p`), não o
  comando de nenhum outro provider

#### Scenario: Provider ausente da lista verificada continua recusado
- **GIVEN** um `provider_id` que não está em `PROVIDERS_EXECUCAO_VERIFICADA`
- **WHEN** um run é iniciado com esse provider
- **THEN** o sistema recusa antes de qualquer efeito colateral

#### Scenario: Run bem-sucedido registra o resultado
- **GIVEN** um provider que completa a tarefa
- **WHEN** o processo termina com sucesso e, se houver gate configurado
  (`execution/deterministic-gate`), o gate também passa
- **THEN** o run é marcado como concluído, com o resultado registrado

#### Scenario: Falha do provider é registrada, não perdida
- **GIVEN** um provider que retorna erro
- **WHEN** o processo termina com falha
- **THEN** o run é marcado como falho, com o motivo registrado
- **AND** o worktree permanece disponível para inspeção

### Requirement: Runs concorrentes não colidem

Dois runs simultâneos SHALL NOT compartilhar worktree ou branch.

#### Scenario: Dois runs simultâneos no mesmo projeto
- **GIVEN** dois runs iniciados ao mesmo tempo no mesmo projeto
- **WHEN** ambos criam seus worktrees
- **THEN** cada um recebe um worktree e uma branch distintos
- **AND** nenhum arquivo de um run é visível no worktree do outro

### Requirement: Worktree nunca é removido em silêncio

Ao final de um run bem-sucedido, o worktree SHALL ser preservado como branch
dedicada. Ao final de um run que falhou ou foi cancelado, o worktree SHALL ser
preservado para inspeção — remoção é sempre uma ação explícita e separada.

#### Scenario: Run bem-sucedido preserva a branch
- **GIVEN** um run concluído com sucesso
- **WHEN** o run finaliza
- **THEN** a branch do worktree permanece disponível para revisão do operador

#### Scenario: Run que falha preserva o worktree
- **GIVEN** um run que falhou
- **WHEN** o run finaliza
- **THEN** o worktree permanece no disco, não é removido automaticamente

### Requirement: Commit base pode ser fixado explicitamente

O sistema SHALL aceitar um commit base explícito na inicialização de um run,
além do padrão (`HEAD` do repositório) já existente — necessário para
reprodutibilidade de casos de eval, que precisam rodar sempre contra o mesmo
commit, não o `HEAD` corrente do repositório de fixture.

#### Scenario: Run sem commit base explícito usa HEAD (comportamento atual)
- **GIVEN** um run iniciado sem commit base explícito
- **WHEN** o worktree é criado
- **THEN** o commit base é o `HEAD` do repositório, como já acontecia antes
  desta capability

#### Scenario: Run com commit base explícito usa esse commit
- **GIVEN** um run iniciado com um commit base explícito, diferente do `HEAD`
  atual do repositório
- **WHEN** o worktree é criado
- **THEN** o commit base do worktree é o commit explícito, não o `HEAD`
