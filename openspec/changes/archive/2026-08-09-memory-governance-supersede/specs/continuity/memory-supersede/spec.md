## Purpose

Blueprint §36: memória é append-only, mas uma correção precisa ficar
ligada ao que corrigiu — sem isso, decisão errada e decisão certa convivem
sem ordem no recall de um agente.

## ADDED Requirements

### Requirement: Supersede nunca edita a nota anterior

Ao registrar uma nota com `--supersedes <id>`, o sistema SHALL gravar a
nota nova normalmente e SHALL marcar a nota anterior com
`superseded_by` apontando para a nova — SHALL NOT alterar `texto`,
`rationale` nem `categoria` da nota anterior.

#### Scenario: Nota anterior permanece intacta após supersede
- **GIVEN** uma nota já registrada
- **WHEN** o operador registra uma nova nota com `--supersedes` apontando
  para ela
- **THEN** a nota anterior continua com o texto original, só ganha
  `superseded_by` preenchido

### Requirement: Supersede de nota de outro Context é recusado

O sistema SHALL recusar `--supersedes <id>` quando a nota referenciada
pertence a um Context diferente do Context ativo — mesma disciplina de
isolamento de `memory-notes`.

#### Scenario: Tentativa de supersede cruzando Context
- **GIVEN** uma nota registrada sob o Context de outro cliente
- **WHEN** o operador tenta `--supersedes` essa nota a partir do Context
  ativo atual
- **THEN** o comando recusa com erro explícito, nenhuma nota é gravada

### Requirement: Recall exclui notas já superseded

`memoria::montar_recall` (continuity/memory-recall) SHALL excluir da
seleção qualquer nota com `superseded_by` preenchido — SHALL NOT injetar
no prompt de um run uma nota já substituída junto com a que a substituiu.

#### Scenario: Nota superseded não aparece no recall
- **GIVEN** uma nota já superseded por outra mais recente
- **WHEN** o recall é montado para o mesmo Context
- **THEN** só a nota mais recente aparece, a superseded fica de fora
