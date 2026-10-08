# Reproducers

This directory turns high-value capability claims into small executable experiments.

## Evidence rule

Source code alone is **not** TESTED BEHAVIOR.

A reproducer becomes evidence only when its exact baseline has run successfully and the verification ledger records the result.

## Current suite

### `surrealdb-3.2.4/`

Target:

```text
SurrealDB Rust SDK: =3.2.4
engine crates (surrealdb-core, -types, -types-derive, -collections, -strand): =3.2.4, Cargo.lock committed
engine modes: Mem + embedded SurrealKV
Rust verification toolchain: 1.96.0 (CI)
```

Checks in `tests/core_contract.rs`:

- intrinsic `id` decodes as `RecordId`, while a `String` model rejects the same row;
- statement-level failures can exist inside an outer successful query response;
- SCHEMAFULL rejects undeclared nested fields while `FLEXIBLE` accepts intentional dynamic keys;
- `SurrealKv` selects the embedded engine while the application handle is `Surreal<Db>`;
- explicit transaction failure does not leave the earlier write committed.

Checks in `tests/schema_and_query_shapes.rs` (added 2026-10-01):

- **The fixture trap.** SCHEMAFULL refuses an undefined top-level field, while a table nobody defined accepts it.
- **`ORDER BY` projection.** Ordering by a field missing from the projection fails the whole request at parse time.
- **JSON timestamps.** Timestamps written through `serde_json` are strings. They:
  - drop out of datetime comparisons;
  - match after a `<datetime>` cast;
  - are refused by a `TYPE datetime` field.
- **`UPSERT` modes.** `MERGE` keeps absent fields, while `CONTENT` replaces the record.
- **SCHEMALESS checks.** SCHEMALESS tables still enforce `TYPE`/`ASSERT` on the fields they define, including an optional enumeration.

Run:

```bash
cargo test --locked --manifest-path reproducers/surrealdb-3.2.4/Cargo.toml --all-targets
```

## Verification history

The first CI execution compiled successfully and passed four tests, but the rollback test's **postcondition was wrong**: because schema creation occurred inside the forced-failure transaction, the later `SELECT` hit a non-existent table. The test was corrected by defining the table before beginning the transaction.

The corrected suite passed all five tests:

```text
GitHub Actions run: 34691471041
head: a837f981cdfc78ad27573101373eff1b37cd2b33
Rust: 1.96.0
SurrealDB Rust SDK: 3.2.4
result: 5 passed, 0 failed
```

The workflow was then hardened to commit-pinned `actions/checkout` v7.0.1 with read-only contents permission. The complete suite passed again:

```text
GitHub Actions run: 34691656259
workflow head: c91d41d938a6e643da69072700dcb40e1baf857e
result: 5 passed, 0 failed
```

This history is kept deliberately. A failed reproducer that exposes a bad test is useful evidence about the harness, but it does not falsify the underlying capability claim.

The workflow `.github/workflows/reproducers.yml` runs on a standard public-repository runner. Public standard GitHub-hosted runners are currently free.

### 2026-10-01: schema and query shapes, engine pinning

Until this pass, the suite pinned only `surrealdb = "=3.2.4"` and had no `Cargo.lock`.

- The SDK depends on `surrealdb-core`, `surrealdb-types` and `surrealdb-types-derive` through caret requirements.
- 3.3.0 of those crates was published on 2026-09-24.
- A fresh resolution of the old manifest on 2026-10-01 gave SDK 3.2.4 with core, types and derive 3.3.0.

The 2026-09-12 runs above predate 3.3.0, and caret requirements do not select the 3.3.0 betas, so their evidence stands. The crates are now pinned `=3.2.4`, `Cargo.lock` is committed, and CI runs with `--locked`. Those three pins turned out to be incomplete; see 2026-10-08.

Local verification of the extended suite:

```text
date: 2026-10-01
Rust: 1.98.1 (local); CI uses 1.96.0
resolved: surrealdb, surrealdb-core, surrealdb-types, surrealdb-types-derive = 3.2.4
cargo test --all-targets:          core_contract 5 passed, schema_and_query_shapes 5 passed
cargo test --locked --all-targets: 10 passed, 0 failed
```

GitHub Actions confirmed it on the pushed head:

```text
GitHub Actions run: 36897934223
head: f029a0a0446661855ce01f9d210ac4c0fbc41890
Rust: 1.96.0, --locked
compiled: surrealdb, surrealdb-core, surrealdb-types, surrealdb-types-derive v3.2.4
result: 10 passed, 0 failed
```

### 2026-10-08: two more engine crates

The 2026-10-01 lock did not hold the whole engine at 3.2.4:

- `surrealdb-core` 3.2.4 depends on `surrealdb-collections` and `surrealdb-strand` through `^3.2.4`.
- The committed lock had resolved both of them to 3.3.0.

So the 2026-10-01 runs above, local and CI, ran core 3.2.4 on collections and strand 3.3.0. Their results stand for that mix, but they were not a pure 3.2.4 engine.

Both crates are now pinned `=3.2.4`, and the lock was updated for exactly those two packages. Only `surrealdb-protocol` keeps its own version line (0.10.2).

Local verification:

```text
date: 2026-10-08
Rust: 1.98.1 (local); CI uses 1.96.0
resolved: surrealdb, surrealdb-core, -types, -types-derive, -collections, -strand = 3.2.4 (surrealdb-protocol 0.10.2)
cargo test --locked --all-targets: core_contract 5 passed, schema_and_query_shapes 5 passed; 10 passed, 0 failed
```

## Cross-version rule

Never use a passing 3.2.4 reproducer as automatic proof for 3.3, 4.x, or another SDK/server combination. Copy or parameterize the reproducer, pin the new baseline, run it, then update the ledger.
