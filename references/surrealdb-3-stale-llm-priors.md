# SurrealDB 3.x: Stale LLM Priors and Current Corrections

**Target baseline:** SurrealDB server/engine 3.2.4 + Rust SDK 3.2.4  
**Last verified:** 2026-09-12

This document is a fast correction layer for AI coding agents whose internal knowledge is dominated by older SurrealDB releases.

## Evidence legend

- **VERIFIED API** — current official SurrealDB documentation / SDK API.
- **TESTED BEHAVIOR** — independently reproduced against the named version.
- **CASE-STUDY EVIDENCE** — observed in a real proving repository but not independently reduced by this skill repository.
- **PROJECT CONVENTION** — application architecture, not a SurrealDB requirement.

---

## 1. `type::thing()` became `type::record()`

**STALE**

```surql
type::thing('person', $id)
```

**CURRENT 3.x**

```surql
type::record('person', $id)
```

**VERIFIED API.**

Official source:
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/type

---

## 2. Intrinsic record IDs are typed records, not strings

**DANGEROUS**

```rust
#[derive(SurrealValue)]
struct Person { id: String }
```

**CURRENT TYPE**

```rust
use surrealdb::types::{RecordId, SurrealValue};

#[derive(SurrealValue)]
struct Person { id: RecordId }
```

A real 3.2.4 persistence test reproduced:

```text
Expected string, got record
```

**VERIFIED API + TESTED BEHAVIOR.**

A valid project alternative is to omit intrinsic `id` and keep an explicit logical key such as `artifact_id`. That is a **PROJECT CONVENTION**.

---

## 3. `SurrealValue` is the native Rust conversion contract

**STALE ASSUMPTION**

> Serde derives alone define native SDK conversion.

Current 3.x typed SDK code uses `SurrealValue`; its `#[surreal(...)]` attributes are distinct from Serde attributes.

```rust
use surrealdb::types::SurrealValue;

#[derive(SurrealValue)]
struct Employee { name: String }
```

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types
- https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes

---

## 4. Prefer current public type paths

Current examples use:

```rust
use surrealdb::types::{RecordId, SurrealValue};
```

Do not reflexively generate old/internal paths such as `surrealdb::sql::Thing` for a 3.x target.

**VERIFIED API.**

---

## 5. Tuple record resources still exist

Do not over-correct by manually constructing `RecordId` for every SDK operation.

```rust
let person: Option<Person> = db.select(("person", "tobie")).await?;
let created: Option<Person> = db.create(("person", "tobie")).content(data).await?;
```

**VERIFIED API.**

---

## 6. `.await?` on `query()` does not prove every statement succeeded

A successful outer request may still contain statement failures.

Fail on the first:

```rust
let response = db.query(sql).await?.check()?;
```

Preserve all indexed statement failures:

```rust
let mut response = db.query(sql).await?;
let failures = response.take_errors();
```

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/query
- https://surrealdb.com/docs/reference/rust/concepts/error-handling

Use structured error kinds/predicates for durable control flow instead of matching message strings.

---

## 7. Do not use application read-modify-write for hot counters

**RACE-PRONE**

```rust
let current = load_checkpoint(db).await?;
save_checkpoint(db, current.records_seen + 1).await?;
```

A proven pattern is one server-side mutation:

```surql
UPSERT type::record('ingest_checkpoint', $source_code) SET
    source_code = $source_code,
    records_seen += $seen,
    records_inserted += $inserted,
    records_replayed += $replayed,
    records_failed += $failed,
    updated_at = time::now();
```

Then inspect statement errors and retry only known retryable conflicts.

**TESTED BEHAVIOR on 3.2.4.**

---

## 8. In-memory success is not disk/restart evidence

`Mem` can be useful, but it does not prove SurrealKV durability, process restart, remote serialization, or embedded-engine reopen behavior.

Strong server evidence:

```text
write → kill server → restart same directory → reconnect → re-read
```

Strong embedded evidence:

```text
writer process writes SurrealKV → process exits → reader process opens same path → re-read
```

ARGOS and DELPHIS provide case-study evidence for process-separated embedded restart tests; Astra independently proves remote/server restart persistence.

---

## 9. SurrealDB does not always mean a remote server

The Rust SDK supports embedded engines.

```rust
use surrealdb::{Surreal, engine::local::{Db, SurrealKv}};

let db: Surreal<Db> = Surreal::new::<SurrealKv>(database_path).await?;
db.use_ns("app").use_db("main").await?;
```

**VERIFIED API + repeated CASE-STUDY EVIDENCE.**

For the current 3.2.4 crate, enable the correct embedded storage feature (`kv-surrealkv`).

---

## 10. `SurrealKv` is an engine selector, not the local client-handle type

A recurring model mistake is to invent:

```rust
Surreal<SurrealKv>
```

for application state.

Across current embedded proving repositories the constructor uses `SurrealKv`, while the resulting handle is:

```rust
Surreal<Db>
```

**CASE-STUDY EVIDENCE across Omphalos, Saturno, ARGOS, and DELPHIS.** Re-check on SDK upgrades.

---

## 11. SCHEMAFULL nested objects are strict in 3.x

**STALE ASSUMPTION**

> Declaring a top-level object permits arbitrary nested keys.

Strict shape:

```surql
DEFINE TABLE order SCHEMAFULL;
DEFINE FIELD shipping ON TABLE order TYPE object;
DEFINE FIELD shipping.city ON TABLE order TYPE string;
```

Intentional dynamic shape:

```surql
DEFINE FIELD metadata ON TABLE order TYPE object FLEXIBLE;
```

Array of objects:

```surql
DEFINE FIELD items ON TABLE order TYPE array<object>;
DEFINE FIELD items.*.sku ON TABLE order TYPE string;
```

Current docs explicitly note stricter 3.0 behavior for undefined nested fields.

**VERIFIED API.**

---

## 12. `NONE` and `NULL` are different

**STALE ASSUMPTION**

> Missing and null are interchangeable.

Current semantics:

```surql
SET field = NONE; -- remove/absence
SET field = NULL; -- stored empty value
```

`TYPE string | NONE` is the optional-field form.

**VERIFIED API.**

Official source:
- https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/none-and-null

### Rust/JSON trap

`Option::None` serialized to ordinary JSON becomes JSON `null`, not SurrealQL `NONE`. DELPHIS treats this as an explicit adapter concern rather than assuming Serde expresses field absence.

**CASE-STUDY EVIDENCE.**

---

## 13. `.bind()` is a typed SDK boundary

Do not assume any arbitrary Serde/JSON value is automatically the same as a SurrealDB native value.

Use the documented variable conversion path and test complex binding types with a tiny fixture before spreading them through a large backend.

**VERIFIED API / GENERAL TESTING RULE.**

See `surrealdb-3-query-shapes-and-sdk-binding.md`.

---

## 14. `RELATE ... SET` uses assignment syntax

**WRONG**

```surql
RELATE a->edge->b SET { strength: 5 };
```

**CURRENT**

```surql
RELATE a->edge->b SET strength = 5;
```

or:

```surql
RELATE a->edge->b CONTENT { strength: 5 };
```

**VERIFIED API.**

---

## 15. Relation tables are first-class typed schema

Current 3.x supports:

```surql
DEFINE TABLE works_at
    TYPE RELATION FROM person TO company
    SCHEMAFULL;
```

`IN person OUT company` is also valid.

Do not reduce every relationship to a foreign-key-like scalar because the model remembers only relational SQL patterns.

**VERIFIED API.**

Official source:
- https://surrealdb.com/docs/reference/query-language/statements/define/table

---

## 16. Full-text index syntax changed in 3.0

**STALE PRE-3.0**

```surql
SEARCH ANALYZER app_text BM25
```

**CURRENT 3.x**

```surql
FULLTEXT ANALYZER app_text BM25
```

with functions such as:

```surql
search::score(1)
search::rrf(...)
```

**VERIFIED API.**

Official source:
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/search

---

## 17. HNSW + BM25 + RRF hybrid search is current, not hypothetical

Current docs support:

```surql
DEFINE INDEX hnsw_embedding
    ON TABLE chunk FIELDS embedding
    HNSW DIMENSION 384 DIST COSINE;

SELECT id, vector::distance::knn() AS distance
FROM chunk
WHERE embedding <|20,100|> $embedding;
```

and RRF fusion with lexical results.

ARGOS implements the same general pattern on 3.2.3.

**VERIFIED API + CASE-STUDY EVIDENCE.**

See `surrealdb-3-graph-search-and-changefeeds.md`.

---

## 18. `CHANGEFEED` is not the same thing as time-travel `VERSION`

A changefeed is an opt-in mutation log with retention:

```surql
DEFINE TABLE opportunity CHANGEFEED 30d;
SHOW CHANGES FOR TABLE opportunity SINCE $cursor LIMIT 100;
```

A historical `SELECT ... VERSION ...` requires a versioning-enabled supported storage engine.

Embedded SurrealKV current Rust setup:

```rust
let db = Surreal::new::<SurrealKv>(path).versioned().await?;
```

Do not teach:

> "SurrealKV automatically makes every table bitemporal."

**VERIFIED API.**

---

## 19. Transactions are all-or-nothing, but prove the actual path

Sequential application `upsert().await?` calls are not a transaction.

Current SurrealDB supports explicit transactions, and the Rust SDK also exposes a manual transaction API.

Use failure injection to prove that a multi-record materialization leaves no partial state.

**VERIFIED API / GENERAL TESTING RULE.**

---

## 20. Dynamic identifiers are not ordinary data bindings

Bind values. When query syntax itself must be dynamic, use a strict grammar/allowlist before interpolation.

ARGOS validates SurrealML model-name/version syntax before constructing a dynamic `ml::<name><version>(...)` function call.

**CASE-STUDY EVIDENCE / GENERAL INJECTION-SAFETY RULE.**

---

## 21. Record display syntax is not automatically a canonical transport string

ARGOS found text record IDs with UUID-shaped keys may be rendered with backtick delimiters when cast to string:

```text
document_envelope:`uuid-text-key`
```

while its transport contract used:

```text
document_envelope:uuid-text-key
```

Backticks are syntax/display delimiters in that case, not stored key characters. Centralize application record-text normalization or keep IDs typed.

**CASE-STUDY EVIDENCE.**

---

## 22. Migration history should be explicit, not an immortal bootstrap string

Multiple mature proving repos converged on:

```text
ordered append-only migrations
+ version table
+ deterministic migration identity
+ statement-error inspection
+ idempotence test
```

ARGOS additionally records a checksum of migration content.

This is **CASE-STUDY EVIDENCE / GENERAL MIGRATION DISCIPLINE**, not a mandatory SurrealDB API.

See `surrealdb-3-migrations-authority-and-rebuildability.md`.

---

## 23. Decide SurrealDB's authority role explicitly

Both of these are valid:

```text
ARGOS:
SurrealDB = operational authority
```

```text
Alexandria:
SurrealDB = rebuildable graph/context projection
```

Do not copy one architecture into the other accidentally. Authority/rebuildability is a system design decision, not a property imposed by SurrealDB.

---

## 24. Version/architecture preflight before generation

Before producing SurrealDB code, answer:

```text
server/SDK version?
Rust toolchain?
remote or embedded?
storage engine and features?
versioning enabled?
SCHEMAFULL or SCHEMALESS?
authoritative or projection store?
Tauri/runtime path ownership if embedded?
expected restart/rebuild proof?
```

Exact version + deployment mode + authority contract + minimal real probe outrank remembered syntax.