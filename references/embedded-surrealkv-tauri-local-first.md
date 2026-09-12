# Embedded SurrealKV + Tauri / Local-First Applications

**Primary baseline:** SurrealDB Rust SDK 3.2.4, SurrealKV embedded storage, Tauri 2.x  
**Case-study baselines:** Brew & Batch, Omphalos, Saturno, ARGOS, DELPHIS  
**Last verified:** 2026-09-12

This reference corrects a common AI failure mode: treating every SurrealDB application as a remote HTTP/WebSocket client. Desktop and local-first Rust applications can embed SurrealDB directly in-process and persist through SurrealKV without running a separate database server.

## Evidence labels

- **VERIFIED API** — current official SurrealDB/Tauri documentation.
- **CASE-STUDY EVIDENCE** — exercised in a proving repository but not independently reduced by this skill repository as a standalone fixture yet.
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
Versioned storage required?
Does the app need a separate SurrealDB server process?
```

For an embedded SurrealKV desktop application:

```text
Tauri process
   ↓
SurrealDB Rust SDK
   ↓
embedded SurrealKV
   ↓
application data directory
```

No HTTP or WebSocket database server is inherently required.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust
- https://surrealdb.com/docs/reference/rust/methods/new

---

## 2. `SurrealKv` is the engine specifier; the local handle is `Surreal<Db>`

A stale or improvised type annotation may assume the constructor engine marker becomes the client handle type.

A pattern repeatedly used across Omphalos, Saturno, ARGOS, and DELPHIS is:

```rust
use surrealdb::{
    Surreal,
    engine::local::{Db, SurrealKv},
};

let db: Surreal<Db> = Surreal::new::<SurrealKv>(database_path).await?;
db.use_ns("app").use_db("main").await?;
```

`SurrealKv` selects the embedded engine for `new::<...>()`; the resulting embedded client handle is typed as `Surreal<Db>`.

**CASE-STUDY EVIDENCE strongly corroborated across multiple 3.x repositories.** Verify against the exact locked SDK when upgrading.

### Feature selection

For SurrealKV, ensure the chosen crate version enables the corresponding embedded storage feature (`kv-surrealkv` in the current 3.2.4 crate).

**VERIFIED API.**

---

## 3. Opening the engine is not the whole boot contract

A useful embedded boot sequence is:

```text
resolve stable path
→ create parent directory
→ open SurrealKV
→ select namespace/database
→ apply/verify migrations
→ run trivial readiness query
→ expose managed state
```

Omphalos explicitly performs a post-`use_ns/use_db` readiness probe (`RETURN 1`) and records runtime metadata including path, namespace, database, schema version, engine, status, and open latency.

This is a **CASE-STUDY PATTERN**, not a SurrealDB requirement, but it creates much better diagnostics than treating “constructor returned Ok” as full application readiness.

---

## 4. Tauri persistent database paths belong under an app-specific data directory

Do not persist a production desktop database relative to the process working directory.

Tauri 2 exposes application-scoped paths through `PathResolver`:

```rust
let app_data = app.path().app_data_dir()?;
let database_path = app_data.join("database");
std::fs::create_dir_all(&database_path)?;
```

`app_data_dir()` resolves to a platform data directory scoped by application identity.

**VERIFIED API.**

Official sources:
- https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html
- https://v2.tauri.app/reference/javascript/api/namespacepath/

### AppData vs AppLocalData

Tauri exposes both application data and local-data locations. Which one should contain the database is an application/platform decision.

**PROJECT CONVENTION.**

---

## 5. Persistence evidence should cross a real lifecycle boundary

Embedded mode removes the external database daemon, but it does **not** remove the need for persistence tests.

A basic reopen test is:

```text
open fixed directory
→ apply schema
→ write records
→ create fresh handle to same directory
→ read records
```

For stronger evidence, prefer a **process-separated restart test**:

```text
writer process
→ opens SurrealKV
→ writes durable state
→ exits completely

reader process
→ opens the same directory
→ reads state
→ verifies relationships / migration state / counts
```

ARGOS and DELPHIS use separate test processes because embedded-engine shutdown can involve internal asynchronous lifecycle work; a same-process drop-and-immediate-reopen can accidentally test handle timing rather than crash/restart durability.

**CASE-STUDY EVIDENCE / GENERAL TESTING RULE.**

### Do not conflate schema idempotence with restart persistence

Saturno tests re-running its migration routine through a live handle and verifies migration rows remain `[1,2,3,4]`. That proves **migration idempotence**, not process restart persistence.

Use the right test for the claim.

---

## 6. Versioned storage is opt-in, not implied by SurrealKV

Do not assume that choosing SurrealKV automatically enables historical `VERSION` queries.

Current Rust SDK documentation exposes an explicit versioned embedded connection:

```rust
let db = Surreal::new::<SurrealKv>("path/to/database")
    .versioned()
    .await?;
```

Current deployment documentation likewise exposes `versioned=true` configuration for supported engines.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/new
- https://surrealdb.com/docs/reference/query-language/statements/select

This corrects an older proving-project assumption that SurrealKV and time-travel history were automatically synonymous.

---

## 7. SCHEMAFULL nested objects are strict in SurrealDB 3.x

On a `SCHEMAFULL` table, nested object fields must be declared, or the object field must intentionally be `FLEXIBLE`.

Strict object example:

```surql
DEFINE TABLE product SCHEMAFULL;
DEFINE FIELD fulfillment ON TABLE product TYPE object;
DEFINE FIELD fulfillment.mode ON TABLE product TYPE string;
DEFINE FIELD fulfillment.lead_time_days ON TABLE product TYPE int;
```

Array-of-object example:

```surql
DEFINE FIELD components ON TABLE recipe TYPE array<object>;
DEFINE FIELD components.*.sku ON TABLE recipe TYPE string;
DEFINE FIELD components.*.quantity ON TABLE recipe TYPE decimal;
```

Intentional dynamic metadata:

```surql
DEFINE FIELD metadata ON TABLE event TYPE object FLEXIBLE;
```

Current documentation notes that as of 3.0, undefined nested fields on schemafull objects produce an error rather than being silently dropped.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/query-language/statements/define/field
- https://surrealdb.com/docs/reference/query-language/statements/define/table

### Fresh-install rule

Validate authoritative seeds/reference bundles against an empty database built from the canonical schema. Upgrading a long-lived developer database can hide missing nested definitions.

This failure mode appeared independently in Brew & Batch and Alexandria-related validation work.

**CASE-STUDY EVIDENCE / GENERAL TESTING RULE.**

---

## 8. Separate schema state, reference overlays, projection state, and business history

These are different ledgers:

```text
schema/module installation state
projection/rebuild checkpoint state
optional reference/cache overlay state
transactional business/history state
```

An optional overlay may have its own replay marker:

```text
version
name
checksum
applied_at
```

A graph projector may have a separate `(target, aggregate, revision/event)` checkpoint. Do not overload the schema migration version to answer all of these questions.

**GENERAL ARCHITECTURAL PATTERN.**

---

## 9. Canonical schema bundle parity

When authoritative schema exists in one location and an application carries a copy, detect drift mechanically.

A useful non-circular strategy:

```text
for canonical scripts in deterministic order:
    hash raw bytes
combine (script name + raw-byte hash)
hash combined material
```

Do not make the checksum depend on a manifest that embeds its own checksum.

Use schema identity in diagnostics/build/seed metadata where it improves traceability.

**GENERAL REPRODUCIBILITY PATTERN**, supported by Brew & Batch and migration-ledger patterns in ARGOS.

---

## 10. Scope limits

This reference does not claim:

- every Tauri app should use SurrealKV;
- SurrealKV is the preferred storage engine for every production workload;
- app data is always preferable to app local data;
- an embedded architecture has the same operational surface as every remote deployment;
- a successful same-handle migration test proves restart durability;
- SurrealKV automatically enables version history;
- a readiness probe replaces deeper database health/consistency testing.

SurrealDB currently describes SurrealKV as an important embedded/local-first path while still documenting deployment trade-offs. Re-evaluate engine and durability assumptions when workload or deployment changes.

---

## General rule

For desktop/local-first applications, **connection mode and lifecycle are part of the API contract**. Establish exact SDK version, engine feature, `Surreal<Db>` handle shape, persistent path ownership, namespace/database selection, migration/readiness behavior, versioning mode, and process-separated durability evidence before declaring the embedded store production-ready.