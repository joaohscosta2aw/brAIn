## ADDED Requirements

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
