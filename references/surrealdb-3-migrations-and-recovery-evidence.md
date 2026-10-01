# SurrealDB 3.x Migrations and Recovery Evidence

**Primary baseline:** SurrealDB 3.2.x + Rust  
**Case-study evidence:** ARGOS, Omphalos, Saturno, DELPHIS  
**Last verified:** 2026-09-12; sections 10–11 on 2026-10-01 (reproducer `tests/schema_and_query_shapes.rs`)

This reference is intentionally narrow. It covers technical patterns for evolving a SurrealDB schema and proving persistence/recovery claims. It does **not** decide whether SurrealDB should be a system of record, a projection, a cache, or one store among several. Those are project-planning and architecture decisions.

## Evidence labels

- **VERIFIED API** — current official SurrealDB API.
- **TESTED BEHAVIOR** — reproduced against 3.2.4 by this repository's reproducers.
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

## 8. Protect administrative credentials during migrations (`argv` vs HTTP API)

Running database migrations or bootstrap scripts via CLI argument passing:

```bash
# ❌ VULNERABLE: Exposes root password in plaintext to all local processes
surreal sql --endpoint http://127.0.0.1:8000 --username root --password "$PASSWORD"
```

creates a high-severity security exposure: any unprivileged local user or process can inspect `/proc/<pid>/cmdline` or `ps aux` and read the plaintext root credentials.

**Hardened pattern**:
Execute migrations over SurrealDB's HTTP REST endpoint (`POST /sql`) with HTTP Basic Auth headers:

```python
import base64
import urllib.request
import json

auth = base64.b64encode(f"{user}:{password}".encode()).decode("ascii")
req = urllib.request.Request(
    f"{endpoint}/sql",
    data=surql_bytes,
    headers={
        "Authorization": f"Basic {auth}",
        "Content-Type": "text/plain",
        "Accept": "application/json",
        "surreal-ns": namespace,
        "surreal-db": database,
    },
    method="POST",
)
with urllib.request.urlopen(req, timeout=300) as resp:
    statements = json.load(resp)
    for index, stmt in enumerate(statements, start=1):
        if stmt.get("status") != "OK":
            raise RuntimeError(f"Statement {index} failed: {stmt.get('detail')}")
```

**SECURITY HARDENING RULE + TESTED BEHAVIOR (Astra deployment hardening).**

---

## 9. Multi-tenant schema lifecycle separation

In multi-tenant systems, avoid treating the database as one monolithic schema file. Separate the architecture into two distinct lifecycles:

1. **Global Control Plane Schema (`control.surql`)**:
   - Manages tenant registration, user accounts, audit ledgers, and capability flags.
   - Applied once at system startup / deployment.
2. **Tenant-Local Schema Template (`tenant_schema.surql`)**:
   - Manages tenant-specific tables, vector embeddings (`memory_chunk` with HNSW indexes), fulltext analyzers (`BM25`), and conversational memory.
   - Applied atomically during tenant provisioning (`DEFINE NAMESPACE` -> provision credentials -> apply tenant schema).

This ensures every newly provisioned tenant namespace is fully SCHEMAFULL and indexed from its very first transaction, without requiring ad-hoc table creation.

**CASE-STUDY EVIDENCE (Astra multi-tenant cognitive memory architecture).**

---

## 10. A new field on a SCHEMAFULL table needs a migration, and tests must run that migration

**TESTED BEHAVIOR** (3.2.4). Reproducer: `schemafull_refuses_an_undefined_top_level_field_an_undefined_table_hides_it`.

On a SCHEMAFULL table, writing a top-level field that nobody defined fails:

```text
Found field 'language_preference', but no such field exists for table 'user'
```

The trap is the test fixture. A test that creates rows in a table it never defined gets a **schemaless** table. The same write passes there, and the suite stays green while production refuses every write.

```surql
-- migration (idempotent)
DEFINE FIELD IF NOT EXISTS language_preference ON TABLE user TYPE option<string>
    ASSERT $value = NONE OR $value IN ['pt-br', 'en'];
```

Rules:

1. Every field the code writes on a SCHEMAFULL table appears in a migration.
2. Integration tests build their tables from the production migration script itself: read the script, or the relevant section of it. Never use hand-written `DEFINE`s or bare `CREATE`s.
3. A regression test for a schema fix must fail when the migration line is removed.

**CASE-STUDY EVIDENCE.** A proving repository added a per-user language preference in code.

- Its tests created users in an undefined table, and they passed.
- In production, every save failed on the SCHEMAFULL `user` table.
- The fix was the migration line, plus a live test that builds the table from the migration script.

---

## 11. SCHEMALESS tables still enforce the fields they define

**TESTED BEHAVIOR** (3.2.4). Reproducer: `schemaless_tables_still_enforce_the_fields_they_define`.

`SCHEMALESS` means "unknown fields are accepted", not "nothing is checked". `TYPE` and `ASSERT` still apply to every field the table defines. That makes it a cheap way to constrain the one value that must be right on an otherwise open table:

```surql
DEFINE TABLE shipment SCHEMALESS;
DEFINE FIELD carrier  ON TABLE shipment TYPE string ASSERT $value IN ['correios'];
DEFINE FIELD language ON TABLE shipment TYPE option<string>
    ASSERT $value = NONE OR $value IN ['pt-br', 'en'];
```

`option<…>` plus `$value = NONE OR …` is the optional-enumeration pattern: an absent value is accepted, and a value outside the list is refused. The error names the rule, normalized to `INSIDE`:

```text
Found 'jadlog' for field `carrier`, with record `s:bad`, but field must conform to: $value INSIDE ['correios']
```

**PROJECT CONVENTION:** when code builds the `DEFINE` statement, generate the list from the code's enum, so there is one source of truth. When the migration is a hand-written script, keep the two in sync with a test.

---

## 12. Out of scope: choosing SurrealDB's architectural authority role

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