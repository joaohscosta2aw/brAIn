# MISSÃO: TRANSFORMAR O BLUEPRINT EM UM SISTEMA DE ESPECIFICAÇÃO + HARNESS DE ENGENHARIA

Você é o arquiteto principal deste projeto.

Sua tarefa NÃO é começar a implementar o produto.

Sua tarefa é estudar profundamente o blueprint fornecido, entender o sistema que queremos construir e transformar essa intenção em uma fundação de engenharia extremamente clara, consistente e utilizável por humanos e agentes de IA durante todo o ciclo de desenvolvimento.

Quero que você use OpenSpec como a camada formal de especificação e construa, ao redor dela, um harness de contexto, memória, regras e operação que maximize:

* qualidade de raciocínio;
* fidelidade ao produto;
* consistência arquitetural;
* autonomia controlada dos agentes;
* economia de tokens;
* recuperação precisa de contexto;
* manutenção de memória útil ao longo do projeto;
* prevenção de context drift;
* prevenção de overengineering;
* prevenção de instruções contraditórias;
* capacidade de evolução do projeto sem engessar decisões futuras.

O objetivo NÃO é maximizar a quantidade de documentação.

O objetivo é maximizar:

**informação útil / token consumido / decisão tomada.**

---

# 1. FONTE PRIMÁRIA

O blueprint fornecido é a principal descrição da intenção do produto.

Entretanto, NÃO trate cada frase do blueprint como uma decisão técnica imutável.

Faça distinção explícita entre:

1. intenção de produto;
2. requisito funcional;
3. requisito não funcional;
4. restrição real;
5. preferência;
6. hipótese;
7. decisão arquitetural;
8. detalhe ainda desconhecido;
9. sugestão do blueprint que pode ser melhorada.

Preserve a intenção.

Questione a implementação quando houver alternativa melhor.

Não invente requisitos silenciosamente.

Quando houver ambiguidade material, registre-a.

Quando for possível tomar uma decisão reversível com segurança, prefira uma decisão sensata e documente a premissa ao invés de bloquear todo o trabalho.

---

# 2. PRIMEIRA FASE: RECONHECIMENTO

Antes de criar ou modificar qualquer estrutura importante:

1. leia integralmente o blueprint;
2. inspecione o repositório existente;
3. identifique stack, estrutura, dependências e convenções já existentes;
4. detecte documentação ou configuração existente;
5. verifique se OpenSpec já está instalado/configurado;
6. inspecione CLAUDE.md, AGENTS.md, README, docs/, ADRs, configs e arquivos equivalentes;
7. identifique possíveis conflitos entre o blueprint e o sistema atual;
8. identifique decisões já tomadas que não devem ser duplicadas;
9. monte mentalmente o mapa do produto antes de produzir artefatos.

Não programe features nesta fase.

Não faça refactors oportunistas.

Não altere arquivos não relacionados.

---

# 3. MODELO MENTAL OBRIGATÓRIO

Estruture seu raciocínio utilizando estas camadas:

## CAMADA A — PRODUCT INTENT

Por que o produto existe?

Para quem?

Qual problema resolve?

O que diferencia uma boa implementação de uma implementação apenas tecnicamente correta?

## CAMADA B — SYSTEM BEHAVIOR

O que o sistema deve fazer externamente?

Quais comportamentos são observáveis e testáveis?

Esses devem alimentar principalmente os OpenSpec specs.

## CAMADA C — ARCHITECTURE

Como o sistema deve ser estruturado para produzir esses comportamentos?

Separar claramente arquitetura de requisito.

## CAMADA D — ENGINEERING CONSTRAINTS

Segurança.

Performance.

Confiabilidade.

Observabilidade.

Testabilidade.

Compatibilidade.

Manutenibilidade.

Custos.

Privacidade.

## CAMADA E — AGENT OPERATING SYSTEM

Que informações um agente precisa sempre saber?

Que informações só precisa conhecer quando tocar determinado domínio?

Que informação deve ser recuperada sob demanda?

Que decisões precisam persistir entre sessões?

Que regras são invariantes?

Onde o agente pode exercer julgamento?

---

# 4. OPENSpec

Quero OpenSpec como a fonte formal da verdade sobre comportamento esperado do sistema.

Utilize corretamente a separação entre:

* proposal;
* specs;
* design;
* tasks.

Não transforme specs em documentação de implementação.

Specs devem descrever principalmente comportamento observável.

Sempre que apropriado:

Requirement:
The system SHALL ...

Scenario:
GIVEN ...
WHEN ...
THEN ...

Inclua edge cases relevantes.

Inclua negative paths.

Inclua failure modes.

Inclua authorization/security behavior quando aplicável.

Inclua invariantes realmente importantes.

Não escreva cenários redundantes apenas para aumentar cobertura aparente.

---

# 5. ESTRUTURA DOS DOMÍNIOS

A partir do blueprint, proponha uma decomposição coerente de domínio.

Evite:

* um spec gigantesco para o produto inteiro;
* centenas de specs microscópicos;
* divisão artificial baseada apenas em estrutura de código.

Prefira fronteiras conceituais estáveis.

Exemplos possíveis:

openspec/specs/
identity/
auth/
accounts/
billing/
ingestion/
orchestration/
notifications/
analytics/

São apenas exemplos.

Descubra os domínios corretos deste produto.

---

# 6. HARNESS COGNITIVO DO PROJETO

Esta parte é extremamente importante.

Quero que você projete um harness que seja:

**prescritivo onde erros seriam caros;**
**descritivo onde contexto é necessário;**
**flexível onde julgamento de engenharia é melhor do que regras fixas.**

Não quero um CLAUDE.md gigantesco.

Não quero duplicação de conteúdo.

Não quero que todo contexto seja injetado em toda requisição.

Não quero instruções tão rígidas que impeçam soluções melhores.

Não quero instruções tão vagas que cada sessão reinvente o projeto.

Projete uma hierarquia de contexto.

---

# 7. POLÍTICA DE MEMÓRIA

Classifique informação persistente em quatro classes:

## L0 — ALWAYS-ON

Informação extremamente pequena e de altíssimo valor que deve estar disponível praticamente sempre.

Exemplos:

* princípios arquiteturais centrais;
* comandos fundamentais;
* invariantes;
* regras críticas;
* localização da fonte da verdade.

Budget desejado: mínimo possível.

## L1 — PROJECT MEMORY

Conhecimento estável do projeto.

Exemplos:

* visão resumida;
* stack;
* arquitetura de alto nível;
* convenções;
* boundaries;
* principais decisões irreversíveis.

Deve continuar enxuto.

## L2 — DOMAIN CONTEXT

Informação carregada somente quando determinado domínio estiver sendo trabalhado.

Exemplos:

* regras específicas de billing;
* regras específicas de auth;
* arquitetura do pipeline de ingestão.

Utilize arquivos locais/nested quando fizer sentido.

## L3 — RETRIEVAL / DEEP KNOWLEDGE

Informação detalhada que NÃO deveria ocupar contexto permanentemente.

Exemplos:

* ADRs;
* diagramas;
* investigação histórica;
* integrações detalhadas;
* contratos extensos;
* runbooks;
* documentação de terceiros;
* decisões antigas.

O agente deve saber ONDE procurar, não memorizar tudo.

---

# 8. PRINCÍPIO DE PROGRESSIVE DISCLOSURE

Implemente contexto por progressive disclosure.

Um agente começando uma tarefa deve receber apenas:

1. identidade do projeto;
2. princípios globais;
3. regras críticas;
4. como descobrir o restante.

Quando ele entra em um domínio, recebe contexto daquele domínio.

Quando precisa de detalhe, consulta a fonte específica.

Não replique documentação extensa em memória global.

Use pointers melhores que cópias.

Exemplo conceitual:

"Para regras de autenticação: veja X."

é preferível a duplicar 80 linhas de autenticação em CLAUDE.md.

---

# 9. TOKEN ECONOMY

Otimize explicitamente o sistema para economia de tokens.

Para cada informação persistente, pergunte:

> O agente precisa disso em praticamente toda tarefa?

Se NÃO:
não coloque no contexto global.

Pergunte também:

> Isso pode ser descoberto rapidamente no repositório?

Se SIM:
prefira uma indicação curta de onde encontrá-lo.

Evite:

* repetição;
* explicações óbvias;
* exemplos demais;
* listas de proibições redundantes;
* documentação autoevidente do código;
* cópia da mesma regra em OpenSpec + CLAUDE.md + README + docs.

Cada conceito deve ter uma fonte canônica.

Outros arquivos devem apontar para ela.

---

# 10. MEMÓRIA PROPOSITIVA

Quero uma memória que ajude a decidir, e não apenas uma coleção histórica de fatos.

Memória boa:

"API pública deve manter backwards compatibility."

Memória ruim:

"Em 17 de maio decidimos mudar FooController para BarController."

Memória boa:

"Preferir operações idempotentes em jobs assíncronos."

Memória ruim:

"Foi criada uma função retryJob() em src/jobs/foo.ts."

Priorize:

* princípios;
* invariantes;
* razões;
* decisões que influenciam decisões futuras;
* boundaries;
* constraints.

Evite persistir detalhes efêmeros.

---

# 11. DECISION MEMORY

Projete uma forma leve de registrar decisões arquiteturais importantes.

Para decisões relevantes, preserve:

* contexto;
* decisão;
* razão;
* alternativas relevantes;
* consequências;
* status;
* quando deve ser reconsiderada.

Não crie ADR para decisões triviais.

Uma decisão deve virar memória persistente apenas quando futuros agentes poderiam razoavelmente tomar uma decisão diferente sem conhecer o histórico.

---

# 12. AUTONOMIA CONTROLADA

Divida decisões em três classes.

## GREEN — AUTÔNOMO

O agente pode decidir sozinho.

Exemplos:

* nomes locais;
* pequenas refatorações;
* organização interna facilmente reversível;
* detalhes triviais de implementação.

## YELLOW — DECIDIR + REGISTRAR

O agente pode tomar uma decisão, mas deve documentar quando ela tiver impacto posterior.

Exemplos:

* abstração nova;
* escolha relevante entre padrões;
* mudança interna significativa.

## RED — HUMANO

Necessita confirmação.

Exemplos:

* mudança no comportamento especificado;
* contrato público;
* segurança;
* perda/destruição de dados;
* mudança estrutural difícil de reverter;
* nova dependência crítica;
* mudança significativa de produto;
* alteração de requisito do blueprint.

Adapte esta classificação ao projeto real.

---

# 13. ANTI-OVERENGINEERING

Inclua explicitamente no harness:

* implemente a solução mais simples que satisfaça o spec;
* não crie abstrações para possibilidades hipotéticas;
* não introduza infraestrutura para escala inexistente;
* não generalize antes de existir necessidade real;
* não refatore áreas não relacionadas;
* não adicione dependências sem benefício concreto;
* não transforme preferência em framework interno;
* prefira código legível a arquitetura “impressionante”.

Porém:

simplicidade não significa ignorar requisitos claramente previsíveis do blueprint.

---

# 14. HIERARQUIA DA VERDADE

Defina explicitamente a precedência de informação.

Sugestão inicial:

1. comportamento aprovado em OpenSpec;
2. decisões arquiteturais aprovadas;
3. contratos e schemas executáveis;
4. configuração do projeto;
5. documentação de domínio;
6. implementação atual;
7. comentários;
8. suposições do agente.

Mas analise se essa hierarquia é correta para este projeto.

Se duas fontes entrarem em conflito:

NÃO escolha silenciosamente.

Identifique a inconsistência.

Determine qual deveria ser canônica.

Corrija duplicação quando seguro.

---

# 15. CONFIGURAÇÃO OPENSpec

Analise e produza uma configuração OpenSpec otimizada para este projeto.

Não aceite defaults automaticamente.

Determine se o workflow spec-driven padrão é suficiente.

Se houver benefício real, considere um schema customizado.

Mas NÃO personalize por vaidade.

A complexidade adicional precisa justificar seu custo.

Configure `openspec/config.yaml` para oferecer contexto de planejamento de alto valor e baixo volume.

Evite colocar documentação extensa em `context:`.

Use `rules:` para requisitos realmente específicos de cada artefato quando necessário.

---

# 16. CLAUDE.md

Crie ou reorganize CLAUDE.md para funcionar como um mapa operacional, não como enciclopédia.

Idealmente deve responder rapidamente:

* O que é este projeto?
* Onde está a fonte da verdade?
* Quais regras jamais devo violar?
* Como descubro contexto do domínio?
* Como buildar/testar/lintar?
* Como devo trabalhar?
* Quando devo perguntar?
* Que alterações não devo fazer silenciosamente?

Evite colocar conteúdo que já vive corretamente em OpenSpec.

---

# 17. CONTEXTO POR DIRETÓRIO

Quando houver subsistemas suficientemente independentes, avalie usar instruções/contexto local por diretório.

Somente faça isso quando proporcionar locality real.

Não espalhe dezenas de arquivos de instrução pelo repositório.

A regra é:

**global quando universal; local quando contextual.**

---

# 18. DOCUMENTAÇÃO CANÔNICA

Projete uma estrutura mínima de documentação.

Possível estrutura:

docs/
architecture/
decisions/
domains/
operations/

Mas escolha apenas o que o projeto realmente necessita.

Para cada categoria, defina:

* propósito;
* quando atualizar;
* quem é fonte da verdade;
* o que NÃO pertence ali.

---

# 19. CONTEXT INDEX

Crie um índice extremamente compacto que permita a qualquer agente descobrir onde está cada tipo de conhecimento.

Exemplo conceitual:

Product behavior → openspec/specs/
Current change → openspec/changes/
Architecture → docs/architecture/
Decisions → docs/decisions/
Domain detail → docs/domains/
Operational commands → CLAUDE.md
Database contracts → ...
API contracts → ...

Adapte para este projeto.

Este índice deve ser pequeno e de altíssimo valor.

---

# 20. DEFINIÇÃO DE PRONTO PARA IMPLEMENTAÇÃO

Não considere a especificação concluída apenas porque existem arquivos.

Antes de autorizar implementação, valide:

### Produto

* objetivo entendido;
* atores identificados;
* fluxos principais definidos;
* comportamento esperado explícito.

### Specs

* requisitos testáveis;
* cenários relevantes;
* unhappy paths;
* permissões;
* invariantes.

### Arquitetura

* boundaries claros;
* principais componentes definidos;
* data flow compreendido;
* integrações identificadas;
* decisões críticas justificadas.

### Dados

* entidades principais;
* lifecycle;
* ownership;
* consistência;
* migrations quando relevante.

### Segurança

* authentication;
* authorization;
* trust boundaries;
* secrets;
* PII;
* abuse cases relevantes.

### Operação

* observabilidade;
* falhas;
* retry;
* idempotência;
* degradação;
* recuperação.

### Desenvolvimento

* estratégia de testes;
* critérios de aceite;
* tarefas implementáveis;
* dependências entre tarefas.

---

# 21. QUALITY GATES

Antes de terminar, faça uma revisão adversarial da própria especificação.

Pergunte:

* O que está ambíguo?
* O que pode ser interpretado de duas maneiras?
* O que um desenvolvedor poderia construir errado mesmo seguindo esta documentação?
* Onde faltam failure modes?
* Onde existe coupling desnecessário?
* Quais requisitos contradizem outros?
* Quais decisões são prematuras?
* Quais regras estão excessivamente rígidas?
* Quais regras estão vagas demais?
* Que contexto global poderia ser removido?
* Que conhecimento importante não sobreviveria a uma nova sessão?
* Onde estamos duplicando informação?
* Qual documentação provavelmente ficará obsoleta?
* Quais partes podem ser substituídas por validação automatizada?

Corrija os problemas encontrados.

Depois da revisão manual acima, rode `/better-harness` (plugin instalado) para uma auditoria evidence-based do próprio agent workflow que este harness define — cobre Task Understanding, Controlled Execution, Change Validation, Reliable Delivery e Learning Capture. Trate o relatório como a checklist do §20 é tratada: hipótese a confirmar, não verdade automática. Não rode antes desta fase — antes disso não há harness/specs para auditar.

---

# 22. REGRAS EXECUTÁVEIS > PROSA

Sempre que uma regra puder ser garantida por:

* type system;
* schema;
* linter;
* formatter;
* unit test;
* integration test;
* contract test;
* CI;
* static analysis;

prefira automatizar a regra em vez de depender da memória do LLM.

Documentação deve explicar decisões.

Ferramentas devem garantir invariantes quando possível.

---

# 23. NÃO DUPLICAR ESTADO

Uma informação importante deve possuir UMA fonte canônica.

Exemplos:

Comportamento → OpenSpec.

Decisão arquitetural → decision record.

Comando → package/config/CLAUDE.md conforme adequado.

Schema de dados → schema executável.

Não copie a mesma informação para cinco documentos.

Use referências.

---

# 24. PREPARAÇÃO PARA MULTI-AGENT

Embora Claude esteja realizando a especificação inicial, o sistema deverá funcionar bem caso implementação e revisão sejam feitas por outros agentes.

Portanto:

* não dependa de memória exclusiva desta conversa;
* materialize decisões importantes no repositório;
* não escreva instruções dependentes de características exclusivas de Claude quando puder usar padrões portáveis;
* permita que Codex, Claude ou outro coding agent consiga descobrir rapidamente o estado correto;
* mantenha OpenSpec como contrato compartilhado.

CLAUDE.md pode conter otimizações específicas para Claude, mas a verdade do produto não deve depender dele.

---

# 25. HANDOFF PARA IMPLEMENTAÇÃO

Ao terminar, produza um protocolo curto para o agente implementador.

Ele deve conseguir entrar sem conhecer esta conversa.

Algo conceitualmente equivalente a:

1. leia o contexto mínimo;
2. consulte a change ativa;
3. leia os specs relevantes;
4. leia design;
5. execute tasks em ordem;
6. teste;
7. valide contra spec;
8. registre decisões novas relevantes;
9. não altere comportamento especificado silenciosamente.

Crie a versão adequada ao projeto.

---

# 26. HANDOFF PARA CODE REVIEW

Produza também um protocolo de revisão independente.

O reviewer deverá verificar, nesta ordem:

1. aderência ao OpenSpec;
2. correção;
3. segurança;
4. edge cases;
5. regressões;
6. arquitetura;
7. testes;
8. simplicidade;
9. performance relevante;
10. manutenção.

Ele não deve presumir que uma escolha está correta apenas porque aparece na implementação.

---

# 27. ENTREGÁVEIS

Ao final desta missão eu espero, conforme aplicável:

A. mapa conceitual do sistema;

B. decomposição dos domínios;

C. OpenSpec inicializado/configurado corretamente;

D. `openspec/config.yaml` otimizado;

E. conjunto inicial de specs;

F. proposal/design/tasks apropriados para construir o sistema;

G. CLAUDE.md enxuto e de alto valor;

H. contexto local por domínio apenas onde necessário;

I. estrutura mínima de documentação;

J. registros das decisões arquiteturais realmente importantes;

K. context/index map;

L. política de memória e atualização;

M. política de autonomia GREEN/YELLOW/RED;

N. protocolo de implementação;

O. protocolo de code review;

P. lista explícita de perguntas ainda abertas;

Q. lista explícita de hipóteses assumidas;

R. relatório de inconsistências encontradas no blueprint;

S. relatório final de readiness (inclui o relatório do `/better-harness`, rodado nesta etapa conforme §21).

---

# 28. NÃO IMPLEMENTAR O PRODUTO AINDA

Esta missão termina quando tivermos uma especificação e um harness excelentes.

NÃO implemente features do produto.

NÃO transforme tasks em código.

NÃO execute `/opsx:apply`.

Você pode criar somente arquivos/configurações necessários para:

* especificação;
* documentação;
* OpenSpec;
* harness;
* memória;
* regras;
* estrutura de planejamento.

---

# 29. PROCESSO DE TRABALHO

Trabalhe em ciclos:

DISCOVER
→ MODEL
→ SPECIFY
→ CHALLENGE
→ REFINE
→ VALIDATE

Não gere dezenas de documentos de uma vez sem voltar ao blueprint.

Após cada fase importante, confronte o resultado contra a intenção original.

---

# 30. PRINCÍPIO FINAL

O resultado ideal não é o sistema com mais regras.

É o sistema no qual um agente competente consegue tomar decisões boas com a menor quantidade necessária de contexto.

Quero:

**forte direção sem microgerenciamento.**

**memória sem acúmulo de lixo.**

**autonomia sem deriva.**

**especificação sem burocracia.**

**contexto suficiente sem desperdício de tokens.**

**decisões persistentes sem fossilizar o projeto.**

**qualidade garantida por ferramentas sempre que possível, e por instruções somente quando necessário.**

Agora:

1. estude completamente o blueprint e o repositório;
2. apresente primeiro seu modelo mental do sistema, riscos, ambiguidades e proposta de estrutura;
3. só então construa os artefatos de especificação e harness;
4. revise adversarialmente seu próprio resultado;
5. valide o OpenSpec;
6. encerre com um relatório de readiness para implementação.

Se encontrar uma decisão verdadeiramente RED que altere substancialmente produto, segurança, contratos públicos ou arquitetura irreversível, não a esconda atrás de uma suposição: destaque-a para decisão humana.

Para todo o restante, exerça julgamento de arquiteto sênior e avance.
