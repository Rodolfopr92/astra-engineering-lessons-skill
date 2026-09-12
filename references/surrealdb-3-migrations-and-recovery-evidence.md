# SurrealDB 3.x Migrations and Recovery Evidence

**Primary baseline:** SurrealDB 3.2.x + Rust  
**Case-study evidence:** ARGOS, Omphalos, Saturno, DELPHIS  
**Last verified:** 2026-09-12

This reference is intentionally narrow. It covers technical patterns for evolving a SurrealDB schema and proving persistence/recovery claims. It does **not** decide whether SurrealDB should be a system of record, a projection, a cache, or one store among several. Those are project-planning and architecture decisions.

## Evidence labels

- **VERIFIED API** — current official SurrealDB API.
- **CASE-STUDY EVIDENCE** — implemented in one or more proving repositories.
- **GENERAL TESTING RULE** — evidence discipline rather than a SurrealDB requirement.

---

## 1. Prefer explicit migration history over one permanently mutable bootstrap string

A mature application benefits from ordered migration units with stable identity rather than continuously rewriting one large `DEFINE ... IF NOT EXISTS` block.

A useful migration record contains at least:

```text
version
name
checksum
applied_at
```

Representative Rust shape:

```rust
struct Migration {
    version: u32,
    name: &'static str,
    body: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "foundation",
        body: include_str!("../schema/001_foundation.surql"),
    },
    Migration {
        version: 2,
        name: "graph",
        body: include_str!("../schema/002_graph.surql"),
    },
];
```

Apply only unapplied versions, inspect statement errors, then record the applied migration.

**CASE-STUDY EVIDENCE:** ARGOS uses ordered `.surql` migration files; Omphalos, Saturno, and DELPHIS use versioned schema runners.

---

## 2. Give migration records deterministic identity

Avoid random migration-ledger rows when a stable version identity exists.

```surql
UPSERT ONLY type::record('schema_migration', $id) SET
    version = $version,
    name = $name,
    checksum = $checksum,
    applied_at = time::now();
```

A unique version index can provide a second integrity backstop:

```surql
DEFINE INDEX IF NOT EXISTS ux_schema_migration_version
    ON TABLE schema_migration COLUMNS version UNIQUE;
```

**CASE-STUDY EVIDENCE.**

The reusable technical point is idempotent migration identity, not one mandatory record-key format.

---

## 3. Record immutable migration content identity

When migration bodies are embedded as files, hashing their exact bytes provides useful provenance:

```text
version 10
name integrity_hardening
sha256 <digest of migration body>
```

This can detect accidental mutation of already-applied migration material and help compare schema bundles across builds/installations.

**CASE-STUDY EVIDENCE:** ARGOS records SHA-256 of migration bodies.

A checksum proves identity, not policy. The application must still decide how to react if an already-applied migration's source bytes no longer match the recorded digest.

---

## 4. Inspect statement-level migration failures

Transport success is not enough for a multi-statement migration.

```rust
let response = db.query(migration.body).await?;
response.check()?;
```

If diagnostics need every failing statement and its index, preserve them with `take_errors()` instead of collapsing to the first failure.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/query
- https://surrealdb.com/docs/reference/rust/concepts/error-handling

---

## 5. Migration idempotence and restart durability are different claims

These tests answer different questions:

```text
apply schema twice through one live handle
→ migration application is idempotent

writer process opens SurrealKV and applies schema/data
→ writer exits
→ fresh reader process opens same directory
→ migration/data state is still present
→ persistence/restart evidence
```

Saturno provides same-handle migration-idempotence evidence. ARGOS and DELPHIS provide stronger process-separated embedded restart evidence.

Do not report an idempotence test as a restart test.

**CASE-STUDY EVIDENCE / GENERAL TESTING RULE.**

---

## 6. A recovery or rebuild claim needs executable evidence

Documentation that says a database or index is recoverable is not enough. The claim becomes strong only when the relevant failure/recovery path is executed.

Depending on the application, a useful proof may be:

```text
create known durable state
→ terminate process / remove disposable derived state
→ restart or reconstruct using the documented mechanism
→ verify identities, counts, hashes, relationships, and migration version
```

The exact recovery source and architecture are project-specific. This skill only requires that the evidence match the claim.

**GENERAL TESTING RULE.**

---

## 7. Keep technical ledgers semantically distinct

If a system has several technical ledgers, do not overload one field to mean several unrelated things.

Examples include:

```text
schema migration version
runtime/replay checkpoint
optional seed/reference bundle marker
```

They describe different technical states and generally need independent identity and tests.

This is a **GENERAL STATE-MODELING PATTERN**, not a SurrealDB API requirement.

---

## 8. Out of scope: choosing SurrealDB's architectural authority role

This reference deliberately does not answer questions such as:

```text
Should SurrealDB be the system of record?
Should it be derived from another store?
Should the application use an outbox?
Should multiple stores exist at all?
```

Those are architecture/planning questions. The capability skill should provide accurate SurrealDB behavior and evidence patterns so the project planner can make that decision with current information, not make the decision on the planner's behalf.

---

## General rule

Use this skill to correct **technical stale priors** about migrations, query responses, persistence, and recovery evidence. Keep application authority, store ownership, and multi-store topology in the project's architecture plan.