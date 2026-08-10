# Brian — Decisões travadas (D-1 … D-17)

> Resumo executável. Detalhe e critério longo: `BRIAN-BLUEPRINT-V1.md`.  
> Premissas de produto: `docs/PREMISSAS-BASICAS.md`.

**Convenção:** decisão travada **para de ser rediscutida** até o critério de reversão.

| ID | Decisão | Reversão |
|----|---------|----------|
| **D-1** | SQLite como banco operacional único, atrás de traits. | Query real de produção > 200 ms com 12 meses de dados. |
| **D-2** | Brian Core em Rust. CLI first. SwiftUI só no v0.3. | Só se CLI provar que o produto não se sustenta sem UI. |
| **D-3** | Workflow é YAML versionado; default curto (fast/direct). | 3 workflows distintos exigirem lógica que YAML não expressa. |
| **D-4** | Adapters: headless JSON → session files → PTY (PTY nunca 1ª escolha). | Provider relevante sem as duas primeiras opções por 2 releases. |
| **D-5** | Context Governor é hipótese (H-1), não pilar. Nada depende dele. | H-1 confirma ≥30% redução de $/change bem-sucedido. |
| **D-6** | Custo reportado pelo provider > price catalog (catalog rotulado). | Nenhum. |
| **D-7** | Todo run orquestrado em git worktree dedicado. | Nenhum (pré-requisito de concorrência). |
| **D-8** | ~~Router por regras + override até n≥30 por célula; depois evals.~~ **[Δ 2026-08-09]** Revertida antecipadamente por decisão consciente do autor — scoring histórico entra no código antes de haver `n≥30` reais por célula. Risco aceito e assumido: "confiança injustificada" nas próprias palavras do blueprint §11.1 até o volume real de uso chegar lá. | ~~Limiar + harness de eval funcionando.~~ Reversão consumida. |
| **D-9** | SQL só em `storage/` via traits. | Nenhum. |
| **D-10** | Brian não orquestra inner loop de um provider. Só N providers / M clientes. | Nenhum (definição de produto). |
| **D-11** | Runtime = “Brian Core”. Termo “Brain” proibido. | Nenhum. |
| **D-12** | Persistir estado do run antes de qualquer efeito colateral externo. | Nenhum. |
| **D-13** | ~~Roteamento adaptativo só depois do harness de eval.~~ **[Δ 2026-08-09]** Revertida antecipadamente junto com D-8 — mesma decisão consciente, mesmo risco assumido. | Reversão consumida. |
| **D-14** | Memória append-only com proveniência; correção supersede. | Nenhum. |
| **D-15** | MCP do Brian = único path de escrita de workflow *por agentes*. Core é writer. | Nenhum. |
| **D-16** | **Zero token perdido.** Usage Control no minuto 0: ledger, janelas, %, restante, burn, assinatura, atribuição, otimização de centavo. | Nenhum. Lei do produto. |
| **D-17** | **Continuidade multi-LLM.** Continuity Pack + memória do Brian; handoff sem reexplicar. | Nenhum. Lei do produto (mínimo no v0.1). |

### Nota — reversão antecipada de D-8/D-13 (2026-08-09)

O autor pediu explicitamente, ciente do trade-off, que Router Fase 2/3 e Learning
Engine entrassem em código antes de existir `n≥30` runs reais por célula —
confirmado duas vezes após eu apontar que isso contraria a trava original.
Consequência prática: qualquer scoring histórico/aprendizado que existir no
código a partir daqui não tem base estatística real ainda. Tratar como
"aposta instrumentada", não como fato — o próprio blueprint (§11.1) chama isso
de "confiança injustificada" enquanto o volume de uso real não chega lá.

## Hipóteses (não são leis)

| ID | Hipótese | Critério |
|----|----------|----------|
| **H-1** | Context Governor reduz custo total por change bem-sucedido ≥30%. | Experimento A/B/C; senão remove o módulo. |

## Ordem sagrada de implementação

```text
D-16 (v0.0) → D-17 mínimo (v0.1) → run/worktree (v0.2) → resto
```

## Princípios operacionais (OP-1…OP-8) — ver `PREMISSAS-BASICAS.md`

| ID | Princípio |
|----|-----------|
| **OP-1** | Eficiência na utilização |
| **OP-2** | Tuning contínuo |
| **OP-3** | Tecnologia sempre atualizada |
| **OP-4** | Inteligência retroalimentada |
| **OP-5** | IA para resolver |
| **OP-6** | Direcionamento claro |
| **OP-7** | Trabalhos assertivos |
| **OP-8** | Poucas indas e vindas |

OpenSpec e código que violem OP-1…OP-8 ou D-16/D-17 são rejeitados por premissa.
