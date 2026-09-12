# Verified Modern Stack Capabilities

A versioned engineering skill for **correcting stale AI-model priors** in fast-moving software stacks.

The repository began as lessons extracted from Astra Analytics Bot, but its primary purpose is broader: give coding agents current, verified capability information when their training data strongly reflects older APIs or behavior.

The first major target is **SurrealDB 3.2.4 + Rust**, where models frequently generate SurrealDB 1.x/2.x patterns that are subtly or completely wrong for 3.x.

## What this repository is

Think of it as a small retrieval-time compatibility layer:

```text
stale model prior
      ↓
versioned correction
      ↓
official API evidence
      ↓
real compile/runtime test
      ↓
agent receives current capability
```

It is deliberately not a replacement for official documentation. The source-of-truth order is:

1. Current target repository / lockfile
2. Official documentation for the target version
3. Reproducible compile or live integration test
4. This skill
5. Model memory

## Current baseline

| Component | Verified baseline |
|---|---|
| SurrealDB server | 3.2.4 |
| SurrealDB Rust SDK | 3.2.4 |
| Rust proving baseline | 1.96 |
| Last verification pass | 2026-09-12 |

Official SurrealDB documentation currently identifies Rust SDK **3.2.4** as the latest SDK and documents `SurrealValue`, `RecordId`, and the 3.x `type::record()` function.

## Repository structure

```text
.
├── SKILL.md
├── README.md
└── references/
    ├── capability-update-template.md
    ├── surrealdb-3-stale-llm-priors.md
    ├── surrealdb-3-contract-and-pitfalls.md
    ├── verification-status.md
    ├── subprocess-sandbox-and-path-containment.md
    ├── multi-agent-clobbering-and-concurrency.md
    ├── authoritative-xml-and-fiscal-parsing.md
    └── rigorous-ci-harness-and-testing-discipline.md
```

`capability-update-template.md` is the standard format for extending the repository to another breaking API/version gap without losing evidence quality.

## Evidence vocabulary

Every important claim should be understood as one of:

- **VERIFIED API** — confirmed by current official documentation or crate API.
- **TESTED BEHAVIOR** — reproduced against the named version in real code/tests.
- **PROJECT CONVENTION** — a design choice that worked in a proving project, not a universal technology requirement.
- **ARCHITECTURAL INTENT** — planned/recommended work, not implementation evidence.

This distinction exists to prevent a common failure mode in agent-generated documentation: a planned feature gets copied into a report, then into a roadmap, then later gets mistaken for code that actually exists.

## Why Astra still appears here

Astra is useful as an empirical test laboratory. Bugs discovered there become reusable corrections only after the pattern is generalized.

Example:

```text
Astra failure:
Expected string, got record
      ↓
root cause:
SurrealDB intrinsic id is a RecordId, not String
      ↓
general capability correction:
Use surrealdb::types::RecordId when modelling intrinsic IDs,
or omit intrinsic id and use a logical domain key.
```

The project incident is evidence. The general rule is the reusable skill.

## Installation in Antigravity

```bash
git clone https://github.com/Rodolfopr92/astra-engineering-lessons-skill.git \
  ~/.gemini/antigravity/skills/verified-modern-stack-capabilities
```

If the repository is already installed under the older directory name, pulling the latest `main` is sufficient as long as the agent loader reads `SKILL.md` frontmatter.

## Maintenance workflow

When a model generates suspicious code for a fast-moving dependency:

1. Identify the likely stale prior.
2. Check the exact dependency/server version.
3. Verify current official documentation.
4. Reproduce behavior with the smallest real compile/runtime test possible.
5. Fix the consuming project.
6. Add a regression test.
7. Add the generalized correction here using `references/capability-update-template.md`.

Do not add claims merely because an AI assistant stated them confidently.

## Scope today

The deepest coverage is SurrealDB 3.2.4 + Rust. The repository also contains verified, reusable lessons about:

- subprocess path containment and symlink handling;
- bounded streaming and payload limits;
- exact-decimal fiscal data;
- deterministic hostile-XML parsing;
- multi-agent worktree clobbering;
- real disk/restart persistence tests;
- strict CI and fixture-grounded assertions.

Future topics should only be added when they address a real stale-prior/capability gap and can be grounded in current evidence.
