# Embedded SurrealKV + Tauri / Local-First Applications

**Primary baseline:** SurrealDB Rust SDK 3.2.4, SurrealKV embedded storage, Tauri 2.11.5 path APIs  
**Case-study baselines:** Brew & Batch, Omphalos, Saturno, ARGOS, DELPHIS  
**Last verified:** 2026-09-12

This reference corrects a common AI failure mode: treating every SurrealDB application as a remote HTTP/WebSocket client. Desktop and local-first Rust applications can embed SurrealDB directly in-process and persist through SurrealKV without running a separate database server.

If the target SurrealDB/Rust SDK differs from 3.2.4, or a Tauri path claim targets a version other than 2.11.5, treat the affected version-sensitive claim as unverified for that target until rechecked.

## Evidence labels

- **VERIFIED API** — current official SurrealDB/Tauri documentation for the named baseline.
- **CASE-STUDY EVIDENCE** — exercised in a proving repository but not independently reduced by this skill repository as a standalone fixture yet.
- **PROJECT CONVENTION** — a design choice, not required by the technology.

---

## 1. Preflight: establish the connection model before writing code

Before generating SurrealDB code, establish:

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

For an embedded SurrealKV desktop application, the supported technical shape is:

```text
Tauri process
   ↓
SurrealDB Rust SDK
   ↓
embedded SurrealKV
   ↓
application-owned persistent directory
```

No HTTP or WebSocket database server is inherently required.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust
- https://surrealdb.com/docs/reference/rust/embedding
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

**CASE-STUDY EVIDENCE**, now also covered by the pinned 3.2.4 reproducer suite. Do not promote it to a different SDK version without recompiling there.

### Feature selection

For SurrealKV, ensure the chosen crate version enables the corresponding embedded storage feature (`kv-surrealkv` in 3.2.4).

**VERIFIED API.**

Official source:
- https://docs.rs/crate/surrealdb/3.2.4

---

## 3. Opening the engine is not the whole boot observation

A useful diagnostic sequence is:

```text
resolve stable path
→ create parent directory
→ open SurrealKV
→ select namespace/database
→ apply/verify schema or migrations if the app has them
→ run a trivial readiness query
→ expose the initialized handle
```

Omphalos uses a post-`use_ns/use_db` readiness probe (`RETURN 1`) and records runtime metadata including path, namespace, database, schema version, engine, status, and open latency.

This is **CASE-STUDY EVIDENCE**, not a requirement imposed by SurrealDB or Tauri.

---

## 4. Tauri 2.11.5 application-scoped paths

Tauri 2.11.5 `PathResolver` exposes:

```rust
let app_data = app.path().app_data_dir()?;
let app_local_data = app.path().app_local_data_dir()?;
```

`app_data_dir()` resolves to the platform data directory plus the configured bundle identifier. `app_local_data_dir()` resolves to the platform local-data directory plus that identifier.

**VERIFIED API.**

Official source:
- https://docs.rs/tauri/2.11.5/tauri/path/struct.PathResolver.html

Choosing which of those locations should hold a particular application's database is a project/platform decision, not a Tauri capability rule.

---

## 5. Persistence evidence should cross a real lifecycle boundary

Embedded mode removes the external database daemon, but it does not remove the need for persistence evidence.

A basic reopen test is:

```text
open fixed directory
→ apply schema
→ write records
→ create fresh handle to same directory
→ read records
```

For stronger evidence, use a process-separated restart test:

```text
writer process
→ opens SurrealKV
→ writes durable state
→ exits completely

reader process
→ opens the same directory
→ reads state
→ verifies the claimed persisted properties
```

ARGOS and DELPHIS use separate test processes because embedded-engine shutdown can involve internal asynchronous lifecycle work; a same-process drop-and-immediate-reopen can accidentally test handle timing rather than a full process boundary.

**CASE-STUDY EVIDENCE / GENERAL TESTING RULE.**

### Do not conflate schema idempotence with restart persistence

Saturno tests re-running its migration routine through a live handle without duplicate migration rows. That proves migration idempotence, not process restart persistence.

---

## 6. Versioned storage is opt-in, not implied by SurrealKV

Do not assume that choosing SurrealKV automatically enables historical `VERSION` queries.

Current 3.2.4 Rust SDK documentation exposes an explicit versioned embedded connection:

```rust
let db = Surreal::new::<SurrealKv>("path/to/database")
    .versioned()
    .await?;
```

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/new
- https://surrealdb.com/docs/reference/query-language/statements/select

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

Official source:
- https://surrealdb.com/docs/reference/query-language/statements/define/field

### Fresh-install testing implication

If an application ships authoritative seed/reference data with nested objects, a fresh database built from the exact schema is stronger evidence than merely reapplying definitions over a long-lived developer database.

**GENERAL TESTING RULE**, supported by case-study failures in Brew & Batch.

---

## 8. Canonical schema-copy parity

If the same schema bundle is intentionally mirrored in two technical locations, drift can be detected mechanically by hashing the canonical scripts in deterministic order.

A non-circular approach is:

```text
for canonical scripts in deterministic order:
    hash raw bytes
combine (script name + raw-byte hash)
hash combined material
```

Do not make a checksum depend on a manifest that embeds its own checksum.

This is a reproducibility technique, not a requirement of SurrealDB or Tauri.

---

## 9. Scope limits

This reference does not claim:

- every Tauri app should use SurrealKV;
- SurrealKV is the preferred storage engine for every production workload;
- AppData is always preferable to AppLocalData;
- embedded and remote deployments have identical operational surfaces;
- same-handle schema idempotence proves restart durability;
- SurrealKV automatically enables version history;
- a readiness probe replaces deeper database validation;
- any particular multi-store or authority architecture should be chosen.

---

## General technical correction

For desktop/local-first work, connection mode and lifecycle affect which SurrealDB API and persistence evidence are relevant. Establish the exact SDK version, embedded engine feature, local handle type, persistent path API, namespace/database selection, versioning mode, and lifecycle boundary before trusting remembered examples.