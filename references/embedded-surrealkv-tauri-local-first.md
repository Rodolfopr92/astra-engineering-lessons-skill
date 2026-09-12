# Embedded SurrealKV + Tauri / Local-First Applications

**Primary baseline:** SurrealDB Rust SDK 3.2.4, SurrealKV embedded storage, Tauri 2.x  
**Last verified:** 2026-09-12

This reference corrects a common AI failure mode: treating every SurrealDB application as a remote HTTP/WebSocket client. Desktop and local-first Rust applications can embed SurrealDB directly in-process and persist through SurrealKV without running a separate database server.

## Evidence labels

- **VERIFIED API** — current official SurrealDB/Tauri documentation.
- **CASE-STUDY EVIDENCE** — exercised in an external proving project (Brew & Batch), but not independently reproduced by this skill repository as a standalone fixture yet.
- **PROJECT CONVENTION** — a design choice, not required by the technology.

---

## 1. Preflight: establish the connection model before writing code

Before generating SurrealDB code, answer all of these:

```text
SurrealDB SDK version?
Storage engine?
Embedded or remote?
If embedded, which feature flag?
Namespace/database names?
Persistent path source?
Does the app need a separate SurrealDB server process?
```

For an embedded SurrealKV desktop application, the intended shape can be:

```text
Tauri process
   ↓
SurrealDB Rust SDK
   ↓
embedded SurrealKV
   ↓
application data directory
```

No HTTP or WebSocket database server is inherently required for this architecture.

**VERIFIED API:** SurrealDB's Rust SDK explicitly supports embedded databases. The 3.2.4 crate exposes the `kv-surrealkv` feature, and SurrealKV can be opened through the local engine or `surrealkv://` endpoint form.

Official sources:
- https://surrealdb.com/docs/reference/rust
- https://surrealdb.com/docs/reference/rust/embedding
- https://surrealdb.com/docs/reference/rust/methods/new
- https://docs.rs/crate/surrealdb-core/3.2.4/features

---

## 2. Enable the storage backend deliberately

A stale or generic model may add `surrealdb` without the storage feature required by the chosen embedded engine.

For SurrealKV, verify the locked crate version and feature set. The 3.2.4 crate exposes:

```text
kv-surrealkv
```

A representative embedded pattern is:

```rust
use surrealdb::{
    Surreal,
    engine::local::SurrealKv,
};

let db = Surreal::new::<SurrealKv>(database_path).await?;
db.use_ns("app").use_db("main").await?;
```

Do not copy an example for `kv-mem` or `kv-rocksdb` and assume the selected engine is interchangeable at build time.

**VERIFIED API.**

---

## 3. Tauri persistent database paths belong under an app-specific data directory

Do not persist a production desktop database relative to the process working directory.

Tauri 2 exposes application-scoped paths through `PathResolver`. In Rust:

```rust
let app_data = app.path().app_data_dir()?;
let database_path = app_data.join("database");
std::fs::create_dir_all(&database_path)?;
```

`app_data_dir()` resolves to the platform's data directory plus the configured application bundle identifier.

**VERIFIED API.**

Official sources:
- https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html
- https://v2.tauri.app/reference/javascript/api/namespacepath/

### Project decision: AppData vs AppLocalData

Tauri exposes both `app_data_dir()` and `app_local_data_dir()`. Which one should contain an embedded database is an application/platform decision. Do not present either location as a universal Tauri rule.

**PROJECT CONVENTION.**

---

## 4. Restart/reopen is part of the persistence contract

Embedded mode removes the external server process, but it does **not** remove the need for persistence tests.

A meaningful durability test is:

```text
resolve fixed application data directory
→ open embedded SurrealKV
→ apply schema
→ write typed records
→ drop/close database handle
→ create a fresh handle against the exact same directory
→ select typed records
→ verify values and relationships
```

Do not substitute an in-memory engine for this test when the claim is durable local storage.

**GENERAL TESTING RULE.**

Brew & Batch also used same-directory reopen testing as part of its local-first conversion.

**CASE-STUDY EVIDENCE.**

---

## 5. SCHEMAFULL nested objects are strict in SurrealDB 3.x

This is a high-value correction for models carrying older behavior.

On a `SCHEMAFULL` table, an object is schemafull by default. Nested object fields must be declared, or the object field must intentionally be marked `FLEXIBLE`.

Example:

```surql
DEFINE TABLE product SCHEMAFULL;
DEFINE FIELD fulfillment ON product TYPE object;
DEFINE FIELD fulfillment.mode ON product TYPE string;
DEFINE FIELD fulfillment.lead_time_days ON product TYPE int;
```

For arrays of objects:

```surql
DEFINE FIELD components ON recipe TYPE array<object>;
DEFINE FIELD components.*.sku ON recipe TYPE string;
DEFINE FIELD components.*.quantity ON recipe TYPE decimal;
```

If arbitrary extra keys are intended:

```surql
DEFINE FIELD metadata ON event TYPE object FLEXIBLE;
```

SurrealDB's current documentation explicitly notes that as of 3.0, undefined nested fields on schemafull objects produce an error rather than being silently dropped.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/query-language/statements/define/field
- https://surrealdb.com/docs/reference/query-language/statements/define/table

### Fresh-install rule

When seeds or reference bundles contain nested data, validate them against an empty database created from the canonical schema. Counting table definitions or successfully upgrading an old developer database does not prove a fresh install works.

**GENERAL TESTING RULE**, reinforced by **CASE-STUDY EVIDENCE** from Brew & Batch, where undeclared nested fields were exposed only during real fresh-install execution.

---

## 6. Separate schema state, reference overlays, and historical business state

Do not automatically treat every data bundle as a schema migration.

Useful separation:

```text
schema/module installation state
    ↓
optional reference/cache overlay state
    ↓
transactional business/history state
```

An optional reference overlay can maintain its own replay marker, for example:

```text
version
name
checksum
applied_at
```

This prevents a recovered UI/reference bundle from being reapplied on every restart while avoiding false claims that it is part of the authoritative schema migration sequence.

**PROJECT/ARCHITECTURAL PATTERN**, supported by Brew & Batch case-study evidence.

---

## 7. Canonical schema bundle parity

When an authoritative schema exists in one location and an application carries a copy, detect drift mechanically.

A useful non-circular checksum strategy is:

```text
for canonical scripts in deterministic order:
    hash raw file bytes
combine (script name + raw-byte hash)
hash the combined manifest material
```

Do **not** make a checksum depend on a manifest that embeds the checksum itself.

Use the resulting schema identity in build metadata, seed metadata, static evidence, or diagnostic output so agents and humans can tell which exact schema bundle was executed.

**GENERAL REPRODUCIBILITY PATTERN**, supported by Brew & Batch case-study evidence.

---

## 8. Scope limits

This reference does not claim:

- every Tauri app should use SurrealKV;
- SurrealKV is the recommended storage engine for every production workload;
- `app_data_dir()` is always preferable to `app_local_data_dir()`;
- an embedded architecture has the same feature surface as every remote transport;
- query syntax tested in one embedded project automatically applies unchanged across future SurrealDB versions.

SurrealDB documentation notes that storage-engine and deployment choices have different operational tradeoffs. Re-evaluate the architecture when the application's durability, clustering, or workload requirements change.

---

## General rule

For desktop/local-first applications, **connection mode is part of the API contract**. Establish embedded-vs-remote architecture, exact storage feature, persistent path ownership, and restart behavior before generating application queries or migrations.