## Purpose

Responde "quanto me resta nesta janela" para cada provider com fonte própria:
consumo, capacidade, percentual, restante, burn rate e tempo até reset — sempre com
a fonte do número identificável, nunca inventada.

## ADDED Requirements

### Requirement: Cálculo de janela de capacidade a partir do ledger

O sistema SHALL calcular, para um provider e uma janela (dia, semana, mês ou ciclo do
plano), o consumo de tokens no período a partir do ledger existente, e — quando a
capacidade da janela é conhecida — o percentual usado e o restante.

#### Scenario: Janela com capacidade conhecida
- **GIVEN** um provider cuja fonte relatou capacidade/percentual para a janela
- **WHEN** a capacidade da janela é consultada
- **THEN** o sistema retorna consumido, capacidade, percentual usado e restante

#### Scenario: Janela sem capacidade conhecida
- **GIVEN** um provider cuja fonte não relatou capacidade para a janela consultada
  (falha temporária ou provider sem fonte)
- **WHEN** a capacidade da janela é consultada
- **THEN** o sistema retorna o consumido medido, quando houver
- **AND** capacidade, percentual e restante são retornados como desconhecidos, não
  como zero

### Requirement: Hierarquia de fonte da capacidade

O sistema SHALL preferir quota/plano reportado pelo provider sobre o consumo medido
pelo Brian. Cada valor apresentado SHALL identificar de qual nível veio.

Um valor medido pelo Brian SHALL NOT ser apresentado como se tivesse a autoridade de
um valor reportado pelo provider.

#### Scenario: Provider reporta quota
- **GIVEN** um provider que reporta percentual usado e reset da janela
- **WHEN** a janela desse provider é consultada
- **THEN** o percentual e o reset vêm do provider
- **AND** a fonte identificada é `provider`

#### Scenario: Provider não reporta quota para a janela
- **GIVEN** um provider cuja fonte não relata percentual para a janela pedida
- **WHEN** a janela é consultada
- **THEN** o sistema retorna o consumo medido pelo Brian, sem fabricar percentual
- **AND** a fonte identificada é `brian_measured`, não `provider`

### Requirement: Fonte de plano/quota por provider verificado

Para cada provider com fonte própria (Claude via `claude auth status`, Codex via
`codex app-server` `account/rateLimits/read`, Gemini via `agy --print "/usage"`), o
sistema SHALL importar plano e/ou percentual restante e instante de reset como fonte
de nível 1 para as janelas de capacidade daquele provider.

#### Scenario: Fonte de um provider disponível
- **WHEN** a fonte de um provider verificado relata percentual restante e reset para
  uma janela
- **THEN** a janela correspondente é atualizada com esse percentual e reset
- **AND** a fonte identificada é `provider`

#### Scenario: Fonte de um provider indisponível
- **WHEN** a consulta à fonte de um provider falha ou está indisponível no momento da
  importação
- **THEN** a janela desse provider cai para consumo medido, sem percentual
- **AND** a importação dos demais providers não é afetada

### Requirement: Burn rate e projeção de esgotamento

O sistema SHALL calcular a taxa de consumo recente (tokens por hora) a partir do
histórico observado, e projetar o instante de esgotamento da janela apenas quando
capacidade e burn forem ambos conhecidos.

A projeção SHALL ser identificada como estimativa linear simples, nunca apresentada
como previsão certa.

#### Scenario: Projeção com capacidade e burn conhecidos
- **GIVEN** uma janela com capacidade conhecida e consumo recente mensurável
- **WHEN** a projeção de esgotamento é consultada
- **THEN** o sistema retorna um instante estimado de esgotamento
- **AND** o identifica como projeção, não como fato

#### Scenario: Projeção sem capacidade conhecida
- **GIVEN** uma janela sem capacidade conhecida
- **WHEN** a projeção de esgotamento é consultada
- **THEN** o sistema retorna a projeção como indisponível, não um instante inventado

### Requirement: Consulta de capacidade em um comando

O operador SHALL conseguir consultar, num único comando, o plano, a janela, o
percentual usado, o restante, o tempo até reset e o burn rate de cada provider com
fonte de plano/quota.

#### Scenario: Consulta de capacidade de todos os providers
- **WHEN** o operador consulta a capacidade sem especificar provider
- **THEN** o sistema retorna, para cada provider com fonte de plano/quota, plano,
  janela, percentual usado, restante, reset e burn rate

#### Scenario: Consulta de capacidade de provider sem fonte
- **WHEN** o operador consulta a capacidade de um provider sem fonte própria de
  plano/quota
- **THEN** o sistema informa explicitamente a ausência de fonte para aquele provider
- **AND** o provider continua aparecendo na lista, não é omitido
