# Brian — Premissas Básicas

> Documento canônico das **premissas não negociáveis**.  
> Fonte para OpenSpec e para o desenvolvimento.  
> Blueprint completo: `BRIAN-BLUEPRINT-V1.md`.  
> Decisões travadas: `docs/DECISIONS.md`.

**Status:** draft canônico (alinhado ao blueprint v1 + D-16/D-17)  
**Público:** o autor + futuros changes OpenSpec  
**Regra:** se um change OpenSpec contradisser este arquivo, o change está errado — a menos que este arquivo seja atualizado de propósito.

---

## 0. Em uma frase

```text
Brian faz a IA resolver de forma assertiva, com direção clara e poucas indas e vindas —
aproveitando cada token, sintonizando o uso o tempo todo, mantendo a stack atual,
aprendendo com o próprio histórico — sem queimar dinheiro e sem perder a cabeça
do trabalho ao trocar de LLM ou de cliente.
```

### Oito princípios operacionais (não podem escapar)

| ID | Princípio | Em uma linha |
|----|-----------|--------------|
| **OP-1** | **Eficiência na utilização** | Cada token, % de janela e minuto de assinatura vão para trabalho útil. |
| **OP-2** | **Tuning contínuo** | Medir → ajustar (modelo, workflow, handoff, limites) → medir de novo. Nunca “configurou e esqueceu”. |
| **OP-3** | **Tecnologia sempre atualizada** | Providers, modelos, adapters e toolchains acompanham o que existe de melhor — com contrato e verificação, não caos. |
| **OP-4** | **Inteligência retroalimentada** | Resultado de runs, custos, falhas e handoffs alimentam o próximo ciclo (dados no Brian, não achismo). |
| **OP-5** | **IA para resolver** | Objetivo é **fechar trabalho**, não “conversar com o modelo”. Sessão sem outcome é desperdício. |
| **OP-6** | **Direcionamento claro** | OpenSpec / objetivo / success criteria explícitos antes de gastar capacidade. |
| **OP-7** | **Trabalhos assertivos** | Preferir a menor ação correta; modelo certo; evidência determinística; sem rodeio. |
| **OP-8** | **Poucas indas e vindas** | Cortar loops de reexplicação, rework e fases inúteis. Handoff denso; fix com log, não com “tenta de novo”. |

```text
OP-1..OP-8  =  COMO o Brian se comporta no dia a dia
D-16        =  nenhum token some (dinheiro)
D-17        =  nenhum contexto some na troca de LLM (tempo)
M1..M4      =  POR QUÊ o produto existe
```

Qualquer feature, OpenSpec change ou default de produto que viole OP-1…OP-8 está **errada**, mesmo que “funcione”.

---

## 1. Missão (não negociável)

| # | Premissa | Implicação |
|---|----------|------------|
| M1 | A IA existe para **poupar trabalho e tempo** e **resolver** (~90% do esforço manual onde couber). | Features que só “fazem IA” sem economizar tempo real são falha. **OP-5** |
| M2 | A IA **não** existe para gastar dinheiro. | Capacidade paga (assinatura/API) é recurso escasso a proteger e atribuir. **OP-1** |
| M3 | Brian **não** é um coding agent. | Não compete com Claude Code, Codex, Gemini, Grok, etc. no *inner loop*. |
| M4 | Brian é o **plano de controle** (capacidade, identidade, memória, atribuição, governança). | Atua onde há **N providers** e/ou **M clientes** — e no tempo. |
| M5 | Trabalho com **direção clara**, **assertivo**, **poucas idas e vindas**. | Default = caminho curto até o done. **OP-6, OP-7, OP-8** |
| M6 | O sistema **aprende com o próprio uso** e se **mantém atual**. | Telemetria + tuning + adapters vivos. **OP-2, OP-3, OP-4** |

```text
Um provider vê    uma sessão.
Brian vê          N providers × M clientes × T tempo
                  + cada token
                  + a memória que sobrevive à troca de LLM
                  + o feedback do que funcionou ou desperdiçou.
```

---

## 1.1 Princípios operacionais — detalhe (OP-1 … OP-8)

### OP-1 — Eficiência na utilização

```text
PREMISSA
  Capacidade paga é finita. Desperdício é falha de produto.

OBRIGA
  - ledger completo (D-16)
  - unattributed = alarme
  - default de workflow curto (direct/fast)
  - modelo pointer adequado à tarefa (não o mais caro por hábito)
  - Continuity Pack denso (não dump de histórico) — D-17 + OP-1
  - alertas de burn e % de janela

PROÍBE
  - fases LLM cerimoniais no default
  - reexplicar o mundo a cada LLM (paga duas vezes)
  - compare multi-provider sem necessidade perto do limite de capacidade
```

### OP-2 — Tuning contínuo

```text
PREMISSA
  O ótimo de ontem (modelo, regra, skill, limite) envelhece.

OBRIGA
  - registrar outcome + custo + retries de todo consumo
  - capacity e costs sempre consultáveis
  - regras de routing/skills em dado (YAML) revisável
  - após release de provider: verify + smoke de extração
  - ajustar limites soft/hard e baselines de plano com uso real

PROÍBE
  - “deixar no automático cego” sem olhar capacity/semana
  - router adaptativo sem n e sem eval (D-8, D-13)
```

Ciclo mínimo:

```text
usar → medir (tokens, $, sucesso, idas-e-vindas)
    → ajustar (pointer, workflow, pack, budget, provider)
    → usar de novo
```

### OP-3 — Tecnologia sempre atualizada

```text
PREMISSA
  Worker e modelo velhos = mais tokens, mais idas e vindas, menos assertividade.

OBRIGA
  - adapters com matriz de compatibilidade versionada
  - detecção de versão do provider no attach/verify
  - model pointers semânticos (coding/quick/review) mapeáveis a nomes novos
  - roadmap de upgrade sem reescrever o core
  - degradar de forma explícita se o adapter quebrar (não falha silenciosa)

PROÍBE
  - hardcode eterno de um model id como verdade do produto
  - PTY como estratégia principal (frágil; D-4)
  - ignorar release notes de provider por meses
```

Atualizar ≠ reescrever Brian. Atualizar = **trocar o worker/ponteiro** com D-16 e D-17 intactos.

### OP-4 — Inteligência retroalimentada

```text
PREMISSA
  O Brian fica mais útil quanto mais o histórico de uso real o alimenta.

OBRIGA
  - todo usage_record e outcome alimentam o ledger (base de feedback)
  - handoffs e decisões ficam na memória do Brian (D-14, D-17)
  - insights determinísticos: burn anômalo, cache hit baixo, retry alto
  - no futuro: learning só com n≥30 e evals (D-8, D-13) — feedback com rigor
  - comparação pareada (--compare) gera preferência humana rotulada

PROÍBE
  - “inteligência” que não grava evidência
  - otimização por achismo de prompt sem medir $/sucesso
  - memória sem proveniência
```

```text
dado de uso  →  ledger + memória  →  tuning (OP-2)  →  melhor uso (OP-1)
```

### OP-5 — IA para resolver

```text
PREMISSA
  Sucesso = trabalho fechado (diff + testes + decisão), não engajamento com o chat.

OBRIGA
  - success_criteria explícitos quando houver tarefa
  - gates determinísticos (tests/lint/secrets) como juiz barato
  - Continuity Pack com next_steps e failed_attempts (evitar loop vazio)
  - preferir direct/fast até o done
  - escalar a humano com estado limpo, não com conversa infinita

PROÍBE
  - workflows que “discutem” em 10 fases o que um fix resolve
  - sessão sem outcome contabilizado como sucesso
```

### OP-6 — Direcionamento claro

```text
PREMISSA
  Sem alvo claro, a IA vagueia — e vaguear custa token e tempo.

OBRIGA
  - objetivo + critérios de sucesso no Continuity Pack / OpenSpec / prompt de run
  - context ativo (cliente/projeto) antes de trabalho atribuído
  - policy e paths sensíveis explícitos quando relevantes
  - “done” definido por workflow (gates + aprovação se preciso)

PROÍBE
  - “faz aí uma melhoria” sem critério como default de produto
  - run sem saber a qual cliente/projeto pertence
```

### OP-7 — Trabalhos assertivos

```text
PREMISSA
  A melhor ação é a menor correta. Certeza > performance de raciocínio longo.

OBRIGA
  - determinístico antes de probabilístico
  - evidência (log de teste, diff, secret scan) manda no fix
  - modelo forte só onde o barato falha ou o risco exige
  - skills com allowed/denied tools
  - max_entries em fix — não spin infinito

PROÍBE
  - “explorar o repo inteiro” como default quando o pack já tem o foco
  - reviewer LLM quando o teste já disse o erro
```

### OP-8 — Poucas indas e vindas

```text
PREMISSA
  Cada ida-e-vinda é imposto de tempo e de capacidade.

OBRIGA
  - D-17: handoff sem reexplicar
  - fix recebe log/erro, não “tente de novo do zero”
  - default 1 fase de trabalho + gates (direct) quando possível
  - max_entries + escalate para humano
  - outcomes tipados (infra ≠ test_fail) para não gastar fix em erro de rede

PROÍBE
  - 10 fases default (erro do v0.1-draft)
  - pedir de novo o mesmo contexto que já está no Continuity Pack
  - loops de “mais um turn” sem critério de parada
```

Métrica proxy de OP-8:

```text
retries médios por change bem-sucedido   → baixo
handoffs que exigiram reexplicação       → 0
fases LLM por change trivial             → 1 (ideal)
```

---

## 2. Os dois ganhos mínimos (pilares)

Tudo o mais (workflow, UI, graph, learning, K8s) é **secundário** até estes dois estarem sólidos.

### Pilar A — Capacidade / dinheiro → **D-16**

| ID | Premissa |
|----|----------|
| P-A1 | **Nenhum token observável some.** Perda silenciosa = bug de release. |
| P-A2 | Todo consumo entra no **ledger** (`usage_record`) com origem rotulada. |
| P-A3 | Janelas são first-class: day / week / month / plan_reset (e custom). |
| P-A4 | O usuário vê **% usado, restante, tempo até reset, burn** por provider/plano. |
| P-A5 | Assinatura ≠ API: em assinatura a moeda primária é **fração de capacidade**. |
| P-A6 | Todo token tem **dono** (cliente) ou fica `unattributed` de forma **explícita e ruidosa**. |
| P-A7 | Cada centavo / cada % da janela deve ir para **trabalho útil** (otimização de aproveitamento). |
| P-A8 | `brian capacity` / `usage` / `costs` existem no **minuto 0** — antes de orquestração e UI. |
| P-A9 | Observe mode: uso **fora** de `brian run` também conta. |
| P-A10 | Import histórico (`--since 30d`) no onboarding; primeiro wow não é ledger vazio. |

### Pilar B — Continuidade / tempo → **D-17**

| ID | Premissa |
|----|----------|
| P-B1 | A **cabeça do trabalho** vive no Brian, não no transcript do provider. |
| P-B2 | Chavear Claude ↔ Codex ↔ outro **não** exige reexplicar o problema. |
| P-B3 | Existe **Continuity Pack**: objetivo, decisões, análise, touched files, falhas, next steps, evidências. |
| P-B4 | Memória é **por Context** (cliente/projeto); nunca vaza entre clientes. |
| P-B5 | Memória é **append-only** com proveniência (D-14); correções supersedem. |
| P-B6 | Handoff é **otimizado** (denso, com orçamento de tokens) — não dump do histórico inteiro. |
| P-B7 | Reexplicar após handoff = **bug de produto** (D-17 falhou). |
| P-B8 | D-17 mínimo sobe no **v0.1** (não espera Memory Engine “completo” do v0.4). |

### Como os pilares se reforçam

```text
D-16  impede a IA de roubar dinheiro (token sumido / janela cega / desperdício)
D-17  impede a IA de roubar tempo    (recomeçar do zero a cada LLM)
Juntos → a IA pode de fato poupar ~90% do trabalho
```

Sem D-17, você **paga duas vezes** o mesmo raciocínio em LLMs diferentes (viola M2 e M1).  
Sem D-16, você **não sabe** se está queimando a assinatura (viola M2).

---

## 3. Fronteira de produto (o que Brian É / NÃO É)

### É

```text
✓ Control plane multi-cliente e multi-provider
✓ Ledger de tokens e capacidade (assinatura + API)
✓ Atribuição / showback / chargeback por cliente
✓ Context + identidade isolada por cliente
✓ Memória e continuidade que sobrevivem à troca de LLM
✓ Observe primeiro; orquestra só quando agregar valor
✓ Gates determinísticos antes de LLM extra
✓ CLI first; UI depois
```

### Não é

```text
✗ Mais um coding agent / “Claude wrapper”
✗ Orquestrador do inner loop (think→tool→observe) de um provider  [D-10]
✗ Substituto de skills/plugins nativos do provider
✗ Graph RAG obrigatório no day 1
✗ Plataforma Kubernetes no ano 1 (só agnosticismo de core)
✗ Learning / router “inteligente” sem dados e sem evals
✗ Context Governor como pilar (é hipótese H-1)
```

### Teste D-10 (escopo)

```text
Se um único provider, numa única sessão, já resolve sozinho
→ Brian NÃO deve reimplementar isso.

Se o problema é N providers, M clientes, T tempo,
  capacidade, identidade, memória entre LLMs, atribuição
→ isso É Brian.
```

---

## 4. Princípios de engenharia (premissas de design)

| ID | Premissa | Notas |
|----|----------|--------|
| E1 | **Context + Run** (ou Context + observe) é a unidade de trabalho, não o prompt. | |
| E2 | **Determinístico antes de probabilístico.** | git, tests, lint, secrets, AST antes de LLM. |
| E3 | **Providers são workers substituíveis** — com custo de adapter. | Nenhum provider é permanente. |
| E4 | **Trait mínima de adapter** + opcionais. | Hierarquia: headless JSON → session files → PTY. |
| E5 | **Custo reportado pelo provider > price catalog.** | Catalog sempre rotulado (D-6). |
| E6 | **CLI first; SwiftUI só depois.** | D-2. |
| E7 | **SQLite** operacional local; storage atrás de traits. | D-1, D-9. |
| E8 | **Workflow = dado (YAML)**, não 10 fases fixas em código. | D-3; default curto (`direct`/`fast`). |
| E9 | Workflow engine **não chama LLM**; só transiciona. | Reasoning propõe. |
| E10 | **Persistir estado antes de efeito colateral.** | D-12. |
| E11 | Run em **git worktree** quando houver execução orquestrada. | D-7. |
| E12 | **Nada depende de hipótese não testada.** | H-1 isolado (D-5). |
| E13 | **Evals antes de roteamento adaptativo.** | D-13. |
| E14 | Hipóteses ≠ pilares. | Governor = H-1; Usage Control = lei. |
| E15 | **Utilidade em cada versão.** | v0.0 útil sozinho; v0.1 útil sozinho; … |

---

## 5. Premissas de capacidade (detalhe D-16)

### 5.1 Integridade do ledger

```text
∀ token observado  →  usage_record
∀ usage_record     →  usage_source + cost_source + occurred_at + billing_mode
∀ usage_record     →  client_id  OU  attribution_status = unattributed (visível)
unattributed silencioso  →  PROIBIDO
```

### 5.2 Níveis de verdade (nunca misturar sem rótulo)

```text
Nível 1  provider reportou (quota, cost, tokens)     → autoritativo
Nível 2  Brian mediu na janela                       → verdade operacional
Nível 3  estimado (catalog / baseline declarada)     → sempre rotulado
```

Ausência de nível 1 **não** desliga nível 2.

### 5.3 Janelas obrigatórias

```text
rolling_hour | calendar_day | calendar_week | calendar_month | plan_reset | session | run | custom
```

Default de visão humana de assinatura: **week** e/ou **plan_reset**.

### 5.4 Superfície mínima (minuto 0)

```text
brian capacity
brian usage [--window] [--by client|provider]
brian costs [--client] [--unattributed] [--export]
brian status
brian import --since 30d
brian attribute <id> --client …
brian plans …
```

### 5.5 Critério de conclusão D-16 (v0.0)

```text
1. capacity < 5s com % / restante / reset / burn por provider anexado
2. usage por cliente soma com a janela
3. costs verificável vs painel (±5% no que for nível 1/2)
4. unattributed = vazio após trabalho real + correção
5. import 30d popula ledger
```

Falha em qualquer item = **não avança** para v0.1.

---

## 6. Premissas de continuidade (detalhe D-17)

### 6.1 Continuity Pack (conteúdo mínimo)

```text
objective + success_criteria
decisions (+ why)
analysis (o que vale e o que foi descartado)
conversation_digest (não log bruto)
touched files/symbols
failed_attempts
next_steps
open_questions
evidence_refs (run_id, paths, usage_ids)
budget_hint (opcional: % janela)
```

### 6.2 Superfície mínima (v0.1)

```text
brian memory note "…"
brian memory decide "…" --why "…"
brian continuity show
brian handoff --to <provider>
brian continuity inject --provider <provider>
```

### 6.3 Critério de conclusão D-17 (mínimo)

```text
20 min de trabalho real no provider A sob um context
→ handoff para provider B
→ usuário NÃO reexplica objetivo / decisões / o que falhou
→ pack cita arquivos reais
→ custo do pack limitado e rotulado
```

### 6.4 O que continuidade NÃO é

```text
✗ Despejar 200 turnos de chat no próximo modelo
✗ Graph obrigatório
✗ Embedding enterprise no day 1
✗ Context Governor (H-1)
```

---

## 7. Premissas de contexto e identidade

| ID | Premissa |
|----|----------|
| C1 | **Context** = fronteira de tenancy (cliente + projeto + políticas + memória + budget). |
| C2 | `brian connect <cliente>` ativa o mundo operacional. |
| C3 | Identidades de provider **isoladas** por cliente (sem vazar conta/secret). |
| C4 | Secrets por **referência** (Keychain), não no SQLite operacional. |
| C5 | Execução sem contexto: falha clara **ou** observe com unattributed ruidoso — nunca mistura silenciosa de clientes. |
| C6 | Troca de contexto &lt; 2s (critério de produto). |

---

## 8. Premissas de execução (quando existir `run`)

| ID | Premissa | OP |
|----|----------|-----|
| R1 | Observe **antes** de exigir mudança de fluxo (`brian run`). | OP-1 |
| R2 | Default de workflow **curto** (`direct` / `fast`); governed é opt-in. | OP-8 |
| R3 | Fase = envelope (role, modelo, gates, humano, budget) — **não** micro-passo mental do agente. | OP-7 |
| R4 | Outer loop = Brian; inner loop = provider. | D-10 |
| R5 | Gates determinísticos antes de nova fase LLM. | OP-5, OP-7 |
| R6 | Outcomes tipados (test_fail vs infra vs policy vs budget). | OP-8 |
| R7 | Hard limits de $ / % / turnos; soft = alerta + política de economia. | OP-1 |
| R8 | Worktree por run; recover após kill. | |
| R9 | Todo run termina em **outcome** (done / failed / escalated) — não em conversa aberta. | OP-5 |
| R10 | Direção (objetivo + success criteria) entra **antes** da primeira chamada cara. | OP-6 |

---

## 9. Ordem de construção (premissa de sequência)

```text
v0.0   D-16  Capacity + ledger + atribuição + import + CLI
       (zero token perdido; sem orquestração)

v0.1   D-17  Context + identity + Continuity Pack mínimo
       (chavear LLM sem perda; connect)

v0.2   Run + worktree + workflow curto + handoff no run

v0.3   Comparação / router por regras / UI

v0.4+  Memory engine rica, code intel, evals largos, …

NUNCA  workflow/UI/learning antes de D-16 verde
NUNCA  “orquestração inteligente” antes de D-17 mínimo + evals (D-13)
```

### OpenSpec — primeiros changes (ordem)

```text
1. client-cost-attribution          (D-16 / v0.0)
2. capacity-windows-and-plans       (D-16 / janelas, %, plans)
3. context-and-identity-switching   (v0.1)
4. continuity-pack-handoff          (D-17 / v0.1)
5. isolated-tracked-run             (v0.2)  — só depois de 1–4
```

Nomes finais podem mudar no OpenSpec; a **ordem e as leis** não.

---

## 10. Premissas de stack (iniciais)

| Escolha | Premissa |
|---------|----------|
| Linguagem | **Rust** no Brian Core (D-2) |
| Interface v0 | **CLI** (`clap`) |
| Storage | **SQLite** + traits (D-1, D-9) |
| Telemetria inicial | eventos de usage/run no SQLite; OTEL completo pode esperar |
| UI nativa | **depois** (v0.3), mesmo protocolo que a CLI |
| K8s | pasta/adapter vazio; **não** construir até demanda real |

---

## 11. Premissas de qualidade e verdade

| ID | Premissa |
|----|----------|
| Q1 | Números sem **origem** não aparecem na UX. |
| Q2 | Explicações de decisão vêm de **sinais gravados**, não de LLM post-hoc. |
| Q3 | Estatísticas de provider mostram **n**; n baixo não dirige router (D-8). |
| Q4 | Evals com variância (taxa, N≥3); não boolean único em agente. |
| Q5 | Correção humana de atribuição é **auditada**. |
| Q6 | ToS / modo assento vs API-key: validar cedo (risco comercial R-4). |

---

## 12. Métricas que importam (premissas de sucesso)

### Primárias (missão + OP)

```text
% do gasto real de AI da semana capturado no ledger     → 100% (D-16, OP-1)
unattributed_tokens                                     → 0
handoffs sem reexplicação (dogfood)                     → regra (D-17, OP-8)
retries médios / idas-e-vindas por change bem-sucedido  → baixo (OP-8)
% changes com success criteria explícitos               → alto (OP-6)
sensação: "a IA RESOLVEU, com poucas voltas"              → qualitativo (OP-5, OP-7)
```

### Operacionais

```text
brian capacity < 5s
brian connect < 2s
erro vs painel do provider < 5% (níveis 1/2)
cache hit ratio (insight de desperdício)                (OP-1, OP-4)
used_percent / remaining / time_to_reset por plano
custo do Continuity Pack << custo de recomeçar a análise
adapter/provider version detectada e verify ok          (OP-3)
ajustes de pointer/regra baseados em dado da semana     (OP-2, OP-4)
```

### North-star de adoção (R-6)

```text
% do gasto de AI do mês que o Brian atribuiu
% de dias com connect/capacity no fluxo real
Se < 50% depois de v0.1 → problema é atrito, não falta de feature
```

---

## 13. Teste de ouro do produto

```text
PASS se, numa semana real de trabalho multi-cliente:
  1. Nenhum token observável ficou de fora do ledger          (D-16, OP-1)
  2. brian capacity responde em < 5s com % e restante
  3. handoff Claude→Codex não exige reexplicar o trabalho     (D-17, OP-8)
  4. tarefas fecham com poucas idas e vindas                  (OP-5, OP-7, OP-8)
  5. direção clara (objetivo/critérios) antes de gastar caro  (OP-6)
  6. houve ao menos um ajuste de uso baseado em dado          (OP-2, OP-4)
  7. providers/adapters não estão “presos” em versão morta    (OP-3)
  8. o usuário sente economia de TEMPO e de DINHEIRO

FAIL se:
  - tokens somem ou unattributed é “normal”
  - trocar de modelo reinicia a história
  - muitas idas e vindas / rework por falta de direção
  - Brian é mais lento/chato que o CLI sem devolver controle nem continuidade
  - o produto “gasta” capacidade em cerimônia (fases demais, handoff gordo)
  - config congelada enquanto o mundo de modelos mudou
```

---

## 14. O que está FORA das premissas básicas (adiado de propósito)

Não apagar do blueprint — **não** tratar como premissa de v0:

```text
Context Governor (H-1)
Learning Engine / router adaptativo
SwiftUI
Kubernetes multi-tenant
Browser bridge, Xcode deep
SkillSpector completo
Graphify / Sourcebot como default
Workflow governed longo como default
24 agent-evals caros no day 1
```

Estes só entram com **dor medida** ou versão explícita.

---

## 15. Glossário mínimo (para OpenSpec)

| Termo | Significado |
|-------|-------------|
| **Context** | Cliente + projeto + policies + memória + budget + identidades ativas |
| **Capacity** | Capacidade paga (tokens/%/$) numa janela |
| **Ledger** | Conjunto de `usage_record` — verdade de consumo |
| **Observe** | Contar e atribuir sem orquestrar o agent |
| **Continuity Pack** | Estado de handoff entre LLMs (D-17) |
| **Worker / Provider** | Claude, Codex, etc. — executa o inner loop |
| **Outer loop** | phase → session → gates → transition (Brian) |
| **Inner loop** | think → tool → observe (provider) |
| **Unattributed** | Consumo sem cliente — alarme, não estado normal |
| **H-1** | Hipótese do Context Governor — **não** é premissa |

---

## 16. Checklist de conformidade OpenSpec

Todo change OpenSpec deve responder:

```text
[ ] Respeita M1–M6 (missão)?
[ ] Respeita OP-1…OP-8? (eficiência, tuning, tech atual, feedback,
    resolver, direção, assertivo, poucas idas-e-vindas)
[ ] Toca D-16 ou D-17? Se sim, como garante zero loss / zero reexplicar?
[ ] Viola D-10 (compete com inner loop de um provider)? Se sim → recusar.
[ ] Introduz dependência de H-1? Se sim → recusar ou isolar.
[ ] Em que versão entra (v0.0 / v0.1 / …)? Respeita a ordem da §9?
[ ] Tem critério de conclusão mensurável (inclui menos idas-e-vindas / $/sucesso)?
[ ] Números/custos têm origem (nível 1/2/3)?
[ ] O que NÃO faz (não-objetivos do change)?
[ ] Aumenta eficiência de utilização ou só “mais feature”?
```

### Anti-padrões (recusar no review)

```text
✗ Mais uma fase LLM no default sem provar menos rework
✗ Handoff que reenvia histórico inteiro (viola OP-1 e OP-8)
✗ Feature sem telemetria de uso (viola OP-2 e OP-4)
✗ Model id cravado sem pointer (viola OP-3)
✗ Chat sem success criteria como fluxo principal (viola OP-5 e OP-6)
✗ “Explorar tudo” quando o pack já tem foco (viola OP-7)
✗ Retry cego sem evidência (viola OP-7 e OP-8)
```

---

## 17. Mapa rápido → blueprint

| Premissa | Blueprint / docs |
|----------|------------------|
| Missão / pilares | §1, §1.0, §104 |
| OP-1…OP-8 | este arquivo §0 e §1.1 |
| D-16 capacity | §2.9, §13, §43–§45, §81, §98 |
| D-17 continuidade | §2.10, §34.0, §82 |
| Decisões D-1…D-17 | Registro de decisões + `docs/DECISIONS.md` |
| Ordem de versões | §4, §81–§85, §106 |
| Diferenciais | §93.0 |
| Riscos | §116 |
| Schema | §60 |

---

## 18. Próximo passo (depois deste doc)

```text
1. Este arquivo é a lei do repo (missão + OP + D-16/D-17)
2. docs/DECISIONS.md (D-1…D-17)
3. OpenSpec change #1: client-cost-attribution + capacity (D-16) — OP-1 em código
4. Implementar v0.0 em Rust + SQLite
5. OpenSpec change #2: continuity-pack-handoff (D-17) — OP-8 em código
```

---

## 19. Cartão de bolso (imprimir mentalmente)

```text
EFICIÊNCIA          cada token trabalha
TUNING              medir → ajustar → medir
TECH ATUAL         workers e ponteiros vivos
RETROALIMENTAÇÃO    uso vira inteligência no Brian
RESOLVER            fechar trabalho, não conversar
DIREÇÃO             objetivo e critério antes do gasto
ASSERTIVO           menor ação correta + evidência
POUCAS VOLTAS       sem reexplicar, sem rework cego

+ D-16 zero token perdido
+ D-17 zero contexto perdido no handoff
```

---

*Premissas básicas. Se não está aqui, ainda não é lei — está no blueprint como visão ou hipótese.*
