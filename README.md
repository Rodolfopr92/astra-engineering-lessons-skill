# Verified Modern Stack Capabilities

A versioned engineering skill for **correcting stale AI-model priors** in fast-moving software stacks.

The repository began as lessons extracted from Astra, but its purpose is broader: give coding agents current, evidence-graded capability information when their training data strongly reflects older APIs or behavior.

The deepest target remains **SurrealDB 3.2.4 + Rust**, informed by several independent local-first, graph, search, persistence, and transactional codebases.

## What this repository is

Think of it as a retrieval-time compatibility layer:

```text
stale model prior
      ↓
versioned correction
      ↓
official API evidence
      ↓
real compile/runtime/case-study evidence
      ↓
agent receives current capability
```

Source-of-truth order:

1. Current target repository / lockfile
2. Official documentation for the target version
3. Reproducible compile or live integration test
4. This skill
5. Model memory

The repository is **not** an architecture planner. It should correct what a technology can do and how its current API behaves. Project-level choices such as system-of-record ownership, multi-store topology, or whether a projection architecture is appropriate belong in that project's planning and ADRs.

## Current baseline

| Component | Verified baseline |
|---|---|
| SurrealDB server/engine | 3.2.4 |
| SurrealDB Rust SDK | 3.2.4 |
| Rust proving baseline | 1.96 |
| Tauri capability coverage | 2.x path/runtime APIs |
| Last verification pass | 2026-09-12 |

Current coverage includes much more than basic CRUD:

- `RecordId`, `SurrealValue`, `type::record()` and 3.x type boundaries;
- remote and embedded SurrealKV architectures;
- Tauri/local-first storage paths and embedded lifecycle evidence;
- SCHEMAFULL nested object/array rules and `FLEXIBLE`;
- `.bind()`, `.check()`, `.take_errors()`, NONE/null, and explicit SDK-value boundaries;
- typed relation tables and graph integrity patterns;
- BM25 full-text, HNSW vector search, KNN distance and RRF hybrid search;
- changefeeds versus versioned-storage time travel;
- transactions, migration ledgers, checksums and schema idempotence;
- process-separated SurrealKV restart/stress testing;
- recovery/rebuild claims matched to executable evidence.

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
    ├── surrealdb-3-graph-search-and-changefeeds.md
    ├── surrealdb-3-migrations-and-recovery-evidence.md
    ├── verification-status.md
    ├── subprocess-sandbox-and-path-containment.md
    ├── multi-agent-clobbering-and-concurrency.md
    ├── authoritative-xml-and-fiscal-parsing.md
    └── rigorous-ci-harness-and-testing-discipline.md
```

`capability-update-template.md` is the standard format for extending the repository without losing evidence quality.

## Evidence vocabulary

Every important claim should be understood as one of:

- **VERIFIED API** — confirmed by current official documentation or crate API.
- **TESTED BEHAVIOR** — independently reproduced against the named version.
- **CASE-STUDY EVIDENCE** — observed in a real proving repository but not yet independently reduced here.
- **PROJECT CONVENTION** — a design decision, not a universal technology requirement.
- **ARCHITECTURAL INTENT** — planned/recommended work, not implementation evidence.

This prevents a common agent failure cascade where a project workaround or roadmap sentence is copied often enough to become “fact.”

## Proving repositories

The SurrealDB knowledge is cross-pollinated from several different technical environments:

| Repository | Reusable technical evidence |
|---|---|
| Astra-bot | RecordId failures, atomic checkpoints, restart persistence, tenant-isolation tests |
| Brew & Batch | embedded Tauri/SurrealKV, SCHEMAFULL fresh-install failures, SDK query/binding probes, schema parity |
| Omphalos-git | embedded boot/readiness, versioned schema runner, relation/agent-memory schema examples |
| Saturno | `Surreal<Db>` handle shape, migration idempotence |
| Alexandria | revision/hash/checkpoint schema patterns and fresh-install validation evidence |
| ARGOS | relation graphs, events, changefeeds, BM25/HNSW/RRF, transactions, checksummed migrations, restart/stress |
| DELPHIS | `take_errors()`, NONE/null adapter semantics, native Value→Serde boundary, process-separated restart |

These repositories are evidence sources, not templates that the skill tells another project to copy wholesale.

## Example: project incident → general capability

```text
Astra failure:
Expected string, got record
      ↓
general correction:
intrinsic id is RecordId, not String
```

```text
DELPHIS boundary:
Option::None → JSON null while storage wanted absence
      ↓
general correction:
NONE, NULL and JSON null are distinct contracts
```

```text
ARGOS search implementation:
BM25 + HNSW + RRF on current SurrealDB
      ↓
general correction:
modern SurrealDB supports this search surface;
verify exact version/query syntax before generating it
```

## Installation in Antigravity

```bash
git clone https://github.com/Rodolfopr92/astra-engineering-lessons-skill.git \
  ~/.gemini/antigravity/skills/verified-modern-stack-capabilities
```

If already installed under the older directory name, pulling latest `main` is sufficient if the loader reads `SKILL.md` frontmatter.

## Maintenance workflow

When a model generates suspicious code for a fast-moving dependency:

1. Identify the likely stale prior.
2. Check exact dependency/server/engine version.
3. Establish connection/storage mode where relevant.
4. Verify current official documentation.
5. Reproduce with the smallest real compile/runtime test possible.
6. Fix the consuming project.
7. Add a regression test.
8. Generalize the **technical correction** using `references/capability-update-template.md`.

If a finding comes from another proving repository but has not been independently minimized, label it **CASE-STUDY EVIDENCE** first.

If a finding is mainly “which architecture should this project choose?”, keep it in the project plan/ADR rather than this capability repository.

Do not add claims merely because an AI assistant stated them confidently.

## What the wider repository scan did *not* find

The current indexed/default branches of `daedalus-inventory`, `goldennest`, `thequietledger`, `chronos-runtime-governor`, and the portfolio/profile sites did not surface additional SurrealDB implementation evidence. `castor-finance` surfaced legacy/design notes rather than strong runtime evidence. `emporion-commerce` currently uses Postgres/sqlx rather than SurrealDB.

That negative result matters too: the skill should grow from evidence, not from repository names.

## Scope beyond SurrealDB

The repository also contains reusable lessons about:

- subprocess path containment and symlink handling;
- bounded streaming and payload limits;
- exact-decimal fiscal data;
- deterministic hostile-XML parsing;
- multi-agent worktree clobbering;
- strict CI, exact-SHA evidence, and fixture-grounded assertions;
- resource-blocked build reporting.

Future topics should only be added when they address a real stale-prior/capability gap and can be grounded in current evidence.