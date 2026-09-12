# SurrealDB 3.x: Stale LLM Priors and Current Corrections

**Target baseline:** SurrealDB server 3.2.4 + Rust SDK 3.2.4  
**Last verified:** 2026-09-12

This document exists specifically for AI coding agents whose internal knowledge is dominated by older SurrealDB releases.

Use it as a fast correction layer before generating or reviewing SurrealDB 3.x Rust code.

## Evidence legend

- **VERIFIED API** — current official SurrealDB documentation / SDK API.
- **TESTED BEHAVIOR** — reproduced against SurrealDB 3.2.4 in live code/tests.
- **PROJECT CONVENTION** — a recommended application pattern, not a SurrealDB requirement.

---

## 1. `type::thing()` became `type::record()`

**STALE PRIOR (2.x and earlier)**

```surql
type::thing('person', $id)
```

**CURRENT 3.x API**

```surql
type::record('person', $id)
```

**Evidence:** VERIFIED API.

Official 3.x documentation explicitly states that `type::record()` was known as `type::thing()` before SurrealDB 3.0. The behavior is the same; the name changed.

Official source:
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/type

---

## 2. Intrinsic record IDs are typed records, not strings

**STALE / DANGEROUS MODEL**

```rust
#[derive(SurrealValue)]
struct Person {
    id: String,
    name: String,
}
```

If a query returns SurrealDB's intrinsic `id`, the native SDK returns it as a structured record identifier.

**CURRENT TYPE**

```rust
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, SurrealValue)]
struct Person {
    id: RecordId,
    name: String,
}
```

**Evidence:** VERIFIED API + TESTED BEHAVIOR.

The failure reproduced in a real 3.2.4 restart test as:

```text
Expected string, got record
```

Official sources:
- https://surrealdb.com/docs/languages/rust
- https://surrealdb.com/docs/reference/rust/methods/select

### Project-level alternative

If application code does not need intrinsic `id`, omit it and store an explicit logical domain key:

```rust
#[derive(Debug, SurrealValue)]
struct Artifact {
    artifact_id: String,
    source_hash: String,
}
```

This is a **PROJECT CONVENTION**, not a SurrealDB requirement.

---

## 3. `SurrealValue` is the native Rust conversion contract

**STALE ASSUMPTION**

> Serde `Serialize` / `Deserialize` alone define native SDK conversion behavior.

**CURRENT 3.x MODEL**

```rust
use surrealdb::types::SurrealValue;

#[derive(Debug, SurrealValue)]
struct Employee {
    name: String,
}
```

`SurrealValue` converts Rust types to and from SurrealDB values. Its `#[surreal(...)]` attributes resemble Serde attributes but are a separate system.

**Evidence:** VERIFIED API.

Official sources:
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types
- https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes

---

## 4. Prefer documented 3.x public type paths

Current official examples import:

```rust
use surrealdb::types::{RecordId, SurrealValue};
```

Do not reflexively generate old examples based on internal/legacy paths such as `surrealdb::sql::Thing` without checking the target version.

**Evidence:** VERIFIED API.

---

## 5. Specific-record SDK operations still accept `(table, id)` tuples

3.x Rust SDK methods can target a specific record using tuple resources:

```rust
let person: Option<Person> = db.select(("person", "tobie")).await?;

let created: Option<Person> = db
    .create(("person", "tobie"))
    .content(data)
    .await?;
```

Do not over-correct stale knowledge by assuming every record operation must manually construct `RecordId`.

**Evidence:** VERIFIED API.

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/select
- https://surrealdb.com/docs/reference/rust/methods/create

---

## 6. Raw multi-statement query success must inspect statement errors

For application code using `db.query(...)`, transport-level `.await` success does not mean every SurrealQL statement succeeded. For critical mutations/schema work, inspect the response with `.check()` before consuming results.

A proven application pattern is:

```rust
let mut response = db
    .query("SELECT * FROM document_artifact WHERE source_hash = $hash LIMIT 1")
    .bind(("hash", source_hash.to_string()))
    .await?
    .check()?;

let rows: Vec<DocumentArtifactRecord> = response.take(0)?;
```

**Evidence:** TESTED BEHAVIOR / application hardening pattern.

---

## 7. Do not use application read-modify-write for hot counters

**RACE-PRONE PATTERN**

```rust
let current = load_checkpoint(db).await?;
let next = current.records_seen + 1;
save_checkpoint(db, next).await?;
```

Two concurrent callers can read the same value and overwrite each other's increments.

**PROVEN PATTERN**

Use one server-side mutation:

```surql
UPSERT type::record('ingest_checkpoint', $source_code) SET
    source_code = $source_code,
    records_seen += $seen,
    records_inserted += $inserted,
    records_replayed += $replayed,
    records_failed += $failed,
    updated_at = time::now();
```

Then inspect statement errors and retry only transaction conflicts that are known to be retryable.

**Evidence:** TESTED BEHAVIOR against SurrealDB 3.2.4 under concurrent updates.

---

## 8. In-memory success is not disk-persistence evidence

`Mem` is useful for some tests, but it does not prove:

- WebSocket/native-SDK serialization behavior;
- SurrealKV disk persistence;
- restart/reopen behavior;
- WAL/recovery behavior;
- session reconnection behavior.

For persistence-sensitive code, run at least one real test that:

```text
start SurrealDB 3.2.4 on SurrealKV
→ write typed records
→ terminate server
→ restart against same directory
→ reconnect
→ read typed records back
```

**Evidence:** TESTED BEHAVIOR. This is how the `RecordId` deserialization issue was caught after ordinary tests had passed.

---

## 9. Record functions also changed naming conventions in 3.x

SurrealDB 3.x documentation uses record-oriented functions such as:

```surql
record::id(person:tobie)
record::table(person:tobie)
```

and documents `type::record()` as the constructor for record pointers.

When a model emits older names, verify the current function reference instead of assuming backward compatibility.

Official sources:
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/record
- https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/record-ids

---

## 10. Version check before generation

Before producing SurrealDB code, an agent should answer:

```text
Server version?
Rust SDK version?
Connection mode? (remote WS/HTTP vs embedded)
Persistence engine?
Exact locked dependency version?
```

If these are known, optimize for them rather than writing generic SurrealDB code from memory.

For the current proving baseline:

```text
Server: 3.2.4
Rust SDK: 3.2.4
Remote protocol: WebSocket
Persistence: SurrealKV
```
