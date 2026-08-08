# Brian
## Plano de Controle de Engenharia de IA para macOS e Kubernetes Enterprise

> **Status:** Product & Architecture Blueprint — revisão crítica do v0.1-draft
> **Versão:** 1.0
> **Plataforma primária:** macOS
> **Evolução enterprise:** Kubernetes
> **Idioma:** português; termos técnicos preservados em inglês
> **Ideia central:** a IA deve **poupar ~90% do trabalho e do tempo** — **não queimar dinheiro**. Brian é o plano de controle que garante isso: **zero token perdido** (capacidade, janelas, atribuição) + **memória compartilhada** para chavear LLM/provider **sem perda** de contexto, conversa, análise ou decisão.
> **Dois ganhos mínimos (invioláveis):**
> 1. **D-16** — nenhum token some; cada centavo da capacidade paga é visível, atribuído e aproveitado.
> 2. **D-17** — memória/continuidade do Brian; trocar Claude↔Codex↔outro **não reinicia a cabeça** do trabalho.

---

# Como ler este documento

Este documento mantém a espinha de 108 seções do `BRIAN-BLUEPRINT.md` (v0.1-draft), na mesma numeração, para comparação lado a lado. Cada seção foi reescrita, corrigida ou confirmada.

As seções **§109–§116** são novas. Elas cobrem lacunas do v0.1 que não eram opcionais: concorrência, recuperação, intervenção humana, avaliação, migração, onboarding, multi-repo e registro de riscos.

Três convenções aparecem ao longo do texto:

```text
[D-n]  Decisão travada. Vale até o critério de reversão ser atingido.
[H-n]  Hipótese sob teste. Não pode virar pilar de produto antes de medida.
[Δ]    Mudança relevante em relação ao v0.1-draft.
```

Uma decisão travada não é uma decisão permanente. É uma decisão que **para de ser discutida** até que a evidência definida no critério de reversão apareça. O objetivo é eliminar deliberação recorrente sobre pontos que não são o gargalo do projeto.

---

# Changelog em relação ao v0.1-draft

| # | Mudança | Seção |
|---|---------|-------|
| 1 | Storage decidido: SQLite. Spike SurrealDB cancelado. | §57–§61 |
| 2 | Sequenciamento invertido: FinOps CLI antes de orquestração e antes de UI. | §80–§85, §98–§100 |
| 3 | Workflow deixa de ser máquina de 10 fases fixas e vira dado versionado, com fast path de 3 fases como padrão. | §15 |
| 4 | Context Governor rebaixado de pilar para hipótese `H-1`, com experimento definido e critério de descarte. | §18–§20, §115 |
| 5 | Estratégia de adapter de provider passa a ser hierárquica: headless JSON → arquivos de sessão → PTY. | §8, §10, §56 |
| 6 | Custo passa a preferir o valor reportado pelo provider; price catalog vira fallback. | §40–§42 |
| 7 | Fronteira Workflow × Reasoning formalizada: reasoning propõe, workflow decide. | §15–§16 |
| 8 | Learning Engine adiado para além do v1.0, com limiar estatístico explícito. | §38 |
| 9 | Concorrência por `git worktree` promovida a regra arquitetural. | §109 |
| 10 | Runs passam a ser retomáveis por construção; persistência antes de efeito colateral. | §110 |
| 11 | Adicionado harness de avaliação como pré-requisito de roteamento adaptativo. | §112 |
| 12 | Nomenclatura unificada em "Brian Core". O termo "Brain" foi eliminado. | §96 |
| 13 | Escopo do MVP reduzido de 13 capacidades para 5. | §81 |
| 14 | Adicionado risco comercial de ToS de assinaturas como item de primeira classe. | §116 |
| 15 | Diferenciais reescritos para só incluir o que não é replicável dentro de um único provider. | §95 |
| 16 | **Controle de capacidade inviolável no minuto 0:** todo token, assinatura, janela de uso, % semanal/diária e tempo restante são first-class. Otimização de cada centavo não é feature opcional — é a lei do produto (D-16). | §2.9, §13, §43–§45, §81, §98 |
| 17 | **Missão e dois ganhos mínimos:** (1) zero token perdido + aproveitar cada centavo; (2) memória compartilhada para chavear LLM sem perda de contexto/conversa/análise. A IA poupa trabalho e tempo — não gasta dinheiro (D-16, D-17). | §1, §2.9–§2.10, §34, §104 |

---

# Registro de decisões travadas

| ID | Decisão | Critério de reversão |
|----|---------|----------------------|
| **D-1** | SQLite como banco operacional único, atrás de traits. | Uma query de produção real exceder 200 ms com 12 meses de dados. |
| **D-2** | Brian Core em Rust. CLI é a primeira interface. SwiftUI só no v0.3. | Nenhum. Reavaliar só se a CLI provar que o produto não se sustenta sem UI. |
| **D-3** | Workflow é dado versionado em YAML, não código. Padrão é o fast path de 3 fases. | Três workflows distintos precisarem de lógica que YAML não expressa. |
| **D-4** | Adapters seguem a hierarquia headless JSON → arquivos de sessão → PTY. PTY nunca é a primeira escolha. | Um provider relevante não oferecer nenhuma das duas primeiras opções por dois releases seguidos. |
| **D-5** | Context Governor é hipótese, não subsistema. Nada depende dele arquiteturalmente. | `H-1` confirmada com redução ≥ 30% de custo total por change bem-sucedido. |
| **D-6** | Custo real reportado pelo provider tem precedência. Price catalog é fallback declarado. | Nenhum. |
| **D-7** | Todo run executa em um `git worktree` dedicado. | Nenhum. É pré-requisito de concorrência. |
| **D-8** | Router é baseado em regras explícitas + override manual até `N ≥ 30` runs por célula. | Limiar estatístico atingido e harness de eval funcionando. |
| **D-9** | Todo acesso a storage passa por trait. Nenhuma query SQL fora de `storage/`. | Nenhum. |
| **D-10** | Brian não orquestra o inner loop de um único provider. Só atua onde há N providers ou M clientes. | Nenhum. É a definição de escopo do produto. |
| **D-11** | O runtime chama-se "Brian Core". O termo "Brain" não é usado. | Nenhum. |
| **D-12** | Estado do run é persistido antes de qualquer efeito colateral externo. | Nenhum. |
| **D-13** | Roteamento adaptativo só existe depois do harness de eval (§112). | Nenhum. |
| **D-14** | Memória é append-only com proveniência. Correções criam novo registro que supersede o anterior. | Nenhum. |
| **D-15** | O MCP Server do Brian é o único caminho de escrita no estado de workflow. | Nenhum. |
| **D-16** | **Zero token perdido — lei inviolável desde o minuto 0.** Todo token/request observável entra no ledger; nenhum some em silêncio. Janelas, %, restante, burn, assinatura e atribuição são first-class. Capacidade paga existe para **trabalho útil**, não para desperdício. Sem isso o Brian está fora de especificação. | Nenhum. Maior ganho econômico do produto junto com D-17. |
| **D-17** | **Memória/continuidade compartilhada é ganho mínimo co-primário.** Ao chavear provider ou modelo sob o mesmo Context, o Brian preserva e reinsere o estado de trabalho (objetivo, decisões, análise, arquivos tocados, erros, próximos passos, evidências) sem o usuário reexplicar. Memória pertence ao Brian, não ao LLM. Perder contexto na troca de LLM é bug de produto. | Nenhum. É o que permite multi-LLM sem pagar duas vezes o mesmo raciocínio. |

---

# 1. Sumário Executivo

## 1.0 Missão (não negociável)

```text
A IA existe para RESOLVER — com direção clara, de forma assertiva,
com poucas indas e vindas — e assim me poupar ~90% do trabalho e do tempo.
A IA NÃO existe para gastar meu dinheiro.

Brian existe para garantir:
  (A) ZERO TOKEN PERDIDO — vejo, controlo e aproveito cada unidade de capacidade paga
  (B) ZERO CONTEXTO PERDIDO AO CHAVEAR LLM — memória compartilhada carrega conversa,
      análise, decisões e estado do trabalho entre providers/modelos
  (C) OPERAÇÃO EFICIENTE — tuning contínuo, tech atual, feedback do uso real
```

### Princípios operacionais (OP) — não podem escapar

Detalhe canônico: `docs/PREMISSAS-BASICAS.md` §1.1.

```text
OP-1  Eficiência na utilização       cada token / % de janela trabalha
OP-2  Tuning contínuo                medir → ajustar → medir
OP-3  Tecnologia sempre atualizada   workers, modelos e adapters vivos
OP-4  Inteligência retroalimentada   uso real alimenta o próximo ciclo
OP-5  IA para resolver               fechar trabalho, não “conversar”
OP-6  Direcionamento claro           objetivo e critério antes do gasto
OP-7  Trabalhos assertivos           menor ação correta + evidência
OP-8  Poucas indas e vindas          sem reexplicar, sem rework cego
```

Esses dois ganhos (A)(B) + os OP são o **mínimo** para o Brian valer a pena. Tudo o mais (workflow, UI, graph, learning) é secundário até (A), (B) e OP-1…OP-8 estarem no DNA do produto.

---

Brian é um **plano de controle de engenharia de IA**, entregue primeiro como binário de linha de comando e depois como aplicação nativa macOS.

Brian não substitui agentes de código como Claude Code, Codex CLI, Gemini CLI, Grok CLI ou ZCode. Também não compete com o que esses agentes fazem *dentro* de uma sessão. Essa distinção é a mudança mais importante em relação ao v0.1-draft.

**[Δ]** O v0.1-draft posicionava Brian como orquestrador de workflow sobre providers. Isso colide frontalmente com o que os próprios providers passaram a oferecer nativamente: plugins versionados, subagents, hooks de ciclo de vida, modo headless com saída JSON estruturada contendo `session_id`, `total_cost_usd` e detalhamento de tokens. Um produto que orquestra fases dentro de um provider está competindo com uma feature gratuita que melhora sozinha.

Brian se define pelo que **nenhum provider individual pode fazer**:

```text
Um provider vê    uma sessão.
Brian vê          N providers × M clientes × T tempo
                  + cada token + a memória que sobrevive à troca de LLM.
```

### Dois ganhos mínimos (pilares)

| Pilar | Lei | Promessa |
|-------|-----|----------|
| **Capacidade** | **D-16** | Nenhum token some. Janela, %, restante, burn, assinatura, dono (cliente). Cada centavo trabalha. |
| **Continuidade** | **D-17** | Chavear LLM/provider **não apaga** objetivo, conversa útil, análise, decisões, erros, próximos passos. |

Disso decorre o escopo defensável completo:

```text
Capacidade       nenhum token perdido; % e tempo restante da assinatura/API
Atribuição       qual cliente consumiu qual token — e por quê
Otimização       trabalho útil por centavo / por % de janela (não gastar à toa)
Continuidade     memória compartilhada para chavear LLM sem reexplicar o mundo
Identidade       conta certa por cliente, isolada do desktop pessoal
Comparação       qual worker resolve melhor — sem pagar o contexto duas vezes
Governança       política, aprovação, o que foi tocado
```

**[Δ][D-16][D-17]** Capacidade e continuidade **não são roadmap**. Sem elas o Brian está **fora de especificação**. Workflow bonito com token sumindo ou com “recomeça a história no Codex” é fracasso de produto.

A abstração central continua sendo **Context**.  
Os eixos invioláveis são **Capacity** (D-16) e **Continuity Memory** (D-17).

O usuário diz:

```text
brian connect xpto
```

E Brian ativa um contexto operacional completo: identidade do cliente, repositório, projeto, OpenSpec, **namespace de memória e continuity pack**, identidades de provider, organização GitHub, ambientes, MCP servers, ferramentas, secrets, políticas, budgets, telemetria, **estado de capacidade por provider/plano** e atribuição de custo.

A separação fundamental permanece válida e é confirmada:

```text
OpenSpec         → define O QUE deve ser construído
Brian            → decide COMO e COM QUAL IDENTIDADE o trabalho acontece
Providers        → executam trabalho cognitivo (workers substituíveis)
Tools            → fornecem capacidades
Gates            → verificam qualidade e segurança
Memory           → preserva conhecimento e CONTINUIDADE entre LLMs     [D-17]
Telemetry        → explica o que aconteceu
Usage Control    → nenhum token some; otimiza cada centavo               [D-16]
FinOps           → atribui consumo a cliente e produz showback/chargeback
Vault            → protege identidades e secrets
```

**Brian é o ambiente onde a IA poupa tempo de verdade — porque o dinheiro é controlado e a cabeça do trabalho não se perde ao trocar de modelo.**

Sempre que houver dúvida de prioridade de engenharia:

```text
PILAR A — DINHEIRO / CAPACIDADE (D-16)
1. O token foi capturado? (zero loss)
2. Foi atribuído (ou unattributed explícito e ruidoso)?
3. Janela, % e restante refletem a verdade?
4. Estou queimando capacidade em lixo (retry cego, unattributed, modelo errado)?

PILAR B — TEMPO / CONTINUIDADE (D-17)
5. Se eu trocar de LLM agora, o próximo worker recebe o estado do trabalho?
6. Objetivo, decisões, análise, erros e próximos passos estão no Brian — não só no chat do provider?

Se qualquer resposta for "não", isso é P0. O resto espera.
```

---

# 2. Princípios de Produto

## 2.1 Contexto acima de prompt

Confirmado do v0.1. A unidade de trabalho é **Context + Run**, não o prompt.

Um prompt é efêmero. Um contexto contém:

```text
quem o usuário representa
qual cliente está ativo
qual projeto está ativo
quais repositórios estão disponíveis
quais identidades de provider são permitidas
quais secrets podem ser resolvidos
qual memória pode ser recuperada
quais políticas se aplicam
qual budget é dono do consumo
```

Esse é o princípio mais forte do documento original e a razão pela qual o produto tem chance de existir.

## 2.2 Providers são workers substituíveis

Confirmado. Claude, Codex, Gemini, Grok e ZCode são workers.

**[Δ]** Com uma qualificação que faltava: a substituibilidade tem custo. Cada adapter é um contrato instável mantido contra um binário de terceiro que muda sem aviso. O princípio não é "todos os providers são iguais", é "**nenhum provider é permanente**". Brian é projetado para sobreviver ao desaparecimento de qualquer um deles, não para tratá-los como intercambiáveis em tempo de execução sem custo.

Capacidades exigidas de um adapter, em ordem de obrigatoriedade:

```text
OBRIGATÓRIO
- detectar presença e versão
- executar uma tarefa e capturar resultado
- reportar sucesso/falha
- extrair consumo de tokens

DESEJÁVEL
- retomar sessão
- listar modelos
- cancelar execução
- reportar custo em USD

OPCIONAL
- reportar quota restante
- reportar limites de rate
- expor invocações de ferramenta
```

Um adapter que só cumpre o bloco obrigatório é um adapter válido. **[Δ]** O v0.1 tratava todos os itens como parte da mesma trait, o que produziria implementações cheias de `unimplemented!()`.

## 2.3 Determinístico antes de probabilístico

Confirmado sem alteração. É o princípio de melhor relação custo-benefício do documento.

```text
"Quem chama esta função?"        → code graph / AST
"Dependências têm CVE?"          → OSV Scanner
"Este diff tem secret?"          → secret scanner
"Os testes passam?"              → test runner
"O endpoint está no ar?"         → health check
"O que mudou?"                   → git diff
"Quanto custou?"                 → banco de FinOps
```

Só depois um LLM recebe a evidência e raciocina sobre ela.

Isso reduz tokens, latência, superfície de alucinação e custo — nessa ordem de importância.

## 2.4 Contexto mínimo por padrão

**[Δ] Rebaixado de princípio para hipótese.** Ver `H-1` em §18.

O v0.1 tratava minimização de contexto como verdade estabelecida. Não é. Duas forças trabalham contra:

**Prompt caching.** Leitura de cache custa uma fração do input não-cacheado. Um contexto grande e *estável* entre turnos pode custar menos que um contexto pequeno e *recalculado*, porque o segundo invalida o cache. Otimizar bytes sem modelar cache aumenta a conta.

**Busca agêntica.** O agente decide o que ler durante o loop, com o repositório inteiro disponível. Um pacote mínimo montado antes tende a chegar incompleto, e o agente lê os arquivos assim mesmo. Paga-se duas vezes.

Além disso, o custo dominante em coding agents não é o pacote inicial — é o **loop**, com N turnos carregando contexto acumulado. Reduzir o turno 1 em 40% mexe pouco no total.

O princípio revisado é mais defensável e mais barato:

> **Brian mede o custo do contexto antes de tentar reduzi-lo.**

Medir é v0.0. Reduzir é condicional ao resultado da medição.

## 2.5 Decisões explicáveis

Confirmado. Brian nunca deve parecer caixa preta.

```text
Por que Brian escolheu Codex?
Por que este run custou tanto?
Por que o workflow voltou para correção?
Por que Brian acredita que refund exige idempotência?
```

**[Δ]** Com uma restrição adicional: explicabilidade se baseia em **sinais registrados**, nunca em pedir a um LLM que explique a decisão depois do fato. Explicação gerada post-hoc é racionalização, não auditoria. Toda decisão de roteamento grava seus insumos no momento da decisão (§11).

## 2.6 Local-first, enterprise-ready

Confirmado como disciplina, negado como cronograma.

```text
Brian Core
├── platform-agnostic
└── adapters/
    ├── macOS
    └── kubernetes    ← existe como pasta vazia até o v1.0
```

**[Δ]** O v0.1 detalhava a evolução Kubernetes em cinco seções (§86–§92). Manter o core agnóstico é disciplina barata: não chamar API de macOS fora de `platform/macos/`. Construir o adapter Kubernetes antes de ter um cliente enterprise pagante é otimização prematura de custo alto.

A regra é: **agnosticismo é obrigatório desde o commit 1; o segundo runtime é opcional até existir demanda.**

## 2.7 Utilidade em cada camada **[Δ novo]**

Princípio que faltava no v0.1 e que determina todo o sequenciamento do §80–§85.

> Cada versão do Brian deve ser útil sozinha, sem a versão seguinte.

O v0.1-draft só entregava valor ao final do §98, depois de contexto, identidade, vault, dois adapters, OpenSpec, workflow, router, governor, telemetria e contabilidade estarem prontos. Isso é meses de construção antes do primeiro sinal de mercado.

O v1.0 deste blueprint reorganiza para que:

```text
v0.0  entrega relatório de custo por cliente         → útil sozinho
v0.1  entrega troca de contexto e identidade         → útil sozinho
v0.2  entrega execução rastreada                     → útil sozinho
v0.3  entrega comparação entre providers             → útil sozinho
```

Nenhuma dessas versões exige a próxima para justificar existência.

## 2.8 Nada depende de hipótese não testada **[Δ novo]**

Componentes marcados `H-n` não podem ter outros subsistemas dependendo deles. Se `H-1` (Context Governor) falhar, a remoção do Governor deve ser uma exclusão de módulo, não uma refatoração.

Concretamente: o Governor produz um `ContextPackage`. O caminho padrão produz um `ContextPackage` trivial (referência ao repositório e à mudança). Providers consomem `ContextPackage` sem saber qual dos dois o produziu.

## 2.9 Zero token perdido + cada centavo trabalha — lei inviolável **[Δ][D-16]**

> **Nenhum token some. Toda assinatura tem janela. Todo centavo tem dono.  
> Capacidade paga existe para poupar meu tempo de trabalho — não para evaporar em ruído.**

Isto **não** é o Context Governor (H-1). Governor tenta reduzir tokens por inteligência de contexto e pode falhar. **Usage Control** não pode falhar em existir: mesmo que o agente gaste o que gastar, o Brian **captura 100% do observável**, **atribui**, **mostra janelas** e **impede surpresa**.

### Obrigações invioláveis (P0 permanente)

```text
1. CAPTURA TOTAL (zero loss)
   Nenhum consumo conhecido de provider anexado fica de fora do ledger.
   "Perdi um token" = bug de release, não débito técnico.
   Sessão fora de `brian run` também conta (modo observe).
   Import histórico no onboarding. Unattributed é alarme, não normal.

2. ATOMISMO DO REGISTRO
   usage_record é a unidade de verdade: tokens (in/cache/out/reasoning),
   modelo, provider, identidade, cliente, projeto, timestamps, origem,
   billing_mode (api|subscription|credits|mixed|unknown).

3. JANELAS
   Toda capacidade é relativa a uma janela explícita:
     rolling_hour | calendar_day | calendar_week | calendar_month |
     plan_reset | custom
   % de uso = consumido / capacidade da janela (quando capacidade é conhecida)
            ou consumido / baseline configurada (quando o provider não expõe quota).

4. TEMPO E CAPACIDADE RESTANTE
   Para cada provider/plano/identidade o usuário consulta em um comando:
     - tokens (ou requests) usados na janela
     - % da janela
     - restante absoluto (se conhecido)
     - tempo até reset (se conhecido)
     - burn rate (tokens/hora, $/hora equivalente)
     - projeção de esgotamento na janela atual

5. ASSINATURA ≠ API
   Em assinatura, a moeda primária é FRAÇÃO DE CAPACIDADE, não só $ de tabela.
   Em API, a moeda primária é $ real / $ reportado.
   Os dois modos coexistem e nunca se misturam sem rótulo.

6. CONTROLE
   Soft limit → alerta + política de economia (modelo mais barato, menos fases).
   Hard limit → bloqueio de novas chamadas no escopo; override auditado com motivo.
   Sem número de origem (D-6 / níveis §13.3), o controle degrada de forma explícita
   — nunca inventa quota “bonita”.

7. OTIMIZAÇÃO = TEMPO SALVO, NÃO TEATRO
   Maximizar trabalho útil por unidade de capacidade.
   Minimizar: retries cegos, unattributed, modelo premium em tarefa trivial,
   reexplicar contexto que a memória já tem (liga com D-17), fases LLM inúteis.
   A meta de negócio do usuário: ~90% menos trabalho manual; a meta do Brian:
   não sabotar isso queimando a assinatura.

8. VISIBILIDADE CONTÍNUA
   `brian status` / `brian usage` / `brian capacity` são tão centrais quanto `run`.
   Se o usuário não consegue responder "quanto me resta esta semana em Claude?"
   em < 5 segundos, o Brian está regredido.
```

### O que isto **não** exige no minuto 0

```text
✗ Context Governor (H-1)
✗ Router adaptativo
✗ UI rica
✗ Previsão perfeita de custo de um run futuro
✗ Quota reportada pelo provider em todos os casos
```

Quando o provider não expõe quota (estado `unknown`), Brian **ainda** cumpre a lei com o que controla: medição própria, janelas configuradas pelo usuário, % sobre baseline do plano, alertas de burn e atribuição 100%.

### Hierarquia de verdade de capacidade

```text
1. Provider reportou quota/remaining/reset     → autoritativo
2. Brian mediu tokens/requests na janela      → verdade operacional local
3. Plano declarado pelo usuário (YAML)        → denominador da %
4. Price catalog / $ equivalente              → comparação e chargeback
```

Nunca apresentar (4) como se fosse (1).

## 2.10 Memória compartilhada e chaveamento de LLM — lei inviolável **[Δ novo][D-17]**

> **A cabeça do trabalho vive no Brian. O LLM é worker substituível.  
> Trocar de modelo/provider não pode me fazer recomeçar a conversa, a análise ou as decisões.**

Este é o **segundo maior ganho**, emparelhado com D-16. Sem ele, multi-provider é teatro: você paga de novo o mesmo raciocínio em cada LLM e perde o tempo que a IA deveria estar economizando.

### O que “sem perda” significa

Na troca Claude → Codex (ou qualquer par), sob o **mesmo Context**, o próximo worker recebe um **Continuity Pack** com:

```text
objetivo atual e critérios de sucesso
resumo da conversa / thread de trabalho (não o log bruto inteiro)
decisões já tomadas e porquês
análise feita (hipóteses, o que foi descartado)
arquivos / símbolos tocados + diff relevante
erros e tentativas que falharam
próximos passos explícitos
memórias de projeto ativas (D-14)
constraints do cliente (policy, paths, budget restante)
ponteiros para evidência (paths, run_ids, usage_ids) — não alucinação
```

O pack é **otimizado para handoff**: denso, estruturado, estável o suficiente para cache quando possível, sem despejar o histórico inteiro de tokens (isso seria gastar dinheiro — viola a missão).

### Obrigações invioláveis (P0 de continuidade)

```text
1. FONTE DA VERDADE
   Conversa útil, análise e decisões materializam-se no Brian
   (continuity + memory), não ficam presas só no transcript do provider.

2. HANDOFF EXPLÍCITO
   brian handoff / troca de provider regenera o Continuity Pack
   e injeta no próximo worker (prompt ou skill nativa).

3. SEM REEXPLICAR
   O usuário não reexplica o projeto ao mudar de LLM.
   Se precisar, D-17 falhou.

4. ISOLAMENTO DE TENANT
   Memória nunca cruza cliente (§37). Continuidade é por Context.

5. APPEND-ONLY + PROVENIÊNCIA (D-14)
   Decisões e correções não apagam história; supersedem.

6. BARATO DE CARREGAR
   Continuity Pack tem orçamento (warn). Handoff não pode custar
   mais que o valor de não recomeçar — senão a "memória" vira desperdício.

7. OBSERVE TAMBÉM CONTRIBUI
   Mesmo sem `brian run`, sessões observadas devem poder
   alimentar notas de continuidade / memória de projeto.
```

### O que **não** é D-17

```text
✗ Graph RAG obrigatório
✗ Embedding cluster enterprise no v0.0
✗ Replicar o transcript completo de 200 turnos em todo handoff
✗ Context Governor (H-1) — outro problema
```

### Relação com a missão

```text
D-16  impede a IA de roubar meu dinheiro (token sumido / janela cega / desperdício)
D-17  impede a IA de roubar meu tempo  (recomeçar do zero a cada LLM)
Juntos → a IA pode de fato poupar ~90% do trabalho
```

### Quando entra na timeline

```text
v0.0   ledger zero-loss + capacity (D-16) — base
v0.1   Continuity Pack mínimo + memory notes por context (D-17 mínimo)
v0.2   handoff automático em run / troca de provider no workflow
v0.4+  Memory Engine completo (retrieval rico, episodic, incident, …)
```

O **mínimo D-17** (pack de handoff + notas de decisão/projeto) **não espera** o Memory Engine “completo” do v0.4.

---

# 3. Arquitetura de Alto Nível

**[Δ]** O diagrama do v0.1 tinha 8 camadas verticais e sugeria que tudo é construído junto. Este diagrama separa o que existe no v0.x do que é posterior.

```text
                            BRIAN CORE
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
     CONTEXTO                EXECUÇÃO                CONTROLE
        │                       │                       │
   Client/Project          Run Manager             Policy Engine
   Identity                Workflow (dados)        Vault
   Vault refs              Provider Adapters       Budgets
   Memory ns               Worktree Manager        Approval Gates
   Environment             Session Manager
        │                       │                       │
        └───────────────────────┼───────────────────────┘
                                │
                          Provider Router
                       (regras → evidência)
                                │
          ┌──────────┬──────────┼──────────┬──────────┐
          ▼          ▼          ▼          ▼          ▼
       Claude      Codex     Gemini      Grok       ZCode
          │          │          │          │          │
          └──────────┴──────────┼──────────┴──────────┘
                                │
                         Capability Layer
                     (Git · Tests · Code Graph)
                                │
                          Quality Gates
                   (OCR · Semgrep · OSV · Secrets)
                                │
                            TELEMETRIA
                                │
              ┌─────────────────┼─────────────────┐
              ▼                 ▼                 ▼
           Traces            Tokens             Custo
              │                 │                 │
              └─────────────────┼─────────────────┘
                                ▼
                             MEMÓRIA
                                │
                                ▼
                        [ LEARNING — pós v1.0 ]
```

Leitura do diagrama:

- **Contexto, Execução e Controle** são o produto. Existem desde o v0.0/v0.1.
- **Provider Router** existe desde o v0.2, mas por regras (`D-8`).
- **Capability Layer** e **Quality Gates** são adicionados incrementalmente, cada um justificado por um problema observado.
- **Telemetria** é a *primeira* coisa construída, não a penúltima. **[Δ]**
- **Learning** é tracejado porque não existe antes do v1.0 (`D-8`, §38).

## 3.1 Fluxo de dados de um run

```text
brian run "<tarefa>"
      │
      ▼
 Context ativo resolvido ──────► client_id, project_id, budget_id
      │
      ▼
 Worktree criado ──────────────► isolamento de filesystem (§109)
      │
      ▼
 Run persistido (status=pending) ──► antes de qualquer efeito (D-12)
      │
      ▼
 Router escolhe provider ──────► decisão gravada com insumos (§11)
      │
      ▼
 Vault resolve credenciais ────► escopo de sessão, nunca em disco
      │
      ▼
 Adapter executa ──────────────► headless JSON preferencial (D-4)
      │
      ├──► stdout/stderr → artifact store
      ├──► usage → token accounting
      └──► eventos → trace
      │
      ▼
 Gates executam ───────────────► testes, lint, security
      │
      ▼
 Resultado avaliado ───────────► critério explícito (§112)
      │
      ▼
 Run finalizado ───────────────► status, custo, atribuição
      │
      ▼
 Memória proposta ─────────────► aprovação antes de durável (§36)
      │
      ▼
 Worktree descartado ou promovido a branch
```

---

# 4. Subsistemas

**[Δ]** O v0.1 listava 24 subsistemas sem indicar quando cada um existe. Isso é o principal fator de risco de escopo do documento original. A lista abaixo é a mesma, ordenada por versão de introdução.

## v0.0 — seis subsistemas **[Δ D-16: capacity no minuto 0]**

```text
Storage               SQLite + traits (D-1, D-9)
Provider Registry     descoberta e attach de CLIs
Usage Collector       leitura de sessões + import histórico
Usage Control Plane   janelas, %, restante, burn, planos, alertas (D-16)
FinOps                atribuição a cliente + showback/export
Brian CLI             usage · capacity · costs · status · import
```

## v0.1 — mais cinco **[Δ D-17 mínimo sobe aqui]**

```text
Context Manager       cliente/projeto ativo
Identity Manager      isolamento de identidade de provider
Brian Vault           credenciais via Keychain
Policy Engine         versão mínima: o que exige aprovação
Continuity Memory     pack de handoff + notas (D-17 mínimo) — chavear LLM sem perda
```

## v0.2 — mais cinco

```text
Run Manager         ciclo de vida, retomada, cancelamento
Worktree Manager    isolamento e concorrência
Workflow Engine     máquina de estados dirigida por dados
Telemetry           traces OpenTelemetry
OpenSpec Adapter    leitura de spec
(+ handoff automático de Continuity Pack em troca de provider no run)
```

## v0.3 — mais cinco

```text
Provider Router     seleção por regras
Model Router        ponteiros semânticos de modelo
Quality Gates       testes, lint, review
Brian.app UI        SwiftUI
Brian Inspector     explicabilidade de decisão
```

## v0.4 — mais quatro

```text
Memory Engine       retrieval rico, episodic, incident (completa D-17)
Code Intelligence   grafo, AST, busca
Security Gates      Semgrep, OSV, secrets, SkillSpector
Eval Harness        avaliação de roteamento (§112)
```

## v1.0+ — o restante

```text
Context Governor    condicional a H-1
Impact Engine       condicional a volume de uso
Reasoning Engine    planner/replanner
Browser Bridge      extensão de navegador
Xcode Bridge        integração Xcode
Learning Engine     condicional a D-8
Brian Chat          interface em linguagem natural
MCP Server          exposição de capacidades a agentes
```

**Total: ~30 subsistemas (Usage Control no v0.0; Continuity Memory mínima no v0.1), distribuídos em seis versões.** O v0.1-draft colocava 13 no MVP. Este documento coloca 6 no v0.0 (D-16) e sobe continuidade multi-LLM (D-17) já no v0.1 — os dois ganhos mínimos da missão.

---

# 5. Context Manager

Context é a abstração central de runtime e a fronteira de tenancy do produto.

Um contexto liga:

```text
Client
Project
Repository
Identity
OpenSpec
Memory namespace
Providers permitidos
Tools permitidas
Secret refs
Policies
Environments
Telemetry namespace
Budget
```

## 5.1 Definição de contexto

```yaml
context:
  id: xpto-checkout
  client: xpto
  project: checkout-api
  created_at: 2026-08-01T10:00:00Z
  schema_version: 3          # [Δ] versionado desde o início (§113)

repository:
  path: ~/Projects/XPTO/checkout-api
  default_branch: main
  worktree_root: ~/.brian/worktrees/xpto-checkout   # [Δ] §109

identity:
  profile: xpto-work

openspec:
  root: ./openspec
  required: false            # [Δ] contexto funciona sem OpenSpec

memory:
  namespace: client:xpto/project:checkout-api

providers:
  allowed: [claude, codex]
  denied: [grok]             # [Δ] deny explícito, não só allow
  claude:
    identity: xpto-work
  codex:
    identity: xpto-work

github:
  organization: xpto-org

environments:
  staging: xpto-staging
  production: xpto-production

budget:
  monthly_tokens: 50000000
  monthly_usd_equivalent: 500
  change_soft_limit_usd: 10
  change_hard_limit_usd: 20

policy:
  set: consultancy-default
```

**[Δ] Três adições em relação ao v0.1:**

1. `schema_version` — sem isso, migração de contexto entre versões do Brian é impossível sem heurística. Ver §113.
2. `providers.denied` — allowlist implícita não expressa "este cliente proíbe modelo X por contrato". Consultoria tem essa exigência com frequência.
3. `openspec.required: false` — o v0.1 acoplava contexto a OpenSpec. Isso impede o v0.0/v0.1, onde não há OpenSpec ainda, e impede uso em repositório que não adota OpenSpec.

## 5.2 Resolução de contexto

Ordem de precedência, do mais específico ao menos:

```text
1. flag explícita          brian run --context xpto-checkout
2. variável de ambiente    BRIAN_CONTEXT=xpto-checkout
3. arquivo do repositório  .brian/context.toml no cwd ou ancestral
4. contexto conectado      brian connect xpto-checkout
5. contexto default        configurado pelo usuário
6. nenhum                  erro explícito, nunca fallback silencioso
```

**[Δ]** O item 6 é regra dura. Executar sem contexto significa consumo não atribuído, que é exatamente o problema que o produto existe para resolver. Brian falha com mensagem clara em vez de rodar em um contexto "default" implícito.

```text
$ brian run "corrige o bug do refund"
erro: nenhum contexto ativo

  O consumo deste run não teria atribuição.

  Escolha um contexto:
    brian connect xpto/checkout-api
    brian run --context xpto/checkout-api "..."

  Ou marque este diretório:
    brian init --client xpto --project checkout-api
```

## 5.3 Comandos

```bash
brian connect xpto                    # cliente, projeto único ou último usado
brian connect xpto/checkout-api       # explícito
brian disconnect
brian whoami                          # contexto + identidade + budget restante
brian context list
brian context show [id]
brian context init                    # cria .brian/context.toml no repo atual
```

## 5.4 O que `connect` realmente faz

`brian connect` é um `cd` semântico que troca simultaneamente:

```text
diretório de trabalho padrão
identidade de provider (config home, auth home)
identidade Git (user.name, user.email)
namespace de memória
conjunto de ferramentas permitidas
conjunto de políticas
budget de atribuição
namespace de telemetria
resolvedor de secrets
```

Isso é o produto inteiro em um comando. É também o teste de valor mais rápido: se `connect` não economizar tempo real de troca de contexto para o usuário, nada mais no documento importa.

---

# 6. Identity Manager

Brian precisa de identidades separadas das identidades normais do desktop.

```text
Terminal (pessoal)
├── Claude → conta pessoal
├── Codex  → conta pessoal
└── Gemini → conta pessoal

Brian (trabalho)
├── Claude → conta da empresa
├── Codex  → conta da empresa
└── Gemini → conta da empresa
```

## 6.1 Mecanismo

O binário do provider é compartilhado. O estado de configuração e autenticação é isolado por variável de ambiente.

```text
/opt/homebrew/bin/codex
        │
        ├── CODEX_HOME=~/.codex                       (pessoal)
        └── CODEX_HOME=~/.brian/identities/xpto/codex (trabalho)
```

**[Δ]** O v0.1 descrevia isso conceitualmente. Concretamente, cada adapter declara quais variáveis controlam seu estado, e o Identity Manager as injeta no processo filho:

```yaml
# providers/codex/identity.yaml
identity_env:
  config_home: CODEX_HOME
  auth_home: CODEX_HOME          # mesmo diretório neste provider
  
isolation_verified: true         # testado: duas identidades simultâneas funcionam
verified_against_version: "0.48.x"
```

O campo `isolation_verified` importa. Se um provider não suportar isolamento de estado por variável de ambiente, Brian **não pode** oferecer identidades paralelas para ele, e a UI deve dizer isso em vez de fingir.

## 6.2 Conteúdo de um perfil

```yaml
profile: xpto-work
client: xpto

provider_bindings:
  codex:
    executable: /opt/homebrew/bin/codex
    config_home: ~/.brian/identities/xpto/codex
    preferred_models: [coding, quick]
  claude:
    executable: /opt/homebrew/bin/claude
    config_home: ~/.brian/identities/xpto/claude
    preferred_models: [reasoning, review]

git:
  user_name: "Joao Costa"
  user_email: "joao@xpto-consultoria.com.br"
  signing_key_ref: keychain://brian/xpto/git/signing

github:
  organization: xpto-org
  token_ref: keychain://brian/xpto/github/pat

environment:
  NODE_ENV: development
  
mcp_config: ~/.brian/identities/xpto/mcp.json

policy_set: consultancy-default
```

## 6.3 Verificação de identidade

Antes de qualquer run, Brian confirma que a identidade ativa é a esperada:

```bash
$ brian whoami

Contexto      xpto / checkout-api
Perfil        xpto-work
Git           Joao Costa <joao@xpto-consultoria.com.br>
GitHub        xpto-org

Providers
  codex       ● autenticado    conta: eng@xpto-consultoria.com.br
  claude      ● autenticado    conta: eng@xpto-consultoria.com.br
  gemini      ○ não vinculado

Budget        R$ 312 de R$ 500 usados este mês (62%)
```

**[Δ]** Mostrar a conta autenticada, não só o status, é o que impede o erro mais caro do produto: rodar trabalho de cliente na conta pessoal e não conseguir cobrar.

---

# 7. Brian Vault

Vault é a abstração de credenciais. No macOS, o backend primário é o Keychain.

## 7.1 Backends

```text
v0.1        macOS Keychain
v1.0+       HashiCorp Vault
            AWS Secrets Manager
            Azure Key Vault
            GCP Secret Manager
            Kubernetes External Secrets
            1Password Connect
```

O banco do Brian armazena **referências**, nunca valores.

```yaml
credential_ref: keychain://brian/xpto/codex/oauth
```

## 7.2 Classes de secret

```text
LOW         credenciais somente leitura
MEDIUM      escrita em repositório
HIGH        acesso a infraestrutura
CRITICAL    acesso a produção
```

## 7.3 Política por operação

```text
ler repositório          automático
git commit               automático
git push                 política / Touch ID opcional
deploy staging           política
deploy produção          Touch ID + aprovação explícita
rotacionar secret        Touch ID
exportar secret          proibido
```

**[Δ]** A última linha é nova e é regra dura. Brian nunca expõe valor de credencial, nem em log, nem em UI, nem em CLI, nem para agentes. Um agente recebe um processo já autenticado, não um token.

## 7.4 Integração macOS

```text
Security.framework          armazenamento
LocalAuthentication         Touch ID
kSecAttrAccessible          política de acesso em repouso
Access Control Lists        binding ao binário assinado do Brian
```

Resolução de credencial é **escopada à sessão**: o valor vive na memória do processo pelo tempo do run e é zerado ao final.

## 7.5 Metadados de credencial

```yaml
credential:
  ref: keychain://brian/xpto/aws/production
  class: CRITICAL
  created_at: 2026-06-01
  expires_at: 2026-09-01
  rotation_policy: 90d
  last_used_at: 2026-08-07T14:22:00Z
  used_by_runs: 47
  requires_biometric: true
```

**[Δ]** `expires_at` e `last_used_at` permitem duas coisas que o v0.1 não previa: aviso proativo de expiração e detecção de credencial órfã (armazenada e nunca usada — candidata a remoção).

---

# 8. Providers Anexados

Brian não instala providers. Ele se anexa a CLIs existentes.

```text
Claude Code
Codex CLI
Gemini CLI
Grok CLI
ZCode
```

## 8.1 Registro de provider

```json
{
  "id": "codex",
  "type": "cli",
  "executable": "/opt/homebrew/bin/codex",
  "version_detected": "0.48.2",
  "version_verified": "0.48.x",
  "managed": false,
  "attached": true,
  "integration_tier": "headless_json",
  "capabilities": {
    "headless": true,
    "json_output": true,
    "session_files": true,
    "session_resume": true,
    "reports_cost": true,
    "reports_quota": false,
    "model_listing": true,
    "identity_isolation": true
  },
  "last_verified_at": "2026-08-08T09:00:00Z"
}
```

**[Δ] `integration_tier` e `capabilities` são a correção mais importante desta seção.** O v0.1 assumia uniformidade. A realidade é que cada provider está em um degrau diferente, e o Brian precisa saber em qual, para não prometer na UI o que não pode entregar.

## 8.2 Degraus de integração (D-4)

```text
TIER 1 — headless_json
  Modo não-interativo com saída JSON estruturada.
  Contém resultado, session_id, tokens, custo.
  Estável entre versões. Contrato público.
  → PREFERENCIAL SEMPRE

TIER 2 — session_files
  Arquivos de sessão em disco (JSONL, SQLite, etc.)
  Lidos após a execução.
  Formato semi-estável. Quebra em releases maiores.
  → USAR PARA ENRIQUECIMENTO E BACKFILL

TIER 3 — pty
  Sessão interativa em pseudo-terminal, saída parseada.
  Formato instável. Quebra sem aviso.
  → APENAS QUANDO 1 E 2 NÃO EXISTEM
  → APENAS PARA MODO INTERATIVO SUPERVISIONADO
```

Regra: **um provider em Tier 3 nunca opera em modo autônomo.** Se não é possível ler o resultado de forma confiável, não é possível decidir automaticamente se ele funcionou.

## 8.3 Verificação de compatibilidade

Toda versão de provider suportada é testada e registrada:

```yaml
# providers/codex/compatibility.yaml
supported:
  - version: "0.48.x"
    tier: headless_json
    verified_at: 2026-08-01
    notes: ""
  - version: "0.47.x"
    tier: headless_json
    verified_at: 2026-06-15
    notes: "campo cost ausente; usar price catalog"
  - version: "0.4x"
    tier: session_files
    verified_at: 2026-03-01
    notes: "sem modo headless"

unsupported_above: "0.49"
```

Quando Brian detecta versão fora da faixa verificada:

```text
aviso: codex 0.49.1 detectado, verificado até 0.48.x

  Brian vai operar em modo degradado:
    - execução: ok
    - leitura de custo: não verificada
    - retomada de sessão: desabilitada

  Rode `brian providers verify codex` para testar.
```

**[Δ]** Isso não existia no v0.1 e é a diferença entre um produto que quebra silenciosamente e um que avisa. Adapters sobre CLIs de terceiros são a maior fonte de manutenção perpétua do projeto (§116, R-3); tratá-los com rigor de contrato é obrigatório.

## 8.4 Coexistência

O CLI continua usável fora do Brian.

```text
Terminal ───────┐
                ├── mesmo binário instalado
Brian ──────────┘   estado de config isolado
```

Brian nunca modifica a configuração pessoal do usuário. Todo estado gerenciado vive sob `~/.brian/`.

---

# 9. Perfis de Provider

Um provider físico expõe múltiplos perfis lógicos.

```text
Codex                Claude
├── Builder          ├── Planner
├── Fixer            ├── Reviewer
└── Quick            └── Architect
```

## 9.1 Definição

```yaml
profile: codex-builder
provider: codex
role: builder
model_pointer: coding

skills:
  - implement
  - debug

capabilities:
  - code.read
  - code.write
  - git.diff
  - tests.run

denied_capabilities:            # [Δ] deny explícito
  - git.push
  - shell.arbitrary
  - network.external

context_budget_tokens: 50000
max_turns: 40                   # [Δ] limite de loop
timeout_seconds: 1800           # [Δ] limite de parede
cost_ceiling_usd: 5.00          # [Δ] limite econômico
```

**[Δ]** Os três últimos campos são novos e são proteções básicas ausentes no v0.1. Um agente sem `max_turns` e sem `cost_ceiling` pode consumir um budget mensal em um run travado. Esses limites são aplicados pelo Run Manager, não pelo provider.

## 9.2 Ação ao atingir limite

```text
max_turns atingido        → run pausa, estado preservado, usuário decide
timeout atingido          → run pausa, processo encerrado com SIGTERM
cost_ceiling atingido     → run pausa, motivo gravado no trace
context_budget excedido   → aviso; não interrompe (é orçamento, não limite)
```

Todos os quatro produzem run em estado `paused`, nunca `failed`. A distinção importa: `failed` significa que o trabalho não funcionou; `paused` significa que Brian parou por precaução e o trabalho pode ser retomado.

---

# 10. Interface de Provider

**[Δ]** O v0.1 definia uma trait única com oito métodos. Isso força implementações incompletas. Este documento separa em uma trait obrigatória e três traits opcionais.

```rust
/// Obrigatório. Todo adapter implementa.
trait Provider {
    fn id(&self) -> ProviderId;
    fn detect(&self) -> Result<ProviderInstallation>;
    fn capabilities(&self) -> ProviderCapabilities;
    fn execute(&self, req: ExecuteRequest) -> Result<ExecuteHandle>;
    fn cancel(&self, handle: &ExecuteHandle) -> Result<()>;
}

/// Opcional. Só quem tem contrato estável de uso.
trait ReportsUsage {
    fn usage(&self, handle: &ExecuteHandle) -> Result<Usage>;
}

/// Opcional. Só quem reporta custo em moeda.
trait ReportsCost {
    fn cost(&self, handle: &ExecuteHandle) -> Result<Cost>;
}

/// Opcional. Só quem suporta retomada.
trait Resumable {
    fn resume(&self, session: SessionId, req: ExecuteRequest)
        -> Result<ExecuteHandle>;
}

/// Opcional. Raro. Poucos providers expõem.
trait ReportsQuota {
    fn quota(&self) -> Result<QuotaStatus>;
}
```

## 10.1 Tipos centrais

```rust
struct ExecuteRequest {
    run_id: RunId,
    context: ContextSnapshot,
    working_dir: PathBuf,        // sempre um worktree (D-7)
    prompt: String,
    context_package: ContextPackage,
    model_pointer: ModelPointer,
    limits: ExecutionLimits,
    identity: ResolvedIdentity,
    allowed_tools: Vec<Capability>,
}

struct ExecuteHandle {
    run_id: RunId,
    provider_session_id: Option<String>,
    pid: Option<u32>,
    started_at: DateTime<Utc>,
    artifact_dir: PathBuf,
}

struct Usage {
    input_tokens: Option<u64>,
    cached_input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    reasoning_tokens: Option<u64>,
    total_tokens: Option<u64>,
    source: UsageSource,          // Provider | SessionFile | Estimated
}

struct Cost {
    amount_usd: Decimal,
    source: CostSource,           // ProviderReported | Calculated
    catalog_version: Option<String>,
}
```

**[Δ]** Todos os campos de `Usage` são `Option`. O v0.1 tinha uma lista fixa de cinco métricas, mas nenhum provider reporta as cinco. Modelar ausência explicitamente evita que zeros falsos entrem no FinOps.

## 10.2 Estrutura de adapters

```text
core/src/providers/
├── mod.rs
├── registry.rs
├── traits.rs
├── capabilities.rs
├── compatibility.rs
├── claude/
│   ├── mod.rs
│   ├── headless.rs        tier 1
│   ├── session_files.rs   tier 2
│   └── compatibility.yaml
├── codex/
├── gemini/
├── grok/
└── zcode/
```

Cada adapter implementa os tiers que consegue, e o registry escolhe o mais alto disponível na versão detectada.

---

# 11. Provider Router

O Router escolhe qual worker executa uma tarefa.

## 11.1 Fases de maturidade (D-8)

```text
FASE 1 — v0.2/v0.3      Regras explícitas + override manual
FASE 2 — v0.4           Regras + evidência histórica como desempate
FASE 3 — pós v1.0       Scoring com pesos aprendidos
```

**[Δ]** O v0.1 descrevia direto a Fase 3, com uma fórmula de scoring de seis termos. O problema é estatístico: para que `historical_quality` por `task_type × provider × model` tenha significado, é preciso dezenas de execuções por célula. Um usuário solo leva meses para sair do ruído. Uma fórmula alimentada por `n=3` produz confiança injustificada.

`D-8` fixa o limiar: **`n ≥ 30` por célula antes de qualquer termo histórico entrar no scoring.** Até lá, evidência histórica aparece na UI como informação, não como insumo de decisão.

## 11.2 Fase 1 — regras

```yaml
# routing/rules.yaml
version: 1

default:
  provider: codex
  model_pointer: coding

rules:
  - when: { task_type: planning }
    then: { provider: claude, model_pointer: reasoning }

  - when: { task_type: review }
    then: { provider: claude, model_pointer: review }

  - when: { task_type: implementation, risk: high }
    then: { provider: codex, model_pointer: coding }

  - when: { task_type: implementation, complexity: low }
    then: { provider: codex, model_pointer: quick }

  - when: { context_tokens_gt: 150000 }
    then: { provider: gemini, model_pointer: long-context }

  - when: { budget_state: soft_limit }
    then: { model_pointer: quick }

constraints:
  - deny: { provider: grok, client: xpto }
    reason: "restrição contratual do cliente"
  - require_available: true
  - respect_context_allowlist: true
```

Regras são avaliadas em ordem; a primeira que casa vence; `constraints` são aplicadas depois e podem vetar.

## 11.3 Sinais coletados

Coletados sempre, usados conforme a fase:

```text
DA TAREFA          task_type, linguagem, framework, complexidade,
                   risco, horizonte, capacidades exigidas

DO CONTEXTO        cliente, projeto, allowlist, denylist, política

DO PROVIDER        disponibilidade, autenticação, versão, tier,
                   rate limit ativo, quota (quando existir)

ECONÔMICOS         budget restante, custo esperado, estado do limite

HISTÓRICOS         taxa de sucesso, retries médios, latência média,
                   custo médio — com n associado
```

## 11.4 Registro de decisão

**Toda decisão de roteamento é gravada no momento em que é tomada**, com seus insumos (§2.5).

```json
{
  "decision_id": "rd_8f21",
  "run_id": "run_1421",
  "phase": "implementation",
  "decided_at": "2026-08-08T10:14:22Z",
  "mode": "rules",
  "chosen": { "provider": "codex", "model_pointer": "coding" },
  "matched_rule": "implementation + risk:high",
  "considered": [
    { "provider": "codex",  "eligible": true },
    { "provider": "claude", "eligible": true,  "not_chosen": "regra não casou" },
    { "provider": "gemini", "eligible": false, "reason": "não autenticado" },
    { "provider": "grok",   "eligible": false, "reason": "denylist do cliente" }
  ],
  "signals": {
    "task_type": "implementation",
    "risk": "high",
    "complexity": "medium",
    "context_tokens": 42000,
    "budget_remaining_usd": 188.40,
    "budget_state": "normal"
  },
  "historical_context": {
    "codex_success_rate": 0.94,
    "codex_n": 12,
    "used_in_decision": false,
    "reason_not_used": "n < 30 (D-8)"
  },
  "override": null
}
```

O campo `used_in_decision: false` com justificativa é o que torna o Brian Inspector (§66) honesto. Mostrar 94% de sucesso e deixar implícito que isso decidiu algo, quando não decidiu, é explicação enganosa.

## 11.5 Override manual

```bash
brian run "..." --provider claude
brian run "..." --model-pointer reasoning
brian run "..." --explain-only     # mostra a decisão sem executar
```

Overrides são gravados. A frequência de override é uma métrica de produto (§101): se o usuário sobrescreve com frequência, as regras estão erradas.

---

# 12. Model Router

Provider e modelo são decisões separadas.

## 12.1 Ponteiros semânticos

```text
reasoning        raciocínio longo, planejamento, arquitetura
coding           implementação de código
quick            tarefas pequenas, baixo custo
compact          resumo, extração, classificação
review           revisão crítica de diff
long-context     entrada muito grande
research         investigação exploratória
```

## 12.2 Mapeamento

```yaml
# models/pointers.yaml
version: 1

pointers:
  reasoning:
    primary:   { provider: claude, tier: strong }
    fallback:  { provider: codex,  tier: strong }

  coding:
    primary:   { provider: codex,  tier: strong }
    fallback:  { provider: claude, tier: strong }

  quick:
    primary:   { provider: codex,  tier: cheap }
    fallback:  { provider: gemini, tier: cheap }

  compact:
    primary:   { provider: gemini, tier: cheap }

  review:
    primary:   { provider: claude, tier: strong }

  long-context:
    primary:   { provider: gemini, tier: strong }

  research:
    primary:   { provider: grok,   tier: strong }
```

## 12.3 Resolução de nome concreto

```yaml
# providers/claude/models.yaml
tiers:
  strong: claude-opus-5
  balanced: claude-sonnet-5
  cheap: claude-haiku-4-5-20251001

resolved_at: 2026-08-01
resolution_source: "brian providers models claude"
```

**[Δ]** Nomes concretos de modelo ficam em configuração de provider, atualizável sem tocar em lógica. O v0.1 já dizia isso; este documento adiciona `resolved_at` e o comando que popula o arquivo, porque nomes de modelo mudam e uma lista escrita à mão apodrece.

## 12.4 Regra de fallback

```text
primary indisponível     → fallback
fallback indisponível    → erro explícito, run não inicia
tier ausente no provider → degradar para o tier mais próximo, avisar
```

Brian nunca escolhe silenciosamente um modelo mais fraco. Degradação é registrada no trace e visível na UI.

---

# 13. Limites, Consumo e Controle de Capacidade

**[Δ][D-16]** Esta seção deixa de ser “status do provider” e passa a ser o **núcleo operacional do Usage Control Plane**. Existe no minuto 0. Sem ela o Brian não shippa.

## 13.1 Status normalizado (provider × identidade × plano)

```json
{
  "provider": "claude",
  "attached": true,
  "authenticated": true,
  "available": true,
  "version": "2.1.4",
  "integration_tier": "headless_json",
  "account": "eng@xpto-consultoria.com.br",
  "identity_profile_id": "prof_xpto_claude",
  "billing_mode": "subscription",
  "plan": {
    "id": "claude_max_seat",
    "label": "Claude Max (assento)",
    "plan_cost_monthly": 100.0,
    "currency": "USD"
  },
  "quota": {
    "state": "limited",
    "remaining_percent": 18.0,
    "reset_at": "2026-08-11T00:00:00Z",
    "source": "provider"
  },
  "windows": {
    "calendar_week": {
      "consumed_tokens": 4200000,
      "capacity_tokens": 10000000,
      "used_percent": 42.0,
      "remaining_tokens": 5800000,
      "remaining_percent": 58.0,
      "resets_at": "2026-08-11T00:00:00Z",
      "burn_tokens_per_hour": 85000,
      "eta_exhaustion_at": null,
      "source": "brian_measured+plan_baseline"
    },
    "calendar_day": {
      "consumed_tokens": 812000,
      "used_percent": 8.1,
      "source": "brian_measured"
    },
    "plan_reset": {
      "consumed_fraction": 0.42,
      "resets_at": "2026-08-11T00:00:00Z",
      "time_remaining_seconds": 214800,
      "source": "provider"
    }
  },
  "runtime": {
    "requests_today": 47,
    "tokens_today": 812000,
    "cost_usd_today_equivalent": 12.4,
    "last_success_at": "2026-08-08T10:12:00Z",
    "last_error_at": null
  },
  "health": {
    "rate_limited": false,
    "consecutive_failures": 0,
    "circuit_state": "closed"
  },
  "optimization": {
    "cache_hit_ratio_7d": 0.64,
    "unattributed_tokens_7d": 0,
    "top_client_share_7d": { "xpto": 0.41, "acme": 0.33, "internal": 0.26 },
    "waste_signals": ["retry_rate_high:codex", "verify_llm_on_trivial_tasks"]
  }
}
```

## 13.2 Estados de quota

```text
available        provider confirma que há quota
limited          provider confirma quota baixa (ou Brian: % acima do alerta)
exhausted        provider confirma quota esgotada (ou hard limit local)
rate_limited     rate limit ativo agora, com reset conhecido ou não
unknown          provider não reporta remaining — comum; Brian ainda mede janelas
```

**[Δ]** `unknown` no *provider* não autoriza cegueira no *Brian*. Se o provider não diz quanto resta, Brian mostra:

```text
medido nesta janela · % sobre baseline do plano · tempo até reset configurado
```

A UI e a CLI tratam `unknown` no nível 1 como “sem cota oficial”, **nunca** como “sem controle”.

## 13.3 Três níveis de informação

```text
NÍVEL 1 — reportado pelo provider
  Autoritativo para quota/remaining/reset e, quando existir, custo.
  Usado em decisão e cobrança quando presente.

NÍVEL 2 — medido em runtime pelo Brian
  Contagem própria de requests, tokens, duração, cache hits observados.
  Verdade operacional das janelas locais. Obrigatório mesmo com nível 1.

NÍVEL 3 — estimado
  Price catalog, $ equivalente, baseline de plano declarada pelo usuário.
  Sempre rotulado. Nunca apresentado como certificado.
```

Regra: **um número de nível 2 ou 3 nunca aparece sem rótulo de origem.**  
Regra D-16: **ausência de nível 1 não desliga nível 2.**

## 13.4 Circuit breaker

**[Δ novo].** Ausente no v0.1 e necessário para modo autônomo.

```text
3 falhas consecutivas       → circuit half_open, backoff de 60s
5 falhas consecutivas       → circuit open, provider removido do routing
                              por 15 min, evento no trace
rate limit detectado        → circuit open até reset_at, ou 5 min se
                              o reset for desconhecido
sucesso em half_open        → circuit closed
```

Sem isso, um provider degradado consome budget em retries até o limite duro do change.

## 13.5 Janelas de uso (obrigatórias no v0.0)

Toda métrica de capacidade é **ancorada em janela**. Janelas built-in:

```text
rolling_5m          burn rate instantâneo (debug / runaway)
rolling_hour        tendência de curto prazo
calendar_day        uso diário
calendar_week       uso semanal  ← visão default de assinatura semanal
calendar_month      showback / chargeback
plan_reset          ciclo do plano do provider (quando conhecido)
session             uma sessão de provider
run                 um run do Brian (quando existir)
custom              janela definida pelo usuário (ex.: sprint, fatura quinzenal)
```

Configuração por provider/plano:

```yaml
# catalog/plans/claude_max.yaml
id: claude_max_seat
provider: claude
billing_mode: subscription
plan_cost_monthly: 100
currency: USD
windows:
  primary: plan_reset          # ou calendar_week se reset for opaco
  report: [calendar_day, calendar_week, calendar_month, plan_reset]
capacity:
  # denominador da % quando o provider não expõe remaining
  baseline_tokens_per_week: 10000000   # declarado; rotulado "plan_baseline"
  baseline_source: user_declared
alerts:
  used_percent: [50, 80, 95]
  burn_tokens_per_hour_above: 200000
limits:
  soft_used_percent: 90
  hard_used_percent: 100       # se baseline confiável; senão só soft
```

## 13.6 Tempo restante e projeção

Campos obrigatórios na superfície de status (null permitido, mentira não):

```text
resets_at
time_remaining_seconds
remaining_tokens | remaining_requests | remaining_percent
burn_tokens_per_hour
burn_usd_equivalent_per_hour
eta_exhaustion_at          # null se burn baixo ou capacidade desconhecida
```

Projeção é **linear simples** no v0.0 (burn das últimas N horas). Não é ML. Rotulada como `projection`.

## 13.7 Superfície CLI do minuto 0

```bash
brian usage                          # janela default (semana ou plan_reset)
brian usage --window week
brian usage --window day --by client
brian usage --provider claude
brian capacity                       # % · restante · reset · burn · alertas
brian capacity --provider claude --json
brian costs                          # $ e/ou fração de assinatura alocada
brian costs --client xpto
brian costs --unattributed           # DEVE tender a vazio
brian status                         # capacity + context + providers em 1 tela
brian plans list
brian plans set claude --baseline-tokens-week 10000000
brian attribute <usage_id> --client xpto   # correção humana auditada
brian import --since 30d             # histórico → ledger
```

Critério de UX (D-16):

```text
brian capacity
→ em < 5s mostra, por provider anexado:
  plano · janela · % usado · restante · tempo até reset · burn · alertas
```

## 13.8 Políticas de otimização (aproveitar cada centavo)

Não confundir com H-1. Estas regras são **determinísticas** e ligadas a capacity:

```text
SE billing_mode = subscription E used_percent_week >= 80
  → preferir model_pointer mais barato em tarefas low-risk
  → desaconselhar --compare multi-provider sem flag explícita
  → alertar antes de governed / evals caros

SE billing_mode = api E burn_usd_hour anômalo (> 3× mediana 7d)
  → pause sugestão + status ALERT

SE unattributed_tokens > 0
  → banner permanente até zerar (integridade do ledger)

SE cache_hit_ratio_7d < 0.3 em runs longos
  → insight: "contexto instável pode estar matando cache" (§40)

SE tarefa trivial (direct) E pointer = strongest
  → warn uma vez: capacidade premium em tarefa barata
```

No v0.0 as políticas **alertam e registram**. Bloqueio hard exige budget configurado (§45). A partir do v0.2, as mesmas políticas podem influenciar default de `model_pointer` e seleção de workflow — sempre com override humano.

## 13.9 Regra de integridade do ledger

```text
∀ token observado ∈ usage_record
∀ usage_record tem client_id OU status = unattributed (visível)
∀ usage_record tem usage_source e cost_source
∀ usage_record tem occurred_at e window keys deriváveis
```

Violar isso é **bug de release**, não débito técnico.

---

# 14. OpenSpec

OpenSpec define intenção. Brian orquestra em volta dela.

```text
requisitos
mudanças
critérios de aceitação
decisões
tarefas
validação
```

## 14.1 Acoplamento opcional

**[Δ]** O v0.1 tratava OpenSpec como parte obrigatória do fluxo. Este documento o torna opcional.

Razões:

1. O v0.0 e o v0.1 não têm orquestração; exigir OpenSpec impede o produto de existir antes do v0.2.
2. A maioria dos repositórios de cliente não adota OpenSpec e não vai adotar como pré-condição para usar uma ferramenta de custo.
3. Acoplar contexto a spec transforma um produto de governança em um produto de metodologia, que é venda muito mais difícil.

```text
sem OpenSpec     → Brian roda tarefas ad-hoc, com atribuição completa
com OpenSpec     → Brian adiciona validação, rastreabilidade requisito→código
                   e critério objetivo de conclusão
```

OpenSpec é um **multiplicador**, não uma dependência.

## 14.2 Fluxo quando presente

```text
OpenSpec Change
      ↓
Planner (opcional)
      ↓
Workflow
      ↓
Provider
      ↓
Testes / Gates
      ↓
Validação OpenSpec
```

## 14.3 Adapter

```rust
trait SpecSource {
    fn current_change(&self) -> Result<Option<Change>>;
    fn tasks(&self, change: &ChangeId) -> Result<Vec<Task>>;
    fn acceptance_criteria(&self, change: &ChangeId) -> Result<Vec<Criterion>>;
    fn validate(&self, change: &ChangeId) -> Result<ValidationReport>;
}
```

**[Δ]** A trait chama-se `SpecSource`, não `OpenSpecAdapter`. Se OpenSpec for substituído por outro formato de spec, ou por issues do GitHub, ou por um markdown estruturado, o resto do sistema não muda.

Implementações previstas:

```text
OpenSpecSource      openspec/ no repositório
MarkdownSource      docs/specs/*.md com frontmatter
GitHubIssueSource   issue com label e checklist
NullSource          sem spec — retorna None, tudo funciona
```

---

# 15. Workflow Engine

**[Δ] Esta é a seção mais alterada do documento.**

## 15.1 O problema do v0.1

O v0.1 definia uma máquina de dez fases fixas:

```text
DISCOVERY → PLANNING → PLAN_REVIEW → ARCHITECTURE → IMPLEMENTATION
→ TESTING → CODE_REVIEW → CORRECTION → VALIDATION → DONE
```

Três problemas:

1. **Cerimônia desproporcional.** A maioria das tarefas reais é pequena. Rodar dez fases para corrigir um off-by-one faz o usuário fechar o Brian e abrir o terminal. Uma ferramenta que é mais lenta que a alternativa não é usada, independentemente do que ela governa.

2. **Fases fixas em código.** Cada cliente, cada tipo de projeto e cada nível de risco quer um pipeline diferente. Codificar dez fases significa que qualquer variação vira `if` no core.

3. **Custo.** Cada fase é ao menos uma chamada de LLM. Dez fases é dez vezes o custo mínimo para tarefas que precisariam de uma.

## 15.2 Decisão (D-3)

> Workflow é **dado versionado**, não código. O padrão é um fast path de três fases. Pipelines longos são opt-in, acionados por risco ou por escolha explícita.

## 15.3 Definição de workflow

```yaml
# workflows/fast.yaml
id: fast
version: 1
description: "Padrão. Tarefas pequenas e médias."

phases:
  - id: implement
    role: builder
    model_pointer: coding
    gates: []
    on_success: verify
    on_failure: fail

  - id: verify
    role: builder
    model_pointer: quick
    gates: [tests, lint]
    on_success: done
    on_failure: fix
    max_entries: 1

  - id: fix
    role: builder
    model_pointer: coding
    gates: []
    on_success: verify
    on_failure: escalate
    max_entries: 2

  - id: escalate
    terminal: true
    action: pause_for_human

  - id: done
    terminal: true

limits:
  max_total_phases: 8
  max_cost_usd: 5.00
  max_wall_seconds: 1800
```

```yaml
# workflows/governed.yaml
id: governed
version: 1
description: "Mudança de alto risco. Cliente regulado, produção, pagamento."

trigger:
  risk: [high, critical]
  or_paths_touching:
    - "src/payment/**"
    - "src/auth/**"
    - "**/migrations/**"

phases:
  - id: plan
    role: planner
    model_pointer: reasoning
    requires_spec: true
    on_success: plan_review

  - id: plan_review
    role: reviewer
    model_pointer: review
    on_success: implement
    on_failure: plan
    max_entries: 2

  - id: implement
    role: builder
    model_pointer: coding
    on_success: test

  - id: test
    role: builder
    gates: [tests, lint]
    on_success: security
    on_failure: fix

  - id: security
    gates: [semgrep, osv, secrets]
    on_success: review
    on_failure: fix
    blocking_severity: high

  - id: review
    role: reviewer
    model_pointer: review
    gates: [ocr]
    on_success: validate
    on_failure: fix

  - id: fix
    role: builder
    model_pointer: coding
    on_success: test
    max_entries: 3
    on_max_entries: replan

  - id: replan
    role: planner
    model_pointer: reasoning
    on_success: implement
    max_entries: 1
    on_max_entries: escalate

  - id: validate
    gates: [spec_validation]
    requires_approval: true
    on_success: done

  - id: escalate
    terminal: true
    action: pause_for_human

  - id: done
    terminal: true

limits:
  max_total_phases: 30
  max_cost_usd: 40.00
  max_wall_seconds: 10800
```

## 15.4 Seleção de workflow

```text
1. flag explícita             brian run --workflow governed
2. trigger por risco/caminho  governed.yaml declara seus gatilhos
3. política do contexto       policy_set define o padrão do cliente
4. padrão global              fast
```

## 15.5 Fronteira Workflow × Reasoning (D-3, §16)

**[Δ]** O v0.1 tinha um Workflow Engine "dono do estado" e um Reasoning Engine com planner, evaluator e replanner que claramente influenciavam transições. A fronteira nunca foi declarada. Isso produz acoplamento em poucos meses.

Fronteira formal:

```text
WORKFLOW ENGINE
  É uma máquina de estados determinística.
  É a ÚNICA autoridade de transição.
  Não chama LLM.
  Não interpreta resultado.
  Recebe PhaseOutcome e aplica a tabela de transições.
  Persiste antes de agir (D-12).

REASONING ENGINE
  Produz PROPOSTAS: plano, diagnóstico, avaliação, replanejamento.
  Nunca transiciona estado.
  Nunca escreve na tabela de runs.
  Sua saída é um input para o Workflow, como qualquer gate.

GATES
  Produzem GateResult determinístico (passou/não passou + achados).
  Também são apenas inputs.
```

Em código:

```rust
// Único ponto de transição do sistema.
impl WorkflowEngine {
    fn advance(&mut self, run: &mut Run, outcome: PhaseOutcome)
        -> Result<Transition>
    {
        let def = self.workflow_def(run.workflow_id, run.workflow_version)?;
        let phase = def.phase(&run.current_phase)?;

        // limites primeiro
        if run.total_phases >= def.limits.max_total_phases {
            return self.terminate(run, TerminationReason::PhaseLimit);
        }
        if run.cost_usd >= def.limits.max_cost_usd {
            return self.terminate(run, TerminationReason::CostLimit);
        }

        let next = match outcome {
            PhaseOutcome::Success => phase.on_success.clone(),
            PhaseOutcome::Failure => phase.on_failure.clone(),
            PhaseOutcome::Paused(r) => return self.pause(run, r),
        };

        let entries = run.phase_entry_count(&next);
        if let Some(max) = phase.max_entries {
            if entries >= max {
                let fallback = phase.on_max_entries
                    .clone()
                    .unwrap_or(PhaseId::escalate());
                return self.transition(run, fallback);
            }
        }

        self.transition(run, next)
    }
}
```

Nenhuma chamada de LLM nesta função. Se aparecer uma, a fronteira foi violada.

## 15.6 Estado persistido

```text
run.current_phase
run.phase_history        [(phase, started_at, ended_at, outcome, cost)]
run.total_phases
run.cost_usd
run.status               pending|running|paused|completed|failed|cancelled
run.pause_reason
run.workflow_id
run.workflow_version     [Δ] congelado no início do run (§113)
```

**[Δ]** `workflow_version` é congelado quando o run começa. Editar `fast.yaml` não pode alterar o comportamento de um run em andamento nem a interpretação de um run histórico.

---

# 16. Reasoning Engine

Produz propostas. Nunca decide transições (§15.5).

```text
reasoning/
├── classifier
├── planner
├── diagnoser
├── evaluator
└── replanner
```

**[Δ]** O v0.1 incluía `router` e `context` dentro de reasoning. Foram movidos: `router` é §11 (é decisão de controle, não de raciocínio) e `context` é §18 (é hipótese separada).

## 16.1 Classifier

Determina os atributos que alimentam roteamento e seleção de workflow.

```json
{
  "task_type": "implementation",
  "risk": "high",
  "complexity": "medium",
  "horizon": "short",
  "required_capabilities": ["code.write", "tests.run"],
  "touches_sensitive_paths": true,
  "sensitive_paths": ["src/payment/RefundService.ts"],
  "confidence": 0.81,
  "method": "heuristic+llm"
}
```

**[Δ]** `method` importa. A classificação começa **heurística** — caminhos tocados, palavras-chave, tamanho do diff, presença de migração — e só chama LLM quando a heurística é ambígua. Classificar toda tarefa com LLM adiciona latência e custo a 100% dos runs para melhorar talvez 20%.

Ordem:

```text
1. regras determinísticas sobre caminhos e padrões
2. se confidence < 0.7 → LLM com model_pointer: compact
3. se ainda < 0.5 → assume o mais conservador (risk: high)
```

Assumir o pior em caso de incerteza é a política correta: erra para o lado de mais governança, não de menos.

## 16.2 Planner

Transforma spec ou pedido em plano executável.

```json
{
  "plan_id": "pl_331",
  "tasks": [
    {
      "id": "T1",
      "description": "Adicionar tratamento idempotente de refund",
      "acceptance": [
        "requisições duplicadas retornam o mesmo resultado",
        "chave de idempotência é persistida",
        "teste de concorrência cobre 2 requisições simultâneas"
      ],
      "estimated_files": [
        "src/payment/RefundService.ts",
        "src/payment/RefundRepository.ts"
      ]
    }
  ],
  "risks": [
    "requisições concorrentes",
    "migração de schema exige janela"
  ],
  "out_of_scope": [
    "refatorar PaymentService",
    "trocar o cliente HTTP"
  ]
}
```

**[Δ]** `out_of_scope` é novo e é o mecanismo de contenção mais barato do sistema. Declarar explicitamente o que não fazer reduz expansão de escopo por parte do agente mais do que qualquer instrução genérica de restrição.

## 16.3 Diagnoser

Quando um gate falha, produz causa raiz antes de tentar correção.

```json
{
  "failure_type": "test_failure",
  "failing_tests": ["RefundService > duplicate request returns same id"],
  "root_cause_hypothesis": "chave de idempotência gerada por requisição, não derivada do payload",
  "evidence": [
    "src/payment/RefundService.ts:112 usa uuid()",
    "teste espera determinismo sobre (order_id, amount)"
  ],
  "confidence": 0.88,
  "plan_still_valid": true
}
```

## 16.4 Replanner

```text
Implementação
    ↓
Falha
    ↓
Diagnoser
    ↓
Plano ainda válido?
   /            \
 sim            não
  ↓              ↓
 fix           replan
```

`max_entries` no workflow (§15.3) impede loop infinito entre `fix` e `verify`.

## 16.5 Evaluator

**[Δ]** O v0.1 mencionava "Judge evaluates compliance" (§80, passo 14) sem definir critério. Um LLM julgando conformidade sem rubrica é carimbo.

O Evaluator recebe uma **rubrica explícita** e retorna evidência por item:

```json
{
  "run_id": "run_1421",
  "rubric": "acceptance_criteria",
  "items": [
    {
      "criterion": "requisições duplicadas retornam o mesmo resultado",
      "verdict": "pass",
      "evidence_type": "test",
      "evidence": "RefundService.spec.ts:44 passou",
      "llm_involved": false
    },
    {
      "criterion": "chave de idempotência é persistida",
      "verdict": "pass",
      "evidence_type": "code",
      "evidence": "migration 0042 adiciona idempotency_key com unique index",
      "llm_involved": true,
      "confidence": 0.91
    },
    {
      "criterion": "teste de concorrência cobre 2 requisições simultâneas",
      "verdict": "fail",
      "evidence_type": "absence",
      "evidence": "nenhum teste com Promise.all encontrado em RefundService.spec.ts",
      "llm_involved": true,
      "confidence": 0.76
    }
  ],
  "overall": "fail",
  "blocking_items": 1
}
```

Regras:

```text
- Todo veredito carrega tipo de evidência.
- evidence_type: test  → determinístico, LLM não participa
- evidence_type: code  → LLM lê, mas cita localização verificável
- evidence_type: absence → maior taxa de erro; nunca bloqueia sozinho
  em modo autônomo sem revisão humana
- Um veredito sem evidência citável é inválido.
```

---

# 17. Impact Engine

Combina sinais determinísticos para estimar alcance de uma mudança.

```text
OpenSpec + Code Graph + Git Diff + Memória + Testes
```

Saída:

```json
{
  "risk": "high",
  "affected_modules": 7,
  "critical_paths": ["checkout → payment → ledger"],
  "recommended_reviews": ["concurrency", "database", "security"],
  "test_coverage_of_changed_lines": 0.62,
  "has_migration": true,
  "touches_public_api": false
}
```

Influencia:

```text
seleção de workflow (fast vs governed)
model_pointer
gates obrigatórios
exigência de aprovação humana
```

**[Δ] Condicionado.** O Impact Engine só é construído no v1.0+ (§4) e sua versão v0.x é deliberadamente burra:

```yaml
# impact/heuristics.yaml
high_risk_paths:
  - "src/payment/**"
  - "src/auth/**"
  - "**/migrations/**"
  - "**/*.tf"

high_risk_signals:
  - files_changed_gt: 15
  - lines_changed_gt: 400
  - has_migration: true
  - deletes_test_file: true
```

Uma heurística de trinta linhas captura a maior parte do valor. O Impact Engine completo, com grafo de código, só se justifica quando houver evidência de que a heurística erra com frequência — o que é mensurável comparando risco previsto contra achados reais de gate.

---

# 18. Context Governor — Hipótese H-1

**[Δ] Rebaixado de subsistema para hipótese.** Esta é a segunda mudança mais importante do documento.

## 18.1 A hipótese

```text
H-1

Pré-montar um pacote de contexto mínimo, usando grafo de código,
busca simbólica, git diff e memória, reduz o CUSTO TOTAL de um
change bem-sucedido em pelo menos 30%, comparado a deixar o agente
fazer sua própria busca com o repositório inteiro disponível.
```

## 18.2 Por que não pode ser assumida

**Prompt caching inverte a economia.** Leitura de cache custa uma fração do input não-cacheado. Um prefixo grande e estável entre turnos pode custar menos que um prefixo pequeno recalculado, porque o segundo invalida o cache a cada turno. Otimizar bytes sem modelar cache aumenta a conta.

**Busca agêntica supera retrieval pré-computado em código.** O agente decide o que ler durante o loop, condicionado ao que já descobriu. Um pacote montado antes do primeiro turno não tem essa informação. Na prática ele chega incompleto, o agente busca assim mesmo, e paga-se duas vezes.

**O custo dominante é o loop, não o turno 1.** Um run de implementação tem 20 a 60 turnos. O contexto inicial é uma fração pequena do total acumulado. Reduzir o turno 1 em 40% pode mover o total em menos de 5%.

**A qualidade pode piorar.** Contexto pré-filtrado que omite o arquivo certo produz implementação errada, que produz retry, que custa mais que o contexto completo teria custado.

## 18.3 Experimento

Executado no Milestone 3 (§100), com o Governor implementado em versão descartável.

```text
POPULAÇÃO
  30 changes reais, do histórico do próprio usuário,
  distribuídos entre: bug pequeno, feature média, refactor.

BRAÇOS
  A — baseline: agente com repositório inteiro, busca própria
  B — governor: pacote mínimo pré-montado, sem busca livre
  C — híbrido: pacote mínimo como dica, busca livre permitida

MÉTRICA PRIMÁRIA
  custo total em USD por change BEM-SUCEDIDO
  (inclui retries; changes que falharam contam o custo gasto)

MÉTRICAS SECUNDÁRIAS
  tokens totais
  tokens de cache read vs input
  turnos até conclusão
  taxa de sucesso na primeira tentativa
  latência de parede

CRITÉRIO DE ACEITAÇÃO
  B ou C reduz a métrica primária em ≥ 30% vs A,
  sem redução de taxa de sucesso maior que 5 pontos.

CRITÉRIO DE DESCARTE
  Redução < 15%, ou queda de sucesso > 10 pontos.
  → Governor é removido do roadmap. §95 perde esse diferencial.
```

## 18.4 Pipeline, se confirmada

```text
Tarefa
 ↓
Filtro por spec
 ↓
Query no grafo de código
 ↓
Busca simbólica
 ↓
Git diff
 ↓
Recuperação de memória
 ↓
Deduplicação
 ↓
Compressão
 ↓
Orçamento de tokens
 ↓
ContextPackage
```

```text
context/
├── retriever
├── graph_selector
├── symbol_selector
├── memory_selector
├── deduplicator
├── compressor
├── reference_store
└── budget_manager
```

## 18.5 Isolamento arquitetural (D-5, §2.8)

Nada depende do Governor. O caminho padrão produz um pacote trivial:

```rust
enum ContextPackage {
    /// Padrão. O agente busca sozinho.
    Repository {
        root: PathBuf,
        change_ref: Option<ChangeId>,
        hints: Vec<String>,       // no máximo instruções curtas
    },
    /// Produzido pelo Governor, se H-1 confirmar.
    Curated {
        symbols: Vec<SymbolRef>,
        diffs: Vec<DiffRef>,
        memories: Vec<MemoryRef>,
        token_estimate: u64,
    },
}
```

Providers consomem `ContextPackage` sem saber a origem. Se `H-1` falhar, remove-se a variante `Curated` e o módulo `context/`. Nenhuma outra parte do sistema muda.

---

# 19. Lazy Context Loading

Condicionado a `H-1`, mas com um componente que vale independentemente.

## 19.1 Referências em vez de conteúdo

Em vez de enviar arquivos inteiros:

```text
PaymentService.capture   @ src/payment.ts:81-143
Checkout.finalize        @ src/checkout.ts:201-248
Ledger.record            @ src/ledger.ts:51-103
```

E expor ferramentas para o agente resolver sob demanda:

```text
code.read_symbol
code.read_range
memory.get
trace.get
```

## 19.2 O valor que não depende de H-1

**[Δ]** Mesmo que `H-1` falhe economicamente, lazy loading entrega uma coisa separada e valiosa: **atribuição**. Quando o agente pede explicitamente um símbolo, Brian sabe o que foi efetivamente usado. Isso alimenta:

```text
auditoria — "quais arquivos influenciaram esta mudança?"
memória   — vincular decisão aos símbolos reais
impacto   — validar a heurística de risco contra uso observado
```

Portanto: a instrumentação de "o que o agente leu" é construída no v0.2 (é barata, vem dos logs de sessão), independentemente do destino do Governor.

---

# 20. Orçamentos de Contexto

```yaml
context_budgets:
  plan:        { max_tokens: 30000,  action_on_exceed: warn }
  implement:   { max_tokens: 120000, action_on_exceed: warn }
  verify:      { max_tokens: 25000,  action_on_exceed: warn }
  review:      { max_tokens: 40000,  action_on_exceed: warn }
  fix:         { max_tokens: 60000,  action_on_exceed: warn }
```

**[Δ] Duas correções:**

1. **Orçamento não é limite.** `action_on_exceed: warn` é o padrão. Cortar contexto no meio de um run degrada qualidade de forma invisível. O orçamento existe para detectar anomalia, não para forçar comportamento. Limites duros ficam em custo e turnos (§9.1), que são observáveis e acionáveis.

2. **Os valores subiram.** O v0.1 propunha 60k para implementação. Runs reais de implementação com busca agêntica acumulam muito mais. Um orçamento irreal só gera alarme constante que o usuário aprende a ignorar.

## 20.1 Escalonamento por confiança

Padrão de economia que vale independentemente do Governor:

```text
1. modelo barato faz reconhecimento
   → resposta com confiança alta?  usa
   → confiança baixa?              escala

2. modelo forte recebe o resultado do reconhecimento
   como contexto, não parte do zero
```

Aplicável a: classificação (§16.1), diagnóstico de falha, triagem de achado de gate.

Não aplicável a: implementação (o custo do reconhecimento não se paga quando a tarefa principal é longa).

---

# 21. Code Intelligence Layer

Capacidades normalizadas de análise de código, independentes de implementação.

```text
code.search
code.symbol
code.references
code.dependencies
code.dependents
code.impact
code.graph
code.architecture
code.read_symbol
code.read_range
```

Providers nunca sabem qual ferramenta respondeu.

```rust
trait CodeGraph {
    fn symbol(&self, name: &str) -> Result<Vec<SymbolRef>>;
    fn references(&self, sym: &SymbolRef) -> Result<Vec<SymbolRef>>;
    fn dependencies(&self, sym: &SymbolRef) -> Result<Vec<SymbolRef>>;
    fn dependents(&self, sym: &SymbolRef) -> Result<Vec<SymbolRef>>;
    fn impact(&self, diff: &Diff) -> Result<ImpactReport>;
}

trait CodeSearch {
    fn search(&self, q: &Query) -> Result<Vec<Match>>;
}

trait StructuralQuery {
    fn find(&self, pattern: &AstPattern) -> Result<Vec<Match>>;
    fn rewrite(&self, pattern: &AstPattern, to: &AstPattern)
        -> Result<Vec<Edit>>;
}
```

## 21.1 Ordem de adoção

**[Δ]** O v0.1 listava cinco ferramentas (§22–§26) como se todas fossem adotadas. Este documento ordena por relação custo-benefício e condiciona cada uma a um problema observado.

```text
1. ast-grep      barato, determinístico, valor imediato        v0.4
2. ripgrep       busca textual — já existe, zero custo         v0.2
3. code graph    caro de manter; só se H-1 confirmar           v1.0+
4. Sourcebot     só com múltiplos repositórios reais           v1.0+
5. Archify       visualização; valor de explicação, não de execução  v1.0+
```

Regra de adoção: **nenhuma ferramenta de code intelligence entra sem uma pergunta concreta que ela responde e que hoje é respondida de forma pior.**

## 21.2 Custo de indexação

**[Δ novo].** Ausente no v0.1 e material.

Todo grafo de código exige indexação. Isso tem custo de tempo, disco e invalidação.

```text
repositório de 50k linhas      índice inicial ~ 30s,  ~40 MB
repositório de 500k linhas     índice inicial ~ 6min, ~400 MB
invalidação                    por commit, incremental
```

Para um usuário com 10 contextos de cliente, isso é gigabytes de índice e minutos de espera no primeiro `connect`. O onboarding de repositório precisa ser explícito e assíncrono (§114).

---

# 22. Graphify

Papel:

```text
mapa do repositório
grafo de símbolos
relações de dependência
descoberta de impacto
seleção de contexto
```

Perguntas que responde:

```text
Quais arquivos importam?
O que depende disto?
O que esta mudança vai afetar?
Que código deve entrar no contexto do LLM?
```

**[Δ]** As três primeiras perguntas têm valor independente de `H-1` — servem a impacto e auditoria. A quarta depende de `H-1`. Se `H-1` falhar, Graphify ainda pode ser adotado, mas com prioridade muito menor, e provavelmente substituído pela heurística de §17.

---

# 23. code-review-graph

Papel:

```text
consultas de grafo avançadas
análise focada em revisão
relações de mudança
análise de impacto
```

Tratado como implementação intercambiável de `CodeGraph` (§21). A escolha entre Graphify e code-review-graph é feita por medição, não por preferência, e é reversível porque ambos ficam atrás da mesma trait.

Critério de escolha:

```text
qualidade do grafo em TypeScript e Swift  (as linguagens do usuário)
tempo de indexação
tamanho do índice
frequência de manutenção do projeto upstream
licença
```

---

# 24. Sourcebot

Papel:

```text
busca de código entre repositórios
definições
referências
navegação
```

**[Δ] Condicionado a multi-repo real.** Sourcebot resolve um problema de organização com muitos repositórios. Um usuário com um repositório por cliente e busca local não tem esse problema. Adotar antes é infraestrutura sem demanda.

Gatilho de adoção: um contexto com **≥ 4 repositórios** onde consultas cruzadas acontecem em runs reais.

---

# 25. ast-grep

Papel:

```text
consultas estruturais em AST
detecção de padrão semântico
codemods
reescritas estruturadas seguras
```

**[Δ] Promovido.** É a ferramenta de melhor relação custo-benefício da lista: binário único, sem índice persistente, sem servidor, resultado determinístico.

Sempre que possível, prefira operação em AST a pedir a um LLM que faça busca e substituição textual ampla.

```yaml
# exemplo de regra
id: no-direct-uuid-in-idempotency
language: typescript
rule:
  pattern: |
    const $KEY = uuid()
  inside:
    kind: method_definition
    has:
      pattern: idempotency
severity: warning
message: "chave de idempotência deve derivar do payload"
```

Usos previstos:

```text
gate determinístico de padrão de código
codemod de refactor mecânico
verificação de convenção antes de acionar review por LLM
```

---

# 26. Archify

Papel:

```text
representação de arquitetura
diagramas de sistema
relações entre módulos
explicação visual
```

```text
Code Graph → Archify → Contexto de arquitetura → Planner / Reviewer
```

**[Δ]** Valor primário é **explicação para humano**, não insumo de LLM. Diagramas gerados raramente melhoram a saída de um agente e frequentemente adicionam tokens. Posicionado como feature de UI (§71), não de pipeline.

---

# 27. Improve

`shadcn/improve` tratado como capacidade de auditoria e planejamento.

```text
O que devemos melhorar?
```

Modos:

```text
quick · standard · deep · security · performance · tests · branch · next · plan
```

Fluxo:

```text
Achado do Improve → revisão do Brian → proposta de OpenSpec
```

**[Δ]** Com uma restrição: achados de auditoria **nunca viram runs automaticamente**. Uma auditoria profunda gera dezenas de achados; convertê-los em trabalho autônomo é a forma mais rápida de queimar budget em melhorias que ninguém pediu.

O fluxo correto é:

```text
achado → fila de propostas → usuário seleciona → vira change → vira run
```

---

# 28. Ponytail

Ponytail é uma **política de contenção**, não uma ferramenta.

```text
Isto precisa existir?
O projeto já tem algo equivalente?
A biblioteca padrão resolve?
Uma dependência existente resolve?
Dá para tocar menos arquivos?
Dá para escrever menos código?
```

## 28.1 Implementação concreta

**[Δ]** O v0.1 descrevia Ponytail conceitualmente. Na prática, contenção funciona melhor como **restrição declarada no prompt** e **gate de diff**, não como fase separada de LLM.

```text
COMO INSTRUÇÃO (custo zero)
  Injetado no prompt de implementação:
  - out_of_scope do plano (§16.2)
  - "prefira modificar arquivos existentes a criar novos"
  - "não adicione dependências sem justificar"

COMO GATE (determinístico, barato)
  - novos arquivos criados > N          → aviso
  - package.json / Cargo.toml alterado  → exige justificativa
  - linhas adicionadas > 3× o estimado  → aviso
  - novo diretório de topo criado       → exige aprovação
```

Uma fase de LLM dedicada a perguntar "isto precisa existir?" custa uma chamada por run para produzir, na maioria das vezes, "sim". O gate de diff captura os casos reais por uma fração do custo.

---

# 29. Open Code Review (OCR)

Motor especializado de revisão de qualidade.

```text
Implementação → Testes → OCR
                          │
                    achados?
                    /       \
                  sim        não
                   ↓          ↓
                  fix    review semântico / validação
```

Achados normalizados:

```json
{
  "finding_id": "f_991",
  "severity": "high",
  "file": "src/payment/RefundService.ts",
  "line": 112,
  "category": "concurrency",
  "rule": "non-deterministic-idempotency-key",
  "message": "chave gerada por requisição não garante idempotência",
  "source": "ocr",
  "confidence": 0.9,
  "suggested_fix": null
}
```

## 29.1 Política de bloqueio

**[Δ novo].** O v0.1 não definia o que um achado faz.

```text
critical  → bloqueia sempre, mesmo em modo autônomo
high      → bloqueia; em fast workflow vira pausa para humano
medium    → registra; não bloqueia; entra no relatório do run
low       → registra apenas
```

Achados não bloqueantes são acumulados por contexto e viram candidatos a change (§27), não trabalho imediato.

---

# 30. Security Gates

```text
Semgrep           SAST
OSV Scanner       dependências vulneráveis
Secret Scanner    credenciais em diff
SkillSpector      skills e plugins de terceiros
```

Capacidades expostas:

```text
security.sast
security.dependencies
security.secrets
security.skill_scan
```

## 30.1 Escopo de execução

**[Δ]** Gates de segurança rodam sobre o **diff**, não sobre o repositório inteiro, exceto na primeira execução por contexto.

```text
primeira execução no contexto   → baseline completo, resultado armazenado
execuções seguintes             → apenas o diff, comparado ao baseline
```

Sem isso, todo run de um repositório legado reporta centenas de achados pré-existentes e o gate vira ruído que o usuário desliga.

```json
{
  "gate": "semgrep",
  "scope": "diff",
  "baseline_id": "bl_2026_08_01",
  "new_findings": 1,
  "pre_existing_findings": 143,
  "resolved_findings": 0,
  "blocking": true
}
```

## 30.2 Secret scanning

Único gate que roda **antes** de qualquer commit ou push, sempre, em qualquer workflow, incluindo o fast path.

```text
TruffleHog ou gitleaks
```

Um secret vazado é o único achado que não tem correção barata depois do fato.

---

# 31. SkillSpector

Toda skill ou plugin de terceiro é código privilegiado.

```text
Nova skill
   ↓
SkillSpector
   ↓
Análise estática
   ↓
Inspeção de capacidades
   ↓
Score de risco
   ↓
Política
├── Allow
├── Restricted
├── Sandbox
└── Block
```

Metadados de confiança:

```yaml
skill:
  name: exemplo
  source: github.com/org/repo
  commit: a3f21b8
  trust: unverified
  risk: medium
  scanned_at: 2026-08-08T09:00:00Z
  declared_capabilities: [code.read, code.write]
  detected_capabilities: [code.read, code.write, network.external]
  discrepancy: true
```

**[Δ]** O campo `discrepancy` é o achado mais valioso: quando as capacidades detectadas por análise estática excedem as declaradas, isso é sinal forte. É também barato de implementar — grep por chamadas de rede, escrita de arquivo fora do worktree, execução de shell.

Esta seção do v0.1 estava correta e é uma das mais prescientes do documento original.

---

# 32. Modelo de Skill

```text
Skill  = como um agente deve executar um tipo de trabalho
Tool   = capacidade que o agente pode invocar
Phase  = o que o workflow exige agora
Role   = papel do perfil de provider (builder, reviewer, planner)
```

**[Δ]** `Role` foi adicionado. O v0.1 usava `role` em perfis (§9) sem posicioná-lo no vocabulário, criando ambiguidade com `skill`.

```yaml
skill:
  name: debug-failure
  version: 2

instructions:
  - leia o erro completo antes de agir
  - identifique a causa raiz, não o sintoma
  - inspecione o diff atual
  - altere apenas arquivos relacionados à causa
  - rode o teste que falhou
  - rode a suíte de regressão

allowed_tools:
  - code.read
  - code.write
  - git.diff
  - tests.run

denied_tools:
  - git.push
  - network.external

applies_to_phases: [fix]
applies_to_roles: [builder]
```

Composição:

```text
implement-feature
├── understand-spec
├── inspect-codebase
├── design-change
├── implement
├── test
└── self-review
```

## 32.1 Relação com skills nativas de provider

**[Δ novo].** Providers modernos têm seu próprio sistema de skills, plugins e subagents. Brian **não reimplementa** isso.

```text
Se o provider suporta skills nativas
  → Brian instala/aponta a skill no diretório de identidade do provider
  → o provider executa nativamente
  → Brian só governa QUAL skill está disponível em QUAL contexto

Se o provider não suporta
  → Brian injeta as instruções no prompt
```

Isso é `D-10` aplicado: não competir com feature nativa de provider, e sim governar a distribuição dela entre clientes.

---

# 33. Camada MCP

Brian expõe um MCP Server para agentes.

```text
openspec.get_change
openspec.get_tasks
openspec.validate

workflow.get_state
workflow.claim_phase
workflow.complete_phase
workflow.reject_phase

code.search
code.read_symbol
code.dependencies
code.impact

workspace.read
workspace.write
git.diff
git.status

tests.run
lint.run

memory.search
memory.suggest

review.submit

finops.current_run_cost      [Δ novo]
```

## 33.1 Autoridade (D-15)

**O MCP Server do Brian é o único caminho de escrita no estado de workflow.** Agentes nunca escrevem no banco diretamente. Não existe caminho alternativo.

```text
Agente → MCP tool → validação → Workflow Engine → persistência
```

Cada chamada é autenticada por `run_id` e verificada contra o perfil ativo:

```rust
fn handle_tool_call(&self, call: ToolCall) -> Result<ToolResponse> {
    let run = self.runs.get(&call.run_id)?;
    let profile = self.profiles.get(&run.profile_id)?;

    if !profile.allows(&call.capability) {
        return Err(Error::CapabilityDenied {
            capability: call.capability,
            profile: profile.id.clone(),
        });
    }

    if call.touches_workflow_state() && !profile.role.can_transition() {
        return Err(Error::WorkflowAuthorityDenied);
    }

    self.dispatch(call)
}
```

## 33.2 `finops.current_run_cost`

**[Δ novo].** Expor o custo acumulado do run ao próprio agente permite comportamento autolimitante:

```text
"você já consumiu $3.20 de um limite de $5.00 neste run"
```

Barato de implementar, e é o único mecanismo que dá ao agente informação para decidir encerrar em vez de continuar explorando.

## 33.3 Prioridade

O MCP Server fica no v1.0+ (§4). Antes disso, Brian executa providers em modo headless com contexto no prompt. A camada MCP só se justifica quando há workflows longos com múltiplas fases interativas — que é v0.3 em diante.

---

# 34. Memória Multi-Provider e Continuidade (D-17)

Memória pertence ao Brian. Não ao Claude, não ao Codex, não ao Gemini.

Este é o ativo mais durável do produto **e o segundo ganho mínimo** junto com D-16: o conhecimento e o estado de trabalho **sobrevivem** à troca de provider, à mudança de modelo e ao fim de uma assinatura — **sem o usuário reexplicar** e **sem re-pagar o mesmo raciocínio**.

**[Δ][D-17]** “Memória” deixa de ser só knowledge base de longo prazo. Inclui **continuidade operacional** para chavear LLM sem perda de contexto, conversa útil, análise e próximos passos.

## 34.0 Continuity Pack — o mínimo que não pode faltar

Artefato versionado por Context (e opcionalmente por run/thread):

```yaml
# continuity/<context_id>/current.yaml  (conceitual)
continuity_pack:
  version: 1
  context: xpto/payments-api
  updated_at: 2026-08-08T12:00:00Z
  from_provider: claude
  objective: "Tornar refund idempotente com chave (order_id, amount, day)"
  success_criteria:
    - "requests duplicados retornam o mesmo resultado"
    - "testes de RefundService passam"
  decisions:
    - id: mem_4821
      summary: "idempotency key derivada, não UUID por request"
  analysis:
    - "timeout do gateway causa retry; UUID por request não protege"
    - "descartado: lock distribuído — overkill no volume atual"
  conversation_digest: |
    Usuário pediu idempotência; exploramos RefundService e gateway;
    concordamos em chave composta; falta persistir e testar.
  touched:
    - path: src/payment/RefundService.ts
      symbols: [RefundService.execute]
  failed_attempts:
    - "teste X falhou por mock incompleto do gateway"
  next_steps:
    - "persistir chave"
    - "adicionar teste de duplicata"
  open_questions: []
  budget_hint:
    claude_week_used_percent: 42
  evidence_refs:
    - run_id: run_1421
    - usage_ids: [u_98, u_99]
```

Comandos:

```bash
brian memory note "..."                 # nota manual rápida
brian memory decide "..." --why "..."   # decisão com rationale
brian handoff --to codex                # gera pack + prepara próximo worker
brian continuity show
brian continuity inject --provider codex
```

Critério de aceitação D-17 (mínimo):

```text
Após 20 min de trabalho em Claude sob um context,
brian handoff --to codex
→ o usuário NÃO reexplica objetivo, decisões nem o que já falhou.
→ o Continuity Pack cita arquivos/símbolos reais do trabalho.
→ custo do pack (tokens injetados) é limitado e rotulado.
```

## 34.1 Tipos

```text
Continuity Pack         estado de handoff atual (D-17 mínimo) — quente
Working Memory          escopo de run/thread
Project Memory          fatos sobre o projeto
Decision Memory         decisões de arquitetura e seus motivos
Architecture Memory     estrutura, módulos, fronteiras
Episodic Memory         o que aconteceu em um run específico
Incident Memory         falhas de produção e suas causas
Provider Memory         desempenho observado por provider e tarefa
Conversation Digest     resumo estruturado da thread (não log bruto)
```

## 34.2 Registro

```json
{
  "id": "mem_4821",
  "type": "decision",
  "status": "active",
  "client": "xpto",
  "project": "payments-api",
  "content": "Operações de refund devem usar chave de idempotência derivada de (order_id, amount, day).",
  "rationale": "Requisições duplicadas do gateway são comuns em timeout. UUID por requisição não protege.",
  "provenance": {
    "run_id": "run_1421",
    "phase": "review",
    "provider": "claude",
    "model": "claude-opus-5",
    "trace_id": "tr_82ac",
    "created_at": "2026-08-08T11:02:00Z",
    "approved_by": "user",
    "approved_at": "2026-08-08T11:14:00Z"
  },
  "evidence": [
    { "type": "code",     "ref": "src/payment/RefundService.ts:112" },
    { "type": "incident", "ref": "inc_0031" },
    { "type": "test",     "ref": "RefundService.spec.ts:44" }
  ],
  "symbols": ["RefundService.execute", "RefundRepository.save"],
  "confidence": 0.94,
  "usage_count": 7,
  "last_used_at": "2026-08-07T09:31:00Z",
  "supersedes": null,
  "superseded_by": null
}
```

**[Δ] Quatro adições em relação ao v0.1:**

1. `rationale` separado de `content` — o motivo é mais valioso que a regra, e é o que impede um agente futuro de "corrigir" a decisão por desconhecimento.
2. `evidence` como lista tipada e referenciável — memória sem evidência verificável é boato.
3. `approved_by` / `approved_at` — proveniência de aprovação, não só de criação (§36).
4. `supersedes` / `superseded_by` — memória é append-only (`D-14`); correções criam novo registro.

## 34.3 Append-only (D-14)

Memória nunca é editada nem deletada. Uma decisão revista cria um novo registro que supersede o anterior:

```text
mem_4821  status: superseded   superseded_by: mem_5033
mem_5033  status: active       supersedes: mem_4821
```

Razões:

```text
auditoria     "o que Brian acreditava em março?"
depuração     "por que o agente fez isso? porque a memória vigente dizia X"
reversão      restaurar uma decisão revertida por engano é trivial
```

O custo é armazenamento, que é irrelevante nessa escala.

---

# 35. Recuperação de Memória

Brian não injeta toda a memória no prompt.

## 35.1 Ranking

```text
FILTRO DURO (não negociável)
  tenant/cliente         — cross-client é negado (§37)
  status = active

RANKING
  projeto                peso alto
  símbolos tocados       peso alto
  change atual           peso alto
  fase                   peso médio
  similaridade semântica peso médio
  recência               peso baixo
  usage_count            peso baixo
  confiança              multiplicador
```

## 35.2 Orçamento de memória

```yaml
memory_injection:
  max_items: 8
  max_tokens: 4000
  min_confidence: 0.7
  always_include_types: [decision, incident]
```

**[Δ]** Limites explícitos e baixos. O v0.1 dizia "apenas um pequeno número" sem quantificar. Oito itens é um número que cabe em atenção e é auditável quando o resultado dá errado.

## 35.3 Implementação por versão

```text
v0.4    SQLite FTS5 + filtro por símbolo tocado
v1.0+   embeddings, se FTS5 provar insuficiente
```

**[Δ]** Começar com busca textual. Um projeto com algumas centenas de memórias não precisa de vetores, e adicionar um backend de embedding é custo de infraestrutura antes de necessidade demonstrada.

---

# 36. Governança de Memória

Agentes sugerem. Brian decide o que se torna durável.

```text
Builder          memory.read ✓   memory.suggest ✓   memory.commit ✗
Reviewer         memory.read ✓   memory.suggest ✓   memory.approve ✓
Brian Core       memory.commit ✓
Usuário          memory.approve ✓   memory.supersede ✓
```

## 36.1 Estados de memória

```text
suggested     proposta por agente, não usada em recuperação
active        aprovada, entra em recuperação
superseded    substituída por registro mais novo
rejected      recusada, mantida para auditoria
```

**[Δ]** `suggested` não participa de recuperação. O v0.1 não separava isso, o que permitiria que uma alucinação de um agente virasse contexto de todos os runs seguintes.

## 36.2 Classes epistêmicas

```text
fact          verificável, com evidência determinística
decision      escolha deliberada, com rationale
hypothesis    não verificada — sempre rotulada como tal no prompt
observation   algo notado, sem conclusão
incident      evento real de produção
lesson        conclusão derivada de incidente
```

Hipóteses entram no contexto com marcação explícita:

```text
[HIPÓTESE, não verificada, confiança 0.6]
O lock de refund pode estar causando contenção sob carga.
```

## 36.3 Aprovação

```text
v0.4     toda memória exige aprovação humana explícita
v1.0+    memória do tipo `fact` com evidência determinística
         (teste passando, resultado de gate) pode ser auto-aprovada
```

Começar exigindo aprovação para tudo produz atrito, mas produz também um conjunto pequeno e confiável. O caminho inverso — aprovar automaticamente e limpar depois — não funciona, porque ninguém limpa.

---

# 37. Isolamento de Memória

Memória é isolada por cliente e projeto.

```text
XPTO
├── architecture
├── decisions
├── incidents
├── patterns
└── outcomes

ACME
└── ...

Interno
└── ...
```

## 37.1 Regra dura

Recuperação cross-client é **negada por padrão** e não tem flag de override no v0.x.

```rust
fn retrieve(&self, q: &MemoryQuery, ctx: &Context) -> Result<Vec<Memory>> {
    // Não é filtro opcional. É pré-condição.
    let scoped = q.clone().with_client(ctx.client_id.clone());
    debug_assert_eq!(scoped.client, Some(ctx.client_id.clone()));
    self.store.search(&scoped)
}
```

## 37.2 Memória compartilhável

**[Δ novo].** Existe conhecimento que não é do cliente: padrões técnicos gerais, aprendizados sobre providers, convenções da própria consultoria.

```yaml
memory:
  namespace: client:xpto/project:checkout-api
  also_read:
    - namespace: org:workwise/shared
      types: [pattern, lesson]
      excludes_client_data: true
```

Regra: um namespace compartilhado **nunca** pode conter memória do tipo `incident`, `decision` ou `fact` originada de um cliente. Só padrões abstraídos, promovidos manualmente, sem referência a código ou dado de cliente.

A promoção é um ato explícito e auditado:

```bash
brian memory promote mem_4821 --to org:workwise/shared --anonymize
```

---

# 38. Learning Engine

**[Δ] Adiado para além do v1.0 (D-8).**

## 38.1 O problema de amostra

O v0.1 propunha estatística por tipo de tarefa:

```text
Migração de banco de dados
  Codex   sucesso 96%   retries 0.8   custo médio $0.71
  Claude  sucesso 91%   retries 1.1   custo médio $1.22
```

A questão não é se isso é útil — é. A questão é **quando esses números existem**.

```text
células = task_type × provider × model_tier
        = 8 × 5 × 3
        = 120 células

n mínimo por célula para diferença de 5 pontos ser detectável ≈ 30

runs necessários ≈ 3600
```

Um usuário solo executando 10 runs úteis por dia leva mais de um ano para preencher isso, e a distribuição real é enviesada — a maior parte cai em três ou quatro células.

## 38.2 Regra (D-8)

```text
n < 30 na célula    → histórico é EXIBIDO, nunca usado em decisão
n ≥ 30              → histórico entra como termo de desempate
n ≥ 100             → histórico pode ter peso maior que regras
```

Toda exibição de estatística carrega o `n`:

```text
Codex · implementação
  sucesso 94%  (n=12 — insuficiente para roteamento)
```

**[Δ]** Mostrar `n` é a diferença entre informação e ilusão de rigor.

## 38.3 O que é construído desde o v0.0

Não o motor de aprendizado, mas o **registro** que ele vai consumir:

```text
todo run grava: task_type, provider, model, sucesso, retries,
                custo, latência, contexto, achados de gate
```

Registrar é barato e irreversível se não for feito. Quando o volume chegar, os dados existem. Essa é a razão pela qual telemetria é v0.0 e não v0.4 (§3).

## 38.4 Alternativa de curto prazo

Enquanto `n` é pequeno, o mecanismo útil não é estatística — é **comparação pareada explícita**:

```bash
brian run "implementa X" --compare codex,claude
```

Executa a mesma tarefa nos dois, em worktrees separados, e apresenta os dois diffs, custos e resultados de gate lado a lado. O usuário escolhe. A escolha é registrada.

Isso gera dado de qualidade muito superior a observação passiva, e é útil no primeiro dia.

---

# 39. Telemetria

**[Δ] Movida para o v0.0.** É o alicerce de FinOps, auditoria e do futuro Learning Engine.

OpenTelemetry é o padrão interno.

```text
Run
└── Trace
    ├── context.resolve
    ├── worktree.create
    ├── router.decide
    ├── vault.resolve
    ├── provider.execute
    │   ├── provider.turn[0..n]
    │   └── provider.usage
    ├── gate.tests
    ├── gate.lint
    ├── gate.security
    ├── evaluator.assess
    ├── memory.propose
    └── worktree.finalize
```

## 39.1 Atributos de span

```text
brian.client_id
brian.project_id
brian.context_id
brian.run_id
brian.change_id
brian.workflow_id
brian.workflow_version
brian.phase
brian.provider
brian.provider_version
brian.integration_tier
brian.model
brian.model_pointer
brian.role
brian.skill
brian.tokens.input
brian.tokens.cached_input
brian.tokens.output
brian.tokens.reasoning
brian.cost.usd
brian.cost.source
brian.usage.source
brian.retry_index
brian.tool
brian.status
brian.worktree_id
```

**[Δ]** `cost.source`, `usage.source` e `integration_tier` propagados em todo span são o que permite auditar a qualidade do próprio dado depois. Um relatório de custo cujos números vieram de estimativa precisa dizer isso na linha, não no rodapé.

## 39.2 Backends

```text
v0.0      arquivo local (OTLP JSON), lido pela CLI
v0.3      exportador opcional para backend do usuário
v1.0+     OTel Collector em enterprise
```

Não é necessário rodar Jaeger local para o produto funcionar. A CLI lê o arquivo.

## 39.3 Traces grandes

Saída bruta de provider (stdout, stderr, diffs) não vai para o trace. Vai para o artifact store, e o span carrega a referência.

```text
traces        SQLite, indexados, consultáveis
artefatos     ~/.brian/artifacts/<run_id>/, comprimidos
retenção      traces: indefinida (são pequenos)
              artefatos: 90 dias por padrão, configurável
```

---

# 40. Contabilidade de Tokens

Brian registra, quando disponível:

```text
input_tokens
cached_input_tokens
output_tokens
reasoning_tokens
total_tokens
```

## 40.1 Origem obrigatória

Toda medição carrega origem:

```text
provider        reportado pelo provider em saída estruturada
session_file    lido de arquivo de sessão do provider
estimated       calculado por tokenizer local
unknown         não foi possível determinar
```

**Regra dura: um número `estimated` nunca é apresentado como consumo certificado.** Relatórios mostram a composição:

```text
Agosto · XPTO

  Tokens        18.4M
    reportados  16.1M   (87%)
    estimados    2.3M   (13%)
```

## 40.2 Cache como categoria de primeira classe

**[Δ novo].** O v0.1 listava `cached_input_tokens` sem tratamento especial. Dado o papel do caching na economia real (§18.2), o cache precisa de visibilidade própria:

```text
input não-cacheado    1.0×  preço base
input cacheado        ~0.1× preço base
escrita de cache      ~1.25× preço base
```

Métrica derivada, exposta na UI e na CLI:

```text
cache_hit_ratio = cached_input_tokens / (input_tokens + cached_input_tokens)
```

Uma taxa de cache baixa em runs longos é o sinal mais acionável de desperdício que o Brian pode oferecer — e é observável desde o v0.0, sem construir nada.

---

# 41. Price Catalog

**[Δ] Rebaixado a fallback (D-6).**

## 41.1 Ordem de precedência

```text
1. custo reportado pelo provider     → autoritativo
2. cálculo via price catalog         → declarado como calculado
3. sem custo                         → exibido como indisponível, nunca zero
```

O v0.1 tratava o catálogo como fonte central. Manter preços versionados de cinco providers é trabalho perpétuo e propenso a erro. Quando o provider reporta `total_cost_usd`, esse número é melhor por definição, e não custa manutenção.

## 41.2 Schema

```text
provider
model
effective_from
effective_to
input_per_million
cached_input_per_million
cache_write_per_million
output_per_million
reasoning_per_million
source_url
recorded_at
recorded_by
```

## 41.3 Manutenção

```text
catálogo versionado no repositório, em YAML
atualização é PR, com source_url obrigatório
run histórico referencia catalog_version usada
recálculo retroativo é possível, mas nunca automático
```

**[Δ]** `catalog_version` gravada por run garante que corrigir um preço errado não altere silenciosamente relatórios já emitidos a clientes. Recalcular é comando explícito:

```bash
brian finops recalculate --from 2026-07-01 --catalog-version 12
```

---

# 42. Modelo de Custo

Cada run registra:

```text
measured_tokens
estimated_api_cost
provider_reported_cost
billing_mode
cost_confidence
```

## 42.1 Modos de cobrança

```text
api             pago por token, custo real ≈ custo calculado
subscription    assinatura fixa, custo marginal ≈ 0
credits         créditos pré-pagos
mixed           contexto usa providers de modos diferentes
unknown         não determinado
```

## 42.2 A distinção central

```text
Custo equivalente em API  ≠  fatura real
```

Quando providers são autenticados por assinatura, o custo marginal de um token é zero até o limite do plano, e infinito depois. O v0.1 já fazia essa distinção, que é um dos pontos mais fortes do documento original.

**[Δ] Extensão necessária:** com assinatura, a métrica útil não é custo, é **fração de capacidade consumida**.

```text
Assinatura de R$ 1.000/mês
XPTO consumiu 42% da capacidade → custo alocado R$ 420
ACME consumiu 31%               → R$ 310
Interno consumiu 27%            → R$ 270
```

Essa alocação proporcional é o que uma consultoria efetivamente precisa para faturar, e é diferente de somar preços de tabela de API.

```yaml
billing:
  mode: subscription
  plan_cost_monthly_brl: 1000
  allocation_method: proportional_tokens   # ou proportional_runs, proportional_cost_equiv
```

## 42.3 Confiança do custo

```text
high      todos os runs do período com custo reportado
medium    > 70% reportado
low       < 70% reportado, resto estimado
```

Exposto em todo relatório. Uma fatura emitida sobre dado `low` deve ser visivelmente marcada.

---

# 43. FinOps e Usage Control Plane

**[Δ][D-16]** Promovido a v0.0 e elevado a **lei do produto**. Não é só “relatório de custo no fim do mês”. É o **controle contínuo** de tokens, assinaturas, janelas, % de uso, tempo restante e otimização de cada centavo — **antes** de orquestração, UI ou Governor.

Dois lados do mesmo subsistema:

```text
USAGE CONTROL (capacidade)
  quanto gastei · quanto resta · em qual janela · a que burn rate
  planos e assinaturas · alertas · limites soft/hard

FINOPS (atribuição e dinheiro)
  de quem é o token · showback · chargeback · export
  $ reportado vs estimado · fração de assinatura alocada
```

Toda unidade de consumo é atribuível a:

```text
cliente
projeto
change
run
fase
provider
modelo
identidade
skill
janela (day/week/month/plan_reset)
```

## 43.1 Relatório de capacidade (minuto 0 — mais importante que o de run)

```text
brian capacity

PROVIDER   PLANO / MODO        JANELA     USADO    RESTANTE   RESET        BURN/h    ALERTA
claude     Max · subscription  week       42%      5.8M tok   2d 11h       85k       —
codex      Plus · subscription week       71%      2.1M tok   2d 11h       120k      ⚠ 80%
gemini     API · paygo         month      $38.2    —          23d          $1.10/h   —

Unattributed (7d):  0 tokens
Cache hit (7d):     64%
Top clients (7d):   xpto 41% · acme 33% · internal 26%
```

## 43.2 Relatório de run

```text
XPTO / antifraud-v2 / run #1421

FASE              PROVIDER   MODELO       TOKENS    CUSTO    ORIGEM
plan              claude     opus-5        42.1k    $1.20    provider
implement         codex      strong       184.3k    $4.82    provider
verify            codex      cheap         12.0k    $0.11    provider
fix               codex      strong        38.7k    $0.93    provider
review            claude     opus-5        21.4k    $0.41    provider
─────────────────────────────────────────────────────────────────────
Total                                     298.5k    $7.47

Cache hit ratio                                      64%
Confiança do custo                                   alta
Retries                                              1
Tempo de parede                                      18m 42s
Impacto na janela week/claude                        +0.4%
```

## 43.3 Cadeia de atribuição

```text
token
  → chamada de provider
    → sessão (observe) ou fase/run (orchestrated)
      → change (se houver)
        → projeto
          → cliente
            → budget
              → janela de capacidade do plano
```

Todo elo é obrigatório **ou** o registro fica `unattributed` de forma explícita e ruidosa. Um token silencioso sem dono é bug (D-16 §13.9). Em observe mode, run/fase podem ser null; cliente **não** pode ser null sem flag unattributed.

## 43.4 Consultas

```bash
brian capacity                           # lei D-16: resposta em < 5s
brian usage --window week --by client
brian usage --window week --by provider
brian costs                              # mês corrente, todos os clientes
brian costs --client xpto
brian costs --client xpto --by provider
brian costs --client xpto --by phase
brian costs --period 2026-07
brian costs --change refund-idempotency
brian costs --export csv > agosto.csv
brian costs --unattributed               # deve retornar vazio
brian import --since 30d
brian attribute <id> --client xpto
```

`--unattributed` é o teste de integridade do sistema.  
`brian capacity` é o teste de **controle** do sistema.

## 43.5 Otimização de aproveitamento (cada centavo)

Objetivo de produto, não experimento:

```text
maximizar  trabalho útil atribuído
minimizar  capacidade desperdiçada (retries cegos, unattributed, modelo errado,
           fases LLM inúteis, compare sem necessidade, burn em off-hours sem dono)
```

No v0.0 (só observe + ledger):

```text
- 100% de captura e atribuição
- % e restante por janela
- alertas de soft limit
- insights determinísticos (§13.8)
- export para decisão humana de onde gastar o resto da semana
```

No v0.2+ (com run):

```text
- defaults de model_pointer por pressão de capacidade
- recusa educada de workflows caros perto do hard limit
- finops.current_run_cost exposto ao agente (quando MCP existir)
```

H-1 (Governor) só entra se **além** deste controle provar redução extra de $/sucesso. O controle de capacidade **não espera** H-1.

---

# 44. Showback e Chargeback

```text
Showback     reportar o que um cliente consumiu
Chargeback   calcular o que deve ser faturado
```

```text
XPTO · Agosto 2026

Consumo de IA (equivalente API)      $124.30
Alocação de assinatura                $87.00
Infraestrutura Brian                  $18.00
──────────────────────────────────────────────
Custo interno                        $229.30

Markup contratado (1.6×)             $366.88
```

**[Δ]** Política comercial pertence à organização, não ao runtime. Brian calcula custo e expõe um campo de markup configurável por cliente. Não decide preço.

```yaml
clients:
  xpto:
    billing:
      markup: 1.6
      minimum_monthly_brl: 2000
      includes_infrastructure: true
```

## 44.1 Formato de entrega

```text
v0.0    CSV e tabela em terminal
v0.3    PDF por cliente, com detalhamento por change
v1.0+   integração com sistema de faturamento
```

---

# 45. Budgets, Planos e Limites de Capacidade

**[Δ][D-16]** Budgets de cliente e **capacidade de plano/assinatura** são duas faces do mesmo controle. Ambos existem no v0.0 (alertas + ledger); bloqueio hard de run entra com execução (v0.2), mas a **definição e o monitoramento** já são minuto 0.

Budgets existem em:

```text
organização
cliente
projeto
change
run
fase
provider
plan / identity          ← capacidade da assinatura ou cota API
```

```yaml
clients:
  xpto:
    budgets:
      monthly_tokens: 50000000
      monthly_usd_equivalent: 500
      weekly_share_of_subscription_percent: 40   # teto de fração da capacidade semanal
      change_soft_limit_usd: 10
      change_hard_limit_usd: 20
      run_hard_limit_usd: 5
      alert_at_percent: [50, 80, 95]

providers:
  claude:
    plan: claude_max_seat
    budgets:
      week_used_percent_soft: 80
      week_used_percent_hard: 98
      day_tokens_soft: 2000000
```

## 45.1 Comportamento em limite suave

```text
notifica o usuário (CLI + registro)
prefere model_pointer mais barato (quando routing existir)
desaconselha compare multi-provider e evals caros
reserva modelos fortes para fases de revisão / governed
em observe mode: só alerta (não pode interceptar o CLI do provider)
```

## 45.2 Comportamento em limite duro

```text
nenhuma nova chamada de LLM iniciada PELO Brian no escopo
runs em andamento entram em `paused`, não `failed`
evento gravado no ledger/trace com motivo
override exige comando explícito e é auditado
em observe mode: Brian não pode matar o processo do provider;
                 grava VIOLATION se consumo continuar após alerta hard
                 (transparência > falsa sensação de bloqueio)
```

```bash
brian budget override --client xpto --reason "entrega crítica, aprovado por João" --limit 800
brian budget override --provider claude --reason "reset amanhã, fechar sprint" --soft 95
```

**[Δ]** Override sempre exige motivo textual e fica registrado. Um limite que pode ser contornado sem rastro não é um controle.

## 45.3 Precedência

Do mais restritivo ao menos: o limite efetivo é sempre o **menor** entre os níveis aplicáveis.

```text
run_hard_limit              $5
change_hard_limit          $20
client weekly share         40% da capacidade do plano
provider week hard          98%
monthly client             $500
                          ────
efetivo = interseção (o mais restritivo que se aplica agora)
```

## 45.4 Assinatura: controle por fração de janela

Quando `billing_mode = subscription`, o budget primário **não** é só USD de catálogo:

```text
used_percent_window     métrica primária de controle
remaining_time          para planejar o que ainda cabe na janela
allocated_plan_cost     $ do plano × fração consumida (showback)
```

Exemplo de decisão humana que o Brian deve tornar óbvia:

```text
Restam 18% da semana no Claude · 2d 11h até reset
XPTO já levou 41% do consumo da semana
→ próxima feature pesada: Codex (29% usado) ou esperar reset?
```

Isso é **otimização de cada centavo da assinatura** — e é P0 de produto, não relatório opcional.

---

# 46. Browser Bridge

Brian integra com o navegador real do usuário, em vez de depender de automação para navegação comum.

```text
Safari / Chrome
      │
 Extensão Brian
      │
      ▼
   Brian.app
```

O usuário **anexa explicitamente** uma aba a um contexto. Não há monitoramento passivo.

Dados possíveis, sujeitos a permissão:

```text
URL
título da página
texto selecionado
snapshot do DOM
árvore de acessibilidade
screenshot
console
diagnóstico de rede
```

## 46.1 Modelo de permissão

**[Δ]** O v0.1 dizia "Brian não deve monitorar silenciosamente". Isso precisa de mecanismo, não de intenção:

```text
padrão                 nenhuma aba anexada
anexar aba             ação explícita do usuário, por aba
escopo                 apenas a aba anexada, apenas enquanto anexada
indicação visual       badge persistente na aba anexada
expiração              desanexa ao fechar a aba ou após 30 min de inatividade
dados sensíveis        campos de senha e inputs marcados nunca são capturados
```

## 46.2 Prioridade

**v1.0+.** O Browser Bridge é uma extensão de navegador — uma quarta toolchain (TypeScript/WebExtension), com processo de revisão de loja, para um caso de uso que não é o gargalo do produto. `D-2` e §2.7 o excluem do caminho crítico.

---

# 47. Política de Automação de Navegador

```text
teste local / staging      Playwright é aceitável
uso real de navegador      navegador real + Browser Bridge
```

Brian **não** inclui lógica de anti-detecção ou bypass de anti-bot. Isso não é uma limitação técnica; é uma decisão de produto que evita uma categoria inteira de risco legal e reputacional.

---

# 48. Integração Xcode

Xcode é tratado como ferramenta de desenvolvedor, atrás de um adapter.

```text
Brian → Provider → Xcode bridge → Build / Test / Diagnostics
```

Casos de uso:

```text
implementar view SwiftUI
compilar projeto
rodar testes
coletar diagnósticos do compilador
```

**[Δ] v1.0+.** A integração Xcode só se justifica quando o Brian é usado para desenvolver Swift, o que na prática significa quando o próprio Brian é o projeto. Isso é dogfooding legítimo, mas não é o caminho crítico de produto — e `xcodebuild` na linha de comando já resolve 80% do valor sem adapter dedicado.

```text
v0.2    `xcodebuild` como gate genérico de teste, sem adapter
v1.0+   adapter Xcode com diagnósticos estruturados
```

---

# 49. Brian Chat

Interface em linguagem natural sobre o plano de controle. **Não é** o plano de controle.

Perguntas:

```text
XPTO está no ar?
Qual foi o último deploy?
Por que o checkout está lento?
Quais PRs estão bloqueados?
Quanto de IA a XPTO consumiu este mês?
O que o OpenSpec atual exige?
Por que Brian escolheu Codex?
```

Ações:

```text
Cria um OpenSpec para este incidente.
Roda a mudança atual.
Revisa este diff.
Muda para ACME.
Pausa o run atual.
```

**[Δ] v1.0+.** Chat é a interface mais cara de construir bem e a menos necessária para provar o produto. A CLI (§79) responde a todas as perguntas acima com comandos determinísticos, sem custo de LLM e sem ambiguidade. Chat é conveniência sobre uma base que precisa existir primeiro.

Chat herda o contexto ativo. Sempre.

---

# 50. Roteamento de Intenção no Chat

```text
"XPTO está no ar?"           → health check / observabilidade
"Quem chama este método?"    → code graph / AST
"Por que o deploy falhou?"   → CI + logs + diff recente + LLM
"Quanto a XPTO custou?"      → consulta SQL no banco de FinOps
"Qual o estado do run?"      → consulta SQL
```

Regra: **Brian não invoca LLM quando uma consulta estruturada resolve.** Isso é §2.3 aplicado à interface.

Implementação:

```text
1. casamento de intenção por padrão (regex/keywords)  → consulta direta
2. sem casamento                                       → LLM com model_pointer: compact
                                                         traduz para chamada de ferramenta
3. LLM nunca responde de memória sobre estado do sistema
```

---

# 51. Arquitetura da Aplicação macOS

```text
UI                   Swift + SwiftUI              v0.3
Integração macOS     Swift                        v0.3
Core Runtime         Rust                         v0.0
IPC                  Unix socket + JSON-RPC       v0.3
Banco                SQLite (rusqlite)            v0.0
Secrets              macOS Keychain               v0.1
Autenticação         LocalAuthentication          v0.1
Extensão navegador   TypeScript/WebExtension      v1.0+
Telemetria           OpenTelemetry (Rust SDK)     v0.0
CLI                  Rust (clap)                  v0.0
Specs                OpenSpec (opcional)          v0.2
```

## 51.1 IPC: Unix socket, não XPC

**[Δ]** O v0.1 propunha XPC. Decisão revista:

```text
XPC              nativo macOS, seguro, mas amarra o core ao macOS
                 e exige bindings Swift↔Rust não triviais

Unix socket      JSON-RPC sobre socket em ~/.brian/brian.sock
+ JSON-RPC       funciona em macOS e Linux sem alteração
                 depurável com nc e jq
                 permite que a CLI, a UI e futuros clientes usem
                 exatamente o mesmo protocolo
                 permissão via modo de arquivo (0600)
```

A vantagem decisiva é que **a UI e a CLI falam o mesmo protocolo**. No v0.0 só existe a CLI; quando a UI chegar no v0.3, ela não exige nenhuma API nova — consome a que já foi validada por meses de uso via terminal.

---

# 52. Workspace de Desenvolvimento

**[Δ]** O v0.1 abria com `Brian.xcworkspace`, o que implica começar pelo app macOS. Dado `D-2`, o repositório começa como projeto Rust puro e ganha o workspace Xcode no v0.3.

## 52.1 v0.0 – v0.2

```text
brian/
├── Cargo.toml              workspace Rust
├── core/
├── cli/
├── adapters/
├── migrations/
├── workflows/
├── routing/
├── models/
├── docs/
└── tests/
```

Uma toolchain. Um comando de build. `cargo test` roda tudo.

## 52.2 v0.3 em diante

```text
brian/
├── Brian.xcworkspace
├── Cargo.toml
├── core/
├── cli/
├── adapters/
├── macos/
│   ├── Brian.xcodeproj
│   ├── BrianApp/
│   └── BrianTests/
├── browser/               v1.0+
├── migrations/
├── workflows/
├── openspec/
├── schemas/
├── docs/
└── tests/
```

Xcode orquestra o build do produto macOS. Cargo continua dono do Rust. A UI consome o socket, não uma biblioteca linkada — o que elimina a necessidade de bindings Swift↔Rust por completo.

---

# 53. Layout do Brian Core

```text
core/src/
├── lib.rs
├── context/
│   ├── manager.rs
│   ├── resolver.rs
│   └── model.rs
├── identity/
│   ├── manager.rs
│   ├── profile.rs
│   └── injection.rs
├── vault/
│   ├── mod.rs
│   ├── keychain.rs
│   └── policy.rs
├── providers/
│   ├── registry.rs
│   ├── traits.rs
│   ├── capabilities.rs
│   ├── compatibility.rs
│   └── <provider>/
├── models/
│   ├── pointers.rs
│   └── resolution.rs
├── run/
│   ├── manager.rs
│   ├── lifecycle.rs
│   └── recovery.rs
├── worktree/
│   ├── manager.rs
│   └── gc.rs
├── workflow/
│   ├── engine.rs
│   ├── definition.rs
│   └── transitions.rs
├── reasoning/
│   ├── classifier.rs
│   ├── planner.rs
│   ├── diagnoser.rs
│   ├── evaluator.rs
│   └── replanner.rs
├── spec/
│   ├── traits.rs
│   └── openspec.rs
├── memory/
│   ├── store.rs
│   ├── retrieval.rs
│   └── governance.rs
├── code_intelligence/
├── tools/
├── quality/
│   ├── gates.rs
│   └── <gate>/
├── policy/
├── telemetry/
│   ├── tracer.rs
│   └── spans.rs
├── finops/
│   ├── accounting.rs
│   ├── catalog.rs
│   └── reporting.rs
├── storage/
│   ├── traits.rs
│   ├── sqlite/
│   └── migrations.rs
├── ipc/
│   ├── server.rs
│   └── protocol.rs
├── runtime/
└── platform/
    ├── mod.rs
    ├── macos/
    └── kubernetes/        vazio até v1.0+
```

**[Δ]** `context/`, `worktree/`, `run/` e `ipc/` são novos ou promovidos. `code_intelligence/` e `reasoning/` existem como diretórios vazios até suas versões.

---

# 54. Adapters de Plataforma

```text
platform/
├── mod.rs                 traits
├── macos/
│   ├── process.rs
│   ├── keychain.rs
│   ├── biometric.rs
│   ├── launchd.rs
│   └── fs.rs
└── kubernetes/            v1.0+
```

```rust
trait PlatformRuntime {
    fn spawn(&self, spec: ProcessSpec) -> Result<ProcessHandle>;
    fn kill(&self, h: &ProcessHandle) -> Result<()>;
    fn secret_store(&self) -> &dyn SecretStore;
    fn workspace_root(&self) -> PathBuf;
    fn schedule(&self, task: ScheduledTask) -> Result<()>;
}
```

**Regra de disciplina (§2.6):** nenhuma chamada de API específica de sistema operacional fora de `platform/`. Isso é verificável por lint no CI:

```bash
# CI: falha se APIs de macOS aparecerem fora de platform/macos
! rg -t rust 'security_framework|core_foundation|objc' \
     core/src --glob '!core/src/platform/macos/**'
```

Agnosticismo verificado automaticamente custa uma linha de CI. Agnosticismo por intenção não sobrevive a seis meses.

---

# 55. Arquitetura de Processos no macOS

```text
brian (CLI)  ──┐
               ├──► ~/.brian/brian.sock ──► brian-core (daemon)
Brian.app    ──┘                                  │
                                                  ├── processos de provider
                                                  ├── SQLite
                                                  ├── worktrees
                                                  ├── telemetria
                                                  └── gates
```

## 55.1 Ciclo de vida do daemon

```text
v0.0    sem daemon; a CLI abre o banco diretamente
v0.2    daemon opcional, iniciado sob demanda pela CLI
v0.3    daemon gerenciado por launchd, sobrevive ao fechamento da UI
```

**[Δ]** Não começar com daemon. No v0.0 o produto é um relatório: a CLI abre o SQLite, lê, escreve, fecha. Um daemon só é necessário quando há runs longos que precisam sobreviver ao término do processo da CLI — o que é v0.2.

## 55.2 Concorrência de acesso ao banco

```text
SQLite em modo WAL
busy_timeout = 5000ms
uma escrita por vez, leituras concorrentes
o daemon é o único escritor quando está ativo
```

---

# 56. Session Manager e PTY

Para providers em Tier 3 (§8.2), Brian mantém sessões em pseudo-terminal.

```text
Session Manager
├── claude:session-38
├── codex:session-72
└── grok:session-04
```

Coleta:

```text
stdout / stderr
status da sessão
invocações de ferramenta, quando observáveis
consumo
estado de saída
```

## 56.1 Restrições (D-4)

```text
PTY nunca é a primeira escolha
PTY nunca opera em modo autônomo
PTY nunca é fonte autoritativa de custo
saída de PTY é sempre arquivada bruta para reprocessamento
parsers de PTY são versionados por versão de provider
```

## 56.2 Reconexão

O usuário pode reconectar a uma sessão viva pela UI ou pela CLI:

```bash
brian attach run_1421
```

**[Δ]** Isso conecta o terminal do usuário ao PTY do provider, com passagem transparente de entrada. É o mecanismo de intervenção humana (§111) e é a razão principal pela qual PTY continua existindo apesar de suas desvantagens: é o único modo que permite ao humano assumir o controle no meio.

---

# 57. Estratégia de Storage

Classes de dado separadas por natureza, não forçadas em um único sistema.

```text
Dados operacionais       SQLite                 (D-1)
Secrets                  macOS Keychain
Artefatos grandes        filesystem, comprimido
Telemetria               SQLite + arquivo OTLP
Índices de código        ferramentas próprias
```

---

# 58. Por que SQLite e não SurrealDB

**[Δ] O v0.1 argumentava a favor do SurrealDB e propunha um spike (§60, §61). Este documento cancela o spike e decide (D-1).**

## 58.1 O argumento do v0.1

SurrealDB é atraente porque Brian precisa de registros tipo documento, relações, consultas tipo grafo, execução embarcada local e caminho para operação remota. Isso está correto como enumeração de necessidades.

## 58.2 Por que não se sustenta

**As consultas propostas no §61 do v0.1 são triviais em SQLite.**

Relações de memória — cadeia projeto → decisão → trace → provider → símbolos:

```sql
WITH RECURSIVE chain(id, depth) AS (
    SELECT id, 0 FROM memory WHERE id = ?1
  UNION ALL
    SELECT m.id, c.depth + 1
    FROM memory m
    JOIN memory_link l ON l.to_id = m.id
    JOIN chain c ON c.id = l.from_id
    WHERE c.depth < 5
)
SELECT m.*, c.depth FROM memory m JOIN chain c ON c.id = m.id;
```

FinOps por cliente, provider, fase — agregação comum com índice composto.

Runs e transições — série temporal com índice em `(run_id, started_at)`.

Nenhuma dessas exige engine de grafo.

**O grafo do v0.1 §58 é modelagem, não requisito de engine.** As arestas descritas — `Client HAS_PROJECT Project`, `Run USED_PROVIDER Provider`, `Symbol CALLS Symbol` — são tabelas de junção. Grafo de verdade se justifica com travessia de profundidade variável e desconhecida em milhões de arestas. O grafo do Brian é raso e conhecido.

**Fatores operacionais decidem:**

```text
                     SQLite              SurrealDB embarcado
maturidade           30 anos             recente
formato              estável, versionado formato próprio
ferramental          universal           limitado
migração             padrão consolidado  próprio
backup               copiar um arquivo   procedimento próprio
depuração            qualquer cliente    cliente próprio
FTS                  FTS5 nativo         próprio
crash safety         testado à exaustão  menos maduro
risco de fornecedor  nenhum              single-vendor
```

**Custo de oportunidade.** O spike do §61 do v0.1 consumiria de duas a três semanas. Essas semanas, aplicadas ao v0.0 (§81), produzem um produto utilizável. Aplicadas ao spike, produzem uma decisão que a análise acima já toma com confiança suficiente.

## 58.3 Critério de reversão (D-1)

```text
Uma query de produção real exceder 200 ms com 12 meses de dados
de uso realista, e a otimização por índice não resolver.
```

Porque `D-9` mantém todo acesso atrás de traits, essa troca é uma implementação nova de trait, não uma reescrita.

---

# 59. Abstração de Storage

Nenhuma query SQL fora de `storage/` (`D-9`).

```rust
trait ContextStore {
    fn get(&self, id: &ContextId) -> Result<Option<Context>>;
    fn list(&self, filter: &ContextFilter) -> Result<Vec<Context>>;
    fn upsert(&self, ctx: &Context) -> Result<()>;
}

trait RunStore {
    fn create(&self, run: &Run) -> Result<RunId>;
    fn get(&self, id: &RunId) -> Result<Option<Run>>;
    fn update_phase(&self, id: &RunId, t: &Transition) -> Result<()>;
    fn list_active(&self) -> Result<Vec<Run>>;
    fn list_orphaned(&self) -> Result<Vec<Run>>;      // §110
}

trait UsageStore {
    fn record(&self, u: &UsageRecord) -> Result<()>;
    fn aggregate(&self, q: &UsageQuery) -> Result<UsageAggregate>;
    fn unattributed(&self) -> Result<Vec<UsageRecord>>;
}

trait MemoryStore {
    fn search(&self, q: &MemoryQuery) -> Result<Vec<Memory>>;
    fn insert(&self, m: &Memory) -> Result<MemoryId>;
    fn supersede(&self, old: &MemoryId, new: &MemoryId) -> Result<()>;
}

trait TraceStore { /* ... */ }
trait PolicyStore { /* ... */ }
trait ProviderStore { /* ... */ }
```

Implementações:

```text
SqliteStore       v0.0
PostgresStore     v1.0+ enterprise
MemoryStore       testes
```

---

# 60. Schema SQLite

**[Δ novo].** O v0.1 não tinha schema. Este é o schema inicial completo do v0.0, que é a fundação de tudo o mais.

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ─────────────────────────────────────── contexto

CREATE TABLE client (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    archived_at     TEXT
);

CREATE TABLE project (
    id              TEXT PRIMARY KEY,
    client_id       TEXT NOT NULL REFERENCES client(id),
    name            TEXT NOT NULL,
    repo_path       TEXT,
    created_at      TEXT NOT NULL,
    UNIQUE(client_id, name)
);

CREATE TABLE context (
    id              TEXT PRIMARY KEY,
    client_id       TEXT NOT NULL REFERENCES client(id),
    project_id      TEXT REFERENCES project(id),
    schema_version  INTEGER NOT NULL,
    config_json     TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- ─────────────────────────────────────── identidade e providers

CREATE TABLE identity_profile (
    id              TEXT PRIMARY KEY,
    client_id       TEXT NOT NULL REFERENCES client(id),
    name            TEXT NOT NULL,
    config_json     TEXT NOT NULL,
    created_at      TEXT NOT NULL
);

CREATE TABLE provider (
    id                  TEXT PRIMARY KEY,
    executable          TEXT NOT NULL,
    version_detected    TEXT,
    integration_tier    TEXT NOT NULL,
    capabilities_json   TEXT NOT NULL,
    attached            INTEGER NOT NULL DEFAULT 0,
    last_verified_at    TEXT
);

-- ─────────────────────────────────────── execução

CREATE TABLE run (
    id                  TEXT PRIMARY KEY,
    context_id          TEXT NOT NULL REFERENCES context(id),
    client_id           TEXT NOT NULL REFERENCES client(id),
    project_id          TEXT REFERENCES project(id),
    change_id           TEXT,
    workflow_id         TEXT NOT NULL,
    workflow_version    INTEGER NOT NULL,
    current_phase       TEXT,
    status              TEXT NOT NULL,
    pause_reason        TEXT,
    worktree_id         TEXT,
    task_type           TEXT,
    risk                TEXT,
    prompt              TEXT NOT NULL,
    total_phases        INTEGER NOT NULL DEFAULT 0,
    cost_usd            REAL NOT NULL DEFAULT 0,
    cost_confidence     TEXT,
    heartbeat_at        TEXT,
    created_at          TEXT NOT NULL,
    started_at          TEXT,
    ended_at            TEXT
);

CREATE INDEX idx_run_client_created  ON run(client_id, created_at DESC);
CREATE INDEX idx_run_status          ON run(status) WHERE status IN ('running','paused');
CREATE INDEX idx_run_change          ON run(change_id);

CREATE TABLE phase_execution (
    id              TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES run(id) ON DELETE CASCADE,
    phase_id        TEXT NOT NULL,
    entry_index     INTEGER NOT NULL,
    provider_id     TEXT REFERENCES provider(id),
    model           TEXT,
    model_pointer   TEXT,
    role            TEXT,
    outcome         TEXT,
    cost_usd        REAL,
    started_at      TEXT NOT NULL,
    ended_at        TEXT
);

CREATE INDEX idx_phase_run ON phase_execution(run_id, started_at);

CREATE TABLE worktree (
    id              TEXT PRIMARY KEY,
    run_id          TEXT REFERENCES run(id),
    project_id      TEXT NOT NULL REFERENCES project(id),
    path            TEXT NOT NULL,
    branch          TEXT NOT NULL,
    base_commit     TEXT NOT NULL,
    status          TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    released_at     TEXT
);

-- ─────────────────────────────────────── decisões de roteamento

CREATE TABLE routing_decision (
    id              TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES run(id) ON DELETE CASCADE,
    phase_id        TEXT NOT NULL,
    mode            TEXT NOT NULL,
    chosen_provider TEXT NOT NULL,
    chosen_pointer  TEXT NOT NULL,
    matched_rule    TEXT,
    signals_json    TEXT NOT NULL,
    considered_json TEXT NOT NULL,
    override_by     TEXT,
    decided_at      TEXT NOT NULL
);

-- ─────────────────────────────────────── consumo e custo

CREATE TABLE usage_record (
    id                      TEXT PRIMARY KEY,
    run_id                  TEXT REFERENCES run(id),
    phase_execution_id      TEXT REFERENCES phase_execution(id),
    session_ref             TEXT,                  -- sessão do provider (observe)
    client_id               TEXT REFERENCES client(id),  -- null só se unattributed
    project_id              TEXT REFERENCES project(id),
    provider_id             TEXT NOT NULL,
    identity_profile_id     TEXT,
    model                   TEXT,
    input_tokens            INTEGER,
    cached_input_tokens     INTEGER,
    output_tokens           INTEGER,
    reasoning_tokens        INTEGER,
    total_tokens            INTEGER,
    requests                INTEGER DEFAULT 1,
    usage_source            TEXT NOT NULL,          -- provider|brian_measured|estimated
    cost_usd                REAL,
    cost_source             TEXT NOT NULL,          -- provider|catalog|allocated_subscription|unknown
    catalog_version         INTEGER,
    billing_mode            TEXT NOT NULL,          -- api|subscription|credits|mixed|unknown
    attribution_status      TEXT NOT NULL DEFAULT 'attributed', -- attributed|unattributed
    occurred_at             TEXT NOT NULL,
    ingested_at             TEXT NOT NULL
);

CREATE INDEX idx_usage_client_time ON usage_record(client_id, occurred_at);
CREATE INDEX idx_usage_run         ON usage_record(run_id);
CREATE INDEX idx_usage_provider    ON usage_record(provider_id, occurred_at);
CREATE INDEX idx_usage_unattr      ON usage_record(attribution_status, occurred_at)
    WHERE attribution_status = 'unattributed';

CREATE TABLE price_catalog (
    id                          INTEGER PRIMARY KEY,
    version                     INTEGER NOT NULL,
    provider_id                 TEXT NOT NULL,
    model                       TEXT NOT NULL,
    effective_from              TEXT NOT NULL,
    effective_to                TEXT,
    input_per_million           REAL,
    cached_input_per_million    REAL,
    cache_write_per_million     REAL,
    output_per_million          REAL,
    reasoning_per_million       REAL,
    source_url                  TEXT,
    recorded_at                 TEXT NOT NULL
);

-- planos / assinaturas (denominador de % e showback)  [D-16]
CREATE TABLE provider_plan (
    id                  TEXT PRIMARY KEY,
    provider_id         TEXT NOT NULL,
    label               TEXT NOT NULL,
    billing_mode        TEXT NOT NULL,
    plan_cost           REAL,
    currency            TEXT,
    primary_window      TEXT NOT NULL,          -- calendar_week|plan_reset|...
    baseline_json       TEXT NOT NULL,          -- capacidade declarada por janela
    alerts_json         TEXT NOT NULL,
    limits_json         TEXT NOT NULL,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE identity_plan_binding (
    identity_profile_id TEXT NOT NULL,
    provider_plan_id    TEXT NOT NULL REFERENCES provider_plan(id),
    active_from         TEXT NOT NULL,
    active_to           TEXT,
    PRIMARY KEY (identity_profile_id, provider_plan_id, active_from)
);

CREATE TABLE capacity_snapshot (
    id                  TEXT PRIMARY KEY,
    provider_id         TEXT NOT NULL,
    identity_profile_id TEXT,
    provider_plan_id    TEXT,
    window_type         TEXT NOT NULL,          -- calendar_day|calendar_week|...
    window_start        TEXT NOT NULL,
    window_end          TEXT NOT NULL,
    consumed_tokens     INTEGER NOT NULL DEFAULT 0,
    consumed_requests   INTEGER NOT NULL DEFAULT 0,
    capacity_tokens     INTEGER,               -- null se desconhecido
    used_percent        REAL,
    remaining_tokens    INTEGER,
    remaining_percent   REAL,
    resets_at           TEXT,
    burn_tokens_per_hour REAL,
    eta_exhaustion_at   TEXT,
    quota_state         TEXT NOT NULL,          -- available|limited|exhausted|unknown|...
    source              TEXT NOT NULL,          -- provider|brian_measured|plan_baseline|mixed
    computed_at         TEXT NOT NULL
);

CREATE INDEX idx_capacity_window ON capacity_snapshot(
    provider_id, window_type, window_start DESC
);

CREATE TABLE budget (
    id              TEXT PRIMARY KEY,
    scope_type      TEXT NOT NULL,              -- org|client|project|provider|plan|run|...
    scope_id        TEXT NOT NULL,
    config_json     TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    UNIQUE(scope_type, scope_id)
);

CREATE TABLE budget_override (
    id              TEXT PRIMARY KEY,
    budget_id       TEXT NOT NULL REFERENCES budget(id),
    reason          TEXT NOT NULL,
    new_limit_json  TEXT NOT NULL,              -- usd e/ou used_percent
    authorized_by   TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    expires_at      TEXT
);

CREATE TABLE attribution_correction (
    id              TEXT PRIMARY KEY,
    usage_record_id TEXT NOT NULL REFERENCES usage_record(id),
    from_client_id  TEXT,
    to_client_id    TEXT NOT NULL,
    reason          TEXT NOT NULL,
    authorized_by   TEXT NOT NULL,
    created_at      TEXT NOT NULL
);

-- ─────────────────────────────────────── memória

CREATE TABLE memory (
    id              TEXT PRIMARY KEY,
    type            TEXT NOT NULL,
    status          TEXT NOT NULL,
    client_id       TEXT NOT NULL REFERENCES client(id),
    project_id      TEXT REFERENCES project(id),
    namespace       TEXT NOT NULL,
    content         TEXT NOT NULL,
    rationale       TEXT,
    confidence      REAL,
    usage_count     INTEGER NOT NULL DEFAULT 0,
    last_used_at    TEXT,
    supersedes      TEXT REFERENCES memory(id),
    superseded_by   TEXT REFERENCES memory(id),
    provenance_json TEXT NOT NULL,
    approved_by     TEXT,
    approved_at     TEXT,
    created_at      TEXT NOT NULL
);

CREATE INDEX idx_memory_ns_status ON memory(namespace, status);

CREATE VIRTUAL TABLE memory_fts USING fts5(
    content, rationale,
    content = 'memory', content_rowid = 'rowid'
);

CREATE TABLE memory_evidence (
    id              TEXT PRIMARY KEY,
    memory_id       TEXT NOT NULL REFERENCES memory(id) ON DELETE CASCADE,
    type            TEXT NOT NULL,
    ref             TEXT NOT NULL
);

CREATE TABLE memory_symbol (
    memory_id       TEXT NOT NULL REFERENCES memory(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    PRIMARY KEY(memory_id, symbol)
);

-- ─────────────────────────────────────── qualidade e auditoria

CREATE TABLE gate_result (
    id                  TEXT PRIMARY KEY,
    run_id              TEXT NOT NULL REFERENCES run(id) ON DELETE CASCADE,
    phase_execution_id  TEXT REFERENCES phase_execution(id),
    gate                TEXT NOT NULL,
    scope               TEXT NOT NULL,
    passed              INTEGER NOT NULL,
    new_findings        INTEGER NOT NULL DEFAULT 0,
    pre_existing        INTEGER NOT NULL DEFAULT 0,
    report_ref          TEXT,
    executed_at         TEXT NOT NULL
);

CREATE TABLE finding (
    id              TEXT PRIMARY KEY,
    gate_result_id  TEXT NOT NULL REFERENCES gate_result(id) ON DELETE CASCADE,
    severity        TEXT NOT NULL,
    category        TEXT,
    rule            TEXT,
    file            TEXT,
    line            INTEGER,
    message         TEXT NOT NULL,
    confidence      REAL,
    status          TEXT NOT NULL DEFAULT 'open'
);

CREATE TABLE audit_log (
    id              INTEGER PRIMARY KEY,
    actor           TEXT NOT NULL,
    action          TEXT NOT NULL,
    subject_type    TEXT NOT NULL,
    subject_id      TEXT NOT NULL,
    client_id       TEXT,
    detail_json     TEXT,
    occurred_at     TEXT NOT NULL
);

CREATE INDEX idx_audit_time ON audit_log(occurred_at DESC);

-- ─────────────────────────────────────── telemetria

CREATE TABLE span (
    id              TEXT PRIMARY KEY,
    trace_id        TEXT NOT NULL,
    parent_id       TEXT,
    run_id          TEXT REFERENCES run(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    attributes_json TEXT NOT NULL,
    status          TEXT,
    started_at      TEXT NOT NULL,
    ended_at        TEXT
);

CREATE INDEX idx_span_trace ON span(trace_id, started_at);
CREATE INDEX idx_span_run   ON span(run_id);

-- ─────────────────────────────────────── migração

CREATE TABLE schema_migration (
    version         INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,
    applied_at      TEXT NOT NULL,
    checksum        TEXT NOT NULL
);
```

## 60.1 Observações de projeto

```text
IDs são TEXT com prefixo semântico (run_1421, mem_4821).
  Legíveis em log, coláveis em comando, ordenáveis por criação.

Timestamps são TEXT ISO-8601 UTC.
  SQLite não tem tipo de data; texto ISO ordena corretamente.

Configuração vive em colunas _json.
  Evolui sem migração. Campos consultados são promovidos a coluna
  quando um índice passa a ser necessário.

Toda tabela de consumo tem client_id desnormalizado.
  Relatório de FinOps é a consulta mais frequente do sistema
  e não deve depender de junção em cascata.

heartbeat_at em run existe para detecção de órfão (§110).
```

---

# 61. Spike de Storage — cancelado

**[Δ]** O spike proposto no §61 do v0.1 está cancelado por `D-1` e §58.

O que **substitui** o spike é um teste de carga sintético de meio dia, executado no v0.0, que gera doze meses de dados plausíveis e mede as consultas reais:

```bash
brian dev seed --months 12 --runs-per-day 15 --clients 5
brian dev bench
```

```text
Consulta                                    alvo      medido
custo por cliente, mês corrente             < 50ms    ...
custo por provider, 12 meses                < 100ms   ...
runs ativos                                 < 10ms    ...
histórico de fases de um run                < 10ms    ...
busca de memória (FTS5), 5k registros       < 50ms    ...
consumo não atribuído                       < 100ms   ...
cadeia de memória, profundidade 5           < 50ms    ...
```

Se todos passarem, `D-1` está confirmada e o assunto encerrado. Se algum falhar, o caminho é índice, não troca de banco — e só se índice não resolver o critério de reversão é acionado.

---

# 62. Filosofia de Design Visual

Brian deve parecer:

```text
calmo
preciso
confiável
nativo
```

Brian não deve parecer:

```text
neon de IA
cyberpunk
parede de dashboards
lançador de providers
sopa de cards
```

Evitar:

```text
gradientes decorativos
brilhos de IA
animação sem função
cards aninhados
métricas sem ação associada
branding de provider dominando a interface
```

## 62.1 Princípio operante

**[Δ]** Um critério concreto substitui a lista de proibições:

> Todo elemento visível responde a uma pergunta que o usuário tem, ou permite uma ação que ele quer tomar. Se não faz nenhum dos dois, sai.

Aplicado ao dashboard (§65), isso elimina a maior parte do que normalmente ocupa uma tela de ferramenta de IA.

## 62.2 Estados vazios e desconhecidos

O estado mais comum do Brian nos primeiros meses é **vazio ou desconhecido**: sem histórico suficiente, quota desconhecida (§13.2), memória vazia, `n` insuficiente.

Design que só funciona com dado cheio falha na semana 1. Cada tela precisa de um estado vazio projetado, que diga o que fazer para preenchê-lo.

```text
Memória · XPTO / checkout-api

  Nenhuma memória ainda.

  Memórias são criadas a partir de decisões tomadas durante runs
  e aprovadas por você. Depois do primeiro run revisado, elas
  aparecem aqui.

  [Rodar primeira tarefa]
```

---

# 63. Arquitetura de Informação

Quatro grupos mentais:

```text
TRABALHO              ENTENDER
├── Projetos          ├── Inspector
├── OpenSpec          ├── Memória
└── Runs              └── Grafo

OBSERVAR              CONTROLAR
├── Traces            ├── Contexto
├── Consumo           ├── Identidade
└── Custo             ├── Vault
                      └── Políticas
```

**[Δ]** Com uma reordenação: em uma consultoria, **Observar** é o grupo de maior uso diário, não Trabalho. A pergunta "quanto a XPTO consumiu?" é feita muito mais vezes que "rode este change". A navegação (§64) reflete isso.

---

# 64. Navegação Principal

```text
Hoje
Custo            ← [Δ] promovido para o topo
Runs
Projetos
Explorar
─────────
Vault
Ajustes
```

Dentro de Explorar:

```text
Providers
Memória
OpenSpec
Skills
Grafo
```

**[Δ]** O v0.1 colocava `Usage` em quinto lugar. Dado que FinOps é o v0.0 e o diferencial primário (§95), custo é item de primeiro nível.

---

# 65. Dashboard

Mostra apenas:

```text
cliente / projeto ativo
objetivo atual
fase atual
worker atual
próxima fase
consumo do run
custo equivalente
tempo decorrido
o que exige atenção
```

```text
XPTO / checkout-api

Implementando idempotência de refund

● Codex trabalhando · fase implement (2 de ~4)
████████████████░░░░ 73%

Próximo
verify → done

48k tokens · $0.38 · 01:42 · cache 61%

                                          [Inspecionar] [Pausar]
```

## 65.1 Quando não há run ativo

**[Δ]** O estado mais frequente. O v0.1 só desenhava o dashboard com run em andamento.

```text
XPTO / checkout-api                              [Rodar tarefa]

Agosto
  $124.30 de $500 (25%)          ████░░░░░░░░░░░░

Últimos runs
  #1421  refund idempotency    ✓ done      $7.47   ontem
  #1419  fix null check        ✓ done      $0.31   ontem
  #1418  migrate user table    ⏸ pausado   $2.10   2 dias

Atenção
  ⚠ 1 run pausado aguardando decisão
  ⚠ credencial AWS staging expira em 12 dias
```

---

# 66. Brian Inspector

> Explica decisões do sistema sem expor raciocínio interno do modelo.

```text
Por que Codex nesta fase?

Regra aplicada
  implementation + risk:high → codex / coding

Considerados
  codex     ✓ escolhido
  claude    — regra não casou
  gemini    ✗ não autenticado
  grok      ✗ negado por contrato do cliente

Sinais no momento da decisão
  task_type      implementation
  risk           high
  complexidade   medium
  contexto       42k tokens
  budget         $188.40 restante · estado normal

Histórico (não usado nesta decisão)
  codex · implementation   94% sucesso   n=12
  ⓘ n < 30, insuficiente para roteamento (D-8)

                                              [Ver trace] [Sobrescrever]
```

**[Δ]** Mostra evidência e fatores registrados no momento da decisão (§11.4), nunca uma explicação gerada depois. A linha sobre `n` insuficiente é o que separa transparência de teatro de transparência.

---

# 67. UI de Providers

Providers aparecem como infraestrutura, não como marcas.

```text
Codex                                        ● Pronto

Identidade      xpto-work
Conta           eng@xpto-consultoria.com.br
Versão          0.48.2 · verificada
Integração      headless JSON (tier 1)
Papel           Builder primário

Quota           desconhecida
                ⓘ este provider não reporta quota

Hoje            812k tokens · $4.21
Este mês        9.8M tokens · $52.30
Sucesso         94% (n=12)
```

**[Δ]** Quatro campos novos em relação ao v0.1: conta autenticada (§6.3), versão verificada (§8.3), tier de integração (§8.2) e a nota explícita sobre quota desconhecida (§13.2). Cada um evita uma classe de erro silencioso.

---

# 68. UI de OpenSpec

```text
refund-idempotency

PROPOSED → PLANNED → IMPLEMENTING ← atual → TESTING → REVIEW → VALIDATED

Critérios de aceitação
  ✓ requisições duplicadas retornam o mesmo resultado
      teste RefundService.spec.ts:44
  ✓ chave de idempotência é persistida
      migration 0042 · confiança 0.91
  ○ teste de concorrência cobre 2 requisições simultâneas
      nenhum teste encontrado · confiança 0.76
  ○ validação OpenSpec

Custo acumulado    $7.47 em 3 runs
```

**[Δ]** Cada critério mostra sua evidência e a confiança do veredito (§16.5). Um checkmark sem evidência clicável não é verificação.

---

# 69. UI de Trace

```text
run #1421 · 18m 42s · $7.47

context.resolve   ▏
worktree.create   ▏
router.decide     ▏
claude:plan          ████
codex:implement           ████████████████████
gate.tests                                    ███
gate.lint                                     ▏
codex:fix                                        ██████
gate.tests                                             ██
claude:review                                            ████
evaluator                                                    █
```

Ao clicar em um span:

```text
codex:implement

provider        codex 0.48.2 (headless JSON)
modelo          coding → gpt-5-codex
identidade      xpto-work
duração         8m 12s
tokens          184.3k   (in 142k · cache 89k · out 42.3k)
custo           $4.82    origem: provider
cache hit       63%
turnos          34
ferramentas     read×61 · write×14 · bash×22
status          success
retry           0

artefatos       stdout.log · diff.patch · session.jsonl
```

---

# 70. UI de Memória

Memória parece conhecimento, não banco vetorial.

```text
Arquitetura · Decisões · Incidentes · Padrões · Aprendizados
```

```text
Operações de refund devem usar chave de idempotência
derivada de (order_id, amount, day).

Por quê
  Requisições duplicadas do gateway são comuns em timeout.
  UUID por requisição não protege.

Tipo         Decisão de arquitetura
Origem       run #1421 · fase review · claude
Aprovada     por você, 08/08 11:14
Evidência    src/payment/RefundService.ts:112
             incidente #0031
             RefundService.spec.ts:44
Usada        7 vezes
Confiança    alta

                        [Ver trace] [Substituir] [Ver histórico]
```

**[Δ]** "Substituir" em vez de "Editar" (`D-14`) e "Ver histórico" para a cadeia de supersessão. Aprovação com autor e data.

---

# 71. UI de Grafo de Código

Grafo só onde entrega valor real.

```text
CheckoutController
        │
        ▼
CheckoutService
   /          \
  ▼            ▼
Payment     Fraud
  │
  ▼
Ledger
```

Inspetor de nó:

```text
referências entrantes
referências saintes
risco
alterado no run atual
cobertura de teste
incidentes recentes
memórias vinculadas
```

**[Δ] v1.0+**, e condicionado a §21.1. Um grafo bonito que ninguém consulta é custo de manutenção sem retorno.

---

# 72. UI do Vault

```text
Brian Vault · XPTO

Providers
  Claude Code        ● armazenado    usado há 2h
  Codex              ● armazenado    usado há 12min
  Gemini             ○ não vinculado

Serviços
  GitHub             ● armazenado    usado ontem
  AWS Staging        ● armazenado    usado há 3 dias
  AWS Production     🔒 Touch ID      usado há 18 dias
                        ⚠ expira em 12 dias

Nunca usadas
  Datadog            ● armazenado    nunca usada · [remover]
```

Valores de credencial nunca são exibidos, em nenhuma circunstância (§7.3).

**[Δ]** "Nunca usadas" e o aviso de expiração vêm dos metadados de §7.5 e são o valor prático da seção — higiene de credencial, não apenas armazenamento.

---

# 73. UI de FinOps

```text
Agosto 2026

CLIENTE       TOKENS      CUSTO      CONFIANÇA
XPTO          18.4M       $124.30    alta
ACME           7.2M        $51.80    média
Interno        3.1M        $18.20    alta
──────────────────────────────────────────────
Total         28.7M       $194.30

Modo de cobrança: assinatura ($1.000/mês)
Alocação proporcional por tokens
```

Detalhamento:

```text
XPTO

Por provider            Por fase              Por change
  Codex      53%          implement   58%       refund-idem     $7.47
  Claude     23%          review      18%       fraud-rules    $22.10
  Gemini     17%          plan        14%       migration-42   $18.30
  Grok        7%          fix         10%       ad-hoc         $76.43

Cache hit ratio    61%
  ⓘ runs com cache < 40% custaram 2.3× mais por change
```

**[Δ]** A última linha é um insight acionável derivado de dado que existe desde o v0.0, sem construir nada (§40.2). É o tipo de observação que justifica o produto sozinha.

---

# 74. UI de Chat

```text
Brian · XPTO / produção

Você
  o cliente está no ar?

Brian
  ● Sim. Serviços primários saudáveis.

  API        ● saudável
  Web        ● saudável
  Database   ● saudável

  Último deploy 23:14

                                    [Detalhes] [Último deploy]
```

Chat herda o contexto ativo. Sempre. **v1.0+** (§49).

---

# 75. Command Palette

`⌘K` como interface de primeira classe.

```text
Conectar XPTO
Conectar ACME
Rodar tarefa
Rodar OpenSpec atual
Revisar diff atual
Comparar providers nesta tarefa
Buscar memória
Ver trace atual
Trocar provider
Pausar run
Custo do mês
Bloquear Vault
```

---

# 76. Menu Bar

```text
🧠 Brian

XPTO / checkout-api
● run #1421 ativo

Codex · implement
48k tokens · $0.31 · 01:42

Providers
  Claude    ●
  Codex     ●
  Gemini    ○
  Grok      ●

Budget       $188 de $500

──────────────
Abrir Brian
Pausar run
Trocar contexto  >
Desconectar XPTO
```

---

# 77. Modos de Autonomia

```text
Manual         Brian recomenda; o usuário executa.
Supervisionado Brian executa e pergunta nos gates críticos.
Autônomo       Brian roteia, executa, testa, revisa e corrige
               dentro da política.
```

## 77.1 Política de operação

```text
edição de código      auto
rodar testes          auto
git commit            auto
git push              perguntar
abrir PR              perguntar
deploy staging        perguntar
deploy produção       sempre perguntar + Touch ID + política
alterar dependência   perguntar
rodar migration       sempre perguntar
comando shell livre   negado por padrão
```

## 77.2 Pré-requisitos do modo autônomo

**[Δ novo].** O v0.1 oferecia o modo autônomo sem condicionar. Modo autônomo exige:

```text
provider em tier 1 ou 2 (§8.2)
  — em tier 3 não há como avaliar resultado com confiança

worktree isolado (§109)
  — sem isso, um run autônomo pode corromper a árvore de trabalho

limites duros configurados (§9.1, §45)
  — custo, turnos e tempo de parede

gates determinísticos passando (§30)
  — testes e secret scan no mínimo

critério de avaliação definido (§16.5)
  — sem rubrica, "sucesso" não é verificável
```

Faltando qualquer um, a UI oferece apenas Manual e Supervisionado, com o motivo explicitado.

---

# 78. Auditabilidade

Toda ação relevante é atribuível.

Perguntas que Brian precisa responder:

```text
Quem alterou este arquivo?
Qual provider fez a mudança?
Qual modelo?
Sob qual identidade?
Qual requisito de OpenSpec motivou?
Qual memória foi usada?
Quais ferramentas foram chamadas?
Quantos tokens foram consumidos?
Quanto custou?
Qual revisão aceitou?
Quem aprovou o que exigia aprovação?
Qual versão de workflow governava?
```

## 78.1 Trilha de commit

**[Δ novo].** A auditoria precisa sobreviver ao Brian. Todo commit gerado carrega proveniência no próprio Git:

```text
feat(payment): idempotência em refund

Deriva a chave de (order_id, amount, day) em vez de uuid().

Brian-Run: run_1421
Brian-Client: xpto
Brian-Context: xpto-checkout
Brian-Provider: codex@0.48.2
Brian-Model: coding/gpt-5-codex
Brian-Workflow: fast@1
Brian-Cost-USD: 7.47
Brian-Trace: tr_82ac
Co-Authored-By: Brian <brian@workwise.com.br>
```

Isso torna a atribuição verificável por qualquer pessoa com acesso ao repositório, mesmo sem acesso ao banco do Brian. É barato e é o único registro que sobrevive a uma reinstalação.

## 78.2 Log de auditoria

Eventos sempre registrados, independentemente de telemetria:

```text
context.connect / disconnect
identity.switch
vault.resolve / vault.rotate
budget.override
policy.approve / policy.deny
memory.approve / memory.supersede
run.start / pause / resume / cancel
provider.attach / detach
workflow.definition.change
```

---

# 79. Brian CLI

A CLI é a interface primária até o v0.3 e permanece completa depois. Tudo que a UI faz, a CLI faz.

## 79.1 Contexto

```bash
brian init                        # marca o diretório atual
brian connect <cliente[/projeto]>
brian disconnect
brian whoami
brian context list
brian context show [id]
brian context edit [id]
```

## 79.2 Clientes e projetos

```bash
brian client add <nome>
brian client list
brian project add <nome> --client <c> --repo <path>
brian project list
```

## 79.3 Providers

```bash
brian providers                        # lista e status
brian providers attach codex
brian providers detach codex
brian providers verify [id]            # testa integração e tier
brian providers models <id>            # popula models.yaml
brian providers usage [--period]
```

## 79.4 Identidade e vault

```bash
brian identity list
brian identity add <perfil> --client <c>
brian identity bind <perfil> <provider>
brian vault status
brian vault set <ref>                  # solicita valor via stdin seguro
brian vault rotate <ref>
brian vault lock
```

## 79.5 Execução

```bash
brian run "<tarefa>"
brian run "<tarefa>" --workflow governed
brian run "<tarefa>" --provider claude
brian run "<tarefa>" --explain-only
brian run "<tarefa>" --compare codex,claude      # §38.4
brian run --spec <change-id>

brian ps                               # runs ativos
brian pause <run-id>
brian resume <run-id>
brian cancel <run-id>
brian attach <run-id>                  # §56.2
brian logs <run-id> [--follow]
```

## 79.6 Observabilidade e custo

```bash
brian usage [--client] [--period]
brian costs [--client] [--by provider|phase|change] [--export csv]
brian costs --unattributed
brian trace current
brian trace <run-id>
brian trace <run-id> --span <span-id>
```

## 79.7 Memória

```bash
brian memory search "<query>"
brian memory list [--type] [--status]
brian memory show <id>
brian memory approve <id>
brian memory supersede <old-id> --with <new-id>
brian memory promote <id> --to <namespace> --anonymize
```

## 79.8 Manutenção

```bash
brian doctor                     # diagnostica instalação e integrações
brian worktree list
brian worktree gc                # limpa worktrees órfãos
brian recover                    # retoma runs órfãos (§110)
brian migrate                    # migração de schema (§113)
brian dev seed / bench           # §61
```

## 79.9 Saída legível por máquina

Todo comando aceita `--json`.

```bash
brian costs --client xpto --json | jq '.total_usd'
```

**[Δ]** Isso permite que o Brian seja usado dentro de scripts, hooks de CI e — recursivamente — por agentes de código. Uma CLI que só produz tabela bonita não compõe com nada.

---

# 80. Run Ponta a Ponta

Usuário:

```bash
brian connect xpto/checkout-api
brian run --spec refund-idempotency
```

Execução:

```text
 1. Context Manager resolve xpto/checkout-api
 2. Identity Manager ativa o perfil xpto-work
 3. Worktree Manager cria worktree isolado a partir de main   [Δ §109]
 4. Run é persistido com status=pending ANTES de qualquer efeito [Δ D-12]
 5. SpecSource carrega o change (opcional — pode não existir)
 6. Classifier determina task_type, risco, complexidade
 7. Workflow é selecionado: risk=high → governed
 8. Workflow version é congelada no run                        [Δ §15.6]
 9. Router decide provider e model_pointer; decisão é gravada  [Δ §11.4]
10. Vault resolve credenciais, escopadas à sessão
11. Provider executa a fase; heartbeat atualizado              [Δ §110]
12. Usage e custo são coletados na fonte de maior tier         [Δ §8.2]
13. Gates determinísticos executam sobre o diff                [Δ §30.1]
14. Evaluator avalia contra rubrica explícita, com evidência   [Δ §16.5]
15. Workflow Engine aplica a transição — e só ele              [Δ §15.5]
16. Fases de correção rodam dentro de max_entries
17. Validação de spec, se houver spec
18. Aprovação humana, se a política exigir
19. Commit com trailers de proveniência                        [Δ §78.1]
20. Memória é PROPOSTA, não gravada; aguarda aprovação         [Δ §36]
21. Telemetria fecha o trace
22. FinOps atribui tokens e custo a XPTO
23. Worktree é promovido a branch ou descartado                [Δ §109]
```

**[Δ]** Dez dos vinte e três passos são novos ou corrigidos. Os mais importantes são o 4 (persistência antes de efeito), o 15 (autoridade única de transição) e o 20 (memória exige aprovação).

## 80.1 O mesmo run no fast path

Para a maioria das tarefas reais:

```text
 1. contexto
 2. identidade
 3. worktree
 4. run persistido
 5. classifier: risk=low → fast
 6. router: regra padrão → codex/coding
 7. implement
 8. verify (testes + lint)
 9. commit com trailers
10. custo atribuído
11. worktree promovido
```

Onze passos, uma chamada de LLM na maior parte dos casos, duas se `verify` falhar. Essa é a diferença entre uma ferramenta usada e uma ferramenta contornada.

---

# 81. Brian v0.0 — Controle de Capacidade e Contabilidade

**[Δ][D-16]** Versão nova. Não existia no v0.1-draft. **Não é opcional nem “fase 2”.**

Objetivo: **controlar cada token e cada % da assinatura desde o minuto 0 — e saber de qual cliente é — sem orquestrar nada.**

## 81.1 Escopo (lei D-16)

```text
CLI em Rust, sem UI, sem daemon
SQLite com o schema de §60 (usage + plans + capacity_snapshot)
Attach de Claude Code e Codex
Import histórico de sessões (--since 30d)
Coleta contínua / sob demanda de consumo (observe mode)
Planos e baselines de assinatura/API por provider
Janelas: day, week, month, plan_reset
brian capacity   → % usado, restante, tempo até reset, burn, alertas
brian usage      → tokens por cliente/provider/janela
brian costs      → $ e/ou fração de assinatura alocada
brian status     → capacity + contexto em uma tela
Atribuição por path de repo + correção humana auditada
Price catalog como fallback de $ (nunca como se fosse quota real)
Alertas soft de % de janela e unattributed
Export CSV
```

## 81.2 O que NÃO tem

```text
sem orquestração
sem workflow
sem router
sem worktree
sem memória
sem gates
sem UI
sem bloqueio hard do CLI nativo do provider (observe não intercepta processo)
```

## 81.3 Por que esta é a primeira versão

```text
CONTROLE INVIOLÁVEL (D-16)
  Sem % de janela, tempo restante e ledger de tokens, o usuário
  continua voando às cegas na assinatura — o Brian não tem desculpa.

VALOR IMEDIATO
  Consultoria multi-cliente: quanto cada cliente queimou da capacidade
  paga esta semana, e quanto ainda cabe até o reset.

RISCO TÉCNICO MÍNIMO
  Ler arquivos/sessões, agregar SQL, configurar plano. Nenhuma parte
  de orquestração está envolvida.

GERA O DADO QUE TUDO DEPOIS PRECISA
  Router, Learning, H-1 e otimização de model_pointer só existem
  em cima de histórico de capacidade real.

VALIDA A PREMISSA MAIS INCERTA CEDO
  Se não for possível extrair consumo com fidelidade, o produto
  não existe — melhor na semana 1 que no mês 8.
```

## 81.4 Critério de conclusão

```text
1) brian capacity
   mostra por provider anexado: janela, % usado, restante (ou baseline),
   tempo até reset (se conhecido), burn, alertas — em < 5s.

2) brian usage --window week --by client
   soma bate com o total da janela (± tolerância de arredondamento).

3) brian costs --client xpto --period <mês>
   número verificável vs painel do provider (tokens e/ou $),
   erro < 5% no que for nível 1/2; estimado sempre rotulado.

4) brian costs --unattributed
   retorna vazio após import + atribuição do período de trabalho real.

5) brian import --since 30d
   popula o ledger; correções via brian attribute ficam auditadas.

Falha em qualquer item = v0.0 incompleto. Não se avança para v0.1.
```

---

# 82. Brian v0.1 — Contexto, Identidade e Continuidade (D-17 mínimo)

Objetivo: **trocar de cliente com um comando — e trocar de LLM sem perder a cabeça do trabalho.**

```text
Context Manager (§5)
Identity Manager (§6) com isolamento verificado
Brian Vault sobre Keychain (§7)
Policy Engine mínimo: o que exige aprovação
brian connect / disconnect / whoami
Atribuição automática de consumo pelo contexto ativo
Continuity Pack + memory notes (§34.0, D-17)
brian handoff --to <provider>
brian memory note / decide
brian continuity show
```

**Critérios de conclusão:**

```text
1) Troca entre dois clientes, identidades distintas, consumo atribuído
   sem intervenção manual. connect < 2s.

2) D-17 mínimo:
   Trabalho real sob context A com provider P1.
   brian handoff --to P2
   → Continuity Pack contém objetivo, decisões, análise, touched files,
     failed attempts, next steps.
   → Usuário NÃO reexplica o problema para P2.
   → Pack tem orçamento de tokens e origem rotulada.

3) D-16 continua verde: capacity + unattributed=vazio no período.
```

**[Δ]** `connect` sem continuidade multi-LLM só resolve metade da missão (identidade). Com D-17 mínimo, o Brian começa a **poupar tempo de verdade** ao chavear workers.

---

# 83. Brian v0.2 — Execução Rastreada

Objetivo: **executar uma tarefa e saber exatamente o que aconteceu.**

```text
Run Manager com ciclo de vida e retomada (§110)
Worktree Manager (§109)
Workflow Engine dirigido por dados, apenas o fast path (§15)
Telemetria OpenTelemetry (§39)
SpecSource com OpenSpecSource e NullSource (§14)
Gates: tests, lint, secrets
Daemon opcional sobre socket (§55)
brian run / ps / pause / resume / cancel / attach / logs / trace
```

**Critério de conclusão:**

```text
Três runs paralelos, em worktrees isolados, sem conflito.
Um run interrompido por kill -9 é retomado com brian recover
sem perda de estado nem duplicação de custo.
```

---

# 84. Brian v0.3 — Comparação e Interface

Objetivo: **escolher o worker certo com evidência, e ver isso.**

```text
Provider Router por regras (§11.2)
Model Router com ponteiros semânticos (§12)
brian run --compare (§38.4)
Workflow governed (§15.3)
Gates: OCR, cobertura
Brian Inspector (§66)
Brian.app em SwiftUI consumindo o mesmo socket (§51.1)
Telas: Hoje, Custo, Runs, Providers, Ajustes
Relatório PDF por cliente (§44.1)
```

**Critério de conclusão:**

```text
A UI não expõe nenhuma capacidade que a CLI não tenha.
O usuário consegue explicar a um cliente, com a tela aberta,
por que uma mudança custou o que custou.
```

---

# 85. Brian v0.4 — Memória e Segurança

Objetivo: **conhecimento que sobrevive à troca de provider.**

```text
Memory Engine com governança (§34–§37)
Recuperação por FTS5 (§35.3)
Security Gates: Semgrep, OSV, secret scanner (§30)
SkillSpector (§31)
ast-grep (§25)
Eval Harness (§112)
Gemini e Grok como providers
Roteamento com evidência histórica, se n ≥ 30 (D-8)
```

---

# 86. Brian v1.0 — Critérios de Sucesso

Brian executa:

```bash
brian connect xpto
brian run --spec refund-idempotency
```

com:

```text
isolamento de contexto
identidade corporativa de provider
carregamento de spec
roteamento explicável
implementação em worktree isolado
testes
revisão de código
gates de segurança
laço de correção limitado
validação contra rubrica
trace completo
memória proposta e aprovada
contabilidade de token
atribuição de custo ao cliente
commit com proveniência verificável
```

exigindo aprovação humana apenas onde a política exige.

## 86.1 Critérios de negócio

**[Δ novo].** Critérios técnicos não medem se o produto funciona.

```text
O usuário usa Brian para ≥ 70% dos runs de agente,
  em vez de abrir o CLI direto.

O tempo de troca de cliente cai de minutos para segundos.

Uma fatura é emitida a um cliente real com base em dado do Brian.

O usuário consegue responder "por que este change custou $X"
  sem consultar mais nada.
```

O primeiro é o mais difícil e o mais importante. Se o usuário contorna o Brian para tarefas rápidas, o produto governa apenas o trabalho que já era governado.

---

# 87. Evolução Enterprise

**[Δ] Reduzida de sete seções (§86–§92 do v0.1) para três (§87–§89).** A razão é `§2.6`: manter o core agnóstico é disciplina barata; construir o segundo runtime antes de ter cliente enterprise é otimização prematura de custo alto.

```text
Local
Brian.app / CLI → brian-core → processos locais de provider

Enterprise
Brian.app / CLI → Brian Enterprise API → Kubernetes → workers
```

```text
                 Brian Enterprise
                        │
                 Control Plane API
                        │
        ┌───────────────┼────────────────┐
        ▼               ▼                ▼
   Context Service  Workflow Engine  Identity/Vault
        │               │                │
        └───────────────┼────────────────┘
                        ▼
                 Provider Router
                        │
        ┌───────────────┼────────────────┐
        ▼               ▼                ▼
   Claude Job       Codex Job       Gemini Job
        │               │                │
        └───────────────┼────────────────┘
                        ▼
                    Workspace
                        │
                  Quality Gates
```

## 87.1 Workers como Jobs

```text
Workflow precisa de implementação
       ↓
Router seleciona Codex
       ↓
Kubernetes Job criado
       ↓
Job recebe: run_id, client_id, project_id, model, skill,
            referência de contexto, budget, workspace
       ↓
worker executa
       ↓
resultados registrados
       ↓
pod termina
```

Workers são descartáveis. Estado vive no control plane.

## 87.2 Gatilho de construção

```text
Um cliente enterprise pagante, com requisito explícito de
execução centralizada, e volume que justifique operação de cluster.
```

Antes disso, `platform/kubernetes/` permanece vazio e o CI verifica agnosticismo (§54).

---

# 88. Mapeamento de Plataforma

```text
Brian Local                Brian Enterprise
────────────────────────────────────────────────────
macOS Keychain             Vault / Secret Manager
Touch ID                   aprovação corporativa / SSO
processo local             Kubernetes Job
git worktree               PVC efêmero por Job
filesystem                 PVC / object storage
SQLite                     PostgreSQL
launchd                    Deployment
OTel local em arquivo      OTel Collector
perfil OAuth local         workload identity
socket Unix                API autenticada
```

As interfaces de domínio (§59) são idênticas nos dois modos. Isso é o que torna a evolução uma troca de implementação de trait, e não uma reescrita — desde que a disciplina de §54 seja mantida por CI desde o primeiro commit.

---

# 89. Multi-Tenant e Execução Híbrida

Cliente permanece a fronteira de tenancy.

```text
XPTO
├── identidade
├── repositórios
├── memória
├── secrets
├── providers
├── budgets
├── traces
└── políticas
```

Opções de isolamento, escolhidas por requisito:

```text
tenant_id lógico
namespace dedicado
workload identity dedicada
storage dedicado
```

Execução híbrida, quando existir:

```text
tarefa rápida        → Mac
refactor grande      → Kubernetes
revisão de produção  → Enterprise
trabalho offline     → Mac
```

Local de execução vira mais uma dimensão da decisão do Router (§11).

---

# 90. Storage Enterprise

```text
DB operacional      PostgreSQL
Memória / vetores   backend vetorial dedicado, se necessário
Artefatos           object storage S3-compatível
Telemetria          OTel Collector
Traces              Tempo / Jaeger
Métricas            Prometheus
Secrets             Vault / secret manager de nuvem
```

---

# 91. Princípios de Segurança

1. Secrets nunca vivem em arquivo de configuração em texto claro.
2. Recuperação de memória cross-client é negada por construção, não por filtro.
3. Identidades de provider são explícitas e a conta autenticada é sempre visível.
4. Ferramentas têm permissão baseada em capacidade, com deny explícito.
5. Transições de workflow pertencem ao Brian Core, nunca aos agentes.
6. Operações de alto risco exigem aprovação de política.
7. Skills e plugins de terceiro são analisados antes de receber confiança.
8. Toda operação privilegiada é registrada no log de auditoria.
9. Credenciais de produção devem ser de vida curta sempre que possível.
10. Workers enterprise usam workload identity, não secrets estáticos de longa duração.
11. **[Δ]** Todo run executa em worktree isolado; nenhum agente escreve na árvore de trabalho principal.
12. **[Δ]** Secret scanning roda antes de qualquer commit ou push, em todo workflow, sem exceção.
13. **[Δ]** Valor de credencial nunca é exposto — nem em log, nem em UI, nem para agentes.
14. **[Δ]** Override de budget exige motivo textual e é auditado.
15. **[Δ]** Comando de shell arbitrário é negado por padrão em todo perfil.

---

# 92. Não-objetivos

Brian v0.x não tenta:

```text
substituir Xcode ou VS Code
substituir a UI de qualquer provider
construir modelo próprio
implementar control plane em nuvem
ser plataforma de monitoramento geral
ser sistema de faturamento completo
substituir GitHub
substituir Kubernetes
```

**[Δ] Adições:**

```text
orquestrar o inner loop de um único provider          (D-10)
reimplementar skills, hooks ou subagents de provider  (§32.1)
fazer retrieval melhor que a busca agêntica nativa    (H-1 não confirmada)
prever custo antes da execução com precisão           (só estimativa declarada)
```

O último merece nota: estimar custo antes de rodar um agente é intrinsecamente impreciso, porque o número de turnos depende do que ele descobre. Brian expõe limites e estimativas com faixa, não previsões pontuais.

---

# 93. Diferenciais

**[Δ]** Reescritos sob o teste de `D-10`: só entra o que **não pode ser replicado dentro de um único provider**.

## 93.1 Fronteira de contexto

Um comando ativa cliente, projeto, identidade, memória, política e budget. Nenhum provider faz isso porque nenhum provider sabe que existem outros clientes.

## 93.2 Fronteira de identidade

O desktop permanece pessoal; Brian usa contas corporativas, com a conta ativa sempre visível. Providers isolam configuração, mas não gerenciam a troca entre identidades por cliente.

## 93.0 Os dois diferenciais mínimos (missão) **[Δ D-16 + D-17]**

```text
A) Zero token perdido + capacity/centavo sob controle     (D-16)
B) Memória/continuidade ao chavear LLM sem recomeçar      (D-17)
```

Nenhum provider individual oferece (A)+(B) juntos para **N providers × M clientes**.  
São o **mínimo** para a IA poupar ~90% do trabalho sem queimar dinheiro. O resto do Brian empilha em cima.

### 93.0.A Capacidade multi-provider / multi-cliente (D-16)

Tokens, % de janela, tempo restante até reset, burn rate e alertas — por provider, plano/assinatura e identidade — com atribuição por cliente. Nenhum provider mostra “quanto da *minha* semana foi o cliente XPTO” nem unifica Claude+Codex+API num único painel de capacidade.

**Primeiro a construir (minuto 0).** Token que some = produto falhou.

### 93.0.B Continuidade multi-LLM (D-17)

Continuity Pack + memória do Brian: objetivo, conversa útil, análise, decisões, erros e próximos passos **seguem** na troca Claude↔Codex↔outro. Nenhum provider entrega a cabeça do trabalho ao concorrente de propósito.

**Segundo a construir (v0.1 mínimo).** Reexplicar = tempo (e tokens) roubados — viola a missão.

## 93.3 Atribuição de custo por cliente

Tokens e custo atribuídos a cliente, projeto, change e fase, com origem do dado rotulada e alocação proporcional em modo assinatura. Um provider reporta o custo da sua sessão; ele não sabe de quem é o custo.

É a face FinOps de §93.0.A. Sem capacity control, atribuição é relatório morto; sem atribuição, capacity é vanidade.

## 93.4 Memória entre providers

Conhecimento e **continuidade operacional** sobrevivem à troca de provider, à mudança de modelo e ao fim de uma assinatura, com proveniência e evidência (D-17 / §34.0). Não é “nice archive” — é o que impede pagar o mesmo raciocínio duas vezes.

## 93.5 Comparação com evidência

Executar a mesma tarefa em dois providers e comparar diff, custo e resultado de gate lado a lado. Nenhum provider tem incentivo para construir isso.

## 93.6 Explicabilidade de decisão

Roteamento, custo e transições de workflow inspecionáveis, com sinais gravados no momento da decisão e com o `n` das estatísticas exibido.

## 93.7 Governança de skills de terceiro

Análise de capacidade declarada versus detectada, com política de confiança por contexto.

## 93.8 Caminho local-para-enterprise

O mesmo core evolui de macOS para Kubernetes, garantido por verificação de CI e não por intenção.

## 93.9 O que saiu da lista

```text
Context Governor    → condicionado a H-1 (§18)
Multi-provider brain → era descrição de orquestração,
                       que colide com D-10
```

---

# 94. Riscos de Mercado

**[Δ novo].** Ver §116 para o registro completo. Os dois que afetam posicionamento:

## 94.1 Absorção pela plataforma

Providers estão subindo na pilha: plugins versionados, subagents, hooks de ciclo de vida, saída estruturada com custo. Cada release reduz a área de orquestração disponível para terceiros.

**Mitigação:** `D-10`. Brian só ocupa o espaço cross-provider e cross-client, que nenhum provider tem incentivo para ocupar — porque ocupá-lo significa facilitar a saída do cliente para um concorrente.

## 94.2 Consolidação de providers

Se o mercado convergir para um ou dois providers dominantes, o valor de "multi-provider" cai.

**Mitigação:** a fronteira **cross-client** permanece válida mesmo com um único provider. Uma consultoria com dez clientes e um só provider ainda precisa de identidade isolada, atribuição de custo e memória por cliente. O v0.0 e o v0.1 funcionam com um provider só — o que é deliberado.

---

# 95. Métricas North-Star

## 95.1 Produto (missão: tempo poupado, não dinheiro queimado)

```text
% do gasto real de AI da semana capturado no ledger    ← primária D-16 (alvo: 100%)
unattributed_tokens                                    ← alvo: 0 (zero loss)
handoffs sem reexplicação (amostra dogfood)            ← primária D-17
tempo até brian capacity útil                          ← alvo: < 5s
tempo de connect até estado produtivo                  ← alvo: < 2s
% de dias com capacity consultada (dogfood)
trabalho manual evitado (proxy: changes fechados / hora)
% de runs via Brian vs CLI direto                      ← após v0.2
frequência de override de roteamento                   ← alta = regras ruins
```

## 95.2 Econômica / capacidade (cada centavo trabalha)

```text
used_percent por provider/janela (day/week/plan_reset)
remaining + time_to_reset por plano
burn rate vs mediana 7d
fração de assinatura alocada por cliente
custo equivalente por change
custo por change BEM-SUCEDIDO                          ← métrica de H-1
tokens re-gastos por reexplicação (proxy de falha D-17)
custo do Continuity Pack por handoff (deve ser << custo de recomeçar)
cache hit ratio                                        ← alavanca acionável
% de custo com origem "provider" vs "estimated"
```

## 95.3 Técnica

```text
runs bem-sucedidos
sucesso na primeira tentativa
retries médios
tokens por change bem-sucedido
latência por fase
precisão de extração vs painel do provider (< 5%)
disponibilidade de provider
runs órfãos recuperados com sucesso                    ← §110
```

## 95.4 Qualidade

```text
achados de revisão por change
regressões pós-merge
achados de segurança novos vs pré-existentes
taxa de aprovação de testes
conformidade com spec
memórias aprovadas vs sugeridas                        ← sinal de qualidade do agente
```

**[Δ]** A métrica primária mudou. O v0.1 media sucesso técnico. Este documento mede **adoção contra a alternativa**: se o usuário abre o CLI direto para tarefas rápidas, Brian governa só o que já era governado, e todo o resto é teatro.

---

# 96. Vocabulário

```text
Brian
→ o produto e o plano de controle

Brian Core
→ o runtime agnóstico de plataforma
   [Δ] o termo "Brain" foi eliminado (D-11)

Context
→ fronteira operacional de cliente/projeto ativa;
   é também a fronteira de tenancy, isolamento e atribuição

Client
→ tenant. A unidade de faturamento e isolamento.

Project
→ subdivisão de cliente, geralmente um repositório

Provider
→ Claude, Codex, Gemini, Grok, ZCode

Integration Tier
→ [Δ] qualidade da integração com um provider:
   headless_json > session_files > pty

Provider Profile
→ configuração e papel de uso de um provider

Role
→ [Δ] papel do perfil: builder, reviewer, planner

Model Pointer
→ papel semântico de modelo: coding, reasoning, quick, review

Run
→ uma execução orquestrada, com identidade própria e custo próprio

Phase
→ estado do workflow durante um run

Phase Execution
→ [Δ] uma entrada em uma fase; a mesma fase pode ser executada
   múltiplas vezes no mesmo run

Workflow
→ [Δ] definição versionada em YAML de fases e transições

Worktree
→ [Δ] árvore Git isolada onde um run executa

Skill
→ instruções reutilizáveis de como executar um tipo de trabalho

Tool / Capability
→ capacidade executável, concedida ou negada por perfil

Gate
→ verificação determinística ou semântica que produz resultado
   consumido pelo Workflow Engine

Memory
→ conhecimento durável, append-only, pertencente ao Brian

Provenance
→ [Δ] origem verificável de uma memória: run, fase, provider,
   trace, evidência, aprovação

Trace / Span
→ registro de observabilidade

Usage
→ consumo de token ou requisição

Usage Source
→ [Δ] origem do dado de consumo: provider, session_file,
   estimated, unknown

Equivalent Cost
→ custo estimado em preço de API

Provider-Reported Cost
→ [Δ] custo informado pelo próprio provider; tem precedência

Actual Billing Cost
→ o que efetivamente será faturado, considerando modo de cobrança

Allocation
→ [Δ] em modo assinatura, a fração de capacidade atribuída
   a um cliente

Budget
→ limite de consumo em um escopo

Vault
→ abstração de credenciais; armazena referências, nunca valores

Decision Record
→ [Δ] registro de uma decisão de roteamento com seus sinais
```

---

# 97. Layout do Repositório

```text
brian/
├── Cargo.toml                     workspace Rust
├── README.md
│
├── core/
│   ├── Cargo.toml
│   └── src/                       ver §53
│
├── cli/
│   ├── Cargo.toml
│   └── src/
│
├── adapters/                      [Δ] fora do core
│   ├── claude/
│   ├── codex/
│   ├── gemini/
│   ├── grok/
│   └── zcode/
│
├── macos/                         v0.3+
│   ├── Brian.xcodeproj
│   ├── BrianApp/
│   └── BrianTests/
│
├── browser/                       v1.0+
│
├── migrations/                    [Δ] SQL versionado
│   ├── 0001_initial.sql
│   └── ...
│
├── workflows/                     [Δ] definições YAML
│   ├── fast.yaml
│   └── governed.yaml
│
├── routing/                       [Δ]
│   └── rules.yaml
│
├── models/                        [Δ]
│   └── pointers.yaml
│
├── catalog/                       [Δ] price catalog versionado
│   └── prices.yaml
│
├── skills/
├── openspec/
├── schemas/
│
├── evals/                         [Δ] §112
│   ├── cases/
│   └── harness/
│
├── tests/
│   ├── integration/
│   └── fixtures/
│
└── docs/
    ├── BRIAN-BLUEPRINT-V1.md
    ├── ARCHITECTURE.md
    ├── DECISIONS.md               [Δ] ADRs, ver D-1..D-15
    ├── PROVIDERS.md               [Δ] matriz de compatibilidade
    ├── SECURITY.md
    └── ENTERPRISE.md
```

**[Δ]** Cinco diretórios novos de configuração no topo (`workflows/`, `routing/`, `models/`, `catalog/`, `evals/`). Eles existem porque `D-3`, `D-6` e `D-8` transformam comportamento em dado versionado. Um pull request que muda uma regra de roteamento é revisável; uma mudança em `match` dentro do Rust não é, para a maioria dos revisores.

---

# 98. Milestone 0 — Extração de Consumo e Capacidade

**[Δ][D-16]** Novo. Dois a três dias. É o experimento de maior informação por hora do projeto — e o **gate de existência** do Brian.

```text
Objetivo
  Determinar se é possível extrair, de forma confiável:
    - tokens (in / cache / out / reasoning quando existirem)
    - modelo, timestamps, session id
    - custo reportado (se houver)
    - sinais de quota/remaining/reset (se houver)
  e se, na ausência de quota do provider, ainda é possível
  operar janelas com baseline de plano + medição Brian.

Método
  Abrir os arquivos de sessão do Claude Code e do Codex.
  Rodar cada um em modo headless com saída JSON.
  Comparar os números extraídos com o painel oficial do provider.
  Documentar o que NÃO existe (quota unknown é ok; token opaco não é).

Entregável
  docs/PROVIDERS.md com matriz:

    provider · versão · tier
    · tokens? · cost? · cache? · quota/remaining/reset?
    · precisão medida · notas de janela
```

## 98.1 Por que primeiro

Se a extração não for confiável, então Usage Control (D-16), FinOps (§43), Learning (§38), o experimento `H-1` (§18.3) e o diferencial primário (§93.3 / §93.0) ficam comprometidos. Descobrir na semana 1 custa dias. Descobrir no mês 8 custa o projeto.

## 98.2 Resultados possíveis

```text
CONFIÁVEL (tokens + timestamps; custo e/ou quota opcionais)
  → segue o plano; capacity com medição Brian + baseline de plano

PARCIAL (tokens presentes, custo/quota ausentes)
  → price catalog para $ equivalente (rotulado)
  → % de janela sobre baseline user_declared
  → confiança "média" onde couber — produto AINDA é válido

NÃO CONFIÁVEL (não dá para cravar tokens por sessão)
  → PARE. Reavaliar premissa do produto.
  → não construir workflow/UI em cima de ledger oco
```

---

# 99. Milestone 1 — Fatia Vertical de Capacidade + Contabilidade

```text
brian client add xpto
brian project add checkout-api --client xpto --repo ~/Projects/XPTO/checkout-api
brian providers attach codex
brian plans set codex --baseline-tokens-week <N>
brian import --since 30d
brian connect xpto/checkout-api
<usuário trabalha normalmente com o CLI do provider>
brian capacity
brian usage --window week --by client
brian costs --client xpto
brian costs --unattributed
```

Prova:

```text
Attachment de provider
Import + coleta de consumo (observe)
Planos / baselines / janelas
capacity (% · restante · reset · burn)
Atribuição por cliente
Correção de unattributed
Storage
CLI status/usage/capacity/costs
```

**[Δ]** Note que **não há run orquestrado**. O usuário continua usando o CLI do provider diretamente; Brian observa, **controla a visibilidade da capacidade** e atribui. Isso remove o maior risco de adoção — pedir que o usuário mude o fluxo de trabalho antes de ter provado valor — e cumpre D-16 no dia um.

---

# 100. Milestone 2 — Execução Rastreada e Comparação

```text
brian run "implementa idempotência de refund"
brian run "implementa idempotência de refund" --compare codex,claude
```

Prova:

```text
Worktree isolado
Run retomável
Workflow fast path
Trace completo
Gates determinísticos
Comparação lado a lado
```

Mede:

```text
tokens · latência · custo · retries · tamanho de contexto
cache hit ratio · resultado de gate · qualidade percebida
```

**[Δ]** O v0.1 propunha orquestração Claude-planeja/Codex-implementa neste milestone. Este documento propõe **comparação** em vez de encadeamento. A razão é que comparação gera dado decisório imediatamente e é mais simples; encadeamento assume que a divisão de trabalho entre providers é benéfica, o que é uma hipótese não testada.

---

# 101. Milestone 3 — Experimento H-1

Executa o desenho de §18.3.

```text
30 changes reais · 3 braços · métrica primária de custo por
change bem-sucedido · critério de aceitação e de descarte
declarados antes da coleta.
```

**Resultado determina se §18, §19, §20, §21 e §22 permanecem no roadmap.**

**[Δ]** O v0.1 colocava este experimento como terceiro milestone mas o descrevia como "validação de uma hipótese econômica importante", sem critério de descarte. Sem critério de descarte declarado antes da coleta, o resultado será interpretado favoravelmente independentemente do que os números disserem.

---

# 102. Regras Arquiteturais

Não-negociáveis até que evidência prove o contrário.

1. **Brian Core é agnóstico de plataforma, verificado por CI** (§54).
2. **macOS e Kubernetes são runtimes, não o core.**
3. **Context é a fronteira primária de isolamento e atribuição** (§5).
4. **Providers são workers substituíveis, com tier de integração declarado** (§8.2).
5. **Autenticação de provider é separada de autorização no Brian.**
6. **Secrets vivem em backends de Vault, nunca em arquivo de projeto** (§7).
7. **Transições de workflow pertencem ao Workflow Engine, e só a ele** (§15.5).
8. **Storage fica atrás de traits; nenhuma SQL fora de `storage/`** (D-9).
9. **Memória é do Brian, append-only, com proveniência** (D-14).
10. **Toda chamada de LLM é atribuível a cliente, projeto e run** (§43.2).
11. **Custo equivalente e faturamento real são conceitos distintos** (§42).
12. **Ferramentas determinísticas são preferidas antes de raciocínio por LLM** (§2.3).
13. **Gates de qualidade e segurança são estágios de primeira classe** (§29, §30).
14. **Decisões de roteamento são registradas no momento em que ocorrem** (§11.4).
15. **Todo run executa em worktree isolado** (D-7, §109).
16. **[Δ]** **Estado é persistido antes de qualquer efeito colateral externo** (D-12, §110).
17. **[Δ]** **Comportamento configurável é dado versionado, não código** (D-3).
18. **[Δ]** **Nada depende arquiteturalmente de uma hipótese não confirmada** (§2.8).
19. **[Δ]** **Todo número exibido carrega a origem do dado** (§13.3, §40.1).
20. **[Δ]** **Cada versão é útil sozinha** (§2.7).

---

# 103. Brian em Um Diagrama

```text
                    ┌──────────────────────────┐
                    │   brian CLI  ·  Brian.app │
                    └────────────┬─────────────┘
                                 │ JSON-RPC / socket
                                 ▼
                    ┌──────────────────────────┐
                    │       Brian Core         │
                    │          Rust            │
                    └────────────┬─────────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        ▼                        ▼                        ▼
    CONTEXTO                 EXECUÇÃO                 CONTROLE
  cliente/projeto          run · worktree           política · vault
  identidade               workflow (YAML)          budget · aprovação
  memória                  session manager
        │                        │                        │
        └────────────────────────┼────────────────────────┘
                                 ▼
                          Provider Router
                        (regras → evidência)
                                 │
          ┌──────────┬───────────┼───────────┬──────────┐
          ▼          ▼           ▼           ▼          ▼
       Claude      Codex      Gemini       Grok       ZCode
          │          │           │           │          │
          └──────────┴───────────┼───────────┴──────────┘
                                 ▼
                          Capability Layer
                     Git · Tests · AST · Search
                                 │
                                 ▼
                           Quality Gates
                   Tests · Lint · Secrets · SAST · OCR
                                 │
                                 ▼
                             TELEMETRIA
                      trace · token · custo · resultado
                                 │
                    ┌────────────┼────────────┐
                    ▼            ▼            ▼
                 FINOPS      AUDITORIA     MEMÓRIA
                    │            │            │
                    └────────────┼────────────┘
                                 ▼
                    ┌──────────────────────────┐
                    │   SQLite  ·  Keychain    │
                    │   artefatos em disco     │
                    └──────────────────────────┘

              [ LEARNING ENGINE — pós v1.0, D-8 ]
              [ CONTEXT GOVERNOR — condicional a H-1 ]
```

---

# 104. Definição Final

**Brian é o plano de controle que faz a IA poupar ~90% do trabalho e do tempo — sem queimar dinheiro.**

Local-first: um Context une cliente, identidade, providers, memória compartilhada e capacidade.  
**Nenhum token some (D-16). Nenhuma troca de LLM apaga a cabeça do trabalho (D-17).**  
Cada centavo de assinatura/API vira trabalho útil atribuído a quem deve pagá-lo.

No macOS, Brian é a interface nativa e o runtime local.

Em ambientes enterprise, o mesmo core evolui para um plano de controle sobre Kubernetes, com workers descartáveis, identidade centralizada, memória compartilhada, secrets corporativos e observabilidade organizacional.

O usuário não deveria precisar pensar:

```text
Qual terminal?
Qual conta?
Qual provider?          ← e se eu trocar, perco a conversa?
Qual repositório?
Qual memória?
De quem é este token?
Quanto isso custou ao cliente?
Quanto me resta esta semana?
Será que estou queimando a assinatura à toa?
```

O usuário deveria poder dizer:

```text
brian connect xpto
```

e trabalhar — **chaveando LLM quando quiser** — sabendo que o dinheiro está sob controle e o contexto não se perde.

### Teste de ouro do produto

```text
PASS se:
  1. Nenhum token observável ficou de fora do ledger na semana
  2. brian capacity responde em < 5s com % e restante
  3. handoff Claude→Codex não exige reexplicar o trabalho
  4. o usuário sente que a IA economizou tempo, não só gerou invoice de tokens

FAIL se:
  - tokens somem ou unattributed é “normal”
  - trocar de modelo é recomeçar do zero
  - o Brian é mais caro/lento que o CLI sem devolver controle nem continuidade
```

---

# 105. Taglines Candidatas

```text
Brian — A IA poupa seu tempo. Cada token tem dono.

Brian — Zero token perdido. Zero contexto perdido ao trocar de LLM.

Brian — Um contexto. Todos os agentes. Cada centavo trabalhando.

Brian — Controle de capacidade + memória que sobrevive ao provider.

Brian — Conecte o cliente. Chaveie o modelo. Não recomece a história.

Brian — Saiba de quem é cada token — e o que ainda cabe na semana.
```

**[Δ]** Taglines alinhadas à missão: poupar trabalho/tempo, não gastar dinheiro; D-16 + D-17.

---

# 106. Próximos Passos Imediatos

```text
 1. Criar o repositório como workspace Rust puro.          §52.1
 2. Adicionar este blueprint em docs/.
 3. Criar docs/DECISIONS.md com D-1 a D-17 (D-16 e D-17 invioláveis).
 4. MILESTONE 0: extração de tokens + quota/reset. 2–3 dias. §98
 5. Escrever docs/PROVIDERS.md (tokens, cost, quota, janelas).
 6. Aplicar o schema de §60 via migrations/0001
    (usage_record, provider_plan, capacity_snapshot, …).
 7. Implementar traits de storage e SqliteStore.           §59
 8. Implementar `brian client/project add` e `brian plans`.
 9. Implementar `brian providers attach` e `verify`.       §8.3
10. Implementar coletor + `brian import --since 30d`.
11. Implementar `brian usage` e `brian capacity` (D-16).   §13.7
12. Implementar `brian costs`, `--unattributed`, `--export`.
13. Implementar `brian attribute` (correção auditada).
14. Alertas soft de % de janela + unattributed.
15. Rodar seed/bench; validar < 5% vs painel do provider.
16. Emitir o primeiro capacity+costs de uma semana REAL.
    → D-16 verde (zero token perdido no período de teste).
17. MILESTONE 1: context + identity + Continuity Pack (D-17). §82
18. Vault + isolamento de duas contas.                     §6–§7
19. `brian handoff` + memory note/decide + inject no provider.
20. Validar handoff real Claude↔Codex sem reexplicar.
21. Só então: worktree, run, workflow.                     §100
```

**[Δ][D-16][D-17]** Ordem sagrada: (1) nenhum token some, (2) handoff de LLM sem perda de cabeça, (3) só então orquestração. Workflow/UI em cima de ledger furado ou memória que morre na troca de provider = fracasso.

---

# 107. Primeiro OpenSpec Candidato

**[Δ]** O candidato do v0.1 (`context-aware-provider-execution`) combinava contexto, identidade, execução e atribuição em um único change. É grande demais para um primeiro change e não é testável de forma incremental. Dividido em três.

## 107.1 `client-cost-attribution` — primeiro

```text
Objetivo
  Permitir que Brian registre clientes e projetos, anexe providers,
  colete consumo de sessões existentes e atribua custo por cliente.

Critérios de aceitação
  - O usuário cria um cliente.
  - O usuário cria um projeto sob esse cliente, apontando para um repo.
  - Brian anexa um CLI de provider e reporta versão e tier.
  - Brian lê o consumo de sessões do provider após execução.
  - Brian atribui esse consumo ao cliente correto.
  - `brian costs --client X` retorna número verificável contra o
    painel do provider, com erro < 5%.
  - `brian costs --unattributed` retorna vazio.
  - Todo registro de consumo carrega usage_source e cost_source.
  - Nenhum secret é armazenado no banco operacional.
```

## 107.2 `context-and-identity-switching` — segundo

```text
Objetivo
  Permitir troca de contexto de cliente com isolamento de identidade
  de provider.

Critérios de aceitação
  - `brian connect <cliente/projeto>` ativa o contexto.
  - `brian whoami` mostra contexto, perfil, conta autenticada por
    provider, identidade Git e budget restante.
  - Dois perfis distintos do mesmo provider coexistem sem colisão
    de estado.
  - Credenciais são resolvidas via Keychain, por referência.
  - `brian disconnect` limpa o contexto ativo.
  - Executar sem contexto ativo falha com mensagem acionável.
  - Troca de contexto leva menos de 2 segundos.
```

## 107.3 `isolated-tracked-run` — terceiro

```text
Objetivo
  Executar uma tarefa em worktree isolado, com run retomável e
  trace completo.

Critérios de aceitação
  - `brian run` cria um worktree dedicado a partir do commit base.
  - O run é persistido antes de qualquer efeito colateral.
  - Três runs concorrentes no mesmo projeto não colidem.
  - Um run interrompido por SIGKILL é detectado como órfão.
  - `brian recover` retoma ou finaliza o órfão sem duplicar custo.
  - O trace contém todos os spans de §39.
  - O commit gerado carrega os trailers de proveniência de §78.1.
  - O worktree é promovido a branch ou removido ao final.
```

---

# 108. Princípio de Fechamento

Brian se torna valioso quando torna simples um ambiente de engenharia multi-agente complexo.

Internamente:

```text
providers · modelos · specs · memória · grafos · MCP · segurança
traces · tokens · budgets · identidades · workflows · worktrees
```

Externamente:

```text
brian connect xpto
brian run "..."
brian costs
```

**[Δ]** E uma adição, que é a diferença entre este documento e o anterior:

> Brian precisa ser útil antes de estar completo.
>
> A versão que só conta tokens já resolve um problema real de quem
> opera agentes de IA para múltiplos clientes. Tudo o que vem depois
> é construído sobre um produto que já funciona, com dados que já
> existem, para um usuário que já o usa.

---

---

# SEÇÕES NOVAS

As seções §109 a §116 cobrem lacunas do v0.1-draft. Nenhuma delas é opcional: cada uma trata de um problema que aparece na primeira semana de uso real e que não tem contorno barato depois.

---

# 109. Modelo de Concorrência

**[Δ novo]. Ausente no v0.1 e bloqueante para o modo autônomo.**

## 109.1 O problema

O v0.1 previa múltiplos runs, session manager com quatro sessões simultâneas e modo autônomo, sem nunca tratar de onde esses runs escrevem. Dois agentes editando a mesma árvore de trabalho produzem:

```text
conflito de arquivo silencioso
diff misturado — impossível saber qual run fez o quê
teste de um run rodando contra código de outro
commit contendo trabalho de dois runs
atribuição de custo correta, mas atribuição de código errada
```

O último é o mais grave: destrói a auditabilidade (§78), que é uma premissa do produto.

## 109.2 Decisão (D-7)

> Todo run executa em um `git worktree` dedicado, criado a partir de um commit base explícito, e descartado ou promovido ao final.

```text
~/Projects/XPTO/checkout-api            árvore principal, do usuário
                                        Brian NUNCA escreve aqui

~/.brian/worktrees/
├── run_1421/                           worktree do run 1421
├── run_1422/                           worktree do run 1422
└── run_1423/
```

## 109.3 Ciclo de vida

```text
CRIAÇÃO
  git worktree add ~/.brian/worktrees/run_1421 \
      -b brian/run_1421 <base_commit>

  base_commit é registrado no banco (§60, tabela worktree)

EXECUÇÃO
  o agente recebe working_dir = o worktree
  todas as ferramentas são escopadas a ele
  escrita fora do worktree é negada pela capability layer

FINALIZAÇÃO — sucesso
  opção A: promover a branch, deixar para o usuário revisar
  opção B: merge/rebase para a branch alvo, se a política permitir
  opção C: gerar patch e descartar o worktree

FINALIZAÇÃO — falha ou cancelamento
  worktree preservado para inspeção
  marcado como released_at = null, status = abandoned
  brian worktree gc remove após retenção configurada
```

## 109.4 Limites

```yaml
concurrency:
  max_parallel_runs_global: 4
  max_parallel_runs_per_project: 2
  worktree_retention_days: 7
  disk_guard_gb: 20          # pausa criação abaixo deste limite livre
```

**Por que limite por projeto:** dois runs no mesmo projeto competem por recursos de teste — porta, banco local, container. Dois runs em projetos diferentes não competem.

## 109.5 Recursos compartilhados

Worktree isola filesystem, não isola tudo:

```text
ISOLADO POR WORKTREE
  arquivos, branch, index do Git

NÃO ISOLADO — exige coordenação
  portas de rede            → alocação dinâmica por run
  banco de dados local      → schema ou database por run
  containers                → nome com run_id
  cache de build            → compartilhado é aceitável e desejável
  quota de provider         → global, gerenciada pelo Router
```

```yaml
# no contexto do projeto
run_environment:
  port_range: [3100, 3199]      # Brian aloca uma por run
  database_url_template: "postgres://localhost/checkout_test_{run_id}"
  container_prefix: "brian_{run_id}_"
```

## 109.6 Interação com o usuário

O usuário continua trabalhando na árvore principal enquanto runs executam. Isso é o ponto: **um agente rodando não bloqueia o humano.** Sem worktrees, todo run autônomo exige que o usuário pare de editar, o que na prática significa que ninguém usa o modo autônomo.

---

# 110. Recuperação e Retomada

**[Δ novo]. Ausente no v0.1.**

## 110.1 O problema

Um run que dura vinte minutos vai ser interrompido: o Mac dorme, o processo é morto, o daemon reinicia, a rede cai, o provider trava. Sem tratamento explícito, cada interrupção produz:

```text
run permanentemente em status "running"
custo já gasto e não contabilizado
worktree órfão consumindo disco
processo de provider zumbi
```

## 110.2 Persistência antes de efeito (D-12)

Toda operação que produz efeito externo grava a intenção antes de executar:

```rust
fn execute_phase(&self, run: &mut Run, phase: &Phase) -> Result<PhaseOutcome> {
    // 1. registra a intenção
    let exec_id = self.store.begin_phase_execution(run.id, phase.id)?;

    // 2. só então age
    let result = self.provider.execute(self.build_request(run, phase, exec_id)?);

    // 3. registra o desfecho, mesmo em erro
    match result {
        Ok(handle) => {
            self.store.record_handle(exec_id, &handle)?;
            let outcome = self.await_completion(handle)?;
            self.store.complete_phase_execution(exec_id, &outcome)?;
            Ok(outcome)
        }
        Err(e) => {
            self.store.fail_phase_execution(exec_id, &e)?;
            Err(e)
        }
    }
}
```

Se o processo morrer entre 1 e 3, o banco mostra uma `phase_execution` sem `ended_at`. Isso é detectável.

## 110.3 Heartbeat

```text
run.heartbeat_at é atualizado a cada 15 segundos enquanto o run vive

heartbeat parado há > 60s e status = running  →  candidato a órfão
```

## 110.4 Recuperação

```bash
brian recover
```

```text
Analisando runs órfãos...

run_1421  xpto/checkout-api  fase: implement
          heartbeat parado há 42min
          processo do provider: não encontrado
          worktree: presente, 14 arquivos modificados
          custo já registrado: $3.82
          sessão do provider: codex session-72 (retomável)

          [1] retomar a sessão do provider
          [2] reiniciar a fase do zero (custo já gasto é preservado)
          [3] pausar e inspecionar o worktree
          [4] cancelar e descartar

Escolha:
```

## 110.5 Idempotência

Retomar não pode duplicar custo nem efeito.

```text
custo         registros de usage têm ID derivado de
              (phase_execution_id, provider_session_id, sequência)
              → reprocessar o mesmo arquivo de sessão não duplica

commit        Brian verifica se o commit esperado já existe
              antes de criar

worktree      recriação é idempotente: se o path existe e o
              base_commit confere, é reusado

memória       proposta com ID determinístico por (run, conteúdo)
```

## 110.6 Reconciliação de consumo

**[Δ]** Caso importante e sutil: o custo pode ter sido gasto no provider mesmo que o Brian não tenha registrado.

Por isso, `brian recover` sempre relê os arquivos de sessão do provider para o período do run, e concilia:

```text
registrado pelo Brian     $3.82
encontrado nas sessões    $4.91
                         ──────
diferença                 $1.09  → registrada com usage_source=session_file
                                    e nota de reconciliação
```

Sem isso, toda interrupção subestima o custo do cliente, que é uma falha direta da proposta de valor.

## 110.7 Sono do sistema

```text
macOS entra em sono   → processos de provider são suspensos
Brian detecta         → NSWorkspaceWillSleepNotification
Ação                  → grava marcador no run, pausa heartbeat
Ao acordar            → verifica se os processos sobreviveram
                        sobreviveram → retoma heartbeat
                        morreram     → marca como órfão
```

---

# 111. Intervenção Humana

**[Δ novo].** O v0.1 mencionava "reconectar a uma sessão viva" sem definir o modelo de interação. Este é o ponto em que a maioria das ferramentas de orquestração falha na prática.

## 111.1 Os três momentos

```text
ANTES        aprovação de plano, de gate, de operação privilegiada
DURANTE      correção de rumo com o agente trabalhando
DEPOIS       revisão de resultado, aprovação de memória
```

O v0.1 tratava bem o "antes" (§77) e o "depois" (§36). O "durante" é o que falta.

## 111.2 Intervenção durante o run

```bash
brian attach run_1421
```

Três modos, escolhidos ao anexar:

```text
OBSERVE
  Streaming de saída em tempo real. Sem entrada.
  Não interfere. Seguro em qualquer tier.

INTERJECT
  Injeta uma mensagem na próxima fronteira de turno.
  O agente vê como instrução adicional do usuário.
  Exige provider com sessão interativa (tier 1 ou 3).
  A mensagem é gravada no trace.

TAKEOVER
  Pausa o agente. O usuário assume o worktree.
  Ao terminar, escolhe: retomar o agente com o novo estado,
  ou finalizar o run manualmente.
```

## 111.3 Interject na prática

```text
$ brian attach run_1421 --interject

[codex] lendo src/payment/RefundService.ts
[codex] lendo src/payment/RefundRepository.ts
[codex] editando src/payment/RefundService.ts
> não crie uma tabela nova, use a coluna idempotency_key
  que já existe em payment_transaction

[injetado — será entregue no próximo turno]

[codex] lendo migrations/
[codex] recebida instrução do usuário
[codex] revisando abordagem...
```

A mensagem é gravada como um span:

```json
{
  "name": "human.interject",
  "attributes": {
    "brian.run_id": "run_1421",
    "brian.phase": "implement",
    "brian.turn_index": 14,
    "message": "não crie uma tabela nova, use a coluna idempotency_key..."
  }
}
```

**[Δ]** Isso importa para auditoria: uma mudança guiada por instrução humana no meio do run tem proveniência diferente de uma mudança autônoma, e o §78 precisa refletir isso.

## 111.4 Takeover

```text
$ brian attach run_1421 --takeover

Pausando codex... ok
Worktree: ~/.brian/worktrees/run_1421
Branch:   brian/run_1421

Você tem o controle. Ao terminar:
  brian resume run_1421       retoma o agente com seu estado
  brian finish run_1421       finaliza o run como concluído por você
  brian cancel run_1421       descarta
```

O trabalho manual do humano é registrado como uma `phase_execution` com `provider_id = null` e `role = human`. Custo zero, mas presente no histórico — o que evita que o Learning Engine futuro (§38) atribua ao provider um sucesso que foi humano.

## 111.5 Aprovações assíncronas

Um run em modo supervisionado pode ficar horas esperando aprovação. Isso não pode bloquear um processo.

```text
run entra em status = paused, pause_reason = awaiting_approval
processo do provider é encerrado, sessão preservada se retomável
notificação: menu bar, notificação do sistema, opcionalmente webhook
brian ps mostra na lista de pendências
aprovação retoma o run do ponto exato
timeout de aprovação configurável → cancela ou escala
```

---

# 112. Avaliação e Evals

**[Δ novo]. Pré-requisito de `D-13`.**

## 112.1 O problema

Um sistema cujo componente central é não-determinístico não pode ser validado só por testes unitários. As perguntas que importam são:

```text
O Router escolhe bem?
A mudança na regra melhorou ou piorou?
O upgrade do provider degradou algo?
O prompt novo é melhor que o anterior?
O Evaluator (§16.5) concorda com julgamento humano?
```

Nenhuma dessas é respondível por teste de unidade. Sem harness, "roteamento adaptativo" é fé, e toda mudança de prompt é aposta.

## 112.2 Casos de eval

```yaml
# evals/cases/refund-idempotency.yaml
id: refund-idempotency
description: "Adicionar idempotência a operação de refund"

fixture:
  repo: fixtures/checkout-api
  base_commit: a3f21b8

task: "Torne a operação de refund idempotente"

expected:
  must_modify:
    - src/payment/RefundService.ts
  must_not_modify:
    - src/checkout/**
  must_pass:
    - "npm test -- RefundService"
  must_contain_pattern:
    - file: src/payment/RefundService.ts
      ast_pattern: "idempotency_key"
  must_not_create_files_over: 3
  max_cost_usd: 3.00
  max_turns: 40

grading:
  automatic: [must_pass, must_modify, must_not_modify, must_not_create_files_over]
  llm_assisted: [must_contain_pattern]
  human_review_sample: 0.2
```

## 112.3 Execução

```bash
brian eval run                              # suíte inteira
brian eval run --case refund-idempotency
brian eval run --provider codex,claude      # comparação
brian eval compare --baseline v0.3.1        # regressão
```

```text
Suite: core · 24 casos · codex@0.48.2

PASSOU      19 (79%)
FALHOU       3
INSTÁVEL     2   (resultado variou entre execuções)

Custo total     $41.20
Custo médio     $1.72 por caso
Turnos médios   28

Regressões vs v0.3.1
  ⚠ migration-rollback: passava, agora falha
  ⚠ custo médio +18%
```

## 112.4 Variância

**[Δ]** Ponto crítico e frequentemente ignorado: agentes são não-determinísticos. Um caso que passa uma vez não passa sempre.

```text
Todo caso roda N=3 por padrão.
Resultado é reportado como taxa, não como booleano.
Casos com taxa entre 0.34 e 0.66 são marcados INSTÁVEL.
Comparações entre configurações exigem N=5 e diferença > 1 desvio.
```

Isso torna evals caros — uma suíte de 24 casos com N=3 custa cerca de $120. Por isso a suíte roda:

```text
antes de release
ao mudar regra de roteamento
ao mudar prompt de skill
ao detectar nova versão de provider
nunca em cada commit
```

## 112.5 Calibração do Evaluator

O Evaluator (§16.5) também precisa ser avaliado. Uma amostra de 20% de seus vereditos vai para revisão humana:

```text
Evaluator vs humano · últimos 100 vereditos

concordância        87%
falso "pass"         8%   ← o mais caro: aprova o que não funciona
falso "fail"         5%

Por tipo de evidência
  test        100%   (determinístico)
  code         89%
  absence      68%   ← confirma §16.5: absence não bloqueia sozinho
```

---

# 113. Migração e Versionamento

**[Δ novo].**

## 113.1 Quatro coisas versionadas

```text
1. schema do banco          migrations SQL numeradas
2. schema de contexto       campo schema_version em context (§5.1)
3. definições de workflow   version no YAML, congelada por run (§15.6)
4. price catalog            version, referenciada por usage_record (§41.3)
```

Cada uma tem regra diferente porque cada uma tem relação diferente com o passado.

## 113.2 Schema do banco

```text
migrations/
├── 0001_initial.sql
├── 0002_add_worktree.sql
└── 0003_add_cache_tokens.sql
```

```text
aplicação    automática na inicialização, em transação
registro     schema_migration com checksum
rollback     não suportado; correções são migrations novas
backup       cópia do arquivo antes de migrar, mantida 30 dias
```

**Regra:** migrations nunca destroem dado. Uma coluna removida é renomeada para `_deprecated_<nome>` e removida em uma versão maior seguinte, com aviso.

## 113.3 Schema de contexto

Contextos são editados pelo usuário e vivem por anos.

```text
schema_version < atual   → migrado na leitura, gravado ao salvar
schema_version > atual   → erro: "este contexto foi criado por uma
                            versão mais nova do Brian"
```

## 113.4 Definições de workflow

```text
workflow_version é congelada no run quando ele inicia (§15.6)
```

Isso significa que definições antigas precisam continuar carregáveis:

```text
workflows/
├── fast.yaml           versão corrente
└── .versions/
    ├── fast.v1.yaml
    └── fast.v2.yaml
```

Um run de seis meses atrás pode ser inspecionado com a definição que efetivamente o governava. Sem isso, a auditoria (§78) mente sobre o passado.

## 113.5 Price catalog

```text
usage_record.catalog_version aponta para a versão usada
recálculo é comando explícito, nunca automático (§41.3)
relatórios já emitidos não mudam por correção de preço
```

---

# 114. Onboarding de Repositório

**[Δ novo].**

## 114.1 O primeiro connect

```bash
$ brian connect xpto/checkout-api

Primeiro acesso a este projeto.

  repositório     ~/Projects/XPTO/checkout-api
  linguagem       TypeScript (detectado)
  testes          npm test (detectado em package.json)
  lint            eslint (detectado)
  build           npm run build
  branch base     main

  ○ Nenhum OpenSpec encontrado — opcional
  ○ Nenhuma memória ainda

  Confirmar? [S/n]
```

Detecção é heurística e barata: ler `package.json`, `Cargo.toml`, `Package.swift`, `Makefile`, workflows do CI. Nada de LLM, nada de índice.

## 114.2 O que NÃO acontece no primeiro connect

```text
sem indexação de código
sem análise por LLM do repositório
sem construção de grafo
sem geração de sumário
```

**[Δ]** Isso é deliberado. Um `connect` que leva seis minutos e custa $2 antes de qualquer trabalho útil é uma barreira de adoção. Indexação, quando existir (§21.2), é comando explícito e assíncrono:

```bash
brian index --project checkout-api
```

## 114.3 Detecção de comandos

Quando a detecção falha ou o projeto é atípico:

```yaml
# .brian/context.toml no repositório
[commands]
test = "npm test"
test_single = "npm test -- {pattern}"
lint = "npm run lint"
build = "npm run build"
typecheck = "npx tsc --noEmit"

[commands.setup]
# rodado uma vez ao criar um worktree novo
install = "npm ci"
```

`commands.setup` é o que torna worktrees viáveis em projetos com dependências pesadas: cada worktree novo precisa de `node_modules`. Alternativas configuráveis: symlink para um cache compartilhado, ou `pnpm` com store global.

## 114.4 Custo do worktree

```text
projeto Node com node_modules de 400 MB
  cópia completa por worktree     400 MB × 4 runs = 1.6 GB
  symlink para store compartilhado  ~5 MB por worktree
```

```yaml
worktree:
  dependency_strategy: symlink   # symlink | copy | install
  shared_store: ~/.brian/stores/checkout-api
```

Esse é o tipo de detalhe que decide se `D-7` é praticável. Um modelo de concorrência que consome dezenas de gigabytes não é usado.

---

# 115. Multi-Repositório

**[Δ novo].** O v0.1 tratava multi-repo como item do v0.4 (§84), mas isso afeta o modelo de Context desde o início.

## 115.1 Três topologias

```text
UM REPO POR PROJETO          o caso simples, e o padrão
MONOREPO                     um repo, múltiplos projetos lógicos
MÚLTIPLOS REPOS POR PROJETO  frontend + backend + infra
```

## 115.2 Modelagem

```yaml
# um repo por projeto — padrão
project:
  id: checkout-api
  repositories:
    - path: ~/Projects/XPTO/checkout-api
      role: primary
```

```yaml
# monorepo
project:
  id: checkout
  repositories:
    - path: ~/Projects/XPTO/platform
      role: primary
      scope: "services/checkout/**"     # [Δ] escopo dentro do repo
```

```yaml
# múltiplos repos
project:
  id: checkout
  repositories:
    - path: ~/Projects/XPTO/checkout-api
      role: primary
    - path: ~/Projects/XPTO/checkout-web
      role: secondary
    - path: ~/Projects/XPTO/infra
      role: reference          # [Δ] leitura apenas
```

## 115.3 Implicações

```text
WORKTREE
  um worktree por repositório com role primary ou secondary
  repositórios reference são montados read-only

ATRIBUIÇÃO
  custo é do projeto, não do repositório
  o mesmo repo pode pertencer a projetos diferentes (monorepo)
  → atribuição por escopo de caminho

GATES
  testes rodam por repositório
  um gate falho em qualquer primary bloqueia

COMMIT
  runs multi-repo produzem commits correlacionados
  o trailer Brian-Run: liga os commits entre repositórios (§78.1)
```

## 115.4 Escopo de caminho em monorepo

```text
services/checkout/**    → projeto checkout,  cliente XPTO
services/billing/**     → projeto billing,   cliente XPTO
libs/shared/**          → projeto platform,  cliente XPTO
```

Uma mudança que toca `libs/shared/` a partir de um run de `checkout` é sinalizada:

```text
⚠ este run modificou libs/shared/, fora do escopo de checkout
  impacto potencial: billing, platform
  [continuar] [reverter esses arquivos] [reclassificar o run]
```

---

# 116. Registro de Riscos

**[Δ novo].** Riscos ordenados por produto de probabilidade e impacto. Cada um tem mitigação e sinal de alerta.

## R-1 — Absorção pela plataforma

```text
Probabilidade   alta
Impacto         alto
Descrição       Providers absorvem orquestração, workflow, skills,
                telemetria e custo como features nativas.

Mitigação       D-10: Brian só ocupa espaço cross-provider e
                cross-client. §93 filtrado por esse teste.

Sinal           Um provider lança atribuição de custo por cliente
                ou gerenciamento de múltiplas identidades.
                → reavaliar §93.3 imediatamente
```

## R-2 — H-1 falha

```text
Probabilidade   média-alta
Impacto         médio
Descrição       Context Governor não reduz custo o suficiente.

Mitigação       §2.8 e D-5: nada depende dele. Remoção é exclusão
                de módulo.

Sinal           Resultado do Milestone 3 (§101).
```

## R-3 — Manutenção de adapters

```text
Probabilidade   certa
Impacto         médio, contínuo
Descrição       Cada release de cada provider pode quebrar o adapter.
                Cinco providers × releases frequentes = trabalho perpétuo.

Mitigação       D-4 (preferir contratos estáveis), §8.3 (matriz de
                compatibilidade versionada), degradação explícita
                em vez de falha silenciosa, evals disparados por
                nova versão detectada (§112.4).

Sinal           Tempo gasto em manutenção de adapter > 30% do
                tempo de desenvolvimento em um trimestre.
                → reduzir para dois providers suportados
```

## R-4 — Termos de uso de assinaturas

```text
Probabilidade   média
Impacto         alto — é risco comercial, não técnico
Descrição       Dirigir CLIs autenticados por assinatura de forma
                programática, atribuindo uso a múltiplos clientes,
                pode conflitar com termos de licenciamento por
                assento ou com cláusulas de revenda.

Mitigação       Verificar os termos de cada provider ANTES de
                posicionar comercialmente. Suportar modo API-key
                por cliente como alternativa limpa. Documentar
                claramente qual modo é adequado a qual uso.

Sinal           Qualquer cláusula sobre uso automatizado, revenda
                ou compartilhamento de assento.
                → deve ser verificado no Milestone 0 (§98)
```

**[Δ]** Este risco não aparecia no v0.1 e é o único que pode inviabilizar o modelo de negócio independentemente da qualidade técnica.

## R-5 — Escopo

```text
Probabilidade   alta
Impacto         alto
Descrição       28 subsistemas (§4). Um projeto solo não termina isso.

Mitigação       §2.7 (utilidade por camada), §81 (v0.0 com cinco
                subsistemas), critérios de conclusão por versão.

Sinal           Uma versão passar de 8 semanas sem entregar seu
                critério de conclusão.
                → cortar escopo, não estender prazo
```

## R-6 — Adoção pelo próprio autor

```text
Probabilidade   média
Impacto         crítico
Descrição       O usuário contorna o Brian para tarefas rápidas
                porque é mais lento que abrir o CLI direto.

Mitigação       §15 (fast path de 3 fases), §99 (v0.1 observa sem
                exigir mudança de fluxo), §95.1 como métrica primária.

Sinal           % de runs via Brian < 50% após o v0.3.
                → o problema é latência ou atrito, não features
```

## R-7 — Confiabilidade dos dados de custo

```text
Probabilidade   média
Impacto         alto
Descrição       Números de custo imprecisos tornam o diferencial
                primário (§93.3) inutilizável para faturamento.

Mitigação       §40.1 (origem obrigatória), §42.3 (confiança
                exposta), §110.6 (reconciliação após interrupção),
                Milestone 0 mede precisão contra painel oficial.

Sinal           Divergência > 5% contra o painel do provider.
```

## R-8 — Custo dos evals

```text
Probabilidade   média
Impacto         baixo-médio
Descrição       Suíte com N=3 custa ~$120 por execução. Rodar com
                frequência é caro; não rodar torna mudanças cegas.

Mitigação       §112.4 (disparo por evento, não por commit),
                suíte pequena e curada, fixtures locais em vez
                de repositórios reais grandes.

Sinal           Evals sendo pulados antes de release.
```

## R-9 — Complexidade de worktrees

```text
Probabilidade   média
Impacto         médio
Descrição       Projetos com dependências pesadas tornam worktrees
                lentos ou caros em disco.

Mitigação       §114.4 (estratégias de dependência), guard de disco
                (§109.4), limite de paralelismo.

Sinal           Criação de worktree > 60 segundos.
```

## R-10 — Deriva entre CLI e UI

```text
Probabilidade   média
Impacto         baixo
Descrição       A UI ganha capacidades que a CLI não tem, ou vice-versa.

Mitigação       §51.1 (ambas consomem o mesmo socket e protocolo),
                critério de conclusão do v0.3 (§84).

Sinal           Uma feature implementada só na UI.
```

---

# Apêndice A — Resumo executivo das mudanças

Para quem já leu o v0.1-draft, as mudanças que importam:

**1. A ordem de construção foi invertida.** Contabilidade de custo vem primeiro (§81, §98, §99), porque é útil sozinha, tem risco técnico mínimo e gera o dado de que todo o resto depende. UI vem em quarto lugar.

**2. O Context Governor virou hipótese.** Deixou de ser pilar de produto (§18) e passou a ter experimento com critério de descarte declarado antes da coleta (§101). Prompt caching e busca agêntica trabalham contra a premissa original.

**3. O Workflow virou dado.** Dez fases fixas em código viraram YAML versionado, com fast path de três fases como padrão (§15). A fronteira com o Reasoning Engine foi formalizada: reasoning propõe, workflow decide, e só o workflow transiciona.

**4. O storage foi decidido.** SQLite, com schema completo (§60) e critério de reversão explícito. O spike SurrealDB foi cancelado e substituído por um benchmark de meio dia (§61).

**5. Adapters de provider ganharam contrato.** Trait obrigatória mínima mais traits opcionais (§10), tiers de integração declarados (§8.2), matriz de compatibilidade versionada (§8.3) e degradação explícita em vez de falha silenciosa.

**6. Oito seções novas cobrem o que faltava.** Concorrência por worktree (§109), recuperação e reconciliação de custo (§110), intervenção humana durante o run (§111), evals com tratamento de variância (§112), migração (§113), onboarding (§114), multi-repo (§115) e registro de riscos (§116).

**7. [D-16] Zero token perdido — lei no minuto 0.** Ledger completo, janelas, %, restante, burn, assinatura, atribuição e otimização de cada centavo. Unattributed silencioso é bug. Distinct de H-1.

**8. [D-17] Memória/continuidade para chavear LLM sem perda.** Continuity Pack + memória do Brian; trocar provider não recomeça conversa, análise nem decisões. Mínimo no v0.1; engine rica depois. Junto com D-16, é o que permite a missão: **IA poupa ~90% do trabalho/tempo — não gasta dinheiro.**

**O que permaneceu intacto:** Context como fronteira de tenancy, determinístico antes de probabilístico, memória pertencente ao Brian, separação entre custo equivalente e faturamento real, classes de secret, SkillSpector e a disciplina de não-objetivos. Esses eram — e continuam sendo — a melhor parte do documento original.

---

# Apêndice B — Índice de decisões e hipóteses

```text
D-1   SQLite                              §58, §60, §61
D-2   Rust core, CLI primeiro             §51, §52, §82–§85
D-3   Workflow como dado                  §15
D-4   Hierarquia de tiers de adapter      §8.2, §10, §56
D-5   Governor isolado                    §18.5, §2.8
D-6   Custo reportado tem precedência     §41.1, §42
D-7   Worktree por run                    §109
D-8   Router por regras até n≥30          §11.1, §38.2
D-9   Storage atrás de traits             §59
D-10  Escopo cross-provider/cross-client  §1, §32.1, §92, §93
D-11  Nomenclatura "Brian Core"           §96
D-12  Persistência antes de efeito        §110.2
D-13  Evals antes de roteamento adaptativo §112
D-14  Memória append-only                 §34.3
D-15  MCP como única escrita em workflow  §33.1
D-16  Zero token perdido / capacity t=0   §1, §2.9, §13, §43–§45, §81
D-17  Continuidade multi-LLM sem perda    §1, §2.10, §34.0, §82

H-1   Context Governor reduz custo ≥30%   §18, §101
```

---

*Fim do documento.*
