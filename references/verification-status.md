# Verification Status Ledger

**Last updated:** 2026-09-12

This file prevents a capability skill from turning roadmap prose, project conventions, architecture choices, or case-study observations into universal facts.

## Evidence classes

- **VERIFIED API** — supported by current official documentation / public API.
- **TESTED BEHAVIOR** — independently reproduced against the named real version.
- **CASE-STUDY EVIDENCE** — observed in a real proving repository, not yet independently minimized by this skill repository.
- **PROJECT CONVENTION** — a design choice; portable only when the same tradeoff applies.
- **ARCHITECTURAL INTENT** — planned/recommended, not implementation evidence.
- **NOT A GENERAL RULE / FALSE** — explicitly prevents over-generalization.

Architecture planning is out of scope for this ledger except where needed to mark a claim as non-universal.

---

## SurrealDB 3.2.4 + Rust SDK 3.2.4

| Claim | Status | Evidence |
|---|---|---|
| Current official Rust SDK version is 3.2.4 | VERIFIED API | SurrealDB Rust SDK docs |
| Rust SDK 3.x exposes `surrealdb::types::RecordId` | VERIFIED API | Official Rust docs/examples |
| Rust SDK 3.x uses `SurrealValue` for native value conversion | VERIFIED API | Official working-with-types docs |
| `#[surreal(...)]` is distinct from Serde attributes | VERIFIED API | Official SurrealValue attribute docs |
| `type::record()` is the 3.x name for pre-3.0 `type::thing()` | VERIFIED API | Official type-function docs |
| `(table, id)` tuple resources remain supported | VERIFIED API | Official Rust method docs |
| Deserializing intrinsic `id` into `String` can fail with `Expected string, got record` | TESTED BEHAVIOR | Astra 3.2.4 restart test |
| Server-side checkpoint increments can avoid application read-modify-write lost updates | TESTED BEHAVIOR | Astra concurrent 3.2.4 checkpoint test |
| Prefer explicit logical IDs in domain structs | PROJECT CONVENTION | Astra architecture |

Official references:
- https://surrealdb.com/docs/reference/rust
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/type

---

## Embedded SurrealKV / Tauri

| Claim | Status | Evidence |
|---|---|---|
| Rust SDK supports embedded SurrealDB engines | VERIFIED API | Official Rust SDK docs |
| Current embedded SurrealKV uses the `kv-surrealkv` feature | VERIFIED API | Official Rust/crate docs |
| `Surreal::new::<SurrealKv>(path)` is used with an application handle typed `Surreal<Db>` | CASE-STUDY EVIDENCE | Omphalos, Saturno, ARGOS, DELPHIS |
| Tauri 2 exposes application-scoped data directories suitable for persistent local state | VERIFIED API | Tauri API docs |
| Post-open readiness probing after namespace/database selection improves boot evidence | PROJECT PATTERN | Omphalos |
| Same-handle schema reapplication proves process restart durability | FALSE | It proves idempotence, not restart |
| Process-separated writer/reader tests provide stronger embedded persistence evidence | CASE-STUDY EVIDENCE | ARGOS, DELPHIS |
| SurrealKV automatically enables `VERSION` history | FALSE | Versioning is opt-in |
| Rust embedded SurrealKV can enable versioning with `.versioned()` | VERIFIED API | Official `new()` docs |

Official references:
- https://surrealdb.com/docs/reference/rust/methods/new
- https://surrealdb.com/docs/reference/query-language/statements/select

---

## SCHEMAFULL / graph schema

| Claim | Status | Evidence |
|---|---|---|
| Undefined nested fields on SCHEMAFULL objects error in current 3.x unless declared / made FLEXIBLE | VERIFIED API | Official DEFINE FIELD docs |
| `TYPE RELATION FROM ... TO ...` and `IN ... OUT ...` are current relation-table syntax | VERIFIED API | Official DEFINE TABLE docs |
| `object FLEXIBLE` is appropriate when arbitrary nested keys are intentional | VERIFIED API | Official schema docs |
| Every graph edge should have unique `(in,out)` | NOT A GENERAL RULE | Domain-dependent integrity policy |
| Fresh-install seeds can expose missing nested schema hidden by upgraded dev state | CASE-STUDY EVIDENCE | Brew & Batch, Alexandria validation |

---

## Query / response / value boundaries

| Claim | Status | Evidence |
|---|---|---|
| Outer `.query(...).await` success can still contain failing statements | VERIFIED API | Rust error-handling docs |
| `.check()` fails on statement error | VERIFIED API | Rust query docs |
| `.take_errors()` preserves indexed statement failures | VERIFIED API | Rust query/error-handling docs |
| Durable retry/security logic should match structured error kinds rather than message text | VERIFIED API | Rust error-handling docs |
| `NONE` means absence; `NULL` is a stored empty value | VERIFIED API | NONE/NULL docs |
| Ordinary JSON serialization of Rust `Option::None` can require adapter handling when storage semantics require absence | CASE-STUDY EVIDENCE | DELPHIS |
| `surrealdb::types::Value -> into_json_value() -> Serde` is a viable explicit legacy-domain boundary | CASE-STUDY EVIDENCE | DELPHIS |
| `<record>$variable` is universally required for dynamic records | NOT A GENERAL RULE | Brew & Batch observation only |
| UUID-shaped text record IDs may render with backtick delimiters when cast to string | CASE-STUDY EVIDENCE | ARGOS |

Official references:
- https://surrealdb.com/docs/reference/rust/concepts/error-handling
- https://surrealdb.com/docs/reference/rust/methods/query
- https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/none-and-null

---

## Search / graph / changefeeds / temporal

| Claim | Status | Evidence |
|---|---|---|
| Pre-3.0 `SEARCH ANALYZER` full-text syntax changed to `FULLTEXT ANALYZER` in 3.x | VERIFIED API | Official search docs |
| `search::score()` and `search::rrf()` are current search functions | VERIFIED API | Official search docs |
| HNSW vector indexes and KNN operators are current | VERIFIED API | Official vector docs |
| BM25 + HNSW candidate lists can be fused with RRF | VERIFIED API | Official hybrid-search docs |
| ARGOS implements BM25 + HNSW + RRF on SurrealDB 3.2.3 | CASE-STUDY EVIDENCE | ARGOS schema/runtime |
| `CHANGEFEED` + `SHOW CHANGES ... SINCE` provides replayable mutation history within retention | VERIFIED API | Official changefeed/SHOW docs |
| Changefeeds are the same thing as `SELECT ... VERSION` history | FALSE | Separate mechanisms |
| `SELECT ... VERSION` requires a versioning-enabled supported storage engine | VERIFIED API | Official SELECT/storage docs |
| Choosing SurrealKV alone automatically enables time-travel history | FALSE | Versioning must be enabled |

Official references:
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/search
- https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search
- https://surrealdb.com/docs/learn/querying/real-time/changefeeds
- https://surrealdb.com/docs/reference/query-language/statements/select

---

## Transactions

| Claim | Status | Evidence |
|---|---|---|
| Explicit transactions are all-or-nothing and can be deliberately aborted | VERIFIED API | SurrealDB transaction docs |
| Rust SDK exposes manual transaction `commit()` / `cancel()` | VERIFIED API | Rust transaction docs |
| Sequential independent `upsert().await?` calls are equivalent to one transaction | FALSE | Separate operations |
| Failure injection should prove multi-record rollback | GENERAL TESTING RULE | High-assurance persistence discipline |
| ARGOS proposal signing persists multiple records/edge/outbox in one transaction and uses uniqueness backstops | CASE-STUDY EVIDENCE | ARGOS R34/R35 |

---

## Migrations / recovery evidence

| Claim | Status | Evidence |
|---|---|---|
| Append-only ordered migrations + migration table are a useful mature-app pattern | CASE-STUDY EVIDENCE | ARGOS, Omphalos, Saturno, DELPHIS |
| Deterministic migration record IDs improve idempotence | CASE-STUDY EVIDENCE | ARGOS, Omphalos/Saturno |
| Recording migration-body checksums improves schema identity evidence | CASE-STUDY EVIDENCE | ARGOS |
| Same-handle migration idempotence and process-restart durability are different claims | GENERAL TESTING RULE | Saturno vs ARGOS/DELPHIS |
| A documented recovery/rebuild design alone proves recovery works | FALSE | Execution evidence required |
| Recovery/rebuild tests should verify the same state properties claimed by the application | GENERAL TESTING RULE | Evidence discipline |

Choosing whether SurrealDB is authoritative, derived, one store among several, or paired with an outbox is **not a capability claim** and belongs in project architecture/planning rather than this ledger.

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
| DELPHIS also records native-build/resource failures separately from source/test failures | CASE-STUDY EVIDENCE | Native validation docs |

---

## Subprocess / filesystem boundary

| Claim | Status | Evidence |
|---|---|---|
| Worker-returned paths must not be trusted | GENERAL SECURITY RULE | Threat model + regression tests |
| Checking raw `symlink_metadata()` before canonicalization detects direct symlink output | TESTED BEHAVIOR | Astra Phase 3.1 tests |
| Canonicalized output must remain under canonical sandbox root | TESTED BEHAVIOR | Astra attack tests |
| `fastrand` is cryptographically secure | FALSE | It is not a CSPRNG |
| `fastrand` may be acceptable for collision avoidance when secrecy is not the boundary | PROJECT CONVENTION | Astra use |

---

## Fiscal XML / NF-e

| Claim | Status | Evidence |
|---|---|---|
| Fiscal monetary values should use exact decimals rather than `f32`/`f64` | GENERAL FINANCIAL RULE | Decimal tests |
| 44-digit access-key Modulo-11 verification is implemented in Astra Phase 5 | TESTED BEHAVIOR | Phase 5 parser tests |
| Raw `NFe` and `nfeProc` are structurally distinguished | TESTED BEHAVIOR | Phase 5 fixtures/tests |
| Parsed NF-e data is legally/fiscally validated by SEFAZ | NOT IMPLEMENTED | No live SEFAZ verification |
| XML signature chain is cryptographically verified | NOT IMPLEMENTED | No signature verification |
| NF-e multi-table materialization is transactionally all-or-nothing | OPEN / NOT YET PROVEN | Current PR uses sequential writes |
| R$ 0.02 tolerance is mandated by NF-e/SurrealDB | FALSE | Astra validation policy |

---

## Proving repositories scanned

### High-value SurrealDB evidence

- **Astra-bot** — remote WS + SurrealKV, tenant-isolation tests, document ingest, concurrency/restart, NF-e parser.
- **DLF merchanting operations** — Kaiju staged-ingest / Bronze patterns, tracked separately from this scan.
- **Brew & Batch** — external Tauri + embedded SurrealKV + SCHEMAFULL/query-shape case study.
- **Omphalos-git** — embedded boot/readiness, versioned schema, graph and agent-memory schema.
- **Saturno** — second embedded implementation, `Surreal<Db>` handle shape, schema-idempotence tests.
- **Alexandria** — revision/hash/checkpoint schema patterns and fresh-install validation evidence.
- **argos-commercial-operations** — graph, events, changefeeds, BM25/HNSW/RRF, transactions, checksummed migrations, restart/stress evidence.
- **delphis-intelligence-studio** — embedded adapter boundary, `take_errors()`, NONE/null handling, Value→JSON→Serde bridge, process-separated restart/stress.

### No new SurrealDB capability evidence found on indexed/default branches

- `daedalus-inventory`
- `goldennest`
- `thequietledger`
- `chronos-runtime-governor`
- portfolio/profile/GitHub Pages repositories

`castor-finance` surfaced legacy/design prose rather than strong runtime evidence. `emporion-commerce` currently uses a Postgres/sqlx Rust workspace rather than SurrealDB.

---

## Proving-project status (Astra)

### Proven / merged or exact-SHA green

- Tool Fabric typed request/response boundary.
- MarkItDown subprocess conversion.
- bounded Telegram file intake.
- worker-output containment and symlink rejection.
- document artifact persistence and unique source-hash dedupe.
- atomic checkpoint increments with conflict retry.
- SurrealKV restart persistence.
- deterministic NF-e parser and exact-decimal model on PR #8 exact green head.

### Known open issue

- NF-e root/child materialization remains sequential rather than one all-or-nothing database transaction. Do not describe Phase 5 persistence as transactionally atomic until failure-injection rollback proves it.

---

## Maintenance rule

Whenever a claim moves categories, update this ledger.

```text
CASE-STUDY EVIDENCE
  → isolate minimal reproducer / confirm official API
  → TESTED BEHAVIOR or VERIFIED API
```

```text
VERIFIED API for 3.2.4
  → dependency upgrade
  → re-check docs + compile/runtime
  → VERIFIED API for new baseline
```

Never preserve an old version claim simply because the old example still looks plausible. And do not promote project architecture decisions into technology-capability corrections.