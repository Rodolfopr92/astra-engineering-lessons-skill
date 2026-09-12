# Verification Status Ledger

**Skill:** `surreal_rust_tauri`  
**Primary baseline:** SurrealDB server/engine 3.2.4 + Rust SDK 3.2.4  
**Last full verification:** 2026-09-12  
**Routine expiry:** 90 days maximum, or immediately on target-version mismatch

This ledger exists to stop the skill from becoming the stale prior it was created to correct.

## Evidence classes

- **VERIFIED API** — supported by official documentation/API for the named baseline.
- **TESTED BEHAVIOR** — independently reproduced against the named baseline.
- **CASE-STUDY EVIDENCE** — observed in a proving repository, not independently minimized here.
- **PROJECT CONVENTION** — a project choice, not a technology requirement.
- **ARCHITECTURAL INTENT** — planned/recommended, not implementation evidence.
- **FALSE / NOT A GENERAL RULE** — retained to prevent known over-generalization.

## Version-transfer rule

If a target repo uses SurrealDB server/engine or Rust SDK other than **3.2.4**, all version-sensitive VERIFIED API / TESTED BEHAVIOR rows below become **UNVERIFIED FOR THAT TARGET** until checked for the target version.

Case-study evidence from another patch/minor version is not automatically transferable. A 3.2.3 observation can motivate a 3.2.4 reproducer, but does not become 3.2.4 TESTED BEHAVIOR by repetition.

---

## SurrealDB Rust SDK / type contract

| Claim | Status | Claim-level evidence |
|---|---|---|
| Current official Rust SDK/server release is 3.2.4 | VERIFIED API | [Rust SDK docs](https://surrealdb.com/docs/reference/rust) |
| Official Rust SDK minimum Rust version is 1.89 | VERIFIED API | [Rust SDK docs](https://surrealdb.com/docs/reference/rust) |
| `SurrealValue` is the native Rust conversion trait | VERIFIED API | [Working with types](https://surrealdb.com/docs/reference/rust/concepts/working-with-types) |
| `#[surreal(...)]` is distinct from Serde attributes | VERIFIED API | [SurrealValue attributes](https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes) |
| Intrinsic record identifiers use `RecordId` | VERIFIED API | [RecordId 3.2.4](https://docs.rs/surrealdb/3.2.4/surrealdb/types/record_id/struct.RecordId.html) |
| `type::record()` replaced pre-3.0 `type::thing()` | VERIFIED API | [type::record docs](https://surrealdb.com/docs/reference/query-language/functions/database-functions/type) |
| `(table, id)` tuple resources remain supported by Rust SDK methods | VERIFIED API | [Working with types / resource examples](https://surrealdb.com/docs/reference/rust/concepts/working-with-types) |
| Deserializing intrinsic `id` into `String` can fail with a record/string type mismatch | TESTED BEHAVIOR | Astra 3.2.4 restart integration evidence; reproducer being added under `reproducers/` |

---

## Query / response / binding boundary

| Claim | Status | Claim-level evidence |
|---|---|---|
| `.bind()` accepts SDK variable forms through `IntoVariables` / `SurrealValue` | VERIFIED API | [query `.bind()` docs](https://surrealdb.com/docs/reference/rust/methods/query) |
| Outer `.query(...).await` success can contain failing statements | VERIFIED API | [Rust error handling](https://surrealdb.com/docs/reference/rust/concepts/error-handling) |
| `.check()` surfaces statement errors | VERIFIED API | [Rust error handling](https://surrealdb.com/docs/reference/rust/concepts/error-handling) |
| `.take_errors()` preserves indexed statement failures | VERIFIED API | [Rust error handling](https://surrealdb.com/docs/reference/rust/concepts/error-handling) |
| Durable control flow should prefer structured error kinds over message matching | VERIFIED API | [Rust error handling](https://surrealdb.com/docs/reference/rust/concepts/error-handling) |
| `<record>$variable` is universally required for bound dynamic records | FALSE / NOT A GENERAL RULE | Brew & Batch observation only; typed `RecordId` and `type::record()` are also supported |
| UUID-shaped text record IDs may render with backtick delimiters when cast to string | CASE-STUDY EVIDENCE | ARGOS record-identity tests; version-specific transport behavior |

---

## Embedded SurrealKV + Tauri

| Claim | Status | Claim-level evidence |
|---|---|---|
| Rust SDK supports embedded database operation | VERIFIED API | [Rust embedding docs](https://surrealdb.com/docs/reference/rust/embedding) |
| SurrealKV is available behind `kv-surrealkv` in current crate line | VERIFIED API | [surrealdb 3.2.4 crate](https://docs.rs/crate/surrealdb/3.2.4) |
| `Surreal::new::<SurrealKv>(...)` can produce a local handle used as `Surreal<Db>` | CASE-STUDY EVIDENCE | Omphalos, Saturno, ARGOS, DELPHIS; target-version compile reproducer added under `reproducers/` |
| SurrealKV historical versioning is opt-in via `.versioned()` | VERIFIED API | [`new()` / versioned backend](https://surrealdb.com/docs/reference/rust/methods/new) |
| Tauri 2 exposes `app_data_dir()` | VERIFIED API | [Tauri PathResolver](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html) |
| Tauri 2 exposes `app_local_data_dir()` | VERIFIED API | [Tauri PathResolver](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html) |
| Every Tauri product should choose AppData rather than AppLocalData | FALSE / NOT A GENERAL RULE | Product/platform decision |
| Same-handle schema reapplication proves process restart durability | FALSE | It proves idempotence, not process restart |
| Process-separated writer/reader tests provide stronger embedded durability evidence | CASE-STUDY EVIDENCE / TESTING RULE | ARGOS + DELPHIS native validation systems |

---

## SCHEMAFULL / relation schema

| Claim | Status | Claim-level evidence |
|---|---|---|
| SCHEMAFULL object fields are strict by default | VERIFIED API | [DEFINE FIELD](https://surrealdb.com/docs/reference/query-language/statements/define/field) |
| `FLEXIBLE` permits undeclared keys in object-containing fields | VERIFIED API | [DEFINE FIELD](https://surrealdb.com/docs/reference/query-language/statements/define/field) |
| As of 3.0, undeclared nested SCHEMAFULL fields error rather than being silently omitted | VERIFIED API | [DEFINE FIELD 3.x behavior note](https://surrealdb.com/docs/reference/query-language/statements/define/field) |
| `TYPE RELATION FROM ... TO ...` is current relation-table syntax | VERIFIED API | [DEFINE TABLE](https://surrealdb.com/docs/reference/query-language/statements/define/table) |
| Every relation edge should be unique by `(in,out)` | FALSE / NOT A GENERAL RULE | Domain-dependent integrity rule |
| Fresh-install execution can expose nested-schema gaps hidden by long-lived dev state | CASE-STUDY EVIDENCE / TESTING RULE | Brew & Batch and Alexandria validation work |

---

## Search / temporal / changefeeds

| Claim | Status | Claim-level evidence |
|---|---|---|
| Pre-3.0 `SEARCH ANALYZER` became `FULLTEXT ANALYZER` | VERIFIED API | [DEFINE overview](https://surrealdb.com/docs/reference/query-language/statements/define/overview) |
| `search::score()` is current | VERIFIED API | [Search functions](https://surrealdb.com/docs/reference/query-language/functions/database-functions/search) |
| `search::rrf()` is current | VERIFIED API | [Search functions](https://surrealdb.com/docs/reference/query-language/functions/database-functions/search) |
| Current SurrealDB supports HNSW KNN vector search | VERIFIED API | [Hybrid/vector search docs](https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search) |
| BM25 and vector candidate lists can be fused with RRF | VERIFIED API | [Hybrid search docs](https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search) |
| ARGOS implements BM25 + HNSW + RRF | CASE-STUDY EVIDENCE | ARGOS used 3.2.3; must not be promoted automatically to 3.2.4 TESTED BEHAVIOR |
| `CHANGEFEED` / `SHOW CHANGES ... SINCE` are mutation-history mechanisms | VERIFIED API | [Changefeeds](https://surrealdb.com/docs/learn/querying/real-time/changefeeds) |
| Changefeeds and `SELECT ... VERSION` are the same mechanism | FALSE | Separate APIs and guarantees |
| Historical `VERSION` reads require versioning-enabled supported storage | VERIFIED API | [`new().versioned()`](https://surrealdb.com/docs/reference/rust/methods/new) |

---

## Transactions / atomicity

| Claim | Status | Claim-level evidence |
|---|---|---|
| Explicit transactions are all-or-nothing and can be aborted | VERIFIED API | [Transactions](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions) |
| Sequential independent `upsert().await?` calls are one transaction | FALSE | Separate calls are not one ACID unit |
| Failure injection is appropriate evidence for multi-record rollback claims | GENERAL TESTING RULE | Evidence-strength discipline |
| Server-side counter mutation can avoid application read-modify-write lost updates | TESTED BEHAVIOR | Astra 3.2.4 concurrent checkpoint tests |

---

## Migrations / recovery evidence

| Claim | Status | Claim-level evidence |
|---|---|---|
| Ordered append-only migration history is a useful mature-app pattern | CASE-STUDY EVIDENCE | ARGOS, Omphalos, Saturno, DELPHIS |
| Deterministic migration record identity improves idempotence | CASE-STUDY EVIDENCE | ARGOS, Omphalos/Saturno |
| Migration-body checksums improve schema identity evidence | CASE-STUDY EVIDENCE | ARGOS |
| Same-handle migration idempotence and process-restart durability are different claims | GENERAL TESTING RULE | Saturno vs ARGOS/DELPHIS evidence |
| A documented rebuild/recovery design alone proves recovery works | FALSE | Execution evidence required |

Architecture choices such as whether SurrealDB is authoritative, derived, one store among several, or paired with an outbox are intentionally **out of scope** for this capability ledger.

---

## Build / verification state

| Claim | Status | Claim-level evidence |
|---|---|---|
| A build killed by OOM/quota/sandbox termination before compiler diagnostics is a pass | FALSE | No completed build evidence |
| The same event automatically proves application source failure | FALSE | Infrastructure may terminate first |
| `BLOCKED / INDETERMINATE` is a valid evidence state | GENERAL EVIDENCE RULE | Reproducibility discipline |
| Standard GitHub-hosted Actions runners are free for public repositories | VERIFIED API | [GitHub Actions billing](https://docs.github.com/en/actions/concepts/billing-and-usage) |

---

## Fiscal XML / NF-e

| Claim | Status | Claim-level evidence |
|---|---|---|
| Fiscal monetary values should use exact decimal arithmetic | GENERAL FINANCIAL RULE | Astra parser tests / deterministic arithmetic requirement |
| Astra implements 44-digit NF-e key validation | TESTED BEHAVIOR | Astra Phase 5 tests |
| Astra structurally distinguishes raw `NFe` and `nfeProc` | TESTED BEHAVIOR | Astra Phase 5 fixtures/tests |
| Parsed NF-e data is legally validated by SEFAZ | NOT IMPLEMENTED | No live SEFAZ verification |
| NF-e XML signature chain is cryptographically verified | NOT IMPLEMENTED | No signature-chain validation |
| NF-e multi-table materialization is transactionally all-or-nothing | OPEN / NOT YET PROVEN | Current Phase 5 path remains sequential until rollback proof exists |

---

## Maintenance rule

When evidence changes category, update this ledger.

```text
CASE-STUDY EVIDENCE
  → exact-version reproducer / official API confirmation
  → TESTED BEHAVIOR or VERIFIED API
```

```text
VERIFIED API for 3.2.4
  → target changes version
  → UNVERIFIED FOR TARGET
  → re-check docs + reproducer
  → VERIFIED for new baseline only after evidence lands
```

Do not preserve a claim merely because it still looks plausible.