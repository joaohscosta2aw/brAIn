## Purpose

Escolher `--provider` toda vez que ele quase sempre seria o mesmo é cerimônia
sem valor (OP-1). Esta capability decide o provider por regra explícita
quando o operador não especifica um, sem esconder a decisão nem trocar
silenciosamente por outro provider quando a regra escolhe mal.

## Requirements

### Requirement: Override explícito sempre vence a regra

Quando o operador especifica `--provider` explicitamente, o sistema SHALL usar
esse provider, sem consultar nenhuma regra.

#### Scenario: Provider explícito ignora regras
- **GIVEN** regras configuradas que apontariam para um provider diferente
- **WHEN** o operador roda `brian run` com `--provider` explícito
- **THEN** o provider usado é o que o operador especificou, não o da regra

### Requirement: Regra decide o provider quando não há override

Quando o operador não especifica `--provider`, o sistema SHALL avaliar as
regras em ordem e usar o provider da primeira regra que casar; sem regra
casando, SHALL usar o `default` configurado.

#### Scenario: Primeira regra que casa decide
- **GIVEN** regras configuradas, mais de uma capaz de casar com o run atual
- **WHEN** o operador roda `brian run` sem `--provider`
- **THEN** o provider usado é o da primeira regra que casa, na ordem em que as
  regras estão declaradas

#### Scenario: Nenhuma regra casa usa o default
- **GIVEN** regras configuradas, nenhuma casando com o run atual
- **WHEN** o operador roda `brian run` sem `--provider`
- **THEN** o provider usado é o `default` configurado

### Requirement: Provider decidido por regra ainda precisa ser válido

Um provider escolhido por regra (ou pelo `default`) SHALL ser validado da mesma
forma que um provider passado explicitamente — se não tiver execução
verificada, o sistema SHALL falhar com erro explícito, nunca substituir
silenciosamente por outro provider.

#### Scenario: Regra aponta para provider sem execução verificada
- **GIVEN** uma regra cujo provider não tem execução não-interativa verificada
- **WHEN** essa regra é a que casa
- **THEN** o run falha com erro explícito nomeando o provider — nenhum outro
  provider é tentado no lugar

### Requirement: Decisão pode ser inspecionada sem executar

O sistema SHALL permitir ver qual provider seria escolhido e por qual regra,
sem criar worktree nem invocar provider nenhum.

#### Scenario: Explain-only mostra a decisão sem efeito colateral
- **GIVEN** um run que seria decidido por regra
- **WHEN** o operador roda `brian run --explain-only`
- **THEN** o provider escolhido e a regra (ou `default`) responsável aparecem
- **AND** nenhum worktree é criado e nenhum provider é invocado
