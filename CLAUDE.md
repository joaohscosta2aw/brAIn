# BrIAn

A verdade do projeto está em **[AGENTS.md](AGENTS.md)** — leia primeiro.
Este arquivo contém apenas o que é específico do Claude Code.

## Específico de Claude Code

- Hooks em `.claude/settings.json` atualizam o grafo estrutural após Edit/Write
  (inertes enquanto não houver código).
- Skills do grafo em `.claude/skills/` — úteis apenas quando houver código.
- Ponytail ativo: a solução mais simples que satisfaça a spec, nunca menos.
