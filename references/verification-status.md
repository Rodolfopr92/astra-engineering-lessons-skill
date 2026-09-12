# Verification Status Ledger

**Last updated:** 2026-09-12

This file prevents a capability skill from turning roadmap prose, project conventions, or old incidents into universal facts.

## Evidence classes

- **VERIFIED API** — supported by current official documentation / current public API.
- **TESTED BEHAVIOR** — independently reproduced in code against the named real version.
- **CASE-STUDY EVIDENCE** — observed in a real proving project, but not yet independently reduced/reproduced by this skill repository.
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
| `.bind()` accepts SDK variable/value forms through `IntoVariables` / `SurrealValue` | VERIFIED API | Official query/working-with-types docs |
| `kv-surrealkv` exists as an embedded storage feature in 3.2.4 | VERIFIED API | crate feature docs / SurrealDB embedding docs |
| Deserializing intrinsic `id` into `String` can fail with `Expected string, got record` | TESTED BEHAVIOR | Live SurrealDB 3.2.4 restart test |
| Real SurrealKV restart tests can expose failures missed by ordinary unit tests | TESTED BEHAVIOR | Astra Phase 4 integration failure/recovery |
| Server-side checkpoint increments prevent application read-modify-write lost updates | TESTED BEHAVIOR | Concurrent Phase 4 checkpoint tests |
| Prefer explicit logical IDs such as `artifact_id` in domain structs | PROJECT CONVENTION | Astra architecture |
| Tenant-per-database isolation is the required architecture for all SurrealDB apps | NOT A GENERAL RULE | Astra-specific design |

### Official references

- https://surrealdb.com/docs/reference/rust
- https://surrealdb.com/docs/reference/rust/embedding
- https://surrealdb.com/docs/reference/rust/methods/new
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types
- https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes
- https://surrealdb.com/docs/reference/rust/methods/query
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/type
- https://surrealdb.com/docs/reference/rust/methods/select
- https://surrealdb.com/docs/reference/rust/methods/create

---

## Embedded SurrealKV + Tauri / local-first

| Claim | Status | Evidence |
|---|---|---|
| SurrealDB Rust SDK can run embedded without an HTTP/WebSocket server | VERIFIED API | Official embedding docs |
| SurrealKV can be selected via `kv-surrealkv` | VERIFIED API | 3.2.4 feature docs |
| Tauri 2 exposes `PathResolver::app_data_dir()` and `app_local_data_dir()` | VERIFIED API | Current Tauri API docs |
| `app_data_dir()` resolves to platform data dir + bundle identifier | VERIFIED API | Tauri PathResolver docs |
| Every Tauri app should store its DB in AppData rather than AppLocalData | NOT A GENERAL RULE | App/platform-specific choice |
| Same-directory reopen should be used to prove embedded persistence | GENERAL TESTING RULE | Persistence boundary logic + Brew & Batch case study |
| Brew & Batch successfully used embedded SurrealKV/Tauri architecture | CASE-STUDY EVIDENCE | External proving-project report supplied by Manus |

Official references:
- https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html
- https://v2.tauri.app/reference/javascript/api/namespacepath/
- https://surrealdb.com/docs/reference/rust/embedding
- https://surrealdb.com/docs/reference/rust/methods/new

---

## SCHEMAFULL nested objects / arrays

| Claim | Status | Evidence |
|---|---|---|
| On SCHEMAFULL tables, nested object fields must be declared unless object-containing field is `FLEXIBLE` | VERIFIED API | Current DEFINE FIELD docs |
| `items.*.field` can define subfields of array<object> | VERIFIED API | Current DEFINE FIELD docs |
| As of SurrealDB 3.0, undefined nested fields error instead of being silently omitted | VERIFIED API | Current DEFINE FIELD docs |
| Fresh-install execution is needed to prove authoritative seeds match SCHEMAFULL declarations | GENERAL TESTING RULE | Schema boundary logic + Brew & Batch case study |
| Brew & Batch exposed undeclared nested object/array fields only during fresh install | CASE-STUDY EVIDENCE | External proving-project report supplied by Manus |

Official references:
- https://surrealdb.com/docs/reference/query-language/statements/define/field
- https://surrealdb.com/docs/reference/query-language/statements/define/table

---

## SurrealQL 3.2.4 query-shape / transaction observations

| Claim | Status | Evidence |
|---|---|---|
| `RELATE ... SET` uses `field = value`; `CONTENT` uses object-literal syntax | VERIFIED API | Official RELATE docs |
| Explicit transactions roll back on statement error / `THROW` | VERIFIED API | Official transaction docs |
| `.check()` should be used to surface statement errors from critical `db.query(...)` calls | VERIFIED API / HARDENING PATTERN | Rust query docs + tested application use |
| Explicit `<record>$variable` casts were reliable in some dynamic Brew & Batch queries | CASE-STUDY EVIDENCE | External proving-project report |
| Materializing IDs via `LET $ids = SELECT VALUE id ...` improved some tested `FOR` loops | CASE-STUDY EVIDENCE | External proving-project report |
| Every `FOR` loop requires pre-materialized IDs | FALSE / NOT GENERALIZED | Context-specific query behavior |
| Every bound record parameter must be `<record>$var` | FALSE / NOT GENERALIZED | Typed `RecordId` and other forms exist |
| Decimal values can cross a JSON/test boundary as strings depending on serialization layer | CASE-STUDY EVIDENCE | Brew & Batch assertion mismatch |

Official references:
- https://surrealdb.com/docs/reference/query-language/statements/relate
- https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions
- https://surrealdb.com/docs/reference/rust/methods/query

---

## Schema bundle parity / overlay state

| Claim | Status | Evidence |
|---|---|---|
| A canonical schema copy should have mechanically detectable identity/parity when mirrored into an app | GENERAL REPRODUCIBILITY PATTERN | Supply-chain/configuration integrity reasoning |
| A checksum must not include a manifest that embeds its own checksum | GENERAL REPRODUCIBILITY PATTERN | Avoid circular identity definition |
| Optional reference/cache overlays are conceptually distinct from schema migrations and transactional history | PROJECT/ARCHITECTURAL PATTERN | Brew & Batch case study |
| Brew & Batch used separate overlay markers with version/name/checksum/applied timestamp | CASE-STUDY EVIDENCE | External proving-project report |

---

## Build/verification state

| Claim | Status | Evidence |
|---|---|---|
| A build terminated by OOM/quota/sandbox death before meaningful compiler diagnostics is a pass | FALSE | No completed build evidence |
| The same event is automatically an application compile failure | FALSE | Infrastructure can terminate first |
| `BLOCKED / INDETERMINATE` is a useful third reporting state for resource-limited verification | GENERAL EVIDENCE RULE | CI/reproducibility discipline |
| Brew & Batch native build was blocked while compiling `surrealdb-core` before application diagnostics | CASE-STUDY EVIDENCE | External proving-project report |

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
CASE-STUDY EVIDENCE
    → reduce to minimal reproducer
    → independently run against named version
    → TESTED BEHAVIOR
```

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
