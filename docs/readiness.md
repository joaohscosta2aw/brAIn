# Relatório de Readiness

Estado da especificação e do harness em 2026-08-08, ao final da missão descrita
em `Prompts/PrimeirosPassos.md`.

Este documento é um instantâneo. As perguntas abertas e hipóteses aqui devem ser
resolvidas ou revisadas conforme o projeto avança — não são verdades permanentes.

---

## 1. Inconsistências encontradas no blueprint

O `BRIAN-BLUEPRINT.md` (v0.1-draft) e o `BRIAN-BLUEPRINT-V1.md` (v1.0) não são
fontes independentes: o segundo é revisão crítica do primeiro. O README já
declara o V1 como canônico e o draft como histórico. As divergências abaixo são
deliberadas, mas importam para quem for citar o documento.

### Divergências entre as duas versões

| Tema | v0.1-draft | v1.0 (canônico) |
|---|---|---|
| Workflow engine | 10 fases fixas em código | dado YAML versionado, fast path de 3 fases (D-3) |
| Context Governor | subsistema central do produto | hipótese isolada H-1, com critério de descarte |
| Storage | spike SQLite vs SurrealDB | SQLite direto, spike cancelado (D-1) |
| MVP | app macOS, 6 telas, 13 capacidades | CLI pura, 6 subsistemas, sem UI |
| IPC | XPC nativo | Unix socket + JSON-RPC |
| Riscos | ausente | §116, 10 riscos registrados |
| D-16 / D-17 | não existem | núcleo declarado do produto |

A mudança de posicionamento é a mais relevante: o draft se define como
orquestrador multi-agente de workflow; o V1 se define primeiro como controlador
de capacidade e continuidade, com orquestração vindo só na v0.2.

### Ambiguidades ainda em aberto

1. **Numeração paralela.** O V1 preserva a numeração de 108 seções do draft para
   comparação lado a lado, mas reescreve o conteúdo. Uma citação a "§N" é
   ambígua sem dizer qual arquivo.

2. **Ordem dos primeiros changes** aparece em dois lugares
   (`docs/PREMISSAS-BASICAS.md` e `BRIAN-BLUEPRINT-V1.md` §107) com ordenação
   ligeiramente diferente no meio da lista. Ambos concordam que
   `client-cost-attribution` é o primeiro.

3. **Granularidade do próximo passo** difere entre o README e as premissas.
   Não é contradição, mas convém alinhar antes de sequenciar.

4. **Nome do repositório.** O remoto é `brAIn` e o diretório local é `BrIAn`,
   enquanto **D-11** determina que o runtime se chama "Brian Core" e proíbe o
   termo "Brain". Vale alinhar.

5. **Modelo de negócio ausente.** Nenhum dos documentos define preço, licença ou
   público-alvo do Brian além do próprio autor. O blueprint registra isso como
   risco R-6 (dogfooding), não como lacuna a resolver agora.

---

## 2. Hipóteses assumidas

Do blueprint, já declarada como hipótese:

- **H-1 — Context Governor.** Redução de custo ≥30% via contexto pré-montado.
  Isolada por construção; nada depende dela. Critério de descarte definido a
  priori pelo próprio blueprint.

Assumidas por mim durante esta missão, e que devem ser derrubadas se a realidade
discordar:

- **Importação sob demanda basta para a v0.0.** Sem processo residente, o
  consumo só aparece no ledger quando o operador importa. Se a latência entre
  gasto e visibilidade incomodar na prática, isso vira argumento para antecipar
  o daemon.
- **As fontes de uso dos providers são estáveis o bastante para deduplicação.**
  A idempotência depende de identificador estável ou de impressão digital
  reproduzível. Não foi verificado em nenhum dos cinco providers.
- **Catálogo de preço versionado por vigência é suficiente** para manter o custo
  equivalente reproduzível ao longo do tempo.
- **Dois valores de custo por registro bastam.** Custo pago e equivalente em API.
  A alocação por plano entra como terceiro na change seguinte.

---

## 3. Perguntas em aberto

Ordenadas por consequência.

### Bloqueantes de negócio

1. **R-4 — Termos de uso.** Planos por assento podem proibir leitura automatizada
   de artefatos de sessão. Vale para os cinco providers. O blueprint classifica
   como o único risco capaz de inviabilizar o modelo independentemente da
   qualidade técnica. **Exige verificação humana antes de a v0.0 depender de
   qualquer fonte de assinatura.** O desenho permite operar só com providers de
   API caso a resposta seja negativa.

### Bloqueantes técnicos da change #1

2. **Quais dos cinco providers expõem fonte de uso utilizável, e em que tier?**
   Desconhecido até que os adapters sejam tentados. Determina se a v0.0 fecha
   com cobertura total ou parcial.

3. **Existe identificador estável de chamada em cada provider?** Sem ele, a
   deduplicação cai para impressão digital, com colisão possível em chamadas
   idênticas no mesmo instante.

### Não bloqueantes

4. Retenção do ledger a longo prazo.
5. Alinhamento do nome do repositório com D-11.
6. Modelo de negócio do próprio Brian.

---

## 4. Estado dos entregáveis

| Entregável | Estado | Onde |
|---|---|---|
| Mapa conceitual e domínios | feito | `AGENTS.md`, §5 abaixo |
| OpenSpec inicializado e configurado | feito | `openspec/config.yaml` |
| Specs iniciais | feito | change #1, 2 capabilities |
| Proposal / design / tasks | feito | `openspec/changes/client-cost-attribution/` |
| CLAUDE.md enxuto | feito | ponteiro para `AGENTS.md` |
| Documentação mínima | feito | `docs/`, `docs/harness/` |
| Índice de contexto | feito | `AGENTS.md` |
| Política de memória | feito | `docs/harness/autonomia-e-memoria.md` |
| Autonomia GREEN/YELLOW/RED | feito | idem |
| Protocolo de implementação | feito | `docs/harness/protocolo-implementacao.md` |
| Protocolo de code review | feito | `docs/harness/protocolo-revisao.md` |
| Decisões arquiteturais | reaproveitado | `docs/DECISIONS.md` (D-1..D-17), sem ADR paralelo |
| Perguntas abertas / hipóteses / inconsistências | feito | este documento |
| Auditoria do harness | ver §6 | `/better-harness` |

---

## 5. Fronteiras de domínio propostas

Fronteiras conceituais que sobrevivem às versões do produto. Apenas `capacity`
tem spec escrita — as demais são mapa, não compromisso.

| Domínio | Cobre |
|---|---|
| `capacity` | ledger de uso, custo, atribuição, janelas, planos, budget, FinOps |
| `context` | tenancy cliente/projeto, identidade, troca de contexto |
| `continuity` | Continuity Pack, handoff entre LLMs, memória |
| `providers` | registry, adapters, tiers de integração, roteamento |
| `execution` | run, worktree, workflow, recuperação |
| `governance` | policy, gates de qualidade e segurança, vault |
| `observability` | telemetria, auditoria, eval harness |

Especificar os demais agora seria decidir sobre subsistemas de v0.2+ que o
próprio blueprint ainda não resolveu.

---

## 6. Auditoria independente do harness

Executada com `better-harness` (QoderAI) sobre o modelo Agent Work Loop: coleta de
evidência versionada, três agentes de evidência independentes e read-only
(sessão, harness do projeto, assets de agente), reconciliação e regradagem pelo
lead. Relatório completo em `docs/auditoria/report.html`.

### Nota sobre a evidência

A primeira coleta usou janela congelada até a meia-noite do dia corrente e
retornou **zero** sessões — o que teria produzido uma auditoria vazia. Como todo
o trabalho deste projeto aconteceu no mesmo dia, a janela excluía 100% da
evidência. Corrigida a janela, a coleta passou a `session-rich`: 1 sessão
elegível, 34 Task Episodes, confiança alta.

Duas limitações permanecem registradas: o portfólio de episódios veio truncado
por orçamento de candidatos, e a própria execução da auditoria foi contabilizada
como atividade do projeto, inflando artificialmente contagens de handoff. São
vieses de instrumentação, não defeitos do projeto.

### Pontuação (Loop Effectiveness)

| Dimensão | Nota | Leitura |
|---|---|---|
| Task Understanding | 72 | mais forte; limitada por rotas que não resolvem |
| Learning Capture | 55 | mecanismos bem desenhados, nenhum exercitado |
| Reliable Delivery | 50 | commits diretos, aceitação por checkbox |
| Change Validation | 48 | intenção acima da média, zero portão mecânico |
| Controlled Execution | 45 | ambiente não declarado em lugar nenhum |

As notas são limitadas por teto de evidência: sem código, sem testes e sem rota
de execução declarada, nenhuma dimensão pode passar de `Present`. São
conservadoras por construção, não julgamento de qualidade da especificação.

### Achados

**Alta severidade**

1. **A parte de maior risco da change é a única sem critério verificável de
   pronto.** Os cinco adapters de provider — que o próprio design nomeia como
   maior risco de cronograma — têm como definição de pronto "declarar tier e
   campos disponíveis". Um adapter pode ser marcado concluído sem nunca ter lido
   dado real, e a lacuna aparece como cobertura declarada em vez de falha.
2. **O ambiente não está declarado justamente onde a primeira task é a mais cara
   de desfazer.** A change abre criando esquema e migrações, e não há toolchain,
   comando de build, teste, reset nem localização do banco em lugar nenhum —
   enquanto os protocolos mandam "rode os testes" e "compila, lint, formatação".
3. **Todas as regras do projeto são prosa, e o único portão mecânico existente
   foi removido.** Contradiz o princípio §22 do próprio projeto (regra executável
   antes de prosa). Na sessão auditada, toda mudança fechou sem checagem
   revisada: a validação que houve foi por disciplina, e nada capturaria sua
   omissão.

**Média severidade**

4. A exigência de revisor independente não tem mecanismo que a sustente — sem PR,
   sem proteção de branch, aceitação por checkbox do próprio implementador.
5. `CLAUDE.md` afirma que os hooks estão inertes; eles executam a cada Edit e
   Write, inclusive nos `.md` que são hoje o único conteúdo do repositório.
6. `.mcp.json` fixa caminho absoluto de máquina e runner sem versão, contradizendo
   a portabilidade que `AGENTS.md` declara.

**Baixa severidade**

7. Procedimento OpenSpec duplicado em duas superfícies de asset.
8. Quatro skills de grafo pressupõem código que `AGENTS.md` declara inexistente.
9. Duas rotas do mapa de contexto não resolvem: `openspec/specs/` ainda não existe,
   e a autoridade de conflito é citada por nome de arquivo malformado.

---

## 7. Veredito

**A especificação está pronta para implementação. O harness de execução não.**

O que está sólido: o produto está compreendido, os domínios têm fronteira, a
change #1 tem requisitos testáveis com unhappy paths, invariantes explícitas e
não-objetivos declarados. Vários requisitos já vêm com o teste que os falseia
embutido. As decisões travadas são obedecidas e citadas. O caminho do dinheiro
foi corretamente classificado como RED.

O que falta é mecanismo, e a concentração dos achados é significativa: das cinco
dimensões, as quatro mais baixas apontam para a mesma causa — **o projeto sabe o
que quer, mas nada além da disciplina do operador garante que aconteça.**

Antes de executar `tasks.md` 1.1, três coisas deveriam ser resolvidas:

1. **Declarar o ambiente** (achado 2). É pré-requisito da própria primeira task e
   barato de fazer agora.
2. **Dar critério de pronto aos adapters** (achado 1). É onde a v0.0 vai falhar
   silenciosamente se falhar.
3. **Tornar executável ao menos uma invariante** (achado 3). O projeto já decidiu
   isso em §22; falta cumprir.

Fora do escopo técnico, permanece bloqueante de negócio a verificação de **R-4**:
se os termos de uso dos planos por assento proíbem leitura automatizada de
artefatos de sessão, o desenho precisa operar só com providers de API — e isso é
decisão humana, não de engenharia.
