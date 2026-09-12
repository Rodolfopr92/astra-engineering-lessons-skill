---
name: verified-modern-stack-capabilities
description: Versioned capability corrections for AI coding agents working with fast-moving engineering stacks, centered on SurrealDB 3.2.4 + Rust. Corrects stale 1.x/2.x priors across RecordId/SurrealValue, embedded SurrealKV + Tauri, SCHEMAFULL, SDK response/value boundaries, relation graphs, BM25/HNSW/RRF search, changefeeds/versioning, transactions, migrations, authority/rebuildability, persistence testing, and evidence discipline.
---

# Verified Modern Stack Capabilities

This skill exists to correct **stale model knowledge** in fast-moving engineering stacks.

Its deepest target is **SurrealDB 3.2.4 + Rust**, where AI models commonly reproduce older 1.x/2.x APIs, remote-only assumptions, outdated full-text syntax, loose schema assumptions, or incorrect Rust value/record types.

The evidence now comes from several independent proving architectures: remote SurrealKV, embedded Tauri/SurrealKV, SurrealDB as operational authority, and SurrealDB as a rebuildable graph projection. Project-specific facts remain evidence, not universal rules.

## Version baseline

- SurrealDB server/engine: **3.2.4**
- SurrealDB Rust SDK: **3.2.4**
- Rust proving baseline: **1.96**
- Tauri coverage: **2.x current path/runtime APIs**
- Last verification pass: **2026-09-12**

When working against another version, verify before copying patterns verbatim.

---

## 1. Source-of-truth hierarchy

When sources disagree:

1. **Current target repository and exact lockfile**
2. **Official documentation for the target/current version**
3. **A reproducible compile/runtime/integration test**
4. **This skill**
5. **Model memory**

This skill is a capability patch, not an oracle.

---

## 2. Evidence labels

- **VERIFIED API** — current official documentation/API.
- **TESTED BEHAVIOR** — independently reproduced against the named real version.
- **CASE-STUDY EVIDENCE** — observed in a real proving repository but not independently minimized here.
- **PROJECT CONVENTION** — a design decision, not a technology requirement.
- **ARCHITECTURAL INTENT** — planned/recommended, not implementation evidence.

Never promote a case-study workaround into a universal API rule merely because several reports repeat it.

---

## 3. High-value SurrealDB 3.x corrections

### Record constructor

```surql
-- stale pre-3.0
type::thing('person', $id)

-- current 3.x
type::record('person', $id)
```

### Intrinsic record IDs

```rust
use surrealdb::types::{RecordId, SurrealValue};

#[derive(SurrealValue)]
struct Row {
    id: RecordId,
}
```

Do not deserialize intrinsic `id` into `String` unless the query deliberately casts it to text and your transport contract expects that representation.

### Native Rust conversion

`SurrealValue` is the native SDK value-conversion contract. Serde and `#[surreal(...)]` are distinct systems.

### Embedded SurrealKV

```rust
use surrealdb::{Surreal, engine::local::{Db, SurrealKv}};

let db: Surreal<Db> = Surreal::new::<SurrealKv>(path).await?;
db.use_ns("app").use_db("main").await?;
```

SurrealDB does not inherently mean a separate WS/HTTP server.

### SCHEMAFULL nested data

```surql
DEFINE FIELD metadata ON TABLE event TYPE object FLEXIBLE;

-- strict nested alternative
DEFINE FIELD shipping ON TABLE order TYPE object;
DEFINE FIELD shipping.city ON TABLE order TYPE string;
```

### Query response errors

```rust
let mut response = db.query(sql).await?;
let errors = response.take_errors(); // indexed statement failures
```

or:

```rust
let response = db.query(sql).await?.check()?;
```

Outer `Ok` does not prove every statement succeeded.

### `NONE` vs `NULL`

```surql
SET field = NONE; -- absence/remove
SET field = NULL; -- stored empty value
```

Do not assume JSON `null` means SurrealQL `NONE`.

### Relation tables

```surql
DEFINE TABLE works_at
  TYPE RELATION FROM person TO company
  SCHEMAFULL;
```

### Full-text syntax

```surql
-- stale pre-3.0
SEARCH ANALYZER app_text BM25

-- current 3.x
FULLTEXT ANALYZER app_text BM25
```

### Hybrid search

Current 3.x supports BM25 full-text, HNSW KNN vector search, `vector::distance::knn()`, and fusion via `search::rrf()`.

### Changefeed vs historical versioning

```surql
DEFINE TABLE opportunity CHANGEFEED 30d;
SHOW CHANGES FOR TABLE opportunity SINCE $cursor;
```

is a replayable mutation feed. It is **not** the same mechanism as:

```surql
SELECT * FROM opportunity:abc VERSION d"2026-09-01T12:00:00Z";
```

which requires a supported storage engine with versioning enabled.

### Transactions

Sequential application writes are not a transaction. Use a real database transaction for all-or-nothing multi-record state and prove rollback with failure injection.

---

## 4. Architecture corrections models often miss

### SurrealDB can be the authority

ARGOS demonstrates a local-first architecture where SurrealDB owns durable operational records and relationships.

### SurrealDB can be a rebuildable projection

Alexandria demonstrates an architecture where another transactional store is authoritative and SurrealDB receives revisioned/idempotent graph projections via an outbox.

Neither is the universal rule.

Before designing schema or recovery, ask:

```text
Is SurrealDB authoritative or derived?
If derived, what source rebuilds it?
What identifies projection revision/content?
What checkpoint makes retry deterministic?
If authoritative, what transaction/constraint protects multi-record invariants?
```

### Schema history is not projection history

Keep distinct:

```text
schema migration ledger
projection checkpoint
optional reference-overlay marker
business/audit history
```

---

## 5. General engineering rules

1. **Use real boundary tests, not mocks alone.**
2. **Match the deployed connection model.** Remote and embedded paths prove different things.
3. **Use process-separated restart evidence for strong embedded durability claims.** Same-handle schema idempotence is not restart proof.
4. **Prefer storage-engine atomicity to application read-modify-write.**
5. **Validate SCHEMAFULL seeds on a fresh database.**
6. **Use exact decimal arithmetic for authoritative money/tax.**
7. **Treat subprocess output as untrusted.**
8. **Use structured error kinds instead of durable substring matching where the SDK exposes them.**
9. **Bind ordinary data; strictly validate any dynamic query identifier/syntax that cannot be bound.**
10. **A resource-killed build is neither pass nor application compile failure.** Report BLOCKED/INDETERMINATE until meaningful diagnostics exist.
11. **The strength of a claim must not exceed the strength of its evidence.**

---

## 6. Topic index

| Topic | Reference | Primary purpose |
|---|---|---|
| **SurrealDB stale priors** | [surrealdb-3-stale-llm-priors.md](references/surrealdb-3-stale-llm-priors.md) | Fast 1.x/2.x → 3.x correction layer. |
| **Core SurrealDB 3.2.4 contract** | [surrealdb-3-contract-and-pitfalls.md](references/surrealdb-3-contract-and-pitfalls.md) | RecordId, SurrealValue, atomic checkpoints, persistence. |
| **Embedded SurrealKV + Tauri** | [embedded-surrealkv-tauri-local-first.md](references/embedded-surrealkv-tauri-local-first.md) | Local handle/lifecycle, Tauri paths, versioning, fresh installs. |
| **Query/value/response boundaries** | [surrealdb-3-query-shapes-and-sdk-binding.md](references/surrealdb-3-query-shapes-and-sdk-binding.md) | `.bind()`, `.check()`, `.take_errors()`, NONE/null, RecordId text, transactions. |
| **Graph/search/changefeeds** | [surrealdb-3-graph-search-and-changefeeds.md](references/surrealdb-3-graph-search-and-changefeeds.md) | Relation schema, BM25/HNSW/RRF, changefeeds, VERSION boundary. |
| **Migrations/authority/rebuildability** | [surrealdb-3-migrations-authority-and-rebuildability.md](references/surrealdb-3-migrations-authority-and-rebuildability.md) | Append-only migrations, checksums, authority vs projection, rebuild evidence. |
| **Verification ledger** | [verification-status.md](references/verification-status.md) | Evidence classifications and proving-repository status. |
| **Subprocess sandboxing** | [subprocess-sandbox-and-path-containment.md](references/subprocess-sandbox-and-path-containment.md) | Path containment, symlinks, bounded streaming. |
| **Multi-agent coordination** | [multi-agent-clobbering-and-concurrency.md](references/multi-agent-clobbering-and-concurrency.md) | Worktree clobbering and optimistic concurrency. |
| **Structured XML & NF-e** | [authoritative-xml-and-fiscal-parsing.md](references/authoritative-xml-and-fiscal-parsing.md) | Deterministic XML/NF-e and exact arithmetic. |
| **CI/testing discipline** | [rigorous-ci-harness-and-testing-discipline.md](references/rigorous-ci-harness-and-testing-discipline.md) | Real-system gates, restart testing, exact-SHA evidence. |

---

## 7. Maintenance rule

When a real project exposes a stale-model failure:

```text
model prior
   ↓
identify exact version + engine + authority mode
   ↓
check current official API
   ↓
reproduce smallest useful behavior
   ↓
fix production code + regression test
   ↓
extract generalized correction
   ↓
retain project-specific incident only as evidence
```

If a finding is supported only by a proving repository, keep it **CASE-STUDY EVIDENCE** until a minimal reproducer or official API source justifies promotion.