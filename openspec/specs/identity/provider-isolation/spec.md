## Purpose

Declara, por provider, como sua configuração e autenticação são isoladas por
identidade — e recusa oferecer isolamento que não foi verificado, em vez de fingir.

## Requirements

### Requirement: Perfil de identidade declara bindings por provider

Um perfil de identidade SHALL conter o cliente ao qual pertence, os bindings de
provider (caminho de configuração isolado por provider), identidade Git (nome,
email) e organização GitHub, quando aplicável.

#### Scenario: Perfil com múltiplos providers vinculados
- **WHEN** um perfil de identidade é criado com bindings para mais de um provider
- **THEN** cada provider tem seu próprio caminho de configuração isolado
- **AND** os caminhos não colidem entre si nem com a configuração pessoal do
  desktop

### Requirement: Isolamento por provider exige verificação explícita

Um provider SHALL declarar `isolation_verified` antes que o sistema ofereça
identidade paralela para ele. Sem essa declaração, o sistema SHALL NOT fingir
isolamento.

#### Scenario: Provider com isolamento verificado
- **GIVEN** um provider com `isolation_verified = true`
- **WHEN** o operador conecta a um contexto que vincula esse provider
- **THEN** o processo do provider recebe a variável de ambiente do caminho de
  configuração isolado do cliente

#### Scenario: Provider sem isolamento verificado
- **GIVEN** um provider sem `isolation_verified` (ausente ou `false`)
- **WHEN** o operador conecta a um contexto que tentaria vincular esse provider
- **THEN** o sistema recusa ativar identidade paralela para esse provider
  especificamente, com aviso explícito
- **AND** os demais providers do mesmo contexto, se verificados, continuam
  funcionando normalmente

### Requirement: Variável de ambiente injetada não vaza para fora do processo filho

A variável de ambiente de isolamento de um provider SHALL ser injetada apenas no
processo filho daquele provider, nunca persistida no ambiente do processo do Brian
nem herdada por contextos subsequentes.

#### Scenario: Processo do provider encerra sem deixar rastro no ambiente do Brian
- **GIVEN** um provider executado sob um contexto com identidade vinculada
- **WHEN** o processo do provider encerra
- **THEN** o ambiente do processo do Brian não contém a variável de isolamento
  daquele provider
