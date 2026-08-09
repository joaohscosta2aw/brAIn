## Purpose

SIGKILL não é capturável — um run cujo processo morre abruptamente fica marcado
como "em execução" para sempre, a menos que algo detecte isso. `brian recover`
fecha essa lacuna sem gastar dinheiro de novo.

## Requirements

### Requirement: Detecção de run órfão

O sistema SHALL identificar, entre os runs marcados como em execução, aqueles cujo
processo do provider não está mais vivo.

#### Scenario: Processo morto é detectado como órfão
- **GIVEN** um run marcado como em execução cujo processo foi encerrado por
  SIGKILL
- **WHEN** o operador consulta a recuperação
- **THEN** esse run é identificado como órfão

#### Scenario: Run realmente em execução não é marcado como órfão
- **GIVEN** um run cujo processo do provider ainda está vivo
- **WHEN** o operador consulta a recuperação
- **THEN** esse run não aparece como órfão

### Requirement: Finalização de órfão nunca duplica custo

Recuperar um run órfão SHALL NOT reexecutar a tarefa automaticamente. O sistema
SHALL apenas finalizar a contabilidade do run (status) e preservar o worktree.

#### Scenario: Recuperação não invoca o provider de novo
- **GIVEN** um run órfão identificado
- **WHEN** o operador o recupera
- **THEN** o sistema finaliza o registro do run sem criar um novo processo de
  provider
- **AND** o worktree permanece preservado para o operador decidir o que fazer

### Requirement: Consumo já capturado permanece íntegro

O consumo já registrado no ledger (D-16) antes da morte do processo SHALL
permanecer intacto — a finalização de um órfão não descarta nem reescreve consumo
já atribuído.

#### Scenario: Consumo parcial de um run órfão não é descartado
- **GIVEN** um run órfão que já havia gerado consumo registrado antes de morrer
- **WHEN** o run é recuperado
- **THEN** o consumo já registrado permanece no ledger sem alteração
