## EXISTING ENGINEERING TOOLKIT

Existe um toolkit instalado para este projeto:

* Graphify — persistent repository knowledge graph;
* code-review-graph (tirth8205) — structural/change-impact graph and graph-assisted review;
* Archify (tt-a1i) — architecture visualization;
* shadcn/improve — read-only audit and implementation planning;
* open-code-review (Alibaba) — independent AI-assisted code review;
* Ponytail — anti-overengineering/minimal-implementation discipline;
* SkillSpector (NVIDIA) — agent skill/plugin security analysis;
* better-harness (QoderAI) — audits the AI agent workflow itself (task understanding, execution, validation, delivery, learning capture) across 5 dimensions. Use ONLY at the quality-gates / readiness-review phase (see PrimeirosPassos.md §21), never before the harness/specs it audits actually exist. Running it earlier audits an empty workflow and produces noise.

Do NOT blindly install, enable, or globally inject all of these tools.

Evaluate each tool against the architecture and workflow you are designing.

For every tool determine:

1. exact responsibility;
2. overlap with other tools;
3. lifecycle phase where it should be invoked;
4. whether invocation should be automatic, conditional, or manual;
5. context/token cost;
6. persistence/storage cost;
7. security implications;
8. hooks or instructions it injects;
9. whether it modifies CLAUDE.md or agent configuration;
10. whether its generated state should be committed or ignored;
11. how its state stays synchronized with the repository;
12. failure mode if unavailable.

Favor composition over accumulation.

Two tools that solve substantially the same problem should not both execute by default.

Prefer one primary tool and one specialized escalation path.

Before installing third-party skills/plugins/hooks:

* inspect their source;
* inspect requested permissions and shell commands;
* review install scripts;
* assess supply-chain risk;
* use available security scanning;
* present anything materially risky before granting broad privileges.

Integrate approved tooling into the progressive-disclosure context architecture.

The agent should know:

* WHAT tool exists;
* WHEN it should use it;
* WHEN it should not use it;
* WHAT authority its output has.

It should NOT carry the complete documentation of every tool in permanent context.

After evaluation, produce a TOOLING MATRIX containing:

| Tool | Purpose | Trigger | Default? | Context cost | Authority | Overlap | Security notes |

Then configure only the set whose benefits justify their complexity.
