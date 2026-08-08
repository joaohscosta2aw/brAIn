# Better Harness Task-Loop Report

## At a Glance

- Loop Effectiveness: 54/100 (changes only after comparable later task outcomes)
- Asset Health / Repair Progress: 0/100 (0 verified, 0 partial, 9 pending)
- Demonstrated autonomy radius: not observed (not observed; not observed confidence)
- Strongest loop: Not enough evidence difference to name one.
- Largest observed leak: Use the priority moves; no single loop is uniquely weakest.
- Top expected gain: No priority benefit is available in this evidence boundary.

## What You Can Rely On Today

- No reliable user outcome has been demonstrated in this evidence boundary yet.

## What You Gain Next

- No priority Harness move is available in this evidence boundary.



### Why these moves matter

### A parte de maior risco da change é a única sem critério verificável de pronto
- Priority: High · Evidence: not observed in this boundary
- Reason: O design.md nomeia 'cinco fontes de uso não documentadas e instáveis' como o maior risco de cronograma, e tasks.md §6 endereça isso com cinco itens cuja definição de pronto é 'identificar a fonte de uso, declarar tier e campos disponíveis'. Não há cenário de spec para nenhum provider concreto, nem fixture ou amostra capturada prevista, e §8.1 só exige cobrir cenários que são todos agnósticos de provider. Um adapter pode ser marcado como concluído declarando tier degradado sem nunca ter lido dado real, e a lacuna aparece como cobertura declarada em vez de falha.
- Expected Output:
  1. Cada adapter só pode ser marcado concluído contra dado real capturado, e cobertura declarada deixa de ser autocertificada.

### O ambiente não está declarado justamente onde a primeira task é a mais cara de desfazer
- Priority: High · Evidence: not observed in this boundary
- Reason: A change abre criando esquema SQLite, migrações versionadas e traits de storage — as decisões mais caras de reverter do projeto — e o repositório não declara toolchain, versão de Rust, comando de build, comando de teste, localização do arquivo de banco nem rotina de reset. O protocolo de implementação manda 'rode os testes' e o de revisão abre com 'compila, lint, formatação, testes passam', ambos contra um ambiente que ninguém definiu. Além disso, o protocolo prescreve `openspec list` como rota canônica sem que nada declare a CLI OpenSpec como requisito.
- Expected Output:
  1. A fundação irreversível é construída sobre um ambiente declarado, não improvisado por quem pegar a task primeiro.

### Todas as regras do projeto são prosa e o único portão mecânico existente foi removido
- Priority: High · Evidence: not observed in this boundary
- Reason: As regras são excepcionalmente concretas — SQL confinado a storage/ (D-9), equivalente nunca somado ao pago, ausente diferente de zero e de desconhecido, isolamento por construção — e dependem inteiramente de o agente lembrar de lê-las. Nenhum CI, pre-commit, config de lint ou formatter existe; o diretório .github/ foi removido durante a curadoria de espelhos de ferramenta. O hook restante é best-effort e só age sobre código que ainda não existe. Na sessão auditada, todos os episódios que produziram mudança fecharam sem checagem revisada: a validação que ocorreu foi por disciplina do operador, e nada teria capturado sua omissão. Isso contradiz o próprio princípio §22 do projeto, que manda preferir regra executável a prosa.
- Expected Output:
  1. As invariantes que o projeto mais teme violar passam a ser garantidas por ferramenta, não por memória do agente.

### A exigência de revisor independente não tem nenhum mecanismo que a sustente
- Priority: Medium · Evidence: not observed in this boundary
- Reason: O protocolo de revisão exige revisor independente e revisão humana obrigatória para caminho do dinheiro, segurança, contrato de CLI e qualquer decisão RED. Nenhum desses gatilhos tem mecanismo: sem PR, sem proteção de branch, sem CODEOWNERS, com commits diretos. A única evidência de aceitação prevista é o implementador marcar checkbox no próprio tasks.md. Num projeto cujo produto é atribuição de custo a cliente, o erro que o protocolo mais teme — cobrar o cliente errado — é exatamente o que o processo permite autoaprovar. Atenuante: repositório local de operador único, onde humano e implementador podem ser a mesma pessoa por desenho.
- Expected Output:
  1. O caminho do dinheiro deixa de depender de autoaprovação para chegar à branch principal.

### O contexto sempre-carregado afirma que os hooks estão inertes, e eles não estão
- Priority: Medium · Evidence: not observed in this boundary
- Reason: CLAUDE.md declara que os hooks são 'inertes enquanto não houver código'. A evidência contradiz: os hooks só abortam se `command -v code-review-graph` falhar, o binário resolve no PATH e o diretório é repositório git. Portanto `code-review-graph update` executa a cada Edit e Write — inclusive nos arquivos .md que são hoje o único conteúdo do repositório — com timeout de 30s, e `status` executa a cada início de sessão. Uma afirmação falsa em contexto sempre-carregado desorienta toda decisão futura sobre custo e efeito desses hooks.
- Expected Output:
  1. O contexto permanente para de afirmar algo falso sobre o comportamento automatizado do repositório.

### A configuração MCP contradiz a portabilidade que AGENTS.md declara
- Priority: Medium · Evidence: not observed in this boundary
- Reason: O .mcp.json versionado fixa um caminho absoluto de máquina como diretório de trabalho, o que aponta o servidor para um diretório inexistente em qualquer clone em outro caminho. Somado ao runner de pacote sem versão fixada, o mesmo arquivo carrega um problema de portabilidade e um de reprodutibilidade — enquanto AGENTS.md é declarado canônico e portável, e o projeto se prepara explicitamente para ser trabalhado por outros agentes.
- Expected Output:
  1. Outro agente ou outra máquina consegue usar o projeto sem corrigir configuração à mão.

### O procedimento OpenSpec existe duplicado em duas superfícies de asset
- Priority: Low · Evidence: not observed in this boundary
- Reason: As cinco skills openspec-* e os cinco comandos opsx/* cobrem o mesmo procedimento par a par, somando cerca de mil linhas de procedimento, todas com advisory de ausência de disclosure progressiva e com o conteúdo integral inline. O ambiente ainda lista skills de escopo externo com as mesmas descrições, o que sugere uma terceira cópia. Qual rota é canônica é indeterminado, e o custo de descrição carregada é permanente.
- Expected Output:
  1. Uma rota por operação, sem cópia concorrente disputando seleção.

### Quatro skills pressupõem um código que o próprio AGENTS.md declara inexistente
- Priority: Low · Evidence: not observed in this boundary
- Reason: As skills de exploração, depuração, refatoração e revisão via grafo instruem consultar estatísticas e visão de arquitetura de um grafo de código. AGENTS.md afirma que o projeto está em fase de especificação e não tem código. CLAUDE.md reconhece a ressalva em prosa, mas nenhuma skill traz o gate na própria descrição, que é o que governa o roteamento — então elas podem ser selecionadas mesmo assim, e suas descrições custam contexto permanentemente sem alvo possível hoje.
- Expected Output:
  1. O roteamento para de oferecer capacidade que não tem alvo, e o custo de contexto some enquanto ela for inútil.

### Duas rotas do mapa de contexto não resolvem para o destino esperado
- Priority: Low · Evidence: not observed in this boundary
- Reason: AGENTS.md roteia 'Comportamento aprovado' para openspec/specs/, diretório que ainda não existe — comportamento esperado antes do primeiro arquivamento, mas a rota não avisa isso a quem chega. E a autoridade de resolução de conflito entre ferramentas é citada, em AGENTS.md e no protocolo de revisão, por um nome de arquivo com vírgula no lugar do ponto. O arquivo existe literalmente assim, então nenhuma verificação estrutural acusa; mas é o documento invocado justamente no momento de conflito, quando há menos margem para adivinhar.
- Expected Output:
  1. O mapa de contexto, que é hoje o principal ativo do projeto, para de ter rota que morre em silêncio.

## Five Lifecycle Dimensions

| Dimension | What the evidence proves | Evidence boundary | Summary | Boundary / blocker |
| --- | --- | --- | --- | --- |
| Task Understanding | Not observed yet | not observed in this boundary | Dimensão mais forte. AGENTS.md carrega leis, fronteira e índice de roteamento; a change declara não-objetivos e conformidade. Limitado por duas rotas do mapa que não resolvem e por uma afirmação falsa no contexto sempre-carregado. | not observed |
| Controlled Execution | Not observed yet | not observed in this boundary | Nenhum manifesto, toolchain, versão, comando de setup, build, teste ou reset existe. Os protocolos mandam 'rode os testes' e 'compila, lint, formatação' sem nomear um comando. Ausência material, não indisponibilidade de coleta. | not observed |
| Change Validation | Not observed yet | not observed in this boundary | A intenção de cobertura é acima da média e alguns requisitos já embutem o teste que os falseia. Mas nenhum portão mecânico existe, e todos os episódios de mudança da sessão fecharam sem checagem revisada. | not observed |
| Reliable Delivery | Not observed yet | not observed in this boundary | Commits diretos, sem PR, sem proteção de branch, sem CODEOWNERS. A evidência de aceitação prevista é o próprio implementador marcar checkbox. O design acerta ao declarar ausência de efeito externo e reversão por deleção do banco. | not observed |
| Learning Capture | Not observed yet | not observed in this boundary | Mecanismos duráveis existem e são bem desenhados (política L0-L3, DECISIONS.md, protocolos). Nenhum foi exercitado ainda, e não há duas Task Episodes comparáveis para sustentar qualquer alegação de efeito. | not observed |

## The 15 Small Checks

| Dimension | Small check | What the evidence proves | Evidence boundary |
| --- | --- | --- | --- |


## Evidence and Boundaries

- Episode coverage: 0 episodes, 0 edited, 0 closed, 0 repaired-and-passed
- Model: agent-work-loop-v4
- Session selection: not observed; 0 sessions analyzed of 0 eligible sessions; not observed confidence
- Delivery grades observed: not observed
- Source gaps: not observed
- Learning comparison: Not observed; 0 declared intervention(s)
