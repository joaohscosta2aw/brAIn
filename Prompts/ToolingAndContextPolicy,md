# Tooling & Context Policy

Use external tooling according to task need, not mechanically.

## Repository understanding

Prefer the cheapest reliable source of context.

For structural questions involving symbols, dependencies, callers, imports, ownership, or cross-module relationships:

1. query the persistent repository graph first;
2. retrieve only the relevant source files needed to verify conclusions;
3. do not read broad portions of the repository when graph-assisted retrieval is sufficient.

Graph data is an index, not ground truth.

Source code remains authoritative.

If graph information conflicts with source code, verify against source code.

## Graph tools

### Graphify

Use Graphify as the primary persistent structural memory of the repository when available.

Prefer it for:

* repository orientation;
* locating relevant components;
* dependency discovery;
* caller/callee relationships;
* architecture exploration;
* reducing repeated repository scans.

Do not query Graphify for trivial tasks where the target files are already known.

### code-review-graph

Use code-review-graph when deeper change analysis is valuable, especially:

* code review;
* blast-radius analysis;
* cross-module changes;
* dependency-sensitive changes;
* identifying affected callers or consumers;
* reviewing non-local consequences of a diff.

Do not automatically run both graph systems for every task.

Start with the simplest sufficient source and escalate when additional structural evidence is valuable.

---

## Specification and planning

OpenSpec is the canonical source for approved product behavior.

Before substantial implementation:

* identify the applicable OpenSpec change/spec;
* understand acceptance criteria;
* identify unresolved decisions;
* distinguish product requirements from implementation choices.

Use `shadcn/improve` or equivalent advisory tooling when performing:

* broad repository audits;
* technical-debt assessment;
* improvement discovery;
* migration planning;
* major refactoring planning.

Its output is advisory.

Do not silently convert recommendations into requirements.

Changes to behavior belong in OpenSpec.

---

## Architecture visualization

Use architecture/diagram tooling only when visualization materially improves reasoning or communication.

Diagrams are derived views of the current architecture, not canonical architecture state unless explicitly designated otherwise.

Never maintain diagrams manually when they can be regenerated reliably.

---

## Implementation discipline

During implementation, follow this decision ladder:

1. Does this code need to exist?
2. Does the platform or standard library already solve it?
3. Does an existing project dependency already solve it?
4. Does an existing project abstraction already solve it cleanly?
5. Can the requirement be satisfied with a smaller implementation?
6. Only then introduce new abstraction or infrastructure.

Prefer the simplest implementation that fully satisfies the approved specification.

Do not optimize for minimum line count at the expense of:

* correctness;
* security;
* clarity;
* necessary validation;
* testability;
* specified behavior.

Use Ponytail or equivalent minimalism tooling as an implementation critic where useful.

It is a constraint against accidental complexity, not an authority over product requirements or architecture.

---

## Code review

The implementation agent should not be considered its own final reviewer.

For meaningful changes, review in layers:

1. deterministic checks;
2. tests;
3. static analysis;
4. OpenSpec compliance;
5. structural/blast-radius analysis when applicable;
6. independent AI review;
7. human review for high-risk changes.

Use open-code-review (Alibaba, `ocr`) for substantial diffs when available.

Use code-review-graph when understanding non-local consequences is important.

### Agent-workflow audit (better-harness)

better-harness audits the harness itself (task understanding, execution control, validation, delivery, learning capture), not the product code. Invoke it only at the quality-gates / readiness-review milestone of a specification or major implementation cycle — running it mid-task or before the harness exists produces noise, not signal. Its report is advisory, same status as an AI code review: confirmed / likely / needs investigation / false positive, never auto-applied.

Review findings are hypotheses until verified.

Do not automatically modify code solely because an AI reviewer reports an issue.

Classify findings as:

* confirmed;
* likely;
* needs investigation;
* false positive.

Fix confirmed problems and investigate materially important uncertain ones.

---

## Skill and plugin security

Treat third-party skills, plugins, hooks, MCP servers, and executable agent extensions as supply-chain dependencies.

Before adopting a new one:

1. inspect its source and permissions;
2. inspect install scripts and hooks;
3. determine filesystem/network/command access;
4. scan it using available security tooling such as SkillSpector;
5. consider additional malware/static scanning when executable content exists;
6. pin/version-lock important dependencies where practical;
7. prefer project-local installation when team reproducibility matters.

No single scanner is sufficient evidence that a skill is safe.

Do not install third-party agent extensions merely because they are popular.

---

## Context economy

Tool output must follow progressive disclosure.

Do not inject entire graph reports, audit reports, review reports, or documentation collections into context by default.

Retrieve:

task
→ relevant domain
→ relevant symbols
→ relevant files
→ deeper evidence only when needed.

Prefer references and targeted queries over repeated full-repository reads.

Persistent tooling should help reduce context consumption, not increase it.

---

## Authority hierarchy

When tools disagree:

1. approved OpenSpec behavior;
2. executable contracts and tests;
3. current source code;
4. architecture decisions;
5. repository graph/indexes;
6. generated diagrams;
7. AI audits/reviews;
8. agent assumptions.

Tools inform decisions.

They do not replace judgment.
