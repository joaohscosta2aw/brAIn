## Purpose

A auditoria precisa sobreviver ao Brian: todo commit gerado por um run carrega,
no próprio Git, quem/o quê/quanto — verificável por qualquer pessoa com acesso ao
repositório, mesmo sem acesso ao banco do Brian (blueprint §78.1).

## Requirements

### Requirement: Commit gerado carrega trailers de proveniência

Quando um run produz um commit no worktree, o sistema SHALL adicionar trailers
identificando o run, o cliente, o contexto, o provider, o modelo e o custo em
dólar equivalente.

#### Scenario: Commit de um run tem os trailers
- **GIVEN** um run que produz um commit
- **WHEN** o commit é criado
- **THEN** ele contém `Brian-Run`, `Brian-Client`, `Brian-Context`,
  `Brian-Provider`, `Brian-Model` e `Brian-Cost-USD`

#### Scenario: Run sem alteração não força commit vazio
- **GIVEN** um run cujo provider não alterou nenhum arquivo
- **WHEN** o run finaliza
- **THEN** nenhum commit é criado — o sistema não fabrica um commit vazio só para
  carregar os trailers

### Requirement: Trilha é legível sem acesso ao banco do Brian

Os trailers de um commit SHALL ser suficientes para identificar de qual run,
cliente e provider ele se origina, sem depender de consulta ao banco do Brian.

#### Scenario: Leitura da trilha só com o Git
- **GIVEN** um commit gerado por um run
- **WHEN** alguém com acesso ao repositório (mas não ao banco do Brian) inspeciona
  o commit
- **THEN** consegue identificar o run, o cliente e o provider responsáveis só
  pelos trailers
