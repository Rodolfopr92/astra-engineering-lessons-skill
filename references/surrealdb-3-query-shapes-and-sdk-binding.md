# SurrealDB 3.2.4 Query Shapes, Binding, Response, and Value Boundaries

**Baseline:** SurrealDB 3.2.4 + Rust SDK 3.2.4  
**Additional case-study baseline:** DELPHIS / ARGOS on SurrealDB 3.2.x  
**Last verified:** 2026-09-12

This reference focuses on small query/SDK details that are expensive when an AI model guesses from older SurrealDB examples.

## Evidence labels

- **VERIFIED API** — current official documentation.
- **CASE-STUDY EVIDENCE** — behavior observed in a real proving project but not independently reduced by this skill repository yet.
- **PROJECT CONVENTION** — design choice, not a universal API rule.

---

## 1. `.bind()` is a typed SDK boundary

The Rust SDK's `.bind()` accepts values through the SDK value-conversion contract, including supported key-value pairs, maps/macros of SurrealDB values, and `SurrealValue` types.

Representative current pattern:

```rust
use surrealdb::types::SurrealValue;

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

Do not assume arbitrary JSON-shaped values are equivalent to native SDK values merely because they implement Serde.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/query
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types

### Minimal binding fixture

Before threading dynamic values through a large backend, create a tiny fixture that exercises the exact types you plan to use:

```text
Surreal<Db>
query(...)
SDK variables / supported maps / SurrealValue
.bind(...)
.await
.check() or take_errors()
response.take(...)
```

If your architecture uses transaction handles or wrapper types, include them in the fixture too.

**GENERAL TESTING RULE.**

---

## 2. `Ok(Response)` does not mean every statement succeeded

A query has two error layers:

```text
outer Result
→ request/transport/query-level failure

response statements
→ individual statement failures
```

Current official Rust documentation explicitly warns that `.query(...).await?` may return `Ok` while individual statements failed.

Use `.check()` when you want to fail on the first statement error:

```rust
let mut response = db.query(sql).await?.check()?;
```

Use `.take_errors()` when the statement indexes themselves are important diagnostic evidence:

```rust
let mut response = db.query(sql).await?;
let errors = response.take_errors();
for (index, error) in errors {
    eprintln!("statement {index}: {error}");
}
```

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/query
- https://surrealdb.com/docs/reference/rust/concepts/error-handling

### Error matching

Do not build durable retry/security logic by substring-matching human error text when the SDK exposes structured error kinds. Current 3.x documentation recommends matching the error kind/predicate because message wording may change.

**VERIFIED API.**

---

## 3. `.check()` and `.take_errors()` serve different reporting goals

They are not competing universal rules.

Use `.check()` when:

- migration or mutation code should abort immediately;
- one failure is enough to fail the operation;
- you do not need to retain the entire per-statement error map.

Use `.take_errors()` when:

- a diagnostic harness needs all failing statement indexes;
- a multi-statement response should preserve successful statements for inspection;
- a test/report needs precise statement-level evidence.

DELPHIS deliberately uses `take_errors()` in its adapter boundary for indexed diagnostics. ARGOS often uses `.check()` in fail-closed mutation/migration paths.

**CASE-STUDY EVIDENCE**, grounded in the current documented API.

---

## 4. Domain types can use `SurrealValue`, or an explicit SDK-Value → JSON → Serde boundary

The preferred native typed path is usually a type implementing `SurrealValue`.

But a legacy/domain model that intentionally derives Serde without `SurrealValue` can decode explicitly:

```rust
let raw: surrealdb::types::Value = response.take(0)?;
let json = raw.into_json_value();
let rows: Vec<MySerdeType> = serde_json::from_value(json)?;
```

DELPHIS uses this boundary to avoid pretending its existing Serde domain types implement the native SDK conversion trait.

**CASE-STUDY EVIDENCE.**

General rule:

> Make the conversion boundary explicit. Do not accidentally mix native SurrealDB values and JSON/Serde assumptions.

---

## 5. `NONE` and `NULL` are not synonyms

Current SurrealQL distinguishes absence from stored emptiness:

```surql
UPDATE person:one SET middle_name = NONE; -- field is absent/removed
UPDATE person:one SET middle_name = NULL; -- field exists with empty value
```

For schema types:

```surql
DEFINE FIELD middle_name ON TABLE person TYPE string | NONE;
```

is equivalent to an optional string.

**VERIFIED API.**

Official source:
- https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/none-and-null

### JSON serialization trap

Rust `Option::None` serialized through ordinary JSON becomes JSON `null`, not SurrealQL `NONE`. If application semantics require field absence rather than a stored null, do not assume a Serde JSON payload expresses that distinction automatically.

DELPHIS strips selected top-level JSON nulls before `CONTENT` writes where absence is the intended storage contract, while preserving intentional nested null values.

**CASE-STUDY EVIDENCE.**

---

## 6. Record parameters: verify construction/casting in the target query

`type::record()` is the current 3.x constructor name. The Rust SDK also exposes typed `RecordId` values.

Do not turn that into a rule that every dynamic record expression must use one exact textual form. Query context matters.

Brew & Batch reported reliable behavior in some dynamic 3.2.4 queries using:

```surql
<record>$record_id
```

Treat this as **CASE-STUDY EVIDENCE**, not a universal replacement. Prefer an actual `RecordId` value when the Rust API naturally supports it, and run the smallest target query against the exact engine/transport when unsure.

---

## 7. Record display text is not necessarily your canonical transport identity

A text-key record may be rendered with syntax delimiters that are not stored key characters. ARGOS encountered UUID-shaped text keys rendered in forms such as:

```text
document_envelope:`3406726d-bd4e-4f24-a49b-2c4fdc8ce514`
```

while its application contract used the canonical transport text:

```text
document_envelope:3406726d-bd4e-4f24-a49b-2c4fdc8ce514
```

It centralized record-id normalization and tested both representations.

**CASE-STUDY EVIDENCE.**

General rule:

> Do not scatter ad-hoc string splitting/quoting logic for record identities across the application. Centralize and test the application's text transport contract, or keep `RecordId` typed for as long as possible.

---

## 8. Dynamic identifiers are not ordinary bound values

Values should be bound. Query syntax/identifiers often cannot be parameterized in the same way.

When dynamic syntax is genuinely unavoidable, validate it against a narrow grammar before interpolation.

ARGOS's optional SurrealML query validates model identifiers/version strings before constructing:

```surql
RETURN ml::<validated_name><validated_version>($features)
```

and rejects punctuation/query-shaping characters.

**CASE-STUDY EVIDENCE / GENERAL INJECTION-SAFETY RULE.**

Do not interpret this as permission to interpolate ordinary user data. Bind ordinary data values.

---

## 9. `RELATE` data assignment uses normal `SET field = value` syntax

Current SurrealQL syntax:

```surql
RELATE person:one->knows->person:two
SET friends = true, strength = 8;
```

or:

```surql
RELATE person:one->knows->person:two
CONTENT { friends: true, strength: 8 };
```

Inside `SET`, assignment is `field = value`, not object-literal `field: value` syntax.

**VERIFIED API.**

Official source:
- https://surrealdb.com/docs/reference/query-language/statements/relate

---

## 10. Explicit transactions roll back on error / `THROW`

SurrealDB transactions are all-or-nothing. A statement error inside an explicit transaction rolls it back; `THROW` can deliberately abort it.

Current Rust SDK also has a manual transaction API with `commit()` and `cancel()`.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/concepts/transaction
- https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions

### Testing implication

For multi-record materialization, prove rollback with failure injection:

```text
begin transaction
→ write root
→ write children
→ inject deterministic failure
→ confirm no partial state survives
```

---

## 11. Query-shape observations still require target-version verification

Brew & Batch reported useful 3.2.4 observations:

- materializing IDs first with `LET $ids = SELECT VALUE id ...` made subsequent `FOR` loops reliable in tested queries;
- ordering/projection combinations required care in exact tested shapes;
- explicit record casts were useful for dynamic bound record identifiers.

These remain **CASE-STUDY EVIDENCE**.

When a model emits a complex SurrealQL loop, graph mutation, projection, or dynamic record expression:

1. reduce to the smallest query;
2. run against the exact SurrealDB version and engine;
3. only then promote to a reusable capability rule.

---

## 12. Decimal precision and representation are separate contracts

Exact decimals should remain exact, but diagnostics/JSON may serialize them as strings depending on the chosen boundary.

Test both:

```text
semantic value / precision
and
wire/JSON representation
```

Do not assume `Decimal` implies a JSON numeric token.

**CASE-STUDY EVIDENCE.**

---

## 13. Scope limits

This document does not claim:

- `<record>$var` is always preferable to typed `RecordId`;
- every domain struct must derive `SurrealValue`;
- JSON/Serde conversion is preferable when native typed conversion is available;
- every error path needs `take_errors()` rather than `.check()`;
- every `FOR` loop requires pre-materialized IDs;
- remote WebSocket and embedded SurrealKV exercise identical internals;
- SDK behavior should be inferred from JSON serialization alone.

When in doubt, exact version + engine + conversion boundary + minimal real query outrank remembered syntax.