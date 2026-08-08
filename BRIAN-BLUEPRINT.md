# Brian
## AI Engineering Control Plane for macOS and Enterprise Kubernetes

> **Status:** Product & Architecture Blueprint  
> **Version:** 0.1-draft  
> **Primary platform:** macOS  
> **Enterprise evolution:** Kubernetes  
> **Core idea:** a context-aware, multi-provider AI engineering control plane that orchestrates coding agents, specifications, memory, tools, security, observability and cost.

---

# 1. Executive Summary

Brian is a native macOS application that acts as an **AI Engineering Control Plane**.

It does not replace coding agents such as Claude Code, Codex CLI, Gemini CLI, Grok CLI, ZCode or future providers. Instead, Brian discovers and attaches to those tools, gives them the correct identity and project context, chooses which provider/model should work, controls workflow, supplies minimal relevant context, observes the execution and attributes usage/cost to the correct client or project.

The central product abstraction is **Context**.

A user can say:

```text
Brian, connect to XPTO.
```

Brian then activates a complete operational context:

- client identity;
- repository;
- project;
- OpenSpec;
- memory namespace;
- provider identities;
- GitHub organization;
- environments;
- MCP servers;
- tools;
- secrets;
- policies;
- budgets;
- telemetry;
- cost attribution.

The user can then ask:

```text
Is XPTO online?
What changed in checkout?
Why is production slow?
How much AI did XPTO consume this month?
Implement the current OpenSpec.
Review the current change.
```

Brian determines which deterministic tools and which AI workers should be used.

The fundamental separation is:

```text
OpenSpec    → defines WHAT should be built
Brian       → decides HOW work should happen
Providers   → perform cognitive/execution work
Tools       → provide capabilities
Gates       → verify quality/security
Memory      → preserves useful knowledge
Telemetry   → explains what happened
FinOps      → attributes tokens/cost
Vault       → protects identities and secrets
```

Brian is not another coding agent.

Brian is the environment in which coding agents work together.

---

# 2. Product Principles

## 2.1 Context over prompts

The unit of work is not the prompt.

The unit of work is the **Context + Run**.

A prompt is temporary. A context contains:

- who the user represents;
- which client is active;
- which project is active;
- which repositories are available;
- which provider identities are allowed;
- which secrets can be resolved;
- which memory can be retrieved;
- which policies apply;
- which budget owns the usage.

---

## 2.2 Providers are replaceable workers

Claude, Codex, Gemini, Grok and ZCode are not the Brain.

They are workers.

Brian must be able to:

- attach a provider;
- detect provider version;
- authenticate a provider profile;
- list models;
- run and resume sessions;
- collect usage;
- collect limits when available;
- switch models;
- switch providers;
- disable a provider;
- evaluate provider performance.

---

## 2.3 Deterministic before probabilistic

Before calling an LLM, Brian should prefer deterministic tools when they can answer the question.

Examples:

```text
"Who calls this function?"
→ code graph / AST / Sourcebot

"Are dependencies vulnerable?"
→ OSV Scanner

"Does this diff contain secrets?"
→ secret scanner

"Do tests pass?"
→ test runner

"Is the endpoint online?"
→ health check / observability

"What changed?"
→ Git diff
```

Only then should an LLM receive the relevant evidence and reason about it.

This reduces:

- tokens;
- latency;
- hallucination surface;
- unnecessary cost.

---

## 2.4 Minimal context by default

No provider should receive the entire repository or the entire conversation unless absolutely necessary.

Brian must implement **lazy context loading**.

The provider first receives references and only reads details when required.

Example:

```text
Relevant symbols:
- PaymentService.capture @ src/payment/PaymentService.ts:81-143
- CheckoutService.finalize @ src/checkout/CheckoutService.ts:201-248
```

Instead of automatically injecting hundreds of source files.

---

## 2.5 Explainable decisions

Brian should never feel like a black box.

For important decisions, the user should be able to ask:

```text
Why did Brian choose Codex?
Why did this run cost so much?
Why did the workflow return to correction?
Why does Brian believe refunds require idempotency?
```

Brian should answer using:

- routing signals;
- traces;
- memory provenance;
- tool results;
- OpenSpec requirements;
- provider history.

---

## 2.6 Local-first, enterprise-ready

Brian starts as a macOS application.

The core architecture must not depend on macOS.

Platform-specific capabilities are adapters.

```text
Brian Core
├── platform-agnostic
└── adapters/
    ├── macOS
    └── Kubernetes
```

The same logical system should evolve from:

```text
macOS local processes
```

to:

```text
Kubernetes Jobs / Pods
```

without rewriting the Brain.

---

# 3. High-Level Architecture

```text
                           BRIAN
                             │
           ┌─────────────────┼─────────────────┐
           │                 │                 │
        Context           Reasoning          Control
           │                 │                 │
      Projects           Classifier         Policies
      Identity           Planner            Budgets
      OpenSpec           Router             Vault
      Memory             Context Gov.       Security
      Environment        Impact Engine      Approval Gates
                         Evaluator
                         Replanner
           │                 │
           └─────────────────┼─────────────────┘
                             │
                       Provider Router
                             │
          ┌──────────┬───────┼────────┬──────────┐
          ▼          ▼       ▼        ▼          ▼
       Claude      Codex   Gemini    Grok      ZCode
          │          │       │        │          │
          └──────────┴───────┼────────┴──────────┘
                             │
                      Capability Layer
                             │
        ┌────────────┬───────┼────────┬─────────────┐
        ▼            ▼       ▼        ▼             ▼
      Git          Code    Tests    Browser       Xcode
                   Graph
                             │
                       Quality Gates
                             │
      ┌──────────────┬───────┼───────────┬────────────┐
      ▼              ▼       ▼           ▼            ▼
   Alibaba OCR    Semgrep    OSV      Secrets     SkillSpector
                             │
                        Telemetry
                             │
             traces / tokens / cost / outcomes
                             │
                          Learning
```

---

# 4. Major Subsystems

Brian is composed of the following major subsystems:

```text
Context Manager
Identity Manager
Brian Vault
Project Registry
Provider Registry
Provider Router
Model Router
OpenSpec Adapter
Workflow Engine
Reasoning Engine
Context Governor
Impact Engine
Memory Engine
Code Intelligence Layer
Capability / Tool Layer
Quality Gates
Policy Engine
Browser Bridge
Xcode Bridge
Telemetry
FinOps
Learning Engine
Brian Chat
Brian CLI
Brian.app UI
```

---

# 5. Context Manager

Context is the central runtime abstraction.

A context binds:

```text
Client
Project
Repository
Identity
OpenSpec
Memory
Providers
Tools
Secrets
Policies
Environment
Telemetry namespace
Budget
```

Example:

```yaml
context:
  id: xpto-checkout
  client: xpto
  project: checkout-api

repository:
  path: ~/Projects/XPTO/checkout-api

identity:
  profile: xpto-work

openspec:
  root: ./openspec

memory:
  namespace: client:xpto/project:checkout-api

providers:
  claude:
    identity: xpto-work
  codex:
    identity: xpto-work
  gemini:
    identity: xpto-work

github:
  organization: xpto-org

environments:
  staging: xpto-staging
  production: xpto-production

budget:
  monthly_tokens: 50000000
  equivalent_usd: 500
```

Commands:

```bash
brian connect xpto
brian connect xpto/checkout-api
brian disconnect
brian whoami
```

`brian connect` is effectively a semantic `cd` plus identity switch, memory switch, tool switch and accounting switch.

---

# 6. Identity Manager

Brian must support identities that are separate from the user’s normal desktop/terminal identities.

Example:

```text
Terminal
├── Claude → personal
├── Codex  → personal
└── Gemini → personal

Brian / Work
├── Claude → company
├── Codex  → company
└── Gemini → company
```

A provider executable may be shared while its configuration/authentication state is isolated.

Conceptually:

```text
/opt/homebrew/bin/codex
        │
        ├── Personal profile
        └── Brian / Company profile
```

Brian profiles may include:

```text
provider executable
config home
auth home
organization/workspace
preferred models
MCP configuration
environment variables
Git identity
GitHub identity
policy set
```

---

# 7. Brian Vault

Brian Vault is the credential abstraction.

On macOS, its primary backend is the macOS Keychain.

The system should also be designed for future enterprise backends:

```text
macOS Keychain
HashiCorp Vault
AWS Secrets Manager
Azure Key Vault
GCP Secret Manager
Kubernetes / External Secrets
1Password Connect
```

Brian's database stores references, never raw secrets.

Example:

```yaml
credential_ref: keychain://brian/xpto/codex/oauth
```

Secret classes:

```text
LOW
- read-only credentials

MEDIUM
- repository write

HIGH
- infrastructure access

CRITICAL
- production access
```

Policy examples:

```text
read repository        → automatic
git commit             → automatic
git push               → policy / optional Touch ID
staging deploy         → policy
production deploy      → Touch ID + explicit approval
```

The Vault should integrate with:

- Security.framework;
- LocalAuthentication;
- Touch ID;
- session-scoped credential resolution;
- expiration/rotation metadata.

---

# 8. Attached Providers

Brian does not need to own provider installation.

It attaches to existing CLIs.

Initial targets:

```text
Claude Code
Codex CLI
Gemini CLI
Grok CLI
ZCode
```

Example provider record:

```json
{
  "id": "codex",
  "type": "cli",
  "executable": "/opt/homebrew/bin/codex",
  "managed": false,
  "attached": true
}
```

The CLI remains usable outside Brian.

```text
Terminal ───────┐
                ├── same installed CLI
Brian ──────────┘
```

---

# 9. Provider Profiles

A physical provider can expose multiple logical profiles.

Example:

```text
Codex
├── Builder
├── Fixer
└── Quick

Claude
├── Planner
├── Architect
└── Reviewer
```

Example profile:

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

context_budget_tokens: 50000
```

---

# 10. Provider Interface

Brian Core should expose a normalized provider interface.

Conceptually:

```rust
trait Provider {
    fn status(&self) -> ProviderStatus;
    fn models(&self) -> Vec<Model>;
    fn authenticate(&self, identity: Identity) -> Result<()>;
    fn run(&self, request: RunRequest) -> Result<RunHandle>;
    fn resume(&self, run: RunHandle) -> Result<RunHandle>;
    fn cancel(&self, run: RunHandle) -> Result<()>;
    fn usage(&self) -> Result<Usage>;
    fn limits(&self) -> Result<ProviderLimits>;
}
```

Provider adapters:

```text
providers/
├── claude
├── codex
├── gemini
├── grok
├── zcode
└── future
```

---

# 11. Provider Router

The Provider Router chooses the best worker for a task.

Initial routing may be rule-based.

Long term, it should use measured outcomes.

Signals:

```text
task type
language/framework
complexity
risk
expected horizon
context requirement
historical success
average retries
latency
estimated cost
remaining quota
current availability
provider health
user policy
client policy
```

Conceptual scoring:

```text
score =
  quality_weight * historical_quality
+ fit_weight * task_affinity
+ quota_weight * quota_health
- cost_weight * expected_cost
- latency_weight * expected_latency
- retry_weight * retry_probability
```

Brian must log every routing decision for later inspection.

---

# 12. Model Router

Provider and model are separate decisions.

Brian should use semantic model pointers:

```text
reasoning
coding
quick
compact
review
long-context
research
```

Example:

```yaml
models:
  reasoning:
    provider: claude
    tier: strong

  coding:
    provider: codex
    tier: strong

  quick:
    provider: codex
    tier: cheap

  compact:
    provider: gemini
    tier: cheap

  long-context:
    provider: gemini
    tier: strong

  investigation:
    provider: grok
    tier: strong
```

Concrete model names are provider configuration, not workflow logic.

---

# 13. Provider Limits and Usage

Brian should normalize provider status into:

```json
{
  "provider": "claude",
  "authenticated": true,
  "available": true,
  "quota": {
    "remaining_percent": 32,
    "reset_at": null,
    "source": "provider"
  },
  "runtime": {
    "requests": 47
  },
  "health": {
    "rate_limited": false
  }
}
```

Three information levels:

```text
Provider-reported
Runtime-measured
Availability / rate-limit state
```

Brian must never invent quota percentages.

If exact quota is unavailable, display:

```text
Unknown
Estimated
Rate limited
Available
```

instead of false precision.

---

# 14. OpenSpec

OpenSpec defines intent.

Brian uses OpenSpec for:

```text
requirements
changes
acceptance criteria
decisions
tasks
validation
```

OpenSpec should remain independent from any single model.

Brian adds orchestration around it.

```text
OpenSpec Change
      ↓
Planner
      ↓
Workflow
      ↓
Provider
      ↓
Tests / Reviews
      ↓
OpenSpec Validation
```

---

# 15. Workflow Engine

Initial workflow:

```text
DISCOVERY
   ↓
PLANNING
   ↓
PLAN_REVIEW
   ↓
ARCHITECTURE
   ↓
IMPLEMENTATION
   ↓
TESTING
   ↓
CODE_REVIEW
   ↓
CORRECTION
   ↓
VALIDATION
   ↓
DONE
```

Possible alternate transitions:

```text
TESTING → CORRECTION
CODE_REVIEW → CORRECTION
CORRECTION → TESTING
VALIDATION → CORRECTION
REPEATED_FAILURE → REPLAN
```

The Workflow Engine, not the provider, owns state.

Agents submit results.

Brian validates transitions.

---

# 16. Reasoning Engine

Structure:

```text
reasoning/
├── classifier
├── planner
├── router
├── context
├── impact
├── evaluator
└── replanner
```

## Classifier

Determines:

```text
task type
risk
complexity
horizon
impact
required capabilities
```

## Planner

Turns OpenSpec into a structured executable plan.

Example:

```json
{
  "tasks": [
    {
      "id": "T1",
      "description": "Add idempotent refund handling",
      "acceptance": [
        "duplicate requests return the same result",
        "idempotency key is persisted"
      ]
    }
  ],
  "risks": [
    "concurrent requests"
  ]
}
```

## Replanner

After failures, Brian evaluates whether the plan remains valid.

```text
Implementation
    ↓
Test Failure
    ↓
Root Cause Analysis
    ↓
Plan still valid?
   / \
 yes  no
 ↓     ↓
fix   replan
```

---

# 17. Impact Engine

Impact Engine combines:

```text
OpenSpec
Code Graph
Git Diff
Memory
Architecture
Tests
```

Output:

```json
{
  "risk": "high",
  "affected_modules": 7,
  "critical_paths": [
    "checkout -> payment -> ledger"
  ],
  "recommended_reviews": [
    "concurrency",
    "database",
    "security"
  ]
}
```

Impact analysis should influence:

- provider choice;
- model strength;
- context budget;
- required tests;
- review gates;
- approval requirements.

---

# 18. Context Governor

Context Governor is one of Brian's most important subsystems.

Goal:

> Reduce the problem before sending it to an LLM.

Pipeline:

```text
Task
 ↓
OpenSpec filter
 ↓
Code graph query
 ↓
Symbol search
 ↓
Git diff
 ↓
Memory retrieval
 ↓
Deduplication
 ↓
Compression
 ↓
Token budget
 ↓
Minimal context package
```

Components:

```text
context/
├── retriever
├── graph-selector
├── symbol-selector
├── memory-selector
├── deduplicator
├── compressor
├── reference-store
└── budget-manager
```

---

# 19. Lazy Context Loading

Brian should use references whenever possible.

Instead of sending:

```text
full payment.ts
full checkout.ts
full ledger.ts
```

send:

```text
PaymentService.capture @ src/payment.ts:81-143
Checkout.finalize @ src/checkout.ts:201-248
Ledger.record @ src/ledger.ts:51-103
```

Agents can call:

```text
code.read_symbol
code.read_range
memory.get
trace.get
```

only when needed.

This reduces context size and improves attribution of what the agent actually used.

---

# 20. Context Budgets

Per-phase budgets:

```yaml
context:
  planning:
    max_tokens: 30000

  implementation:
    max_tokens: 60000

  review:
    max_tokens: 25000

  quick_fix:
    max_tokens: 12000
```

Policies may escalate:

```text
cheap scout
→ low confidence
→ strong model
```

rather than starting every task with the most expensive context/model combination.

---

# 21. Code Intelligence Layer

Initial tools:

```text
Graphify
code-review-graph
Sourcebot
ast-grep
Archify
```

Brian should expose normalized capabilities:

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
```

Providers should not need to know which implementation answered the query.

---

# 22. Graphify

Role:

```text
repository map
symbol graph
dependency relationships
impact discovery
context selection
```

Primary use cases:

```text
Which files matter?
What depends on this?
What will this change affect?
What code should enter the LLM context?
```

---

# 23. code-review-graph

Role:

```text
advanced graph queries
review-focused graph analysis
change relationships
impact analysis
```

It can complement or replace parts of Graphify depending on quality and maintenance.

Brian should treat both as interchangeable graph providers behind a `CodeGraph` interface.

---

# 24. Sourcebot

Role:

```text
cross-repository code search
definitions
references
search navigation
```

Useful for:

- large organizations;
- many repositories;
- retrieval before LLM calls.

---

# 25. ast-grep

Role:

```text
AST structural queries
semantic pattern detection
codemods
safe structured rewrites
```

Whenever possible, Brian should prefer AST operations over asking an LLM to perform broad textual search/replace.

---

# 26. Archify

Role:

```text
architecture representation
system diagrams
module relationships
visual explanation
```

Typical workflow:

```text
Code Graph
 ↓
Archify
 ↓
Architecture Context
 ↓
Planner / Reviewer
```

---

# 27. Improve

`shadcn/improve` is treated as an audit/planning capability.

Purpose:

```text
What should we improve?
```

Potential modes:

```text
quick
standard
deep
security
performance
tests
branch
next
plan
```

Brian can turn selected findings into OpenSpec changes.

```text
Improve finding
      ↓
Brain review
      ↓
OpenSpec proposal
```

---

# 28. Ponytail

Ponytail is treated as a restraint policy.

Purpose:

> Prevent over-engineering.

Pre-implementation questions:

```text
Does this need to exist?
Does the project already have something equivalent?
Can the standard library solve it?
Can an existing dependency solve it?
Can we touch fewer files?
Can the implementation be smaller?
```

Ponytail should operate before high-cost implementation phases.

---

# 29. Alibaba Open Code Review

Open Code Review / OCR is treated as a specialized quality engine.

Flow:

```text
Implementation
    ↓
Tests
    ↓
OCR
    ↓
Findings?
  /      \
yes      no
 ↓        ↓
Fix    semantic review / validation
```

Brian should normalize findings:

```json
{
  "severity": "high",
  "file": "src/payment.ts",
  "line": 81,
  "category": "concurrency",
  "message": "..."
}
```

---

# 30. Security Gates

Initial security stack:

```text
Semgrep
OSV Scanner
Secret Scanner
SkillSpector
```

Possible secret scanners:

```text
TruffleHog
GitGuardian
```

Brian should expose:

```text
security.sast
security.dependencies
security.secrets
security.skill_scan
```

---

# 31. SkillSpector

Every third-party skill/plugin should be considered potentially privileged code.

Flow:

```text
New Skill
   ↓
SkillSpector
   ↓
Static analysis
   ↓
Capability inspection
   ↓
Risk score
   ↓
Policy
├── Allow
├── Restricted
├── Sandbox
└── Block
```

Skill trust metadata:

```yaml
skill:
  name: example
  source: github
  trust: unverified
  risk: medium
  scanned_at: ...
```

---

# 32. Skill Model

Brian distinguishes:

```text
Skill = how an agent should perform a type of work
Tool  = capability the agent can invoke
Phase = what the workflow requires now
```

Example:

```yaml
skill:
  name: debug-failure

instructions:
  - read the complete error
  - identify root cause
  - inspect current diff
  - change only related files
  - rerun failing test
  - run regression suite

allowed_tools:
  - code.read
  - code.write
  - git.diff
  - tests.run
```

Skills can be composed:

```text
implement-feature
├── understand-spec
├── inspect-codebase
├── design-change
├── implement
├── test
└── self-review
```

---

# 33. MCP Layer

Brian can expose an MCP Server to agents.

Conceptual capabilities:

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

tests.run
lint.run

memory.search
memory.remember
memory.get_decisions

review.submit
```

Brian remains the authority over workflow state.

Agents must not directly edit the workflow database.

---

# 34. Multi-Provider Memory

Memory belongs to Brian.

Not to Claude.

Not to Codex.

Not to Gemini.

Types:

```text
Working Memory
Project Memory
Decision Memory
Architecture Memory
Episodic Memory
Incident Memory
Provider Performance Memory
```

Example:

```json
{
  "type": "decision",
  "client": "xpto",
  "project": "payments-api",
  "content": "Refunds must use idempotency keys.",
  "source": {
    "provider": "claude",
    "phase": "architecture"
  },
  "confidence": 0.94,
  "trace_id": "tr_82ac"
}
```

---

# 35. Memory Retrieval

Brian should not inject all memories into prompts.

Retrieval should rank by:

```text
tenant/client
project
change
file/symbol
phase
skill
similarity
recency
importance
confidence
evidence quality
```

Only a small number of memories should enter the context package.

---

# 36. Memory Governance

Agents can suggest memory.

Brian decides whether it becomes durable.

Example permissions:

```text
Builder
memory.read      ✓
memory.suggest   ✓
memory.commit    ✗

Reviewer
memory.read      ✓
memory.approve   ✓

Brain
memory.commit    ✓
```

Memory statuses:

```text
fact
decision
hypothesis
observation
incident
lesson
```

Hypotheses remain explicitly unverified.

---

# 37. Memory Isolation

Memory is isolated per client/project.

```text
XPTO
├── architecture
├── decisions
├── incidents
├── patterns
└── outcomes

ACME
└── ...

Internal
└── ...
```

Cross-client retrieval is denied by default.

---

# 38. Learning Engine

Brian should learn from outcomes.

Initial implementation can be simple statistics.

Example:

```text
Task type: database migration

Codex
success: 96%
avg retries: 0.8
avg equivalent cost: $0.71

Claude
success: 91%
avg retries: 1.1
avg equivalent cost: $1.22
```

The router uses this historical evidence.

No complex ML is required for v0.x.

---

# 39. Telemetry

OpenTelemetry is the preferred internal standard.

Every execution produces:

```text
Run
└── Trace
    ├── context.build
    ├── memory.search
    ├── graph.query
    ├── provider.plan
    ├── provider.implement
    ├── tests.run
    ├── review.ocr
    ├── provider.fix
    └── judge
```

Span attributes:

```text
client_id
project_id
change_id
run_id
phase
provider
model
skill
tokens
cost
latency
retry
tool
status
```

---

# 40. Token Accounting

Brian records, when available:

```text
input_tokens
cached_input_tokens
output_tokens
reasoning_tokens
total_tokens
```

Each measurement has a source:

```text
provider
runtime
estimated
```

Brian must never present estimated numbers as provider-certified usage.

---

# 41. Price Catalog

Brian maintains a versioned price catalog.

Example schema:

```text
provider
model
effective_from
effective_to
input_per_million
cached_input_per_million
output_per_million
other_tool_costs
source
```

The catalog allows historical cost calculations to remain stable when provider pricing changes.

---

# 42. Cost Model

Each run records:

```text
measured_tokens
estimated_api_cost
actual_billing_cost
billing_mode
```

Billing modes:

```text
api
subscription
credits
unknown
```

Important distinction:

```text
Equivalent API cost != actual invoice
```

when providers are authenticated through subscriptions or bundled plans.

---

# 43. FinOps

Every unit of AI usage should be attributable to:

```text
client
project
OpenSpec change
run
phase
provider
model
skill
```

Example:

```text
XPTO / antifraud-v2

Planning            Claude      $1.20 equiv.
Implementation      Codex       $4.82 equiv.
Investigation       Grok        $0.76 equiv.
OCR review          Review      $0.41 equiv.
Corrections         Codex       $0.93 equiv.
-------------------------------------------
Total                           $8.12 equiv.
```

---

# 44. Client Showback and Chargeback

Brian supports:

```text
Showback
→ report what a client consumed

Chargeback
→ calculate what should be billed
```

Example:

```text
AI API equivalent      $124
Brian infrastructure     $18
--------------------------------
Internal cost           $142

Commercial pricing      $200
```

Actual billing policy belongs to the organization, not the runtime.

---

# 45. Budgets

Budgets can exist at:

```text
organization
client
project
change
run
phase
provider
```

Example:

```yaml
clients:
  xpto:
    budgets:
      monthly_tokens: 50000000
      monthly_usd_equivalent: 500
      change_soft_limit_usd: 10
      change_hard_limit_usd: 20
```

Soft limit behavior:

```text
prefer cheaper model
reduce context
avoid deep audit
reserve premium models
```

Hard limit behavior:

```text
stop new LLM calls
record trace
request policy override
```

---

# 46. Browser Bridge

Brian should integrate with the user's real browser rather than relying on browser automation for normal browsing.

Primary approach:

```text
Safari / Chrome
      │
 Brian Extension
      │
      ▼
   Brian.app
```

User explicitly attaches a tab to a context.

Possible data, subject to permission:

```text
URL
page title
selected text
DOM snapshot
accessibility tree
screenshot
console
network diagnostics
```

Brian should not silently monitor all browsing.

---

# 47. Browser Automation Policy

For internal testing:

```text
local / staging
→ Playwright is acceptable
```

For ordinary real browser use:

```text
real browser + Brian Browser Bridge
```

Brian should not include anti-detection or anti-bot bypass logic.

---

# 48. Xcode Integration

Xcode is treated as a developer tool.

Conceptually:

```text
Brian
  ↓
Provider
  ↓
Xcode bridge / MCP
  ↓
Build
Test
Diagnostics
Project operations
```

Use cases:

```text
implement SwiftUI view
build project
run tests
collect compiler diagnostics
review generated UI
```

Xcode integration should be an adapter, not hard-coded into the Brain.

---

# 49. Brian Chat

Brian Chat is a natural-language interface to the control plane.

It is not the control plane itself.

Example questions:

```text
Is XPTO online?
What was the latest deploy?
Why is checkout slow?
Which PRs are blocked?
How much AI did XPTO consume this month?
What does the current OpenSpec require?
Why did Brian choose Codex?
```

Example actions:

```text
Create an OpenSpec for this incident.
Run the current change.
Review this diff.
Switch to ACME.
Pause the current run.
```

---

# 50. Intent Routing in Chat

Example:

```text
"Is XPTO online?"
   ↓
Context Manager
   ↓
observability / health checks

"Who calls this method?"
   ↓
code graph / AST

"Why did the deployment fail?"
   ↓
CI + logs + recent diff + LLM reasoning

"How much did XPTO cost?"
   ↓
FinOps store
```

Brian should avoid invoking an LLM when direct structured queries are sufficient.

---

# 51. macOS Application Architecture

Recommended stack:

```text
UI                  Swift + SwiftUI
macOS Integration   Swift
Core Runtime        Rust
IPC                 XPC / local IPC boundary
Database            SurrealDB Embedded candidate
Secrets             macOS Keychain
Authentication      LocalAuthentication / Touch ID
Browser Extension   TypeScript / WebExtension
Telemetry           OpenTelemetry
CLI                 Rust
Specifications      OpenSpec
```

---

# 52. Xcode Workspace

The product can be developed from one repository/workspace.

```text
Brian/
├── Brian.xcworkspace
│
├── macos/
│   ├── Brian.xcodeproj
│   ├── BrianApp/
│   ├── BrianXPC/
│   └── BrianSafariExtension/
│
├── core/
│   ├── Cargo.toml
│   └── src/
│
├── cli/
│   ├── Cargo.toml
│   └── src/
│
├── browser/
│   ├── package.json
│   └── src/
│
├── openspec/
├── skills/
├── docs/
└── schemas/
```

Xcode acts as the macOS product build orchestrator.

Cargo remains the Rust build system.

npm/pnpm remains the WebExtension build system.

---

# 53. Brian Core Layout

Suggested Rust structure:

```text
core/src/
├── context/
├── identity/
├── providers/
├── models/
├── openspec/
├── workflow/
├── reasoning/
├── memory/
├── code_intelligence/
├── tools/
├── quality/
├── policy/
├── vault/
├── telemetry/
├── finops/
├── storage/
├── runtime/
└── platform/
```

---

# 54. Platform Adapters

Brian Core must remain platform-agnostic.

```text
platform/
├── macos/
│   ├── local_process
│   ├── keychain
│   ├── touch_id
│   ├── launchd
│   └── filesystem
│
└── kubernetes/
    ├── jobs
    ├── pods
    ├── workload_identity
    ├── secrets
    ├── pvc
    └── object_storage
```

---

# 55. Process Architecture on macOS

```text
Brian.app
    │
    │ IPC
    ▼
brian-core
    │
    ├── provider processes / PTYs
    ├── storage
    ├── workflow
    ├── telemetry
    └── tools
```

`brian-core` can run as a helper/background service so long-running tasks do not depend on the main window being open.

---

# 56. PTY / Session Manager

For CLI providers Brian may maintain real PTY-backed sessions.

```text
Session Manager
├── claude:session-38
├── codex:session-72
├── gemini:session-11
└── grok:session-04
```

The user can reconnect to a live session from the UI.

Brian should collect:

```text
stdout
stderr
session status
tool invocations when available
usage
exit state
```

without breaking the native CLI behavior.

---

# 57. Storage Strategy

Brian needs multiple storage classes.

Do not force all data into one database.

Recommended separation:

```text
Primary operational DB
→ SurrealDB Embedded candidate

Secrets
→ macOS Keychain

Large artifacts
→ filesystem

Telemetry transport
→ OpenTelemetry

Large raw traces
→ compressed files / future object storage

Code indexes
→ specialized code intelligence tools
```

---

# 58. Why SurrealDB Is a Strong Candidate

SurrealDB is attractive because Brian needs:

```text
document-like records
relationships
graph-like queries
memory links
run relationships
provider relationships
local embedded execution
future remote/enterprise operation
```

Potential graph:

```text
Client ──HAS_PROJECT──► Project
Project ──HAS_RUN────► Run
Run ──USED_PROVIDER──► Provider
Run ──GENERATED──────► Memory
Change ──AFFECTS─────► Symbol
Symbol ──CALLS───────► Symbol
Memory ──SUPPORTED_BY► Trace
```

This is a more natural model for Brian than a purely relational design.

---

# 59. Storage Abstraction

Brian must not depend directly on SurrealDB APIs throughout the core.

Define interfaces:

```text
ContextStore
ProjectStore
MemoryStore
RunStore
UsageStore
GraphStore
PolicyStore
```

Potential implementations:

```text
SurrealStore
SQLiteStore
PostgresStore
```

This makes storage replaceable.

---

# 60. SQLite Alternative

SQLite remains a valid fallback if simplicity wins.

Potential local stack:

```text
SQLite
├── operational data
├── FTS5
└── usage/finops

Filesystem
├── artifacts
└── raw traces

Keychain
└── secrets
```

Brian should perform a technical spike comparing SQLite and SurrealDB before final commitment.

---

# 61. Suggested Storage Spike

Test both candidates with real Brian workloads:

## Test A — Memory relationships

```text
project
→ decision
→ source trace
→ provider
→ affected symbols
```

## Test B — FinOps

Queries:

```text
cost by client
cost by provider
tokens by phase
cost per successful change
```

## Test C — Runs and workflow

```text
active runs
state transitions
retries
provider outcomes
```

Measure:

```text
developer complexity
query clarity
performance
migration strategy
backup/recovery
enterprise path
```

---

# 62. Visual Design Philosophy

Brian should feel:

```text
calm
precise
trustworthy
native
```

Brian should not look like:

```text
AI neon
cyberpunk
dashboard wall
provider launcher
card soup
```

Avoid:

- decorative gradients;
- AI glows;
- unnecessary animation;
- nested cards;
- excessive dashboards;
- provider branding dominating UI.

---

# 63. Information Architecture

Four main mental groups:

```text
WORK
├── Projects
├── OpenSpec
└── Runs

UNDERSTAND
├── Brain Inspector
├── Memory
└── Graph

OBSERVE
├── Traces
├── Usage
└── Cost

CONTROL
├── Context
├── Identity
├── Vault
└── Policies
```

The visible sidebar can remain smaller through progressive disclosure.

---

# 64. Main Navigation

Suggested visible navigation:

```text
Today
Projects
Runs
Explore
Usage

────────

Vault
Settings
```

Inside Explore:

```text
OpenSpec
Memory
Graph
Providers
Skills
```

---

# 65. Dashboard

The Dashboard should show only:

```text
Active client / project
Current objective
Current phase
Current worker
Next phase
Token usage
Equivalent cost
Elapsed time
Anything requiring attention
```

Example:

```text
XPTO / Checkout

Implementing refund idempotency

● Codex is working
████████████████░░ 73%

Next
Tests → Review

48k tokens · $0.38 equiv. · 01:42

[Inspect]
```

---

# 66. Brain Inspector

Purpose:

> Explain system decisions without exposing hidden model chain-of-thought.

Example:

```text
Why Codex?

Best fit for backend implementation
94% historical success
Expected cost below task budget
78% quota available

Alternatives:
Claude    0.82
Gemini    0.76
Grok      0.61
```

Show evidence and routing factors, not private reasoning traces.

---

# 67. Providers UI

Providers appear as infrastructure, not product brands.

Example:

```text
Codex
● Ready

Identity
Work / XPTO

Role
Primary Builder

Today
812k tokens

This month
9.8M tokens

Historical success
94%
```

---

# 68. OpenSpec UI

Represent changes as a workflow.

```text
refund-idempotency

PROPOSED
   ↓
PLANNED
   ↓
IMPLEMENTING  ← current
   ↓
TESTING
   ↓
REVIEW
   ↓
VALIDATED
```

Acceptance criteria:

```text
✓ Duplicate requests return same result
✓ Idempotency key persisted
○ Concurrent request test
○ OpenSpec validation
```

---

# 69. Trace UI

Visual timeline:

```text
Context   ███
Claude       █████
Graph             ██
Codex                █████████████████
Tests                                    ███
OCR                                        █████
```

Clicking a span shows:

```text
provider
model
tokens
cost
duration
tools
status
retry count
inputs/outputs references
```

---

# 70. Memory UI

Memory should look like knowledge, not a vector database.

Categories:

```text
Architecture
Decisions
Incidents
Patterns
Learned Outcomes
```

Memory item:

```text
Refund operations must use idempotency keys.

Type: Architecture Decision
Source: OpenSpec payment-v2
Trace: run #1421
Used by Brian: 7 times
Confidence: High
```

---

# 71. Code Graph UI

Graph UI is used only where it provides real value.

Example:

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

Node inspector:

```text
incoming references
outgoing references
risk
changed in current run
tests
recent incidents
```

---

# 72. Vault UI

Example:

```text
Brian Vault · XPTO

Providers
Claude Code      ● Stored
Codex            ● Stored
Gemini           ● Stored

Services
GitHub           ● Stored
AWS Staging      ● Stored
AWS Production   🔒 Touch ID
```

Never reveal credential values in the UI.

---

# 73. FinOps UI

Example:

```text
August

CLIENT       TOKENS      API EQUIV.
XPTO         18.4M       $124.30
ACME          7.2M        $51.80
Internal      3.1M        $18.20
```

Drilldown:

```text
XPTO

By Provider
Codex     53%
Claude    23%
Gemini    17%
Grok       7%

By Work
Implementation 58%
Planning       14%
Review         18%
Debug          10%
```

---

# 74. Chat UI

Chat should remain visually simple.

```text
Brian · XPTO / Production

You:
Is the client online?

Brian:
● Yes. Primary services are healthy.

API       ● Healthy
Web       ● Healthy
Database  ● Healthy

Last deploy: 23:14

[Details] [Last deploy]
```

Chat must inherit the active Context.

---

# 75. Command Palette

`⌘K` should be a first-class interface.

Examples:

```text
Connect XPTO
Connect ACME
Run current OpenSpec
Review current diff
Search memory
Show current trace
Switch provider
Lock Vault
Pause current run
```

---

# 76. Menu Bar

Example:

```text
🧠 Brian

XPTO / checkout-api
● Brain active

Codex · implementation
48k tokens · $0.31

Claude    ●
Codex     ●
Gemini    ●
Grok      ●

──────────────
Open Brian
Pause Run
Switch Context >
Disconnect XPTO
```

---

# 77. Autonomy Modes

Brian should support explicit autonomy levels.

```text
Manual
→ Brian recommends; user executes.

Supervised
→ Brian executes but asks at critical gates.

Autonomous
→ Brian routes, executes, tests, reviews and fixes automatically within policy.
```

Example operation policy:

```text
code edit            auto
tests                auto
git commit           auto
git push             ask
staging deploy       ask
production deploy    always ask + policy
```

---

# 78. Auditability

Every important action should be attributable.

Questions Brian must answer:

```text
Who changed this file?
Which provider made the change?
Which model?
Which OpenSpec requirement caused it?
Which memory was used?
Which tools were called?
How many tokens were consumed?
What did it cost?
Which review accepted it?
```

---

# 79. Brian CLI

Initial CLI:

```bash
brian status
brian connect <context>
brian disconnect
brian whoami

brian providers
brian providers status
brian providers usage

brian run "<task>"
brian pause
brian cancel

brian usage
brian costs
brian trace current
brian trace <run-id>

brian memory search "<query>"
brian vault status
```

---

# 80. Example End-to-End Run

User:

```text
Brian, connect to XPTO and implement the current change.
```

Execution:

```text
1. Context Manager activates XPTO
2. Identity Manager activates XPTO provider identities
3. Vault resolves required credentials
4. OpenSpec loads current change
5. Impact Engine analyzes affected code
6. Context Governor builds minimal context
7. Planner creates execution plan
8. Ponytail applies complexity restraint
9. Router selects provider/model
10. Provider implements
11. Tests run
12. OCR reviews diff
13. Semgrep / OSV / secret scans execute
14. Judge evaluates compliance
15. Corrections run if necessary
16. OpenSpec validation runs
17. Workflow moves to DONE
18. Memory stores approved outcomes
19. Telemetry closes trace
20. FinOps attributes tokens/cost to XPTO
```

---

# 81. Brian v0.1 — MVP

The MVP must remain intentionally small.

## macOS app

Screens:

```text
Projects
Chat
Current Run
Providers
Usage
Settings
```

## Core capabilities

```text
Context Manager
Identity Manager
Brian Vault
Provider Registry
Claude Code adapter
Codex adapter
OpenSpec reader
simple Workflow Engine
basic Provider Router
Context Governor v1
Telemetry
Token accounting
Equivalent cost
```

## Storage

Perform SQLite vs SurrealDB spike.

Choose one after measurement.

---

# 82. Brian v0.2

Add:

```text
Gemini CLI
Grok CLI
multi-provider memory
Graphify / CodeGraph interface
ast-grep
Alibaba OCR
Browser Bridge
basic Xcode integration
```

---

# 83. Brian v0.3

Add:

```text
ZCode
Improve
Ponytail
Semgrep
OSV
Secret Scanner
SkillSpector
Brain Inspector
Budgets
Adaptive routing
Provider performance memory
```

---

# 84. Brian v0.4

Add:

```text
Sourcebot
Archify
advanced graph review
multi-repository context
better semantic memory
provider experimentation
advanced FinOps
```

---

# 85. Brian 1.0 Success Criteria

Brian should successfully execute:

```text
Brian, connect to XPTO and implement the current OpenSpec.
```

with:

```text
context isolation
corporate provider identity
OpenSpec loading
minimal code context
provider routing
implementation
tests
code review
security gates
correction loop
validation
trace
memory
token accounting
client cost attribution
```

while requiring human approval only where policy requires it.

---

# 86. Enterprise Evolution

Brian Enterprise moves execution from local processes to Kubernetes.

Local:

```text
Brian.app
   ↓
brian-core
   ↓
local provider processes
```

Enterprise:

```text
Brian.app
   ↓
Brian Enterprise API
   ↓
Kubernetes
   ↓
provider workers
```

---

# 87. Enterprise Architecture

```text
                 Brian Enterprise
                        │
                 Control Plane API
                        │
        ┌───────────────┼────────────────┐
        ▼               ▼                ▼
   Context Service  Workflow Engine   Identity/Vault
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

---

# 88. Kubernetes Runtime

Enterprise provider workers should normally run as disposable Jobs.

Example:

```text
Workflow needs implementation
       ↓
Router selects Codex
       ↓
Kubernetes Job created
       ↓
Job receives:
- run_id
- client_id
- project_id
- model
- skill
- context reference
- budget
- workspace
       ↓
worker executes
       ↓
results recorded
       ↓
pod terminates
```

---

# 89. Enterprise Platform Mapping

```text
Local Brian                Enterprise Brian
------------------------------------------------
macOS Keychain             Vault / Secret Manager
Touch ID                   Enterprise approval / identity
Local process              Kubernetes Job
Filesystem                 PVC / object storage
Local DB                   Remote DB
launchd                    Deployment
Local OTel                 OTel Collector
Local OAuth profile        Workload identity
```

---

# 90. Enterprise Storage

Potential stack:

```text
Operational DB
→ PostgreSQL or SurrealDB service/cluster

Memory / vectors
→ dedicated vector backend if required

Artifacts
→ S3-compatible object storage

Telemetry
→ OpenTelemetry Collector

Traces
→ Tempo / Jaeger-compatible backend

Metrics
→ Prometheus-compatible backend

Secrets
→ Vault / cloud secret manager
```

The domain interfaces remain the same as local mode.

---

# 91. Hybrid Execution

Brian may eventually support:

```text
Local
Enterprise
Hybrid
```

Examples:

```text
quick local task
→ Mac

large refactor
→ Kubernetes

production review
→ Enterprise

offline work
→ Mac
```

Routing can include execution location as another decision dimension.

---

# 92. Multi-Tenant Enterprise

Client context remains the central tenancy boundary.

```text
XPTO
├── identity
├── repositories
├── memory
├── secrets
├── providers
├── budgets
├── traces
└── policies
```

Isolation options:

```text
logical tenant_id
dedicated namespace
dedicated workload identity
dedicated storage
```

chosen according to enterprise requirements.

---

# 93. Security Principles

1. Secrets never live in plain configuration files.
2. Cross-client memory retrieval is denied by default.
3. Provider identities are explicit.
4. Tools have capability-based permissions.
5. Workflow state is controlled by Brian, not agents.
6. High-risk operations require policy approval.
7. Skills/plugins are scanned before trust.
8. Every privileged operation is traced.
9. Production credentials should be short-lived whenever possible.
10. Enterprise workers should use workload identity instead of static long-lived secrets.

---

# 94. Non-Goals for Early Versions

Brian v0.x should not try to:

```text
replace Xcode
replace VS Code
replace every provider UI
build its own foundation model
implement a full cloud control plane
be a universal monitoring platform
be a full billing system
replace GitHub
replace Kubernetes
```

Brian orchestrates existing systems.

---

# 95. Key Product Differentiators

## Context Boundary

One command activates the complete client/project environment.

## Identity Boundary

Desktop can remain personal while Brian uses company identities.

## Multi-Provider Brain

Providers become interchangeable cognitive workers.

## Context Governor

Brian actively reduces token consumption by minimizing context.

## Multi-Provider Memory

Knowledge survives provider changes.

## Explainability

Routes, cost and workflow decisions remain inspectable.

## FinOps

Tokens and equivalent cost are automatically attributed to clients/projects.

## Local-to-Enterprise Path

The same core can evolve from macOS to Kubernetes.

---

# 96. Product Vocabulary

Use consistent vocabulary.

```text
Brian
→ the product/control plane

Brain Core
→ platform-agnostic runtime

Context
→ active client/project operational boundary

Provider
→ Claude/Codex/Gemini/etc.

Provider Profile
→ role/configuration using a provider

Model Pointer
→ semantic model role (coding, quick, reasoning...)

Run
→ one orchestrated execution

Phase
→ workflow state

Skill
→ reusable behavior/instructions

Tool
→ executable capability

Memory
→ durable Brian-managed knowledge

Trace
→ observability record

Vault
→ credential abstraction

Usage
→ token/request consumption

Equivalent Cost
→ estimated API-equivalent cost

Actual Billing Cost
→ provider-reported actual charge, when available
```

---

# 97. Suggested Repository Layout

```text
brian/
├── Brian.xcworkspace
│
├── macos/
│   ├── Brian.xcodeproj
│   ├── BrianApp/
│   ├── BrianXPC/
│   ├── BrianSafariExtension/
│   └── BrianTests/
│
├── core/
│   ├── Cargo.toml
│   └── src/
│       ├── context/
│       ├── identity/
│       ├── providers/
│       ├── openspec/
│       ├── workflow/
│       ├── reasoning/
│       ├── memory/
│       ├── code_intelligence/
│       ├── tools/
│       ├── quality/
│       ├── policy/
│       ├── telemetry/
│       ├── finops/
│       ├── storage/
│       ├── runtime/
│       └── platform/
│
├── cli/
│   ├── Cargo.toml
│   └── src/
│
├── browser/
│   ├── package.json
│   └── src/
│
├── skills/
├── openspec/
├── schemas/
├── docs/
│   ├── BRIAN-BLUEPRINT.md
│   ├── ARCHITECTURE.md
│   ├── PRODUCT.md
│   ├── SECURITY.md
│   └── ENTERPRISE.md
│
└── README.md
```

---

# 98. First Technical Milestone

Build a vertical slice before expanding the ecosystem.

Goal:

```text
Brian.app
  ↓
connect XPTO
  ↓
use XPTO Codex identity
  ↓
load OpenSpec
  ↓
run one Codex task
  ↓
collect trace + tokens
  ↓
attribute usage to XPTO
```

This proves:

```text
Context
Identity
Vault
Provider attachment
OpenSpec
Run
Telemetry
FinOps
```

which are the foundations of the product.

---

# 99. Second Technical Milestone

Add two-provider orchestration.

```text
Claude
→ plan/review

Codex
→ implementation/fix
```

Flow:

```text
OpenSpec
 ↓
Claude Plan
 ↓
Codex Build
 ↓
Tests
 ↓
Claude Review
 ↓
Done
```

Measure:

```text
tokens
latency
cost
retries
context size
quality outcome
```

---

# 100. Third Technical Milestone

Add Context Governor + Code Intelligence.

Goal:

```text
Graph / AST / Git
      ↓
minimal relevant context
      ↓
providers
```

Compare token consumption before/after.

This milestone validates one of Brian's most important economic hypotheses.

---

# 101. North-Star Metrics

Technical:

```text
successful runs
success on first attempt
average retries
average context tokens
total tokens per successful change
latency per phase
provider availability
```

Economic:

```text
equivalent cost per change
equivalent cost per successful change
cost by client
cost by project
cost by phase
cost saved through context reduction
```

Quality:

```text
review findings
post-merge regressions
security findings
test pass rate
OpenSpec compliance
```

Product:

```text
time from connect → productive state
number of manual context switches avoided
percentage of runs with correct attribution
number of user approvals required
```

---

# 102. Core Architectural Rules

These rules should be treated as non-negotiable until evidence proves otherwise.

1. **Brain Core is platform-agnostic.**
2. **macOS and Kubernetes are runtimes, not the Brain itself.**
3. **Context is the primary isolation and attribution boundary.**
4. **Providers are replaceable workers.**
5. **Provider auth is separate from Brian/MCP authorization.**
6. **Secrets are stored in Vault backends, not project files.**
7. **Workflow transitions belong to Brain.**
8. **LLM context is minimized and loaded lazily.**
9. **Memory is Brian-owned and provider-agnostic.**
10. **Every LLM call is attributable to client/project/run.**
11. **Equivalent cost and actual billing are separate concepts.**
12. **Deterministic tools are preferred before LLM reasoning.**
13. **Quality and security gates are first-class workflow stages.**
14. **Provider decisions are explainable.**
15. **Storage is behind interfaces and replaceable.**

---

# 103. Brian in One Diagram

```text
                       ┌──────────────────────┐
                       │      Brian.app       │
                       │   SwiftUI / macOS    │
                       └──────────┬───────────┘
                                  │
                                  ▼
                       ┌──────────────────────┐
                       │     Brian Core       │
                       │        Rust          │
                       └──────────┬───────────┘
                                  │
           ┌──────────────────────┼──────────────────────┐
           │                      │                      │
           ▼                      ▼                      ▼
       Context                 Reasoning               Control
  identity/project       planner/router/judge    policy/vault/budget
           │                      │                      │
           └──────────────────────┼──────────────────────┘
                                  ▼
                          Provider Router
                                  │
              ┌──────────┬────────┼────────┬──────────┐
              ▼          ▼        ▼        ▼          ▼
           Claude      Codex    Gemini    Grok      ZCode
              │          │        │        │          │
              └──────────┴────────┼────────┴──────────┘
                                  ▼
                              Tools/MCP
                                  │
           ┌─────────────┬────────┼────────┬────────────┐
           ▼             ▼        ▼        ▼            ▼
        OpenSpec       Code      Git     Browser       Xcode
                      Graph
                                  │
                                  ▼
                            Quality Gates
                                  │
                         OCR / Security / Tests
                                  │
                                  ▼
                             Telemetry
                                  │
                      traces/tokens/cost/outcome
                                  │
                                  ▼
                               Memory
                                  │
                                  ▼
                               Learning
```

---

# 104. Final Definition

**Brian is a local-first AI Engineering Control Plane that turns projects, client identities, specifications, coding agents, memory, tools, security, observability and cost into one coherent operational context.**

On macOS, Brian is the native interface and local runtime.

In enterprise environments, the same Brain evolves into a Kubernetes-based control plane with disposable agent workers, centralized identity, shared memory, enterprise secrets and organization-wide observability.

The user should not have to think:

```text
Which terminal?
Which account?
Which provider?
Which repository?
Which memory?
Which token belongs to which client?
Which tool should I open?
```

The user should be able to say:

```text
Brian, connect to XPTO.
```

and work from there.

---

# 105. Product Tagline Candidates

```text
Brian — Your AI Engineering Control Plane.

Brian — One context. Every coding agent.

Brian — The brain behind your coding agents.

Brian — Engineering context, orchestrated.

Brian — Connect the project. Brian handles the rest.
```

---

# 106. Immediate Next Steps

1. Create the repository.
2. Add this blueprint as `docs/BRIAN-BLUEPRINT.md`.
3. Create the first OpenSpec for `Context Manager`.
4. Create the second OpenSpec for `Provider Attachment`.
5. Build a storage spike: SQLite vs SurrealDB.
6. Prototype `brian-core` in Rust.
7. Prototype `Brian.app` shell in SwiftUI.
8. Implement Keychain-backed `Brian Vault`.
9. Attach Codex as the first provider.
10. Add Claude Code as the second provider.
11. Record the first end-to-end trace.
12. Attribute the first token usage to a client.
13. Build the minimal Chat + Current Run UI.
14. Measure context size and token consumption.
15. Only then add graph, memory and quality extensions.

---

# 107. First OpenSpec Candidate

```text
Change:
context-aware-provider-execution

Goal:
Allow Brian to connect to a client/project context and execute
an attached provider using an isolated Brian identity while
recording the run, trace and token attribution.

Acceptance Criteria:

- User can create a client.
- User can create a project under that client.
- User can connect to the project.
- Brian exposes the active context.
- Brian can attach a Codex CLI executable.
- Brian can associate a work identity with Codex.
- Brian can run Codex from the active context.
- Brian records run status.
- Brian records available token usage.
- Brian attributes usage to the active client/project.
- Secrets are not stored in the operational database.
- Disconnect clears the active client context.
```

---

# 108. Closing Principle

Brian becomes valuable when it makes a complex multi-agent engineering environment feel simple.

Internally:

```text
providers
models
OpenSpec
memory
graphs
MCP
security
traces
tokens
budgets
identities
workflows
```

Externally:

```text
Brian, connect to XPTO.

Brian, what is happening?

Brian, fix it.
```
