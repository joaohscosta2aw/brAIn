## Purpose

Detecta e registra, por provider com fonte própria, o plano de billing vigente —
o denominador sem o qual nenhuma janela de uso vira percentual. Providers sem fonte
própria ficam explicitamente fora, nunca omitidos em silêncio.

## Requirements

### Requirement: Detecção automática de plano por provider

Para um provider com fonte própria de plano/conta, o sistema SHALL consultar essa
fonte e registrar `billing_mode` e o identificador do plano relatado, sem exigir
digitação do operador.

#### Scenario: Provider relata plano de assinatura
- **WHEN** a fonte do provider é consultada e relata um plano de assinatura
- **THEN** o sistema registra `billing_mode` = `subscription` e o identificador do
  plano relatado
- **AND** a origem do dado é identificável como `provider`

#### Scenario: Provider relata plano de API
- **WHEN** a fonte do provider é consultada e relata cobrança por API
- **THEN** o sistema registra `billing_mode` = `api`

#### Scenario: Consulta de plano falha
- **WHEN** a consulta à fonte do provider falha (rede, autenticação expirada, etc.)
- **THEN** o sistema mantém o último plano conhecido, se houver
- **AND** identifica a informação como potencialmente desatualizada, não a descarta

### Requirement: Provider sem fonte de plano/quota é excluído e documentado

Um provider sem fonte própria de plano ou quota SHALL NOT ter plano fabricado ou
inferido. O sistema SHALL listá-lo explicitamente como sem fonte, nunca omiti-lo em
silêncio das superfícies de capacidade.

#### Scenario: Consulta de plano de provider sem fonte
- **WHEN** o operador consulta o plano de um provider sem fonte própria de
  plano/quota
- **THEN** o sistema informa que não há fonte disponível para esse provider
- **AND** o provider continua aparecendo nas superfícies de capacidade, marcado como
  sem fonte, não removido da lista

### Requirement: Vigência de plano por provider

Um provider SHALL ter no máximo um plano vigente por vez. O sistema SHALL preservar
planos anteriores como histórico, nunca sobrescrevendo silenciosamente.

#### Scenario: Plano detectado muda
- **GIVEN** um provider com um plano vigente
- **WHEN** uma nova consulta à fonte do provider relata um plano diferente
- **THEN** o plano anterior deixa de ser vigente e permanece consultável como
  histórico
- **AND** o novo plano passa a ser o vigente a partir do momento da detecção

#### Scenario: Janela histórica usa o plano vigente à época
- **GIVEN** um provider que teve o plano alterado no meio de um período consultado
- **WHEN** a capacidade desse período é calculada
- **THEN** cada trecho da janela usa o plano que estava vigente naquele instante
