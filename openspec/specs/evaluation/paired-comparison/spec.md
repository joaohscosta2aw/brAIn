## Purpose

Com pouco histórico real, estatística de sucesso por provider é ruído
(`routing/historical-scoring`). Comparação pareada explícita — mesma tarefa,
providers diferentes, escolha humana — é dado de qualidade alta desde o
primeiro uso (blueprint §38.4).

## Requirements

### Requirement: Comparação roda a mesma tarefa em cada provider da lista

Quando o operador pede uma comparação, o sistema SHALL executar a mesma
tarefa uma vez para cada provider informado, cada execução em seu próprio
worktree isolado.

#### Scenario: Dois providers geram dois runs isolados
- **GIVEN** uma comparação com dois providers
- **WHEN** a comparação termina
- **THEN** existem dois runs reais, cada um em worktree próprio, ligados à
  mesma comparação

### Requirement: Candidato inválido falha a comparação inteira, sem pular silenciosamente

Quando um dos providers pedidos para comparação não tem execução verificada,
o sistema SHALL recusar a comparação com erro explícito nomeando esse
provider — SHALL NOT executar só os providers válidos e ignorar o inválido
sem avisar.

#### Scenario: Provider não verificado nomeado explicitamente
- **GIVEN** uma comparação pedida com um provider sem execução verificada
- **WHEN** o operador roda a comparação
- **THEN** a comparação falha, e o erro nomeia o provider inválido

### Requirement: Resultado é apresentado lado a lado, nunca escolhido automaticamente

O sistema SHALL apresentar o status de cada candidato (provider, status do
run, worktree) sem decidir um vencedor sozinho.

#### Scenario: Comparação termina sem vencedor definido
- **GIVEN** uma comparação com todos os candidatos concluídos
- **WHEN** a comparação termina
- **THEN** nenhum vencedor é registrado automaticamente

### Requirement: Escolha do vencedor é sempre uma ação explícita separada

O sistema SHALL registrar um vencedor de comparação só por ação explícita do
operador, nunca como parte da execução dos candidatos.

#### Scenario: Operador escolhe o vencedor depois
- **GIVEN** uma comparação já concluída, sem vencedor
- **WHEN** o operador escolhe explicitamente um dos candidatos
- **THEN** esse candidato é registrado como vencedor da comparação
