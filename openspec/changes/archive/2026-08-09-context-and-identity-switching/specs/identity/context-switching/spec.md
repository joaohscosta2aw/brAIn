## Purpose

O "cd semântico" do Brian: um único comando troca cliente, projeto, identidade de
provider, identidade Git e reserva o namespace de memória — impedindo o erro mais
caro do produto, rodar trabalho de um cliente autenticado na conta de outro.

## ADDED Requirements

### Requirement: Troca simultânea de contexto

O sistema SHALL ativar, num único comando, o cliente, o projeto (quando informado ou
determinável sem ambiguidade), a identidade de provider vinculada, a identidade Git e
o namespace de memória do cliente.

#### Scenario: Conectar a cliente com projeto único
- **GIVEN** um cliente com exatamente um projeto configurado
- **WHEN** o operador conecta a esse cliente sem especificar projeto
- **THEN** o contexto ativa esse projeto único
- **AND** identidade de provider, identidade Git e namespace de memória do cliente
  ficam ativos

#### Scenario: Conectar a cliente com múltiplos projetos, sem especificar qual
- **GIVEN** um cliente com mais de um projeto configurado
- **WHEN** o operador conecta a esse cliente sem especificar projeto
- **THEN** o sistema recusa a conexão com erro explícito listando os projetos
  disponíveis
- **AND** nenhum contexto parcial fica ativo

#### Scenario: Conectar a cliente ou projeto inexistente
- **WHEN** o operador conecta a um cliente ou projeto não configurado
- **THEN** o sistema recusa com erro explícito
- **AND** o contexto anterior (se houver) permanece ativo

### Requirement: Desconexão explícita

O sistema SHALL permitir encerrar o contexto ativo, retornando providers e
identidade Git ao estado padrão do desktop.

#### Scenario: Desconectar com contexto ativo
- **GIVEN** um contexto ativo
- **WHEN** o operador desconecta
- **THEN** nenhum contexto fica ativo
- **AND** providers voltam a usar a configuração padrão do desktop, não a do
  cliente anterior

#### Scenario: Desconectar sem contexto ativo
- **WHEN** o operador desconecta sem haver contexto ativo
- **THEN** o sistema trata como no-op, não como erro

### Requirement: Consulta do contexto ativo mostra a conta autenticada

O sistema SHALL responder, num único comando, cliente, projeto, perfil, identidade
Git, organização GitHub e — para cada provider — se está autenticado e **qual conta**
está autenticada, não apenas o status binário.

#### Scenario: Consulta com contexto ativo
- **GIVEN** um contexto ativo com providers autenticados
- **WHEN** o operador consulta o contexto
- **THEN** o sistema mostra, por provider, o status de autenticação e a conta
  autenticada

#### Scenario: Consulta sem contexto ativo
- **WHEN** o operador consulta o contexto sem haver um ativo
- **THEN** o sistema informa explicitamente a ausência de contexto ativo, não um
  contexto vazio ou de valores zerados

### Requirement: Isolamento entre contextos por construção

Trocar de contexto SHALL NOT deixar identidade, variável de ambiente ou
configuração do contexto anterior visível ao novo contexto.

#### Scenario: Troca sequencial de contexto não retém identidade anterior
- **GIVEN** um contexto A ativo, depois trocado para um contexto B
- **WHEN** o contexto B executa uma operação que depende de identidade de provider
- **THEN** a identidade usada é a de B, nunca a de A
