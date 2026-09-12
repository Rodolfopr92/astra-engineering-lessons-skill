---
name: verified-modern-stack-capabilities
description: Versioned capability corrections for AI coding agents working with fast-moving engineering stacks, starting with SurrealDB 3.2.4 + Rust. Use this skill to override stale 1.x/2.x SurrealDB priors, distinguish current verified APIs from project conventions, and apply empirically tested patterns for persistence, sandboxing, exact-decimal fiscal parsing, concurrency, and CI.
---

# Verified Modern Stack Capabilities

This skill exists to correct **stale model knowledge** in fast-moving engineering stacks.

Its first and deepest target is **SurrealDB 3.2.4 + Rust**, where AI models commonly reproduce older 1.x/2.x APIs, type names, query functions, or serialization assumptions. The repository also captures reusable, verified patterns for subprocess isolation, exact-decimal financial parsing, multi-agent concurrency, and real persistence testing.

Astra is a **proving ground and case study**, not the scope of the skill. Project-specific facts should remain clearly labeled as project conventions or evidence.

## Version Baseline

- SurrealDB server: **3.2.4**
- SurrealDB Rust SDK: **3.2.4**
- Rust baseline used by the proving project: **1.96**
- Capability baseline last verified: **2026-09-12**
- Primary evidence: official documentation + code that compiled + live integration/restart tests

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

Treat statements in this repository according to four categories:

- **VERIFIED API** — confirmed by current official documentation or crate API.
- **TESTED BEHAVIOR** — reproduced against the named real version in a compile/runtime/integration test.
- **PROJECT CONVENTION** — a design decision that worked for Astra but is not required by the technology.
- **ARCHITECTURAL INTENT** — planned or recommended behavior, not yet implementation evidence.

Do not silently promote a project convention into a universal API rule, or a roadmap item into a completed capability.

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

See `references/surrealdb-3-stale-llm-priors.md` for the dedicated correction table.

---

## 4. General Engineering Rules Proven by Real Failures

1. **Use real boundary tests, not mocks alone.** Unit tests and mocks are useful, but they do not replace WebSocket/native-SDK, disk persistence, restart, or subprocess-boundary tests.
2. **Do not deserialize SurrealDB intrinsic `id` as `String`.** Use `RecordId` when you need it, or explicit logical IDs when you do not.
3. **Use exact decimal arithmetic for money and fiscal values.** Avoid binary floating-point for authoritative monetary data.
4. **Validate hostile input at boundaries.** Enforce byte limits, structural limits, safe filenames, and deterministic parser behavior before durable materialization.
5. **Treat subprocess output as untrusted.** Reject raw symlinks and non-regular files before canonicalization, then canonicalize and enforce sandbox containment.
6. **Prefer storage-engine atomicity to application read-modify-write.** Use server-side increments/constraints and test concurrency against the real database.
7. **Never confuse collision-avoidance randomness with cryptographic randomness.** Document the actual security property.
8. **A green unit suite is evidence, not proof of persistence correctness.** Kill/restart the real database when persistence matters.

---

## 5. Topic Index

| Topic | Reference | Primary purpose |
|---|---|---|
| **SurrealDB stale LLM priors** | [surrealdb-3-stale-llm-priors.md](references/surrealdb-3-stale-llm-priors.md) | Fast correction table for 2.x → 3.x mistakes. |
| **SurrealDB 3.2.4 contract** | [surrealdb-3-contract-and-pitfalls.md](references/surrealdb-3-contract-and-pitfalls.md) | RecordId, SurrealValue, atomic checkpoints, real persistence. |
| **Verification status** | [verification-status.md](references/verification-status.md) | Separates verified API, tested behavior, project convention, and open work. |
| **Subprocess sandboxing** | [subprocess-sandbox-and-path-containment.md](references/subprocess-sandbox-and-path-containment.md) | Path containment, symlinks, bounded streaming, nonce classification. |
| **Multi-agent coordination** | [multi-agent-clobbering-and-concurrency.md](references/multi-agent-clobbering-and-concurrency.md) | Worktree clobbering, TOCTOU, canonical identities. |
| **Structured XML & NF-e** | [authoritative-xml-and-fiscal-parsing.md](references/authoritative-xml-and-fiscal-parsing.md) | Deterministic XML/NF-e extraction and exact arithmetic. |
| **CI/testing discipline** | [rigorous-ci-harness-and-testing-discipline.md](references/rigorous-ci-harness-and-testing-discipline.md) | Real-system gates, restart testing, fixture grounding. |

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

Prefer **verified corrections over accumulated prose**. If a lesson cannot be tied to an exact version, official source, compile result, or reproducible behavior, label it as architectural intent rather than fact.
