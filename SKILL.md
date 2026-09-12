---
name: verified-modern-stack-capabilities
description: Versioned capability corrections for AI coding agents working with fast-moving engineering stacks, centered on SurrealDB 3.2.4 + Rust and now including embedded SurrealKV + Tauri/local-first patterns. Use this skill to override stale 1.x/2.x SurrealDB priors, distinguish verified APIs from project conventions, and apply empirically tested patterns for persistence, schemafull data, query binding, sandboxing, exact-decimal fiscal parsing, concurrency, and CI evidence.
---

# Verified Modern Stack Capabilities

This skill exists to correct **stale model knowledge** in fast-moving engineering stacks.

Its first and deepest target is **SurrealDB 3.2.4 + Rust**, where AI models commonly reproduce older 1.x/2.x APIs, type names, query functions, serialization assumptions, or remote-server-only architecture. It now also covers **embedded SurrealKV inside Tauri/local-first applications**, SCHEMAFULL nested-object behavior, SDK binding/query-shape pitfalls, subprocess isolation, exact-decimal financial parsing, multi-agent concurrency, and evidence-aware CI.

Astra and Brew & Batch are **proving grounds and case studies**, not the scope of the skill. Project-specific facts must remain clearly labeled as project convention or case-study evidence.

## Version Baseline

- SurrealDB server/engine: **3.2.4**
- SurrealDB Rust SDK: **3.2.4**
- Rust proving baseline: **1.96**
- Tauri coverage: **2.x current path/runtime APIs**
- Capability baseline last verified: **2026-09-12**
- Primary evidence: official documentation + code that compiled + live integration/restart tests + clearly labeled external proving-project evidence

When working against another version, verify the relevant API before copying these patterns verbatim.

---

## 1. Source-of-Truth Hierarchy

When sources disagree, use this order:

1. **Current target repository and exact dependency lockfile**
2. **Official documentation for the exact/current version**
3. **A reproducible compile or integration test against the real system**
4. **This skill**
5. **Model memory / prior knowledge**

This skill is a capability patch, not an oracle. If current code or official versioned documentation contradicts it, re-verify and update the skill.

---

## 2. Evidence Labels

Treat statements in this repository according to five categories:

- **VERIFIED API** — confirmed by current official documentation or crate API.
- **TESTED BEHAVIOR** — independently reproduced against the named real version in a compile/runtime/integration test.
- **CASE-STUDY EVIDENCE** — observed in a real proving project, but not yet independently reduced/reproduced by this skill repository.
- **PROJECT CONVENTION** — a design decision that worked in a proving project but is not required by the technology.
- **ARCHITECTURAL INTENT** — planned or recommended behavior, not implementation evidence.

Do not silently promote a project convention into a universal API rule, a case-study observation into independently tested behavior, or a roadmap item into a completed capability.

---

## 3. High-Value Stale-Prior Corrections

### SurrealDB 3.x Rust types

**STALE MODEL PATTERN**

```rust
use surrealdb::sql::Thing;
#[derive(Serialize, Deserialize)]
struct Record { id: String }
```

**CURRENT VERIFIED PATTERN (3.2.4)**

```rust
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, SurrealValue)]
struct Record {
    id: RecordId,
}
```

For domain records where the intrinsic database ID is not needed, a **project convention** is to use explicit logical IDs such as `artifact_id`, `checkpoint_id`, or `nfe_id` and omit intrinsic `id` from the domain struct.

### SurrealQL record constructor

**STALE MODEL PATTERN (pre-3.0)**

```surql
type::thing('person', $id)
```

**CURRENT VERIFIED PATTERN (3.x)**

```surql
type::record('person', $id)
```

### Native Rust value conversion

`SurrealValue` is the current native Rust SDK conversion contract. Its `#[surreal(...)]` attributes are inspired by Serde but are not inherited from `#[serde(...)]`.

### Connection model

**STALE ASSUMPTION**

> SurrealDB always means connecting to a separate HTTP/WebSocket server.

The Rust SDK also supports embedded engines. For local-first desktop applications, SurrealKV can run in-process with the `kv-surrealkv` feature.

```rust
use surrealdb::{Surreal, engine::local::SurrealKv};

let db = Surreal::new::<SurrealKv>(database_path).await?;
```

### SCHEMAFULL nested objects

**STALE ASSUMPTION**

> Defining a top-level object field is enough for arbitrary nested keys.

In 3.x, nested object paths on SCHEMAFULL tables must be declared unless the containing object is intentionally `FLEXIBLE`.

```surql
DEFINE FIELD metadata ON event TYPE object FLEXIBLE;

-- or strict nested paths
DEFINE FIELD shipping ON order TYPE object;
DEFINE FIELD shipping.city ON order TYPE string;
```

See `references/surrealdb-3-stale-llm-priors.md` for the dedicated correction table.

---

## 4. General Engineering Rules Proven by Real Failures

1. **Use real boundary tests, not mocks alone.** Unit tests and mocks are useful, but they do not replace deployed-protocol, embedded-engine, disk persistence, restart, or subprocess-boundary tests.
2. **Do not deserialize SurrealDB intrinsic `id` as `String`.** Use `RecordId` when you need it, or explicit logical IDs when you do not.
3. **Connection mode is part of the contract.** Remote WS/HTTP and embedded SurrealKV are different architectures; establish the mode before generating code.
4. **SCHEMAFULL nested data must be proven against a fresh install.** Old developer state can hide undeclared nested-field defects.
5. **Use exact decimal arithmetic for money and fiscal values.** Avoid binary floating-point for authoritative monetary data.
6. **Validate hostile input at boundaries.** Enforce byte limits, structural limits, safe filenames, and deterministic parser behavior before durable materialization.
7. **Treat subprocess output as untrusted.** Reject raw symlinks and non-regular files before canonicalization, then canonicalize and enforce sandbox containment.
8. **Prefer storage-engine atomicity to application read-modify-write.** Use server-side increments/constraints/transactions and test concurrency/rollback against the real database.
9. **Never confuse collision-avoidance randomness with cryptographic randomness.** Document the actual security property.
10. **A green unit suite is evidence, not proof of persistence correctness.** Kill/restart or close/reopen the real database when persistence matters.
11. **A resource-killed build is neither pass nor code failure.** Report verification as blocked/indeterminate until the compiler actually reaches meaningful diagnostics.

---

## 5. Topic Index

| Topic | Reference | Primary purpose |
|---|---|---|
| **SurrealDB stale LLM priors** | [surrealdb-3-stale-llm-priors.md](references/surrealdb-3-stale-llm-priors.md) | Fast correction table for 2.x → 3.x mistakes. |
| **SurrealDB 3.2.4 contract** | [surrealdb-3-contract-and-pitfalls.md](references/surrealdb-3-contract-and-pitfalls.md) | RecordId, SurrealValue, atomic checkpoints, real persistence. |
| **Embedded SurrealKV + Tauri** | [embedded-surrealkv-tauri-local-first.md](references/embedded-surrealkv-tauri-local-first.md) | Embedded engine preflight, Tauri app data paths, SCHEMAFULL nested fields, schema parity, overlay state. |
| **SurrealDB query/binding shapes** | [surrealdb-3-query-shapes-and-sdk-binding.md](references/surrealdb-3-query-shapes-and-sdk-binding.md) | `.bind()`, record parameters, `RELATE`, transactions, query-shape probes. |
| **Verification status** | [verification-status.md](references/verification-status.md) | Separates verified API, tested behavior, case-study evidence, project convention, and open work. |
| **Subprocess sandboxing** | [subprocess-sandbox-and-path-containment.md](references/subprocess-sandbox-and-path-containment.md) | Path containment, symlinks, bounded streaming, nonce classification. |
| **Multi-agent coordination** | [multi-agent-clobbering-and-concurrency.md](references/multi-agent-clobbering-and-concurrency.md) | Worktree clobbering, TOCTOU, canonical identities. |
| **Structured XML & NF-e** | [authoritative-xml-and-fiscal-parsing.md](references/authoritative-xml-and-fiscal-parsing.md) | Deterministic XML/NF-e extraction and exact arithmetic. |
| **CI/testing discipline** | [rigorous-ci-harness-and-testing-discipline.md](references/rigorous-ci-harness-and-testing-discipline.md) | Real-system gates, restart/reopen testing, fixture grounding, blocked-build reporting. |

---

## 6. Quick Correctness Patterns

### SurrealDB record IDs

```rust
// Technology-level option: keep intrinsic ID with its real type.
#[derive(Debug, SurrealValue)]
struct DbRecord {
    id: RecordId,
    title: String,
}

// Project-level option: omit intrinsic ID and use a logical domain key.
#[derive(Debug, SurrealValue)]
struct Artifact {
    artifact_id: String,
    title: String,
}
```

### Embedded SurrealKV in a desktop app

```rust
let app_data = app.path().app_data_dir()?;
let db_dir = app_data.join("database");
std::fs::create_dir_all(&db_dir)?;

let db = Surreal::new::<SurrealKv>(db_dir).await?;
```

The exact application directory (`AppData` vs `AppLocalData`) is a project/platform decision. The general rule is to resolve a stable application-owned path rather than relying on the process working directory.

### Worker output containment

```rust
let raw = Path::new(output_path);
let raw_meta = fs::symlink_metadata(raw)?;
if raw_meta.file_type().is_symlink() || !raw_meta.is_file() {
    return Err(SecurityError::RejectedOutput);
}

let canonical_sandbox = fs::canonicalize(sandbox)?;
let canonical_output = fs::canonicalize(raw)?;
if !canonical_output.starts_with(&canonical_sandbox) {
    return Err(SecurityError::PathEscape);
}
```

The order matters: checking only `symlink_metadata()` **after** canonicalization no longer tells you whether the original worker-returned path was itself a symlink.

### Atomic checkpoint update

Use a single server-side mutation, inspect statement errors with `.check()`, and retry only conflicts known to be retryable. See the exact tested pattern in the SurrealDB reference rather than inventing a read-modify-write loop.

### Transactional multi-record materialization

When root and child records must succeed or fail together, use a real transaction and prove rollback with failure injection. Sequential `upsert().await?` calls are not an all-or-nothing transaction merely because each call is individually atomic.

---

## 7. Maintenance Rule

When a real project uncovers a stale-model failure:

```text
model prior
   ↓
reproduce against exact version
   ↓
confirm current official API / runtime behavior
   ↓
fix production code
   ↓
add regression test
   ↓
extract the general correction into this skill
   ↓
keep project-specific incident as evidence/case study
```

If the finding comes from another proving project but has not yet been independently minimized, record it as **CASE-STUDY EVIDENCE** rather than silently upgrading it to TESTED BEHAVIOR.

Prefer **verified corrections over accumulated prose**. If a lesson cannot be tied to an exact version, official source, compile result, or reproducible behavior, label it appropriately rather than presenting it as universal fact.
