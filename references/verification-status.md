# Verification Status Ledger

**Skill:** `surreal_rust_tauri`  
**Primary baseline:** SurrealDB server/engine 3.2.4 + Rust SDK 3.2.4  
**Tauri documentation baseline:** 2.11.5 path APIs  
**Rust reproducer toolchain:** 1.96.0  
**Last full verification:** 2026-09-12  
**Last partial verification:** 2026-10-30 (staged-code pitfalls reproducers; earlier: 2026-10-12 `ASSERT` widening, 2026-10-01 schema/query shapes and engine pinning)  
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
- **Correction (2026-10-08):** three pins were not enough. `surrealdb-core` 3.2.4 depends on `surrealdb-collections` and `surrealdb-strand` through `^3.2.4`, and the committed lock had resolved both to 3.3.0. So the 2026-10-01 runs below, local and CI 36897934223, ran core 3.2.4 on collections and strand 3.3.0. Both are now pinned `=3.2.4` too; see the 2026-10-08 runs.

Local runs, 2026-10-01:

- Rust 1.98.1, all four crates resolved to 3.2.4.
- `cargo test --all-targets` and `cargo test --locked --all-targets` each passed 10 tests: `core_contract` 5 and `schema_and_query_shapes` 5.

GitHub Actions, 2026-10-01:

- Run **36897934223**, exact head **`f029a0a0446661855ce01f9d210ac4c0fbc41890`**.
- Rust 1.96.0, with `--locked`. The log shows surrealdb, surrealdb-core, surrealdb-types and surrealdb-types-derive compiled at 3.2.4.
- Result: 10 passed, 0 failed.

### 2026-10-08: the five-crate engine pin

Local run, 2026-10-08:

- Rust 1.98.1, with `--locked`. The lock resolves surrealdb, surrealdb-core, -types, -types-derive, -collections and -strand to 3.2.4; `surrealdb-protocol` stays at 0.10.2.
- `cargo test --locked --all-targets` passed 10 tests: `core_contract` 5 and `schema_and_query_shapes` 5.

GitHub Actions, 2026-10-08:

- Run **37824036016**, exact head **`79b51bc830869bb10fa29aa71c7eb67ed4dd0a37`**.
- Rust 1.96.0, with `--locked`. The log shows surrealdb-collections, surrealdb-strand, surrealdb-core, surrealdb-types, surrealdb-types-derive, surrealdb compiled at 3.2.4.
- Result: 10 passed, 0 failed.

---

### 2026-10-12: widening an `ASSERT`

An audit-table incident in the proving repository was reduced to `if_not_exists_cannot_widen_an_existing_assert_overwrite_can` in `schema_and_query_shapes.rs`.

```text
date: 2026-10-12
SurrealDB CLI: 3.2.4+20260803.93ab219 (in-memory engine) — behavior confirmed before writing the test
Rust: 1.98.1 (local), --locked
cargo test --locked --test schema_and_query_shapes: 6 passed, 0 failed
```

GitHub Actions, 2026-10-12:

- Run **38314604229**, exact head **`37006d4260b4e4b99e52767cf09174cadfbb79d6`**.
- Rust 1.96.0, with `--locked`. The log shows surrealdb-collections, surrealdb-strand, surrealdb-core, surrealdb-types, surrealdb-types-derive, surrealdb compiled at 3.2.4.
- Result: `core_contract` 5 passed, `schema_and_query_shapes` 6 passed, 0 failed.

### 2026-10-30: staged-code pitfalls (DLF Block 04)

Fourteen repairs made while integrating a block of SurrealQL + SDK code that had never been
compiled (DLF Block 04, merged through PR #7) were reduced to
`reproducers/surrealdb-3.2.4/tests/staged_code_pitfalls.rs`, seven tests. Two corrected the
integration record itself: only four parameter names are protected (crate source
`PROTECTED_PARAM_NAMES`), and a UNIQUE index over rows that lack the field is accepted. The SDK
facts (`IndexedResults`, `check(self)`, `take_errors() -> HashMap`, no `Display` on `RecordId`)
were read from the 3.2.4 crate sources in the Cargo registry and are pinned by the file compiling.
Narrative: [integrating-staged-uncompiled-code.md](integrating-staged-uncompiled-code.md).

```text
date: 2026-10-30
Rust: 1.97.0 (local), --locked
resolved: surrealdb, surrealdb-core, -types, -types-derive, -collections, -strand = 3.2.4 (surrealdb-protocol 0.10.2)
cargo test --locked --all-targets: core_contract 5, schema_and_query_shapes 6, staged_code_pitfalls 7; 18 passed, 0 failed
```

GitHub Actions: pending (the workflow runs on pull requests and on pushes to main); record the
run id and head here when it runs.

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
| Pinning those three crates `=3.2.4` next to the SDK resolves the whole family to 3.2.4 | FALSE (corrected 2026-10-08) | The lock kept `surrealdb-collections` and `surrealdb-strand` at 3.3.0; see the next row. Original evidence: `reproducers/surrealdb-3.2.4/Cargo.lock`; local runs 2026-10-01 and CI run 36897934223 |
| `surrealdb-core` 3.2.4 depends on `surrealdb-collections` and `surrealdb-strand` through `^3.2.4`; pinning all five crates (core, types, types-derive, collections, strand) resolves the engine to 3.2.4 | TESTED BEHAVIOR | crates.io dependency metadata for surrealdb-core 3.2.4; `reproducers/surrealdb-3.2.4/Cargo.lock`; local run 2026-10-08 and CI run 37824036016 |
| `db.select()` requires `SurrealValue`; domain types deriving only Serde must select into `Option<serde_json::Value>` | VERIFIED API + CASE-STUDY EVIDENCE | [Rust SDK docs](https://docs.rs/surrealdb/3.2.4/surrealdb/struct.Surreal.html#method.select); Astra contract gateway |
| The query response type in 3.2.4 is `surrealdb::IndexedResults`; `surrealdb::Response` (2.x) does not exist | VERIFIED API | crate source `surrealdb-3.2.4/src/lib.rs`; `staged_code_pitfalls.rs` imports it, local run 2026-10-30 |
| `IndexedResults::check(self) -> Result<Self>` consumes the response, so `q.check()?; q.take(0)` is E0382 | VERIFIED API + CASE-STUDY EVIDENCE | crate source `src/method/query.rs`; DLF Block 04 patch B04-P008 |
| `RecordId { table, key }` implements `ToSql` and `Debug` but not `Display`; `to_sql()` gives `item:sku1`, `key.to_sql()` gives `sku1` | VERIFIED API + TESTED BEHAVIOR | no `Display` impl in `surrealdb-types-3.2.4`; reproducer `record_id_text_comes_from_record_id_in_sql_or_to_sql_in_rust`, local run 2026-10-30; DLF B04-P007 (E0599) |

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
| `ORDER BY` a field missing from the projection is a parse error (``Missing order idiom `x` in statement selection``) that fails the whole request, so its earlier valid statements do not run | TESTED BEHAVIOR | CLI 3.2.4; reproducer `order_by_needs_its_field_selected_and_the_parse_error_fails_the_whole_request`, local runs 2026-10-01 and CI run 36897934223 |
| Timestamps serialized through `serde_json` are stored as strings and silently fail datetime comparisons; a `<datetime>` cast matches them | TESTED BEHAVIOR | Reproducer `json_timestamps_are_strings_and_drop_out_of_datetime_comparisons`, local runs 2026-10-01 and CI run 36897934223 |
| A `TYPE datetime` field refuses a JSON timestamp string instead of storing it | TESTED BEHAVIOR | CLI 3.2.4; same reproducer (SDK `.create().content()` path), local runs 2026-10-01 and CI run 36897934223 |
| `UPSERT … MERGE` keeps fields absent from the payload; `UPSERT … CONTENT` replaces the record | TESTED BEHAVIOR | Reproducer `upsert_merge_keeps_absent_fields_while_content_replaces_the_record`, local runs 2026-10-01 and CI run 36897934223 |
| A lookup ordering by an unselected field shipped and failed on every call without surfacing the error | CASE-STUDY EVIDENCE | Astra message-log lookup, found in review, 2026-09 |
| Escaping line-continuation backslashes in multi-line query strings (`\\`) causes `Parse error: Invalid token '\'`; single `\` or multi-line strings must be used | CASE-STUDY EVIDENCE | Astra contract gateway live queries |
| `take_errors()` is a `HashMap<usize, Error>`; in a failed transaction the statements before the cause say "not executed due to a failed transaction", later ones "… cancelled transaction", `COMMIT` "Cannot COMMIT: the transaction was aborted due to a prior error"; the cause is the lowest non-cascade index | VERIFIED API + TESTED BEHAVIOR | crate source `src/method/query.rs`; reproducer `take_errors_is_unordered_and_the_cause_hides_among_cascade_errors`, local run 2026-10-30; DLF B04-P010 |
| Exactly `$access`, `$auth`, `$token`, `$session` are protected: `LET` and `.bind()` fail with "'x' is a protected variable and cannot be set" | VERIFIED API + TESTED BEHAVIOR | `surrealdb-core-3.2.4/src/cnf/mod.rs` `PROTECTED_PARAM_NAMES`; reproducer `the_four_protected_parameter_names_are_refused_by_let_and_by_bind`, local run 2026-10-30; DLF B04-P009 |
| `$value` and `$this` are accepted by `LET` (context-defined, not protected), so a collision shadows silently | TESTED BEHAVIOR | same reproducer, local run 2026-10-30 |
| `IF … ELSE IF … ELSE … END` closes with one `END`; a second `END` is "Parse error: Unexpected token `END`, expected Eof" and fails the whole request | TESTED BEHAVIOR | reproducer `an_if_else_if_chain_closes_with_one_end`, local run 2026-10-30; DLF B04-P002 |
| `ORDER BY` accepts projected idioms only; an `IF` expression after `ORDER BY` is a parse error, a projected alias orders correctly | TESTED BEHAVIOR | reproducer `order_by_takes_projected_idioms_not_expressions`, local run 2026-10-30; DLF B04-P003 |

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
| SCHEMAFULL refuses an undefined top-level field (`Found field 'x', but no such field exists for table 'y'`) | TESTED BEHAVIOR | CLI 3.2.4; reproducer `schemafull_refuses_an_undefined_top_level_field_an_undefined_table_hides_it`, local runs 2026-10-01 and CI run 36897934223 |
| With default settings, a write to a table that was never defined succeeds with any field, so such a fixture cannot detect SCHEMAFULL refusals | TESTED BEHAVIOR | Same reproducer, local runs 2026-10-01 and CI run 36897934223 |
| SCHEMALESS tables enforce `TYPE` / `ASSERT` on the fields they define; `option<string>` with `ASSERT $value = NONE OR $value IN [...]` is an optional enumeration | TESTED BEHAVIOR | CLI 3.2.4; reproducer `schemaless_tables_still_enforce_the_fields_they_define`, local runs 2026-10-01 and CI run 36897934223 |
| A missing SCHEMAFULL field migration passed tests that created rows in an undefined table, and failed in production | CASE-STUDY EVIDENCE | Astra per-user language preference on a SCHEMAFULL `user` table, 2026-09 |
| A SCHEMAFULL field `TYPE record<tx>` refuses `NONE` and an omitted field ("Couldn't coerce value for field … Expected `record<tx>` but found `NONE`"); `option<record<tx>>` accepts absence | TESTED BEHAVIOR | reproducer `a_schemafull_record_link_field_refuses_none_unless_declared_option`, local run 2026-10-30; DLF B04-P004 |
| A UNIQUE index defined over rows lacking the field is accepted, later rows lacking it are accepted, duplicates fail the definition, and the index enforces afterwards | TESTED BEHAVIOR | reproducer `a_unique_index_ignores_absent_values_so_normalize_rows_before_defining_it`, local run 2026-10-30 |
| Defining a UNIQUE index while rows lack the field fails | FALSE / NOT A GENERAL RULE | DLF Block 04 register assumed it (B04-P006); the reproducer above shows it is accepted in 3.2.4 |

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
| A data-rewriting migration run outside the transactional path left normalized rows behind when a later statement failed | CASE-STUDY EVIDENCE | DLF Block 04 first populated-upgrade run, patch B04-P011 |
| Normalize rows before defining a new UNIQUE index, so a duplicate fails at the definition with consistent data | TESTED BEHAVIOR (index behavior) + CASE-STUDY EVIDENCE (ordering) | reproducer `a_unique_index_ignores_absent_values_so_normalize_rows_before_defining_it`; DLF B04-P006 |

Architecture choices such as whether SurrealDB is authoritative, derived, one store among several, or paired with an outbox are intentionally **out of scope** for this capability ledger.

---

## SCHEMA evolution (2026-10-12)

| Claim | Status | Claim-level evidence |
|---|---|---|
| `DEFINE FIELD IF NOT EXISTS` with a different `ASSERT` is a silent no-op on an existing field; `OVERWRITE` replaces it | TESTED BEHAVIOR | `if_not_exists_cannot_widen_an_existing_assert_overwrite_can`; local run 2026-10-12 and CI run 38314604229 |

## Multi-tenancy / session isolation

| Claim | Status | Claim-level evidence |
|---|---|---|
| Sharing cloned `Surreal<Ws>` handles across concurrent tasks races internal SDK authentication replay | TESTED BEHAVIOR | Astra concurrent WebSocket tenant worker tests |
| Multi-tenant integration tests on shared servers collide when using static tenant slugs; tests require dynamic unique slugs | CASE-STUDY EVIDENCE | Astra multi-tenant integration test suite (`support::unique`) |

---

## Build / verification state

| Claim | Status | Claim-level evidence |
|---|---|---|
| A build killed by OOM/quota/sandbox termination before compiler diagnostics is a pass | FALSE | No completed build evidence |
| The same event automatically proves application source failure | FALSE | Infrastructure may terminate first |
| `BLOCKED / INDETERMINATE` is a valid evidence state | GENERAL EVIDENCE RULE | Reproducibility discipline |
| Standard GitHub-hosted Actions runners are free for public repositories | VERIFIED API | [GitHub Actions billing](https://docs.github.com/en/actions/concepts/billing-and-usage) |
| Cargo test filter matches test names by substring; tests without matching prefix silently skip and exit code 0 | CASE-STUDY EVIDENCE | Astra contract gateway integration tests (`live_tests` filter) |
| Brittle single-line regex assertions in verification scripts break when `cargo fmt` reflows code into multi-line formatting | CASE-STUDY EVIDENCE | Astra pre-commit / test verification harness |
| `#[serde(default)]` does not omit fields during serialization; missing `skip_serializing_if` emits empty values altering cryptographic provenance digests | CASE-STUDY EVIDENCE | Astra audited contract event provenance hashing |
| A test that returns early when its environment variable is missing is reported `ok`; a live stage filtered by `live_tests` misses `live_db_tests` / `live_session_tests` modules | CASE-STUDY EVIDENCE | Astra gate review 2026-10-11 (seven live tests never ran) |
| `git stash` is shared by all worktrees; `stash push -- <paths>` with no changes creates nothing, and a following `pop` applies and drops another worktree's stash | TESTED BEHAVIOR | git 2.43.0 throwaway repository, 2026-10-12; Astra incident 2026-10-10 |
| Unprivileged `bwrap --uid N` maps the caller inside a user namespace; outside, the process keeps the caller's uid | TESTED BEHAVIOR | bubblewrap 0.9.0, `id` inside vs `ps` outside, 2026-10-09 |
| Code written and statically "refined" without compiling needed fourteen repairs (parse, protected name, schema, index, SDK API, migration order and path, guard mismatch, Clippy limits) before its first test passed | CASE-STUDY EVIDENCE | DLF Block 04 patch register, 2026-10-29 |
| Clippy `too_many_arguments` fires above seven parameters and fails a `-D warnings` build; grouping into a struct keeps behavior without suppression | VERIFIED API + CASE-STUDY EVIDENCE | Clippy lint docs (default threshold 7); DLF B04-P013 |
| Futures passed to `tokio::join!` that borrow temporaries fail with E0716; bind the values with `let` first | CASE-STUDY EVIDENCE | DLF Block 04 acceptance-test fixtures |
| A test asserting a literal registry size (15 migrations) breaks when the next migration lands; compare with `MIGRATIONS.len()` | CASE-STUDY EVIDENCE | DLF B04-P014 |
| A package's "cumulative corrections" may already exist in the integrated tree; re-applying them duplicates behavior | CASE-STUDY EVIDENCE | DLF Block 04 sections D and E (would have duplicated PICK tasks) |

---

## Fiscal XML / NF-e

| Claim | Status | Claim-level evidence |
|---|---|---|
| Fiscal monetary values should use exact decimal arithmetic | GENERAL FINANCIAL RULE | Astra parser tests / deterministic arithmetic requirement |
| Astra implements 44-digit NF-e key validation | TESTED BEHAVIOR | Astra Phase 5 tests |
| Astra structurally distinguishes raw `NFe` and `nfeProc` | TESTED BEHAVIOR | Astra Phase 5 fixtures/tests |
| Parsed NF-e data is legally validated by SEFAZ | NOT IMPLEMENTED | No live SEFAZ verification |
| NF-e XML signature chain is cryptographically verified | NOT IMPLEMENTED | No signature-chain validation |
| NF-e issuer signature uses inclusive C14N 1.0, rsa-sha1 and sha1, fixed by the official schema | VERIFIED API | MOC (NT 2026.007) §4.2.4; PL_010_V1.30 `xmldsig-core-schema_v1.01.xsd` |
| `protNFe` `cStat 100` inside a file proves SEFAZ authorization | FALSE | `digVal` is a public copy and the protocol signature is optional; only an online check confirms authorization or cancellation |
| C14N drops comments, so a comment inside a signed value keeps the signature valid while first-text-node readers see a different value | CASE-STUDY EVIDENCE | Astra signature prototype, 2026-10-10 |
| lxml `tostring(el, method="c14n")` on a sub-element adds `xmlns=""` to nested default-namespace elements | CASE-STUDY EVIDENCE | lxml 6.1.3 / libxml2 2.14.6, Astra prototype, 2026-10-10 |
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