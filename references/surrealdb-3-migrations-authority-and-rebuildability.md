# SurrealDB 3.x Migrations, Authority, and Rebuildability

**Primary baseline:** SurrealDB 3.2.x + Rust  
**Case studies:** ARGOS, Alexandria, Omphalos, Saturno  
**Last verified:** 2026-09-12

This reference captures two recurring problems that AI agents often blur together:

1. how to evolve a SurrealDB schema safely; and
2. whether SurrealDB is the authoritative store or a rebuildable projection.

Those are separate decisions. A project can have excellent migrations and still have an unclear recovery model, or a clean projection architecture with poor schema-version discipline.

## Evidence labels

- **VERIFIED API** — current official SurrealDB API.
- **CASE-STUDY EVIDENCE** — implemented in one or more proving repositories.
- **PROJECT CONVENTION** — a chosen architecture, not a database requirement.

---

## 1. Prefer append-only migration history over mutable bootstrap strings

A mature application should not treat one ever-growing `DEFINE ... IF NOT EXISTS` string as its permanent migration system.

A useful migration record contains at least:

```text
version
name
checksum
applied_at
```

A robust application pattern is:

```rust
struct Migration {
    version: u32,
    name: &'static str,
    body: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration { version: 1, name: "foundation", body: include_str!("../schema/001_foundation.surql") },
    Migration { version: 2, name: "graph", body: include_str!("../schema/002_graph.surql") },
];
```

Apply only unapplied versions, execute the migration, inspect statement errors, then record the migration identity.

**CASE-STUDY EVIDENCE:** ARGOS uses ten ordered `.surql` migration files; Omphalos/Saturno use a versioned embedded schema runner.

---

## 2. Make migration identity deterministic

Avoid random migration-history records. A deterministic record ID such as `v10` makes the migration ledger itself idempotent:

```surql
UPSERT ONLY type::record('schema_migration', $id) SET
    version = $version,
    name = $name,
    checksum = $checksum,
    applied_at = time::now();
```

Back it with a unique version index:

```surql
DEFINE INDEX IF NOT EXISTS ux_schema_migration_version
    ON TABLE schema_migration COLUMNS version UNIQUE;
```

**CASE-STUDY EVIDENCE.**

The general rule is to give each migration a stable identity that survives repeated application attempts and restart/recovery paths.

---

## 3. Store a checksum of immutable migration content

When migrations are embedded as source files, compute a stable digest of the exact bytes:

```text
version 10
name r35_integrity_hardening
sha256 <digest of migration body>
```

A checksum helps answer:

- Did the migration file change after it was applied?
- Is this binary using the schema bundle we think it is?
- Are two installations running the same migration material?

**CASE-STUDY EVIDENCE:** ARGOS records SHA-256 of each migration body.

### Important distinction

A checksum is evidence of identity, not an automatic migration policy. Decide explicitly what the application should do if an already-applied version's current source checksum differs from the stored checksum. In most audited systems, silently rewriting history is the wrong answer.

---

## 4. Check statement-level migration errors

A successful outer Rust `await` is not enough for a multi-statement migration. Inspect the query response:

```rust
let response = db.query(migration.body).await?;
response.check()?;
```

If you need to preserve every failing statement index for diagnostics, use `take_errors()` rather than collapsing the response to the first error.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/query
- https://surrealdb.com/docs/reference/rust/concepts/error-handling

---

## 5. Test both migration idempotence and durable reopen

These claims are different:

```text
apply_schema() twice through one live handle
→ proves application-level idempotence

exit process, reopen same SurrealKV directory, inspect migration table
→ proves durable persistence/reopen behavior
```

Saturno explicitly tests same-handle reapplication without duplicate migration rows. ARGOS and DELPHIS use process-separated persistence/restart evidence for stronger durability claims.

Do not call the first test a restart proof.

**CASE-STUDY EVIDENCE / GENERAL TESTING RULE.**

---

## 6. SurrealDB can be the authority

In a single-authority local application, SurrealDB may own the domain records and relationships directly:

```text
Tauri/Rust command
      ↓
SurrealDB transaction
      ↓
operational nodes + edges + receipts/outbox
```

When several records must change together, use a database transaction and integrity constraints. Application locks can serialize one local process, while unique indexes and transaction rules remain the database backstop.

ARGOS demonstrates this architecture for a local-first commercial operations system.

**PROJECT CONVENTION / CASE-STUDY EVIDENCE.**

---

## 7. SurrealDB can instead be a rebuildable projection

In a multi-store system, SurrealDB may be deliberately non-authoritative:

```text
authoritative transaction store
      ↓
domain row + outbox event
      ↓
projector
      ↓
SurrealDB graph/context projection
```

A robust projection record should carry enough identity to reject stale writes and rebuild deterministically. Useful fields include:

```text
tenant_id
external_id
revision
content_hash
updated_at
```

A separate checkpoint can track:

```text
tenant_id
target
aggregate_id
revision
event_id
applied_at
```

Alexandria implements this shape and treats SurrealDB/LanceDB as wipeable, rebuildable projections from authoritative Gold state.

**CASE-STUDY EVIDENCE.**

---

## 8. Do not independently dual-write the same fact to two authorities

A dangerous pattern is:

```text
command
  ├── write database A
  └── write database B
```

with no durable ordering/repair contract. If the process dies between writes, the system has two competing truths.

Prefer one authoritative commit that also records the intent to project:

```text
transaction in authority
  ├── domain mutation
  └── outbox event
        ↓ retryable worker
      projection store
```

The projector should be:

- idempotent;
- revision-aware;
- tenant-scoped where applicable;
- retryable;
- rebuildable from authority.

This is a **GENERAL DISTRIBUTED-SYSTEMS PATTERN**, reinforced by Alexandria case-study evidence.

---

## 9. A rebuild promise needs executable evidence

Documentation that says "this database is reconstructible" is architectural intent until a rebuild path is implemented and tested.

A meaningful proof looks like:

```text
seed authoritative source
→ build SurrealDB projection
→ delete/wipe projection
→ recreate schema
→ replay authority/outbox/checkpoints
→ compare identities, revisions, counts, hashes and selected graph relations
```

Omphalos has a documented staged rebuild design, but parts of that file remain TODO skeletons. Therefore the **reconstruction invariant there is architectural intent, not completed behavior**.

Alexandria's portable projection tests provide stronger evidence of idempotence/retry/rebuild logic, while its native SurrealDB projector still has its own integration gates.

This distinction is exactly why the skill maintains evidence labels.

---

## 10. Keep schema migration state separate from data projection state

These ledgers answer different questions:

```text
schema_migration
→ which database definitions have been installed?

projection_checkpoint
→ which source revisions/events have been projected?

reference_overlay_marker
→ which optional seed/reference bundle has been applied?
```

Do not collapse all three into a single version integer. Their replay and recovery semantics differ.

**GENERAL ARCHITECTURAL PATTERN**, supported by ARGOS, Alexandria, and Brew & Batch case-study evidence.

---

## 11. Scope limits

This reference does not claim:

- every application needs a custom migration runner;
- every migration must use SHA-256 specifically;
- SurrealDB should always be authoritative;
- SurrealDB should always be derived;
- an outbox is required in a single-store system;
- a documented rebuild design is evidence that rebuild actually works;
- same-handle schema idempotence proves process restart durability.

The reusable rule is to make **schema history, authority, projection identity, and recovery evidence explicit rather than implicit**.