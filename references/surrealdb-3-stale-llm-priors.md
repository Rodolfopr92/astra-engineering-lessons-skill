# SurrealDB 3.x: Stale LLM Priors and Current Corrections

**Target baseline:** SurrealDB server/engine 3.2.4 + Rust SDK 3.2.4  
**Last verified:** 2026-09-12

This document exists specifically for AI coding agents whose internal knowledge is dominated by older SurrealDB releases.

Use it as a fast correction layer before generating or reviewing SurrealDB 3.x Rust code.

## Evidence legend

- **VERIFIED API** — current official SurrealDB documentation / SDK API.
- **TESTED BEHAVIOR** — independently reproduced against SurrealDB 3.2.4 in live code/tests.
- **CASE-STUDY EVIDENCE** — observed in a real external proving project but not yet reduced to an independent minimal reproducer in this skill repository.
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

- remote protocol serialization;
- SurrealKV disk persistence;
- restart/reopen behavior;
- WAL/recovery behavior;
- session reconnection behavior.

For persistence-sensitive code, run a real test matching the deployed connection model.

Remote/server example:

```text
start SurrealDB 3.2.4 on SurrealKV
→ write typed records
→ terminate server
→ restart against same directory
→ reconnect
→ read typed records back
```

Embedded example:

```text
open embedded SurrealKV on fixed directory
→ write typed records
→ close/drop handle
→ open fresh handle on same directory
→ read typed records back
```

**Evidence:** TESTED BEHAVIOR / GENERAL TESTING RULE.

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

## 10. Connection mode is part of the contract

**STALE ASSUMPTION**

> SurrealDB means “connect to a server over WebSocket/HTTP.”

The Rust SDK also supports embedded engines. SurrealKV is available behind the `kv-surrealkv` feature and can persist directly from the application process.

Representative embedded pattern:

```rust
use surrealdb::{Surreal, engine::local::SurrealKv};

let db = Surreal::new::<SurrealKv>(database_path).await?;
db.use_ns("app").use_db("main").await?;
```

**Evidence:** VERIFIED API.

Official sources:
- https://surrealdb.com/docs/reference/rust/embedding
- https://surrealdb.com/docs/reference/rust/methods/new
- https://docs.rs/crate/surrealdb-core/3.2.4/features

Before generation, ask:

```text
remote or embedded?
which storage feature?
which persistence path?
which runtime owns database lifetime?
```

See `embedded-surrealkv-tauri-local-first.md` for desktop/Tauri guidance.

---

## 11. SCHEMAFULL nested objects are stricter in 3.x

**STALE PRIOR**

> If the top-level object field is declared, arbitrary nested keys will simply work or be silently ignored.

On `SCHEMAFULL` tables, object fields are schemafull by default. Nested fields must be declared, or the object-containing field must intentionally use `FLEXIBLE`.

```surql
DEFINE TABLE order SCHEMAFULL;
DEFINE FIELD shipping ON order TYPE object;
DEFINE FIELD shipping.city ON order TYPE string;
DEFINE FIELD shipping.postal_code ON order TYPE string;
```

For arrays of objects:

```surql
DEFINE FIELD items ON order TYPE array<object>;
DEFINE FIELD items.*.sku ON order TYPE string;
DEFINE FIELD items.*.quantity ON order TYPE decimal;
```

For intentionally open nested objects:

```surql
DEFINE FIELD metadata ON order TYPE object FLEXIBLE;
```

Current documentation explicitly notes that before 3.0, undefined nested fields could be omitted; as of 3.0 the write errors on the first undefined nested field.

**Evidence:** VERIFIED API.

Official source:
- https://surrealdb.com/docs/reference/query-language/statements/define/field

---

## 12. `.bind()` uses the SDK variable/value contract, not “anything Serde can serialize”

**STALE ASSUMPTION**

> Any arbitrary `serde_json::Value` can be dropped into `.bind()` and will behave exactly like a native SDK value.

Current `.bind()` accepts values through `IntoVariables`, including key-value pairs, `vars!`, `object!`, supported maps, and `SurrealValue`-compatible structs.

```rust
use surrealdb::types::{SurrealValue, vars};

#[derive(SurrealValue)]
struct Filters {
    min_age: i64,
}

let result = db
    .query("SELECT * FROM person WHERE age >= $min_age")
    .bind(Filters { min_age: 18 })
    .await?;
```

**Evidence:** VERIFIED API.

Official source:
- https://surrealdb.com/docs/reference/rust/methods/query

For complex dynamic bindings, compile a tiny fixture first. See `surrealdb-3-query-shapes-and-sdk-binding.md`.

---

## 13. `RELATE ... SET` uses assignment syntax

**WRONG SHAPE**

```surql
RELATE a->edge->b SET { strength: 5 };
```

**CURRENT DOCUMENTED SHAPE**

```surql
RELATE a->edge->b SET strength = 5;
```

or:

```surql
RELATE a->edge->b CONTENT { strength: 5 };
```

**Evidence:** VERIFIED API.

Official source:
- https://surrealdb.com/docs/reference/query-language/statements/relate

---

## 14. Transactions roll back on error / `THROW`

Do not implement multi-record all-or-nothing semantics with sequential application writes and hope cleanup succeeds.

Current SurrealQL supports explicit transactions where an error or `THROW` rolls back the transaction.

```surql
BEGIN TRANSACTION;
UPDATE account:one SET balance -= $amount;
UPDATE account:two SET balance += $amount;
IF account:one.balance < 0 { THROW "insufficient funds"; };
COMMIT TRANSACTION;
```

**Evidence:** VERIFIED API.

Official source:
- https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions

Failure-injection tests should prove rollback for the target materialization path.

---

## 15. Complex query shapes still require minimal real probes

Brew & Batch reported useful 3.2.4 observations involving:

- bound record IDs using explicit `<record>$variable` casts in some dynamic queries;
- materializing IDs with `LET $ids = SELECT VALUE id ...` before certain `FOR` loops;
- projection/order combinations that required care in the tested query shape.

These are **CASE-STUDY EVIDENCE**, not yet universal SurrealDB API rules in this skill.

General rule:

> Fix known stale major-version priors first, then reduce complex query behavior to the smallest real probe against the exact version and engine before generalizing it.

---

## 16. Version and architecture check before generation

Before producing SurrealDB code, an agent should answer:

```text
SDK/server/engine version?
Rust toolchain version?
Connection mode? remote WS/HTTP vs embedded?
Persistence engine?
Exact locked dependency version?
SCHEMAFULL or SCHEMALESS?
Desktop/runtime framework if embedded?
```

Current proving evidence spans two shapes:

```text
Astra:
  SDK/server 3.2.4
  remote WebSocket
  SurrealKV server persistence
  independently tested in restart/concurrency gates

Brew & Batch:
  SDK/engine 3.2.4
  embedded SurrealKV inside a Tauri/local-first architecture
  case-study evidence supplied from the target conversion
```

Do not collapse those architectures into one generic connection model.
