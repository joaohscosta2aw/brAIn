# Brian

Plano de controle de engenharia de IA (macOS → enterprise depois).

**Missão:** a IA **resolve** com direção clara, de forma assertiva e com poucas idas e vindas — poupando ~90% do trabalho/tempo e **sem** queimar dinheiro.

**Dois ganhos mínimos:**

1. **D-16** — zero token perdido (capacidade, janelas, %, atribuição)
2. **D-17** — memória/continuidade para chavear LLM sem perda de contexto

**Princípios operacionais (não podem escapar):**

```text
OP-1  Eficiência na utilização
OP-2  Tuning contínuo
OP-3  Tecnologia sempre atualizada
OP-4  Inteligência retroalimentada
OP-5  IA para resolver
OP-6  Direcionamento claro
OP-7  Trabalhos assertivos
OP-8  Poucas indas e vindas
```

## Documentos

| Arquivo | Uso |
|---------|-----|
| [`docs/PREMISSAS-BASICAS.md`](docs/PREMISSAS-BASICAS.md) | **Lei do produto** (missão + OP + pilares) — ler antes de OpenSpec / código |
| [`docs/DECISIONS.md`](docs/DECISIONS.md) | D-1 … D-17 + ponte para OP-1…OP-8 |
| [`BRIAN-BLUEPRINT-V1.md`](BRIAN-BLUEPRINT-V1.md) | Blueprint completo (canônico) |
| [`BRIAN-BLUEPRINT.md`](BRIAN-BLUEPRINT.md) | v0.1-draft histórico — **não** usar para implementar |

## Ordem de construção

```text
v0.0  capacity + ledger + costs          (D-16)
v0.1  context + identity + handoff       (D-17)
v0.2  run + worktree + workflow curto
…
```

## Próximo passo

1. Aceitar / ajustar `docs/PREMISSAS-BASICAS.md`
2. OpenSpec: primeiro change = **client-cost-attribution + capacity** (D-16)
3. Scaffold Rust + SQLite

Ainda **não** há código de produto neste repositório.
