## Purpose

Provider e modelo são decisões separadas (blueprint §12) — um ponteiro
semântico como "coding" deve resolver para o nome concreto certo sem que
quem descreve a tarefa precise saber qual modelo exato está em vigor hoje.

## ADDED Requirements

### Requirement: Ponteiro resolve provider e tier antes do nome concreto

Um ponteiro semântico SHALL resolver, em ordem, para um provider e um tier, e
só então para o nome concreto de modelo desse provider.

#### Scenario: Pointer resolve para o nome concreto do primary
- **GIVEN** um pointer cujo `primary` aponta para um provider disponível com
  o tier configurado
- **WHEN** o operador roda `brian run --model-pointer <nome>`
- **THEN** o run usa o nome concreto de modelo correspondente ao tier do
  `primary`

### Requirement: Primary indisponível usa o fallback

Quando o provider do `primary` de um pointer não tem execução verificada, o
sistema SHALL usar o `fallback` do pointer.

#### Scenario: Primary indisponível cai para o fallback
- **GIVEN** um pointer cujo `primary` aponta para um provider sem execução
  verificada
- **WHEN** o pointer é resolvido
- **THEN** o provider e o tier usados são os do `fallback`

### Requirement: Fallback também indisponível recusa o run explicitamente

Quando nem `primary` nem `fallback` de um pointer têm provider disponível, o
sistema SHALL recusar o run com erro explícito nomeando o pointer — o run
SHALL NOT iniciar.

#### Scenario: Nenhum dos dois provider disponível
- **GIVEN** um pointer cujos `primary` e `fallback` apontam para providers
  sem execução verificada
- **WHEN** o operador tenta rodar com esse pointer
- **THEN** o run não inicia e o erro nomeia o pointer que falhou

### Requirement: Tier ausente degrada com aviso, nunca em silêncio

Quando o provider resolvido não tem o tier pedido configurado em
`models.json`, o sistema SHALL degradar para o tier mais próximo disponível
desse provider e SHALL emitir um aviso — nunca escolhe um modelo mais fraco
sem avisar.

#### Scenario: Tier ausente gera aviso visível
- **GIVEN** um provider resolvido sem o tier pedido em `models.json`
- **WHEN** o pointer é resolvido
- **THEN** o sistema usa o tier mais próximo disponível
- **AND** um aviso explícito acompanha a saída, nomeando o tier pedido e o
  tier realmente usado

### Requirement: Override explícito de modelo vence o ponteiro

Quando o operador especifica `--model` (nome concreto) explicitamente, o
sistema SHALL usar esse nome, sem consultar nenhum pointer — mesma
disciplina de override já aplicada a `routing/provider-rules`.

#### Scenario: --model explícito ignora --model-pointer
- **GIVEN** um run com `--model` e `--model-pointer` informados juntos
- **WHEN** o run é resolvido
- **THEN** o nome de modelo usado é o de `--model`, não o resolvido pelo
  pointer
