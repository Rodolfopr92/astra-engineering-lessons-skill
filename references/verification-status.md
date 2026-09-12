# Verification Status Ledger

**Last updated:** 2026-09-12

This file prevents a capability skill from turning roadmap prose, project conventions, or old incidents into universal facts.

## Evidence classes

- **VERIFIED API** — supported by current official documentation / current public API.
- **TESTED BEHAVIOR** — reproduced in code against the named real version.
- **PROJECT CONVENTION** — a design choice in a proving project; portable only when the same tradeoff applies.
- **ARCHITECTURAL INTENT** — planned/recommended, not implementation evidence.

---

## SurrealDB 3.2.4 + Rust SDK 3.2.4

| Claim | Status | Evidence |
|---|---|---|
| Current official Rust SDK version is 3.2.4 | VERIFIED API | SurrealDB Rust SDK docs |
| Rust SDK 3.x exposes `surrealdb::types::RecordId` | VERIFIED API | Official Rust examples |
| Rust SDK 3.x uses `SurrealValue` for native value conversion | VERIFIED API | Official working-with-types docs |
| `#[surreal(...)]` is separate from Serde attributes | VERIFIED API | Official SurrealValue attribute docs |
| `type::record()` is the 3.x name for pre-3.0 `type::thing()` | VERIFIED API | Official type-function docs |
| `(table, id)` tuple resources remain supported in Rust SDK methods | VERIFIED API | Official `select` / `create` docs |
| Deserializing intrinsic `id` into `String` can fail with `Expected string, got record` | TESTED BEHAVIOR | Live SurrealDB 3.2.4 restart test |
| Real SurrealKV restart tests can expose failures missed by ordinary unit tests | TESTED BEHAVIOR | Astra Phase 4 integration failure/recovery |
| Server-side checkpoint increments prevent application read-modify-write lost updates | TESTED BEHAVIOR | Concurrent Phase 4 checkpoint tests |
| Prefer explicit logical IDs such as `artifact_id` in domain structs | PROJECT CONVENTION | Astra architecture |
| Tenant-per-database isolation is the required architecture for all SurrealDB apps | NOT A GENERAL RULE | Astra-specific design |

### Official references

- https://surrealdb.com/docs/reference/rust
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types
- https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/type
- https://surrealdb.com/docs/reference/rust/methods/select
- https://surrealdb.com/docs/reference/rust/methods/create

---

## Subprocess / filesystem boundary

| Claim | Status | Evidence |
|---|---|---|
| Worker-returned paths must not be trusted | GENERAL SECURITY RULE | Threat model + regression tests |
| Checking raw `symlink_metadata()` before canonicalization detects direct symlink output | TESTED BEHAVIOR | Astra Phase 3.1 tests |
| Canonicalized output must remain under canonical sandbox root | TESTED BEHAVIOR | Outside-path and nested-symlink tests |
| `tempfile::TempDir` provides a stronger isolation primitive than predictable manual temp names | PROJECT/GENERAL PATTERN | Astra hardening |
| `fastrand` is cryptographically secure | FALSE | It is not a CSPRNG |
| `fastrand` may be acceptable for collision-avoidance when secrecy is not the security boundary | PROJECT CONVENTION | Astra inner filename use |

---

## Fiscal XML / NF-e

| Claim | Status | Evidence |
|---|---|---|
| NF-e amounts should use exact decimals rather than `f32`/`f64` | GENERAL FINANCIAL RULE | Decimal integration tests |
| 44-digit access-key Modulo-11 verification is implemented | TESTED BEHAVIOR | Phase 5 parser tests |
| Raw `NFe` and `nfeProc` are structurally distinguished | TESTED BEHAVIOR | Phase 5 fixtures/tests |
| `nfeProc` protocol fields can be extracted deterministically | TESTED BEHAVIOR | Phase 5 fixture/tests |
| Parsed NF-e data is legally/fiscally validated by SEFAZ | NOT IMPLEMENTED | No live SEFAZ verification |
| XML signature chain is cryptographically verified | NOT IMPLEMENTED | No signature verification |
| NF-e multi-table materialization is transactionally all-or-nothing | OPEN / NOT YET PROVEN | Current Phase 5 persistence uses sequential writes |
| R$ 0.02 item-total tolerance is mandated by SurrealDB or NF-e technology | FALSE | It is an Astra validation policy |

---

## Agent / multi-agent engineering

| Claim | Status | Evidence |
|---|---|---|
| Two agents can overwrite each other's changes in a shared worktree | TESTED INCIDENT | Astra repository clobbering incident |
| `git status`/re-read before write reduces clobbering risk | GENERAL PRACTICE | Incident remediation |
| Separate branches/worktrees provide stronger isolation | GENERAL PRACTICE | Git workflow design |
| mtime alone is a complete concurrency-control mechanism | FALSE | It can be a signal, not a transactional guarantee |

---

## Proving-project status (Astra)

This section is evidence bookkeeping, not a general capability contract.

### Proven / merged or exact-SHA green

- Tool Fabric typed request/response boundary.
- MarkItDown subprocess conversion.
- bounded Telegram file intake.
- worker-output containment and symlink rejection.
- Rust-owned source/canonical hashes.
- document artifact persistence.
- unique source-hash dedupe.
- atomic checkpoint increments with conflict retry.
- SurrealKV restart persistence.
- deterministic NF-e parser and exact-decimal model on PR #8 exact green head.

### Known open issue

- NF-e root/child materialization is currently sequential rather than one all-or-nothing database transaction. Do not describe Phase 5 persistence as transactionally atomic until a failure-injection rollback test proves it.

### Architectural intent, not implementation evidence

- supplier email intake.
- production backup/restore/monitoring baseline.
- Docling/OCR fallback pipeline.
- DuckDB/Polars analytics worker.
- RAG/retrieval.
- speech transcription worker.
- browser-action worker.

---

## Maintenance rule

Whenever a claim moves categories, update this ledger.

Examples:

```text
ARCHITECTURAL INTENT
    → implementation lands
    → integration test proves behavior
    → TESTED BEHAVIOR
```

or:

```text
VERIFIED API for 3.2.4
    → dependency upgrades
    → re-check official docs and compile
    → VERIFIED API for new baseline
```

Never preserve an old version label simply because the example still looks plausible.
