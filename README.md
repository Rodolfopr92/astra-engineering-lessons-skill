# Verified Modern Stack Capabilities

A versioned engineering skill for **correcting stale AI-model priors** in fast-moving software stacks.

The repository began as lessons extracted from Astra Analytics Bot, but its primary purpose is broader: give coding agents current, verified capability information when their training data strongly reflects older APIs or behavior.

The deepest target remains **SurrealDB 3.2.4 + Rust**, now covering both remote/server and **embedded SurrealKV + Tauri/local-first** architectures.

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
| SurrealDB server/engine | 3.2.4 |
| SurrealDB Rust SDK | 3.2.4 |
| Rust proving baseline | 1.96 |
| Tauri capability coverage | 2.x path/runtime APIs |
| Last verification pass | 2026-09-12 |

Official SurrealDB documentation currently identifies Rust SDK **3.2.4** as the latest SDK and documents `SurrealValue`, `RecordId`, embedded engines, `kv-surrealkv`, SCHEMAFULL nested-object behavior, and the 3.x `type::record()` function. Current Tauri 2 APIs expose application-scoped data directories suitable for persistent local-first state.

## Repository structure

```text
.
├── SKILL.md
├── README.md
└── references/
    ├── capability-update-template.md
    ├── surrealdb-3-stale-llm-priors.md
    ├── surrealdb-3-contract-and-pitfalls.md
    ├── embedded-surrealkv-tauri-local-first.md
    ├── surrealdb-3-query-shapes-and-sdk-binding.md
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
- **TESTED BEHAVIOR** — independently reproduced against the named version in real code/tests.
- **CASE-STUDY EVIDENCE** — observed in a real proving project but not yet independently reduced/reproduced by this skill repository.
- **PROJECT CONVENTION** — a design choice that worked in a proving project, not a universal technology requirement.
- **ARCHITECTURAL INTENT** — planned/recommended work, not implementation evidence.

This distinction prevents several common agent errors:

- a planned feature gets copied into reports until it is mistaken for implemented code;
- one project's workaround becomes a supposed universal API requirement;
- an external case-study observation is repeated as independently proven behavior;
- a resource-killed build is reported as either a pass or a source-code failure.

## Why proving projects still appear here

Real projects are empirical laboratories. Bugs discovered there become reusable corrections only after the pattern is generalized.

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

A second proving path now comes from Brew & Batch:

```text
Tauri/local-first conversion
      ↓
embedded SurrealKV + SCHEMAFULL fresh-install failures
      ↓
general corrections:
connection mode is part of the contract;
use stable app-owned data paths;
declare nested SCHEMAFULL object/array fields;
validate fresh installs against canonical schema;
report resource-blocked native builds as indeterminate.
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
2. Check the exact dependency/server/engine version.
3. Establish connection/storage mode where relevant.
4. Verify current official documentation.
5. Reproduce behavior with the smallest real compile/runtime test possible.
6. Fix the consuming project.
7. Add a regression test.
8. Add the generalized correction here using `references/capability-update-template.md`.

If a finding comes from an external proving project but has not been independently reduced, label it **CASE-STUDY EVIDENCE** first.

Do not add claims merely because an AI assistant stated them confidently.

## Scope today

The strongest coverage is SurrealDB 3.2.4 + Rust, including:

- stale 1.x/2.x record/type/query priors;
- native `SurrealValue` / `RecordId` contracts;
- remote vs embedded connection architecture;
- embedded SurrealKV and Tauri application-data paths;
- SCHEMAFULL nested object/array declarations and `FLEXIBLE` behavior;
- SDK `.bind()` / query-response hardening;
- transaction/rollback semantics and query-shape verification;
- real disk/restart or close/reopen persistence testing;
- schema-bundle parity and replay markers for optional overlays;
- resource-blocked native-build reporting.

The repository also contains verified, reusable lessons about:

- subprocess path containment and symlink handling;
- bounded streaming and payload limits;
- exact-decimal fiscal data;
- deterministic hostile-XML parsing;
- multi-agent worktree clobbering;
- strict CI and fixture-grounded assertions.

Future topics should only be added when they address a real stale-prior/capability gap and can be grounded in current evidence.
