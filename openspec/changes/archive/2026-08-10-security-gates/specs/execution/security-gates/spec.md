## Purpose

Blueprint §30: secret scanning é "o único achado que não tem correção
barata depois do fato" — precisa rodar sempre, sem exceção, em qualquer
run. SAST e dependências vulneráveis ficam disponíveis sob demanda,
porque têm custo real de tempo que o operador deve escolher pagar.

## ADDED Requirements

### Requirement: Secret scan roda sempre, em todo run, sem opção de desligar

O sistema SHALL rodar um scan de segredos no worktree ao final de todo
`brian run`, independente de gate configurado — SHALL NOT expor nenhuma
forma de desabilitar esse scan.

#### Scenario: Run sem gate configurado ainda roda secret scan
- **GIVEN** um run sem `--gate` informado
- **WHEN** o provider termina
- **THEN** o secret scan roda mesmo assim, antes do run ser considerado
  concluído

### Requirement: Segredo encontrado reprova o run, mesmo com provider e gate bem-sucedidos

Quando o secret scan encontra qualquer achado, o sistema SHALL marcar o
run como falho — SHALL NOT marcar como concluído mesmo que o provider e o
gate configurado tenham tido sucesso.

#### Scenario: Provider e gate passam, mas há segredo no diff
- **GIVEN** um provider que termina com sucesso e um gate que passa
- **WHEN** o secret scan encontra um achado no worktree
- **THEN** o run é marcado como falho, com o achado no motivo

#### Scenario: Worktree limpo de segredos não é afetado
- **GIVEN** um provider que termina com sucesso e um gate que passa
- **WHEN** o secret scan não encontra nenhum achado
- **THEN** o run é marcado como concluído normalmente

### Requirement: SAST e dependências vulneráveis são sob demanda, não automáticos

`brian security scan` SHALL expor SAST (`--sast`) e checagem de
dependências vulneráveis (`--dependencies`) como comandos separados,
explicitamente invocados pelo operador — SHALL NOT rodar automaticamente
em todo `brian run`.

#### Scenario: brian run não dispara SAST nem checagem de dependências
- **WHEN** um `brian run` é executado
- **THEN** nem SAST nem checagem de dependências rodam automaticamente —
  só o secret scan roda

#### Scenario: Operador roda SAST manualmente
- **WHEN** o operador roda `brian security scan --sast --path <dir>`
- **THEN** o resultado do SAST é mostrado, com ferramenta, arquivo, linha
  e severidade de cada achado
