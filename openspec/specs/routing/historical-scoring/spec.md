## Purpose

Regra explícita (Fase 1) decide bem quando o operador já sabe o que quer;
score histórico serve para quando o próprio uso real deveria orientar a
escolha — mas só quando essa base de dados existe e nunca escondendo o
tamanho dela.

## Requirements

### Requirement: Score só entra quando explicitamente pedido

O sistema SHALL usar o modo de decisão por regra explícita (Fase 1,
`routing/provider-rules`) por padrão. Scoring histórico SHALL só ser
consultado quando o operador pedir explicitamente (`--scored`).

#### Scenario: Sem --scored usa só regra
- **GIVEN** um run sem a flag `--scored`
- **WHEN** o provider é decidido
- **THEN** o mecanismo usado é o mesmo de `routing/provider-rules` (regra ou
  default), sem consultar histórico nenhum

### Requirement: Score é calculado sobre runs reais do cliente

O score de um provider SHALL ser calculado a partir dos runs já finalizados
(`Concluido` ou `Falhou`) desse `client_id`, nunca de dados inventados ou de
outro cliente.

#### Scenario: Score usa só o histórico do cliente ativo
- **GIVEN** runs finalizados de dois clientes diferentes, com providers
  distintos
- **WHEN** o score é calculado para o cliente ativo
- **THEN** só os runs desse cliente entram na conta

### Requirement: Decisão por score nunca esconde o tamanho da base

Toda decisão de provider por score SHALL expor quantos runs históricos
alimentaram o cálculo (`n`) — nunca apresenta um score sem o `n` que o
sustenta.

#### Scenario: n pequeno aparece explícito
- **GIVEN** um provider cujo score foi calculado a partir de poucos runs
- **WHEN** a decisão é reportada (`--explain-only` ou `brian router score`)
- **THEN** o `n` usado aparece junto ao score, sem arredondar a confiança
  para cima

### Requirement: Provider sem histórico nenhum não é penalizado silenciosamente

Um provider disponível sem nenhum run histórico para esse cliente SHALL ser
elegível para o score (não excluído por falta de dado), mas SHALL ser
identificável como `n=0` na explicação.

#### Scenario: Provider novo aparece com n=0, não descartado
- **GIVEN** um provider disponível que nunca rodou para o cliente ativo
- **WHEN** o score é calculado entre os candidatos elegíveis
- **THEN** esse provider participa do cálculo com `n=0` visível, não é
  simplesmente omitido da decisão
