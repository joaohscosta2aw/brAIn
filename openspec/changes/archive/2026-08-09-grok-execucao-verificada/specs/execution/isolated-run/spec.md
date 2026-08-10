## MODIFIED Requirements

### Requirement: Execução despacha pelo provider verificado, não hardcoded a um único

O sistema SHALL invocar o comando não-interativo correto para cada
`provider_id` presente em `PROVIDERS_EXECUCAO_VERIFICADA` — SHALL NOT
assumir que existe um único provider de execução possível.

#### Scenario: Grok executa via seu próprio comando não-interativo
- **GIVEN** um run com `provider_id = "grok"`
- **WHEN** o provider é invocado
- **THEN** o comando `grok` é chamado em modo não-interativo (`-p`), não o
  comando de nenhum outro provider

#### Scenario: Provider ausente da lista verificada continua recusado
- **GIVEN** um `provider_id` que não está em
  `PROVIDERS_EXECUCAO_VERIFICADA`
- **WHEN** um run é iniciado com esse provider
- **THEN** o sistema recusa antes de qualquer efeito colateral, mesmo
  comportamento já existente
