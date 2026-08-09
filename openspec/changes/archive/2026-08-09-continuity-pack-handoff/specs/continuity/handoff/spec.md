## Purpose

Materializa o Continuity Pack do Context ativo para o próximo worker — o comando
que faz a troca de LLM não custar o raciocínio de novo.

## ADDED Requirements

### Requirement: Handoff materializa o pack para um provider de destino

O sistema SHALL, ao receber um provider de destino, montar o Continuity Pack
atual do Context ativo e apresentá-lo pronto para o operador levar a esse worker.

#### Scenario: Handoff com contexto ativo e notas registradas
- **GIVEN** um Context ativo com objetivo, decisões e notas registradas
- **WHEN** o operador executa handoff para um provider
- **THEN** o pack é montado e apresentado, citando as decisões e notas reais
  registradas

#### Scenario: Handoff sem contexto ativo
- **WHEN** o operador executa handoff sem Context ativo
- **THEN** o sistema recusa com erro explícito

### Requirement: Handoff nunca exige reexplicação do operador

O critério de aceitação do D-17 mínimo SHALL ser satisfeito: depois de um handoff,
o operador não precisa reexplicar objetivo, decisões nem tentativas que já
falharam — essas informações SHALL estar todas presentes no pack apresentado, sem
exigir novo input do operador para o worker seguinte já ter esse contexto.

#### Scenario: Pack cita arquivos reais do trabalho
- **GIVEN** um Context com arquivos alterados no repositório
- **WHEN** o handoff é executado
- **THEN** o pack apresentado cita os arquivos reais alterados, não uma descrição
  genérica

### Requirement: Handoff nunca mistura Context

Um handoff SHALL usar exclusivamente os dados do Context ativo no momento da
chamada — nunca dados de outro Context.

#### Scenario: Handoff usa só o Context ativo
- **GIVEN** notas registradas sob Contexts distintos
- **WHEN** o operador executa handoff estando conectado a um deles
- **THEN** o pack apresentado contém apenas notas do Context ativo no momento
