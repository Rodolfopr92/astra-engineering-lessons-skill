# surreal_rust_tauri

A versioned capability-correction skill for AI coding agents working with **SurrealDB 3.x + Rust + Tauri/local-first applications**.

Its purpose is narrow: correct stale model priors, show the current API contract, grade evidence, and provide small reproducers. It is **not** a project architecture planner.

## Current verified baseline

| Component | Baseline |
|---|---|
| SurrealDB server/engine | 3.2.4 |
| SurrealDB Rust SDK | 3.2.4 |
| Rust proving baseline | 1.96 |
| Official SDK minimum Rust | 1.89 |
| Tauri path/API documentation | 2.11.5 |
| Last full verification | 2026-09-12 |
| Maximum routine reverification interval | 90 days |

The current official SurrealDB Rust docs identify SDK/server **3.2.4** as current. The Tauri path APIs verified for this skill are from **Tauri 2.11.5**.

## The rule that prevents this skill becoming stale

If the target repository uses a SurrealDB server/engine or Rust SDK version other than **3.2.4**, or a Tauri-sensitive claim targets a Tauri version other than **2.11.5**, affected version-sensitive claims become **unverified for that target** until checked against the target version.

Reverify when:

- the target SurrealDB server or SDK version changes;
- the relevant Tauri API version changes;
- official docs deprecate or alter a covered API;
- a runtime result contradicts the skill;
- a relevant security advisory lands;
- or 90 days pass after the last full verification.

The target repo/lockfile always outranks this repository.

## Source-of-truth order

1. Target repository + exact lockfile/manifests
2. Official documentation/API for the exact target version
3. Reproducible compile/runtime/integration evidence
4. This skill
5. Model memory

## Evidence vocabulary

- **VERIFIED API** — current official API/docs for the named baseline.
- **TESTED BEHAVIOR** — independently reproduced against the named baseline.
- **CASE-STUDY EVIDENCE** — observed in a proving project, not independently minimized here.
- **PROJECT CONVENTION** — a project choice, not a technology requirement.
- **ARCHITECTURAL INTENT** — planned/recommended, not implementation evidence.
- **FALSE / NOT A GENERAL RULE** — retained to prevent a known over-generalization.

Case-study evidence from a different patch/minor version does **not** transfer automatically. A 3.2.3 observation may guide a 3.2.4 probe, but does not become `TESTED BEHAVIOR` for 3.2.4 until verified there.

## Current coverage

- `RecordId`, `SurrealValue`, `type::record()` and 3.x value/type boundaries;
- remote and embedded SurrealKV;
- `SurrealKv` engine selection vs `Surreal<Db>` application handle;
- Tauri `app_data_dir()` / `app_local_data_dir()` path APIs;
- SCHEMAFULL nested objects/arrays and `FLEXIBLE`;
- `.bind()`, `.check()`, `.take_errors()`, structured error kinds;
- `NONE` vs `NULL` vs JSON `null`;
- relation tables;
- BM25 full-text, HNSW KNN, `search::rrf()`;
- changefeeds vs versioned historical reads;
- transactions and rollback evidence;
- migration identity/checksums and recovery verification;
- process-separated durability tests;
- evidence-aware CI and blocked-build reporting.

## Repository structure

```text
.
├── SKILL.md
├── README.md
├── reproducers/
│   ├── README.md
│   └── surrealdb-3.2.4/
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

## Reproducers

High-value corrections should increasingly have executable minimal checks under `reproducers/`.

A reproducer only upgrades a claim to `TESTED BEHAVIOR` after it actually passes against the named baseline. Merely adding test source is not evidence.

## Installation in Antigravity

The skill directory name should match the frontmatter name:

```bash
git clone https://github.com/Rodolfopr92/astra-engineering-lessons-skill.git \
  ~/.gemini/antigravity/skills/surreal_rust_tauri
```

The GitHub repository slug is still the historical `astra-engineering-lessons-skill` because repository-settings rename is not exposed by the connected GitHub interface used for this maintenance pass. The intended repository name is **`surreal_rust_tauri`** as well.

## Maintenance workflow

When a model produces suspicious code for this stack:

1. identify the likely stale prior;
2. read the target versions from lockfiles/manifests;
3. check official docs for those versions;
4. reduce uncertainty to the smallest reproducer;
5. run it against the named target;
6. fix the consuming project;
7. preserve a regression test;
8. update this repository with the generalized technical correction and evidence.

Do not add architecture-planning doctrine here. Do not add claims merely because an AI assistant stated them confidently.