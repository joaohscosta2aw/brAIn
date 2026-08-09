## Purpose

Abstração de credencial do Brian: o banco guarda referências, nunca valores. Classes
de secret determinam quanta fricção de autenticação uma leitura exige.

## ADDED Requirements

### Requirement: Só referência é persistida, nunca o valor do secret

O sistema SHALL armazenar, para cada credencial, apenas uma referência opaca ao
backend de segredo (macOS Keychain). O valor do secret SHALL NOT ser gravado no
banco do Brian, em log, em trace ou em qualquer saída de CLI.

#### Scenario: Registro de credencial grava só a referência
- **WHEN** o operador registra uma credencial no Vault
- **THEN** o banco grava a referência ao item do Keychain
- **AND** nenhum campo do banco contém o valor do secret

#### Scenario: Erro ao resolver credencial não vaza valor parcial
- **GIVEN** uma resolução de credencial que falha
- **WHEN** o erro é reportado
- **THEN** a mensagem de erro não contém o valor do secret nem fragmento dele

### Requirement: Resolução de credencial é escopada à sessão

O valor de uma credencial resolvida SHALL viver apenas na memória do processo pelo
tempo do uso, e SHALL ser descartado ao final — nunca persistido em disco pelo
Brian fora do backend do Keychain.

#### Scenario: Valor não sobrevive ao fim do uso
- **GIVEN** uma credencial resolvida para uma operação
- **WHEN** a operação termina
- **THEN** o Brian não mantém o valor acessível para operações futuras sem nova
  resolução

### Requirement: Classe de secret determina exigência de autenticação

Toda credencial SHALL ter uma classe (`low` | `medium` | `high` | `critical`).
Resolver uma credencial de classe `high` ou `critical` SHALL exigir autenticação
biométrica (Touch ID) bem-sucedida antes de liberar o valor.

#### Scenario: Resolver credencial de classe alta com biometria disponível
- **GIVEN** uma credencial de classe `high` ou `critical`
- **WHEN** o operador a resolve e a autenticação biométrica é bem-sucedida
- **THEN** o valor é liberado para o uso solicitado

#### Scenario: Resolver credencial de classe alta sem biometria disponível
- **GIVEN** uma credencial de classe `high` ou `critical` e biometria indisponível
  no momento
- **WHEN** o operador tenta resolvê-la
- **THEN** o sistema recusa a resolução
- **AND** SHALL NOT cair para um método de autenticação mais fraco em silêncio

#### Scenario: Resolver credencial de classe baixa ou média
- **GIVEN** uma credencial de classe `low` ou `medium`
- **WHEN** o operador a resolve
- **THEN** o sistema libera o valor sem exigir biometria

### Requirement: Metadados de uso e expiração são rastreados

O sistema SHALL registrar, por credencial, o instante da última resolução
bem-sucedida. Quando a credencial tiver expiração declarada e o instante atual a
ultrapassar, o sistema SHALL sinalizar isso explicitamente, sem bloquear a
resolução em silêncio nem falhar sem explicação.

#### Scenario: Resolução bem-sucedida atualiza o último uso
- **WHEN** uma credencial é resolvida com sucesso
- **THEN** o instante dessa resolução fica registrado como o último uso

#### Scenario: Consulta de credencial expirada alerta, não bloqueia sem explicação
- **GIVEN** uma credencial cuja expiração declarada já passou
- **WHEN** o operador a consulta ou resolve
- **THEN** o sistema sinaliza explicitamente que a credencial está expirada
- **AND** a decisão de prosseguir ou não cabe ao operador, não a uma falha
  silenciosa

### Requirement: Exportação do valor de um secret é proibida

Nenhum comando do sistema SHALL imprimir, logar ou exportar o valor de uma
credencial.

#### Scenario: Tentativa de exportar valor de secret
- **WHEN** o operador tenta obter o valor bruto de uma credencial via qualquer
  comando do Brian
- **THEN** o sistema recusa a operação
