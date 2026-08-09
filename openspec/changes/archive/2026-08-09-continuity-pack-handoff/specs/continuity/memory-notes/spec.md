## Purpose

Registro manual, append-only, do que um operador quer que sobreviva à troca de LLM:
objetivo, decisões e seus porquês, análise, tentativas que falharam, próximos
passos — a matéria-prima do Continuity Pack.

## ADDED Requirements

### Requirement: Nota registrada sob o Context ativo

O sistema SHALL registrar uma nota associada ao Context ativo no momento do
registro, com categoria (`objetivo` | `decisao` | `analise` | `tentativa_falha` |
`proximo_passo` | `nota`), texto e instante.

#### Scenario: Registrar nota com contexto ativo
- **GIVEN** um Context ativo
- **WHEN** o operador registra uma nota
- **THEN** a nota é gravada associada a esse Context, com categoria e instante

#### Scenario: Registrar nota sem contexto ativo
- **WHEN** o operador tenta registrar uma nota sem Context ativo
- **THEN** o sistema recusa com erro explícito, não grava nota órfã

### Requirement: Decisão exige o porquê

Uma nota de categoria `decisao` SHALL incluir o resumo e o motivo (rationale)
separadamente — uma decisão sem porquê registrado perde o valor de evitar
retrabalho.

#### Scenario: Registrar decisão com motivo
- **WHEN** o operador registra uma decisão com resumo e motivo
- **THEN** ambos ficam gravados, recuperáveis separadamente

#### Scenario: Registrar decisão sem motivo
- **WHEN** o operador tenta registrar uma decisão sem motivo
- **THEN** o sistema recusa com erro explícito

### Requirement: Isolamento entre Contexts por construção

Uma consulta de notas de um Context SHALL NOT retornar notas de outro Context —
memória nunca cruza cliente (D-14, BRIAN-BLUEPRINT-V1.md §37.1: negado por padrão,
sem flag de override).

#### Scenario: Notas de um cliente não vazam para outro
- **GIVEN** notas registradas sob dois Contexts distintos
- **WHEN** o operador consulta as notas de um deles
- **THEN** apenas as notas daquele Context são retornadas

### Requirement: Notas são append-only

Uma nota já registrada SHALL NOT ser editada ou removida. Uma correção SHALL ser
uma nova nota, nunca uma sobrescrita silenciosa (D-14).

#### Scenario: Duas notas sobre o mesmo assunto coexistem
- **GIVEN** uma nota já registrada
- **WHEN** o operador registra uma nova nota que corrige ou complementa a anterior
- **THEN** ambas permanecem recuperáveis, a anterior nunca é apagada
