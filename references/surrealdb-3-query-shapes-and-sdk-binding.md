# SurrealDB 3.2.4 Query Shapes, Binding, and Transaction Evidence

**Baseline:** SurrealDB 3.2.4 + Rust SDK 3.2.4  
**Last verified:** 2026-09-12

This reference focuses on small query/SDK details that are expensive when an AI model guesses from older SurrealDB examples.

## Evidence labels

- **VERIFIED API** — current official documentation.
- **CASE-STUDY EVIDENCE** — behavior reported from a real SurrealDB 3.2.4 proving project (Brew & Batch), but not independently reproduced by this skill repository as a standalone fixture yet.
- **PROJECT CONVENTION** — design choice, not a universal API rule.

---

## 1. `.bind()` is a typed SDK boundary

The Rust SDK's `.bind()` accepts values implementing its variable conversion contract, including key-value pairs, `vars!`, `object!`, maps of SurrealDB `Value`, and structs that implement `SurrealValue`.

Current official example:

```rust
use surrealdb::types::{SurrealValue, vars};

#[derive(SurrealValue)]
struct Filters {
    min_age: i64,
}

let mut response = db
    .query("SELECT * FROM person WHERE age >= $min_age")
    .bind(Filters { min_age: 18 })
    .await?
    .check()?;
```

Do not assume arbitrary JSON-shaped values are equivalent to native SDK values merely because they serialize with Serde.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/query
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types

### Minimal binding fixture

Before threading dynamic values through a large application backend, create a tiny compile/runtime fixture that exercises the exact types you plan to use:

```text
Surreal<Db>
query(...)
Variables / vars!
SurrealValue-derived struct or supported map
.bind(...)
.await
.check()
response.take(...)
```

If your architecture uses explicit transaction handles or wrappers such as `SerdeWrapper`, include them in the fixture too.

**GENERAL TESTING RULE.**

Brew & Batch found this useful for catching binding assumptions before they spread through a large Tauri command layer.

**CASE-STUDY EVIDENCE.**

---

## 2. Record parameters: verify construction/casting in the target query

`type::record()` is the current 3.x constructor name. The Rust SDK also exposes typed `RecordId` values.

However, do not turn that into a rule that every dynamic record expression must use one exact textual form. Query context matters.

Brew & Batch reported reliable behavior in some dynamic 3.2.4 queries using explicit casts:

```surql
<record>$record_id
```

Treat this as **CASE-STUDY EVIDENCE**, not a universal syntax replacement. Prefer an actual `RecordId` value when the Rust API naturally supports it, and run the smallest target query against the exact engine/transport when unsure.

General rule:

> Correct the stale `type::thing()` prior first, then verify the exact record-binding expression required by the query you are actually executing.

---

## 3. Always inspect statement errors

Transport success from `db.query(...).await` does not prove every statement succeeded.

For mutations, schema installation, or multi-statement queries:

```rust
let mut response = db
    .query(sql)
    .bind(vars)
    .await?
    .check()?;

let rows: Vec<MyRow> = response.take(0)?;
```

**VERIFIED API / TESTED APPLICATION PATTERN.**

Official source:
- https://surrealdb.com/docs/reference/rust/methods/query

---

## 4. `RELATE` data assignment uses normal `SET field = value` syntax

Current SurrealQL syntax is:

```surql
RELATE person:one->knows->person:two
SET friends = true, strength = 8;
```

or:

```surql
RELATE person:one->knows->person:two
CONTENT { friends: true, strength: 8 };
```

Inside `SET`, assignment is `field = value`. Do not paste object-literal `field: value` syntax into a `SET` clause.

**VERIFIED API.**

Official source:
- https://surrealdb.com/docs/reference/query-language/statements/relate

---

## 5. Explicit transactions roll back on error / `THROW`

SurrealDB transactions are all-or-nothing. A statement error inside an explicit transaction rolls it back; `THROW` can deliberately abort the transaction.

```surql
BEGIN TRANSACTION;
UPDATE account:one SET balance -= $amount;
UPDATE account:two SET balance += $amount;
IF account:one.balance < 0 {
    THROW "insufficient funds";
};
COMMIT TRANSACTION;
```

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions
- https://surrealdb.com/docs/reference/query-language/language-primitives/transactions

### Testing implication

For multi-record materialization, prove rollback with failure injection rather than merely trusting a transaction keyword is present.

```text
begin transaction
→ write root record
→ write child rows
→ inject deterministic failure
→ confirm no partial state survives
```

**GENERAL TESTING RULE.**

---

## 6. Query-shape observations that require target-version verification

Brew & Batch reported several useful SurrealDB 3.2.4 observations while converting a SCHEMAFULL local-first app:

- materializing IDs first with `LET $ids = SELECT VALUE id ...` made subsequent `FOR` loops reliable in the tested queries;
- ordering/projection combinations required care in the exact tested shape;
- explicit record casts were useful for bound record identifiers in dynamic expressions.

These are **CASE-STUDY EVIDENCE**, not yet generalized VERIFIED API rules in this skill.

When a model emits a complex SurrealQL loop, projection, graph mutation, or dynamic record expression:

1. reduce it to the smallest query that still expresses the behavior;
2. run it against the exact SurrealDB version and engine;
3. only then promote the result to a reusable capability entry.

---

## 7. Decimal serialization can cross representation boundaries

Exact decimals should remain exact, but their representation in diagnostics or JSON-shaped test output may be strings rather than native JSON numbers depending on the serialization layer.

Tests should assert the representation actually produced by the chosen boundary instead of assuming `Decimal` implies a JSON numeric token.

Brew & Batch caught assertion drift where a workflow expected numeric JSON while the actual SurrealDB/Rust serialization path produced decimal strings.

**CASE-STUDY EVIDENCE.**

General rule:

> Test semantic precision and boundary representation separately.

---

## 8. Scope limits

This document does not claim:

- `<record>$var` is always preferable to a typed `RecordId`;
- every `FOR` loop requires pre-materialized IDs;
- every `ORDER BY` query has the same projection constraints;
- remote WebSocket and embedded SurrealKV necessarily exercise identical transport code;
- SDK behavior should be inferred from JSON serialization alone.

When in doubt, version + engine + minimal real query outrank remembered syntax.