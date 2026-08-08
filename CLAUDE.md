# BrIAn

A verdade do projeto está em **[AGENTS.md](AGENTS.md)** — leia primeiro.
Este arquivo contém apenas o que é específico do Claude Code.

## Específico de Claude Code

- **Hooks ativos.** `.claude/settings.json` executa `code-review-graph update`
  após cada Edit e Write, e `code-review-graph status` a cada início de sessão.
  Eles rodam hoje, inclusive sobre os `.md` que são o único conteúdo do
  repositório — a única guarda é a presença do binário e ser um repo git.
  Mantidos ligados porque passam a ter valor na primeira task de código, e
  desligar agora só criaria churn.
- Skills do grafo em `.claude/skills/` — sem alvo enquanto não houver código.
- Ponytail ativo: a solução mais simples que satisfaça a spec, nunca menos.
