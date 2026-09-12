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
Rust verification toolchain: 1.96
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

The GitHub workflow `.github/workflows/reproducers.yml` runs this suite on a standard public-repository runner. Public standard GitHub-hosted runners are currently free; no cache/artifact upload is configured.

## Cross-version rule

Never use a passing 3.2.4 reproducer as automatic proof for 3.3, 4.x, or another SDK/server combination. Copy or parameterize the reproducer, pin the new baseline, run it, then update the ledger.
