## Purpose

Um provider que sai com sucesso não é prova de que o trabalho está correto — o
gate determinístico (testes, lint) é o juiz barato que decide o status final do
run, não a autoavaliação do provider (OP-5, OP-8).

## ADDED Requirements

### Requirement: Gate determinístico decide o status final do run

Quando um run é iniciado com um gate configurado, o sistema SHALL executar esse
gate dentro do worktree, após o provider terminar, antes de marcar o run como
concluído.

#### Scenario: Provider e gate bem-sucedidos concluem o run
- **GIVEN** um run com gate configurado
- **WHEN** o provider termina com sucesso e o gate também passa
- **THEN** o run é marcado como concluído

#### Scenario: Gate reprovado marca o run como falho mesmo com provider verde
- **GIVEN** um run com gate configurado, cujo provider termina com sucesso
- **WHEN** o gate falha
- **THEN** o run é marcado como falho
- **AND** a saída do gate é registrada como motivo da falha
- **AND** o worktree permanece disponível para inspeção (nenhuma remoção
  automática)

#### Scenario: Provider já falho não chega a rodar o gate
- **GIVEN** um run com gate configurado, cujo provider termina com falha
- **WHEN** o run finaliza
- **THEN** o run é marcado como falho pelo motivo do provider
- **AND** o gate não é executado

### Requirement: Ausência de gate preserva o comportamento anterior

Um run sem gate configurado SHALL ter seu status decidido só pelo resultado do
provider, exatamente como antes desta capability existir.

#### Scenario: Run sem gate configurado
- **GIVEN** um run iniciado sem gate
- **WHEN** o provider termina com sucesso
- **THEN** o run é marcado como concluído sem nenhuma etapa de gate

### Requirement: Gate reprovado nunca reexecuta o provider automaticamente

Um gate reprovado SHALL apenas finalizar o run como falho — nunca aciona uma nova
tentativa do provider automaticamente.

#### Scenario: Falha do gate não gera retry automático
- **GIVEN** um run cujo gate falhou
- **WHEN** o run finaliza
- **THEN** nenhum novo processo de provider é criado para esse run
- **AND** uma nova tentativa exige um novo `brian run` explícito do operador
