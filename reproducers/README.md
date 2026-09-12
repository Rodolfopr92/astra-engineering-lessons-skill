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
engine modes: Mem + embedded SurrealKV
Rust verification toolchain: 1.96.0
```

Checks:

- intrinsic `id` decodes as `RecordId`, while a `String` model rejects the same row;
- statement-level failures can exist inside an outer successful query response;
- SCHEMAFULL rejects undeclared nested fields while `FLEXIBLE` accepts intentional dynamic keys;
- `SurrealKv` selects the embedded engine while the application handle is `Surreal<Db>`;
- explicit transaction failure does not leave the earlier write committed.

Run:

```bash
cargo test --manifest-path reproducers/surrealdb-3.2.4/Cargo.toml
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

This history is kept deliberately. A failed reproducer that exposes a bad test is useful evidence about the test harness, but it does not falsify the underlying capability claim.

The workflow `.github/workflows/reproducers.yml` runs on a standard public-repository runner. It uses a commit-pinned `actions/checkout` release and grants only `contents: read`. Public standard GitHub-hosted runners are currently free.

## Cross-version rule

Never use a passing 3.2.4 reproducer as automatic proof for 3.3, 4.x, or another SDK/server combination. Copy or parameterize the reproducer, pin the new baseline, run it, then update the ledger.
