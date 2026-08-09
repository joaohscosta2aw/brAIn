## MODIFIED Requirements

### Requirement: Execução não-interativa do provider dentro do worktree

O sistema SHALL invocar o provider em modo não-interativo, com o diretório de
trabalho do processo igual ao worktree do run.

#### Scenario: Run bem-sucedido registra o resultado
- **GIVEN** um provider que completa a tarefa
- **WHEN** o processo termina com sucesso e, se houver gate configurado
  (`execution/deterministic-gate`), o gate também passa
- **THEN** o run é marcado como concluído, com o resultado registrado

#### Scenario: Falha do provider é registrada, não perdida
- **GIVEN** um provider que retorna erro
- **WHEN** o processo termina com falha
- **THEN** o run é marcado como falho, com o motivo registrado
- **AND** o worktree permanece disponível para inspeção
