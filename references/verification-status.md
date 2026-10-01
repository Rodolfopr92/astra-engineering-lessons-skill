# Verification Status Ledger

**Skill:** `surreal_rust_tauri`  
**Primary baseline:** SurrealDB server/engine 3.2.4 + Rust SDK 3.2.4  
**Tauri documentation baseline:** 2.11.5 path APIs  
**Rust reproducer toolchain:** 1.96.0  
**Last full verification:** 2026-09-12  
**Last partial verification:** 2026-10-01 (schema and query-shape reproducers; engine crate pinning)  
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

If a target repo uses SurrealDB server/engine or Rust SDK other than **3.2.4**, or a Tauri-sensitive claim targets a Tauri version other than **2.11.5**, the affected version-sensitive VERIFIED API / TESTED BEHAVIOR rows below become **UNVERIFIED FOR THAT TARGET** until checked for the target version.

Case-study evidence from another patch/minor version is not automatically transferable. A 3.2.3 observation can motivate a 3.2.4 reproducer, but does not become 3.2.4 TESTED BEHAVIOR by repetition.

## Independent reproducer evidence

The first reproducer run exposed an assertion-design flaw in the rollback test: the transaction rolled back the table creation too, so a subsequent `SELECT` failed because the table no longer existed. The test was corrected by defining the table outside the transaction.

The corrected suite passed all five tests on run **34691471041**, exact head **`a837f981cdfc78ad27573101373eff1b37cd2b33`**, using Rust **1.96.0** and `surrealdb = "=3.2.4"`.

The workflow itself was then hardened from deprecated `actions/checkout@v4` to commit-pinned checkout **v7.0.1** (`3d3c42e5aac5ba805825da76410c181273ba90b1`) with `contents: read` only. The complete five-test suite passed again on run **34691656259**, exact workflow head **`c91d41d938a6e643da69072700dcb40e1baf857e`**.

Source: `reproducers/surrealdb-3.2.4/tests/core_contract.rs`.

### 2026-10-01: schema and query shapes, engine pinning

Six findings from a proving repository (Astra, 2026-09-28 to 10-01) were reduced to `reproducers/surrealdb-3.2.4/tests/schema_and_query_shapes.rs`, which holds five tests. The error texts were first confirmed with the SurrealDB CLI `3.2.4+20260803.93ab219` against an in-memory engine.

**The reproducer now pins the engine.**

- **Before:** until 2026-10-01 it pinned only `surrealdb = "=3.2.4"`, with no `Cargo.lock`.
- **The gap:** the SDK reaches `surrealdb-core`, `surrealdb-types` and `surrealdb-types-derive` through caret requirements. 3.3.0 of those crates was published on 2026-09-24. A fresh resolution of the old manifest on 2026-10-01 gave SDK 3.2.4 on core, types and derive 3.3.0.
- **Earlier evidence stands:** the 2026-09-12 runs predate 3.3.0, and caret requirements do not select the 3.3.0 betas, so those runs resolved 3.2.4.
- **Now:** the three crates are pinned `=3.2.4`, `Cargo.lock` is committed, and CI runs `--locked`.

Local runs, 2026-10-01:

- Rust 1.98.1, all four crates resolved to 3.2.4.
- `cargo test --all-targets` and `cargo test --locked --all-targets` each passed 10 tests: `core_contract` 5 and `schema_and_query_shapes` 5.

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
| `(table, id)` tuple resources remain supported by Rust SDK methods | VERIFIED API | [Working with types](https://surrealdb.com/docs/reference/rust/concepts/working-with-types) |
| A row containing intrinsic `id` decodes into `RecordId`, while the same row does not decode into a struct declaring `id: String` | TESTED BEHAVIOR | Reproducer runs 34691471041 and 34691656259, `intrinsic_id_is_record_id_not_string` |
| `surrealdb = "=3.2.4"` alone does not pin `surrealdb-core` / `-types` / `-types-derive`; since 2026-09-24 a fresh resolution selects 3.3.0 for them | TESTED BEHAVIOR | `cargo generate-lockfile` on the unpinned reproducer manifest, 2026-10-01; crates.io publish dates |
| Pinning those three crates `=3.2.4` next to the SDK resolves the whole family to 3.2.4 | TESTED BEHAVIOR | `reproducers/surrealdb-3.2.4/Cargo.lock`; local runs 2026-10-01 |

---

## Query / response / binding boundary

| Claim | Status | Claim-level evidence |
|---|---|---|
| `.bind()` accepts SDK variable forms through `IntoVariables` / `SurrealValue` | VERIFIED API | [query `.bind()` docs](https://surrealdb.com/docs/reference/rust/methods/query) |
| Outer `.query(...).await` success can contain failing statements | VERIFIED API + TESTED BEHAVIOR | [Rust error handling](https://surrealdb.com/docs/reference/rust/concepts/error-handling); reproducer runs 34691471041 and 34691656259 |
| `.check()` surfaces statement errors | VERIFIED API | [Rust error handling](https://surrealdb.com/docs/reference/rust/concepts/error-handling) |
| `.take_errors()` preserves indexed statement failures | VERIFIED API + TESTED BEHAVIOR | [Rust error handling](https://surrealdb.com/docs/reference/rust/concepts/error-handling); reproducer runs 34691471041 and 34691656259 |
| Durable control flow should prefer structured error kinds over message matching | VERIFIED API | [Rust error handling](https://surrealdb.com/docs/reference/rust/concepts/error-handling) |
| `<record>$variable` is universally required for bound dynamic records | FALSE / NOT A GENERAL RULE | Brew & Batch observation only; typed `RecordId` and `type::record()` are also supported |
| UUID-shaped text record IDs may render with backtick delimiters when cast to string | CASE-STUDY EVIDENCE | ARGOS record-identity tests; version-specific transport behavior |
| `ORDER BY` a field missing from the projection is a parse error (``Missing order idiom `x` in statement selection``) that fails the whole request, so its earlier valid statements do not run | TESTED BEHAVIOR | CLI 3.2.4; reproducer `order_by_needs_its_field_selected_and_the_parse_error_fails_the_whole_request`, local runs 2026-10-01 |
| Timestamps serialized through `serde_json` are stored as strings and silently fail datetime comparisons; a `<datetime>` cast matches them | TESTED BEHAVIOR | Reproducer `json_timestamps_are_strings_and_drop_out_of_datetime_comparisons`, local runs 2026-10-01 |
| A `TYPE datetime` field refuses a JSON timestamp string instead of storing it | TESTED BEHAVIOR | CLI 3.2.4; same reproducer (SDK `.create().content()` path), local runs 2026-10-01 |
| `UPSERT … MERGE` keeps fields absent from the payload; `UPSERT … CONTENT` replaces the record | TESTED BEHAVIOR | Reproducer `upsert_merge_keeps_absent_fields_while_content_replaces_the_record`, local runs 2026-10-01 |
| A lookup ordering by an unselected field shipped and failed on every call without surfacing the error | CASE-STUDY EVIDENCE | Astra message-log lookup, found in review, 2026-09 |

---

## Embedded SurrealKV + Tauri

| Claim | Status | Claim-level evidence |
|---|---|---|
| Rust SDK supports embedded database operation | VERIFIED API | [Rust embedding docs](https://surrealdb.com/docs/reference/rust/embedding) |
| SurrealKV is available behind `kv-surrealkv` in 3.2.4 | VERIFIED API | [surrealdb 3.2.4 crate](https://docs.rs/crate/surrealdb/3.2.4) |
| `Surreal::new::<SurrealKv>(...)` can be assigned to a local `Surreal<Db>` handle and queried | TESTED BEHAVIOR | Reproducer runs 34691471041 and 34691656259, `surrealkv_engine_selector_produces_local_db_handle` |
| SurrealKV historical versioning is opt-in via `.versioned()` | VERIFIED API | [`new()` / versioned backend](https://surrealdb.com/docs/reference/rust/methods/new) |
| Tauri 2.11.5 exposes `app_data_dir()` | VERIFIED API | [Tauri 2.11.5 PathResolver](https://docs.rs/tauri/2.11.5/tauri/path/struct.PathResolver.html) |
| Tauri 2.11.5 exposes `app_local_data_dir()` | VERIFIED API | [Tauri 2.11.5 PathResolver](https://docs.rs/tauri/2.11.5/tauri/path/struct.PathResolver.html) |
| Every Tauri product should choose AppData rather than AppLocalData | FALSE / NOT A GENERAL RULE | Product/platform decision |
| Same-handle schema reapplication proves process restart durability | FALSE | It proves idempotence, not process restart |
| Process-separated writer/reader tests provide stronger embedded durability evidence | CASE-STUDY EVIDENCE / TESTING RULE | ARGOS + DELPHIS native validation systems |

---

## SCHEMAFULL / relation schema

| Claim | Status | Claim-level evidence |
|---|---|---|
| SCHEMAFULL object fields are strict by default | VERIFIED API + TESTED BEHAVIOR | [DEFINE FIELD](https://surrealdb.com/docs/reference/query-language/statements/define/field); reproducer runs 34691471041 and 34691656259 |
| `FLEXIBLE` permits undeclared keys in object-containing fields | VERIFIED API + TESTED BEHAVIOR | [DEFINE FIELD](https://surrealdb.com/docs/reference/query-language/statements/define/field); reproducer runs 34691471041 and 34691656259 |
| As of 3.0, undeclared nested SCHEMAFULL fields error rather than being silently omitted | VERIFIED API + TESTED BEHAVIOR | [DEFINE FIELD 3.x behavior](https://surrealdb.com/docs/reference/query-language/statements/define/field); reproducer runs 34691471041 and 34691656259 |
| `TYPE RELATION FROM ... TO ...` is current relation-table syntax | VERIFIED API | [DEFINE TABLE](https://surrealdb.com/docs/reference/query-language/statements/define/table) |
| Every relation edge should be unique by `(in,out)` | FALSE / NOT A GENERAL RULE | Domain-dependent integrity rule |
| Fresh-install execution can expose nested-schema gaps hidden by long-lived dev state | CASE-STUDY EVIDENCE / TESTING RULE | Brew & Batch validation work |
| SCHEMAFULL refuses an undefined top-level field (`Found field 'x', but no such field exists for table 'y'`) | TESTED BEHAVIOR | CLI 3.2.4; reproducer `schemafull_refuses_an_undefined_top_level_field_an_undefined_table_hides_it`, local runs 2026-10-01 |
| With default settings, a write to a table that was never defined succeeds with any field, so such a fixture cannot detect SCHEMAFULL refusals | TESTED BEHAVIOR | Same reproducer, local runs 2026-10-01 |
| SCHEMALESS tables enforce `TYPE` / `ASSERT` on the fields they define; `option<string>` with `ASSERT $value = NONE OR $value IN [...]` is an optional enumeration | TESTED BEHAVIOR | CLI 3.2.4; reproducer `schemaless_tables_still_enforce_the_fields_they_define`, local runs 2026-10-01 |
| A missing SCHEMAFULL field migration passed tests that created rows in an undefined table, and failed in production | CASE-STUDY EVIDENCE | Astra per-user language preference on a SCHEMAFULL `user` table, 2026-09 |

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
| Explicit transactions are all-or-nothing and can be aborted | VERIFIED API + TESTED BEHAVIOR | [Transactions](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions); reproducer runs 34691471041 and 34691656259 |
| Sequential independent `upsert().await?` calls are one transaction | FALSE | Separate calls are not one ACID unit |
| Failure injection is appropriate evidence for multi-record rollback claims | GENERAL TESTING RULE | Reproducer demonstrates deterministic rollback assertion |
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