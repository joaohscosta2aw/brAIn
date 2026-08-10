## Purpose

Algumas transições não deveriam ser automáticas — um humano decide se o
workflow segue, sem precisar virar um novo run gastando dinheiro sozinho.

## Requirements

### Requirement: Fase com aprovação obrigatória pausa o workflow

Uma fase com `requires_approval: true` SHALL pausar o `workflow_run`
imediatamente após terminar com sucesso, sem transicionar automaticamente
para a próxima fase.

#### Scenario: Fase de aprovação pausa em vez de avançar
- **GIVEN** uma fase com `requires_approval: true` que termina com sucesso
- **WHEN** o motor decidiria a próxima transição
- **THEN** o `workflow_run` fica com `status = paused`, sem avançar

### Requirement: Aprovação explícita retoma o workflow pausado

O sistema SHALL retomar um `workflow_run` pausado só por ação explícita do
operador — nunca automaticamente.

#### Scenario: Aprovação explícita avança para a próxima fase
- **GIVEN** um `workflow_run` pausado aguardando aprovação
- **WHEN** o operador aprova explicitamente
- **THEN** o workflow transiciona para a fase declarada em `on_success` da
  fase que pausou
