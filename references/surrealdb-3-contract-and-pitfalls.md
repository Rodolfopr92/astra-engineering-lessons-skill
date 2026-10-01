# SurrealDB 3.2.4 Contract & Engineering Pitfalls

**Target baseline:** SurrealDB server 3.2.4 + Rust SDK 3.2.4  
**Last verified:** 2026-09-12

This reference separates the current technology contract from project-specific conventions.

For a fast 2.x → 3.x correction table, read `surrealdb-3-stale-llm-priors.md` first.

---

## 1. Intrinsic Record IDs: `RecordId`, Not `String`

### VERIFIED API

Current official Rust examples model an intrinsic SurrealDB record identifier with:

```rust
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, SurrealValue)]
struct Person {
    id: RecordId,
    name: String,
}
```

Official references:
- https://surrealdb.com/docs/languages/rust
- https://surrealdb.com/docs/reference/rust/methods/select

### TESTED BEHAVIOR

A real SurrealDB 3.2.4 persistence/restart test failed when a Rust struct declared:

```rust
#[derive(SurrealValue)]
struct ArtifactProbe {
    id: String,
}
```

with:

```text
Failed to deserialize field 'id' on type 'ArtifactProbe':
Expected string, got record
```

The important correction is not “never have a field named `id`.” It is:

> If the field represents SurrealDB's intrinsic record ID, model it with the real record-ID type.

### PROJECT CONVENTION: explicit logical IDs

Astra often omits intrinsic `id` from domain structs and uses a logical key:

```rust
#[derive(Clone, Debug, SurrealValue)]
struct DocumentArtifactRecord {
    artifact_id: String,
    source_hash: String,
    status: String,
}
```

then addresses the database record explicitly:

```rust
let saved: Option<DocumentArtifactRecord> = db
    .upsert(("document_artifact", artifact.artifact_id.as_str()))
    .content(artifact.clone())
    .await?;
```

That is an application design choice, not a SurrealDB requirement.

---

## 2. `SurrealValue` Is the Native Rust Value Contract

### VERIFIED API

SurrealDB 3.x documents `SurrealValue` as the trait used to convert Rust types to and from SurrealDB values.

```rust
use surrealdb::types::SurrealValue;

#[derive(Debug, SurrealValue)]
struct Employee {
    name: String,
}
```

Its `#[surreal(...)]` attributes resemble Serde attributes, but they are a separate attribute system.

Official references:
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types
- https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes

Do not use intermediate JSON serialization as a generic workaround for native type errors. It can change value semantics and make database behavior harder to reason about.

---

## 3. `type::record()` Replaced `type::thing()` in 3.x

### VERIFIED API

Current SurrealDB 3.x:

```surql
type::record('person', $id)
```

Pre-3.0 examples often show:

```surql
type::thing('person', $id)
```

The official docs explicitly state that `type::record()` was known as `type::thing()` before 3.0 and that the behavior did not otherwise change.

Official reference:
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/type

---

## 4. Raw Query Responses: Inspect Statement Errors

### TESTED HARDENING PATTERN

For critical raw SurrealQL queries, treat successful transport as distinct from successful statements:

```rust
let mut response = db
    .query("SELECT * FROM document_artifact WHERE source_hash = $hash LIMIT 1")
    .bind(("hash", source_hash.to_string()))
    .await
    .map_err(|e| format!("query transport failed: {e}"))?
    .check()
    .map_err(|e| format!("query statement failed: {e}"))?;

let rows: Vec<DocumentArtifactRecord> = response
    .take(0)
    .map_err(|e| format!("deserialization failed: {e}"))?;
```

This pattern is especially useful for schema setup, multi-statement queries, and mutations where silently ignoring a statement error would corrupt application assumptions.

---

## 5. Atomic Counters: Keep the Mutation in SurrealQL

### The application-level race

This is vulnerable to lost updates:

```rust
let current = get_checkpoint(db, source_code).await?;
let next = current.records_seen + seen;
save_checkpoint(db, next).await?;
```

### TESTED BEHAVIOR / proven pattern

The proving implementation uses one server-side UPSERT:

```surql
UPSERT type::record('ingest_checkpoint', $source_code) SET
    source_code = $source_code,
    records_seen += $seen,
    records_inserted += $inserted,
    records_replayed += $replayed,
    records_failed += $failed,
    last_source_record_id = IF $last_record_id != NONE {
        $last_record_id
    } ELSE {
        last_source_record_id
    },
    last_source_time = IF $last_record_id != NONE {
        time::now()
    } ELSE {
        last_source_time
    },
    updated_at = time::now();
```

The Rust caller:

1. binds parameters;
2. awaits the query;
3. calls `.check()`;
4. retries only recognized transaction-conflict errors with bounded backoff;
5. deserializes the returned record.

A concurrent regression test verified that 20 parallel increments produced exactly 20, rather than losing updates.

Do not simplify this lesson into a different SurrealQL expression unless that expression is separately verified against the target version.

---

## 6. Uniqueness Must Live in the Database When It Defines Identity

### TESTED PATTERN

If `source_hash` defines artifact identity for a tenant, enforce it in SurrealDB:

```surql
DEFINE INDEX OVERWRITE idx_document_artifact_source_hash
ON TABLE document_artifact COLUMNS source_hash UNIQUE;
```

Then use an indexed parameterized lookup:

```rust
let mut response = db
    .query("SELECT * FROM document_artifact WHERE source_hash = $hash LIMIT 1")
    .bind(("hash", source_hash.to_string()))
    .await?
    .check()?;
```

An application preflight such as `SELECT → if absent → INSERT` is not a substitute for a storage-engine uniqueness constraint under concurrency.

---

## 7. Multi-Tenant Isolation Is an Application Architecture, Not a 3.x Requirement

### PROJECT CONVENTION

Astra uses a control database plus isolated tenant namespace/database sessions. Each company receives a dedicated namespace (e.g. `astra_company_<slug>`) and database (`operations`).

### CRITICAL CONCURRENCY PITFALL: Do Not Share Cloned WebSocket Handles Across Concurrent Tenants

While the SurrealDB 3.0+ Rust SDK documents `Surreal<C>::clone()` as creating an independent logical session, in high-concurrency multi-threaded runtimes (e.g. async web servers or bot workers), caching cloned `Surreal<Client>` handles in an in-memory `HashMap` can race the SDK's internal asynchronous session replay and sign-in state (`use_ns`, `use_db`, `signin`). 

Under concurrent multi-tenant traffic, this can lead to session bleeding or queries executing under the wrong tenant context!

**TESTED HARDENING**:
Open a fresh dedicated WebSocket connection (`Surreal::new::<Ws>(&endpoint).await`) per tenant session rather than caching and sharing cloned handles across worker threads:

```rust
pub async fn get_tenant_session(
    endpoint: &str,
    namespace: &str,
    database: &str,
    credentials: &Credentials,
) -> Result<Surreal<Client>, String> {
    // Open a fresh WebSocket for every returned session. Cloning a previously
    // scoped SurrealDB 3.2.4 handle can race the SDK's asynchronous session
    // replay when the clone's first request is a plain query.
    let tenant_db = Surreal::new::<Ws>(endpoint)
        .await
        .map_err(|e| format!("connection error: {e}"))?;

    tenant_db
        .signin(credentials.clone())
        .await
        .map_err(|e| format!("auth error: {e}"))?;

    tenant_db
        .use_ns(namespace)
        .use_db(database)
        .await
        .map_err(|e| format!("scope error: {e}"))?;

    Ok(tenant_db)
}
```

The reusable lesson is:

> Test the tenancy boundary you actually claim, against the real connection/session mechanism you actually deploy under concurrent load.

### Multi-Tenant Integration Tests: Shared Server Namespace Collisions on Static Slugs

When executing integration tests against a shared test server, hardcoding static tenant slugs (such as `astra_company_acme` or `company_acme`) creates cross-test race conditions and state leakage:

1. Parallel tests concurrently run `DEFINE NAMESPACE` and table definition migrations on the same namespace.
2. An aborted test leaves state behind, causing unique index conflicts in subsequent runs.
3. Tests asserting clean-state or count invariants fail non-deterministically.

**TESTED ISOLATION PATTERN**:
In test fixtures, always generate process-unique tenant slugs:

```rust
let test_slug = support::unique("cr_acme");
let test_ns = format!("astra_company_{test_slug}");
```

Where `support::unique(prefix)` appends an atomic counter and timestamp. Ensure tests initialize and execute strictly within their unique namespace, preventing test pollution and race conditions across concurrent runners.

**CASE-STUDY EVIDENCE.**

---

## 8. Real SurrealKV Restart Testing

### What `Mem` can and cannot prove

In-memory engines are useful for fast tests. They do **not** prove:

- disk persistence;
- process restart/recovery;
- remote WebSocket serialization behavior;
- reconnection behavior;
- the exact production storage path.

### TESTED BEHAVIOR

Astra's restart test:

```text
start SurrealDB 3.2.4 on surrealkv://<temp-dir>
→ connect over WebSocket
→ write typed records
→ assert seed state
→ terminate server process
→ start a fresh server on the same directory
→ reconnect
→ re-read typed records and exact numeric fields
```

This test caught the `Expected string, got record` bug after ordinary tests had passed.

The lesson is not “never mock.” It is:

> Mocks and in-memory tests must not substitute for the real persistence boundary when persistence correctness is part of the claim.

---

## 9. Current Official 3.x Rust Patterns Worth Remembering

Current official documentation shows:

```rust
use surrealdb::types::{RecordId, SurrealValue};
```

Specific records can still be targeted ergonomically:

```rust
let person: Option<Person> = db.select(("person", "tobie")).await?;

let created: Option<Person> = db
    .create(("person", "tobie"))
    .content(data)
    .await?;
```

And SurrealQL record construction uses:

```surql
type::record('person', 'tobie')
```

Useful official references:
- https://surrealdb.com/docs/reference/rust
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types
- https://surrealdb.com/docs/reference/rust/methods/select
- https://surrealdb.com/docs/reference/rust/methods/create
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/type
- https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/record-ids

---

## 10. Agent Preflight for SurrealDB Work

Before generating code, establish:

```text
server version
SDK version
language
remote vs embedded engine
connection protocol
storage engine
schema mode (schemafull/schemaless)
exact dependency lockfile
```

Then generate for that environment. Do not silently fall back to remembered 2.x syntax because it looks familiar.
