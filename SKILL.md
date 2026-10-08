---
name: surreal_rust_tauri
description: Use when writing, debugging, migrating, or reviewing SurrealDB 3.x + Rust/Tauri code, especially when model knowledge may reflect SurrealDB 1.x/2.x APIs. Provides version-checked corrections for Rust SDK types, SurrealQL, embedded SurrealKV, schema behavior, search, transactions, persistence, and Tauri local-data integration.
---

# surreal_rust_tauri

This skill is a **versioned capability correction layer** for AI coding agents working with SurrealDB 3.x + Rust and Tauri/local-first applications.

Its job is technical: correct stale model knowledge about what the current stack supports, how its APIs behave, and which failure modes have been verified. It is **not** an architecture planner. Store ownership, topology, system-of-record decisions, and product architecture belong in the target project's planning/ADRs.

## Verified baseline

- SurrealDB server/engine: **3.2.4**
- SurrealDB Rust SDK: **3.2.4**
- Rust proving baseline: **1.96**
- Official SDK minimum Rust version: **1.89**
- Tauri documentation baseline: **2.11.5** (`PathResolver` application-data APIs)
- Last full verification pass: **2026-09-12**
- Last partial verification: **2026-10-01** (schema and query-shape reproducers; engine crate pinning)
- Routine reverification interval: **90 days maximum**

## 1. Hard version-decay guard

This rule overrides every example in this skill.

Before applying a version-sensitive claim, inspect the target repository's lockfile/manifests and identify the actual SurrealDB server/engine, Rust SDK, Rust toolchain, connection mode, and Tauri version where relevant.

For an embedded engine, the engine version is the resolved `surrealdb-core` in `Cargo.lock`, not the `surrealdb` line in `Cargo.toml`. See [An exact SDK pin does not pin the engine](#an-exact-sdk-pin-does-not-pin-the-engine).

If the target uses a SurrealDB server/engine or Rust SDK version **other than 3.2.4**, or a Tauri-sensitive claim targets a Tauri version **other than 2.11.5**:

1. Treat the affected version-sensitive `VERIFIED API` or `TESTED BEHAVIOR` claims here as **UNVERIFIED FOR THE TARGET VERSION**.
2. Re-check official documentation for the target version.
3. Run the relevant reproducer when one exists.
4. State the baseline mismatch before relying on the skill's version-sensitive guidance.
5. Promote the claim for the new baseline only after the evidence ledger is updated.

Reverification is also required when any of these occurs:

- a SurrealDB minor or major version is adopted;
- the Rust SDK version changes;
- the relevant Tauri API version changes;
- official docs deprecate or alter a covered API;
- a runtime result contradicts the skill;
- a relevant security advisory lands;
- **90 days** have passed since the last full verification pass.

A newer upstream release existing somewhere does not automatically invalidate 3.2.4 claims for a project still locked to 3.2.4. The target lockfile remains first authority.

## 2. Source-of-truth hierarchy

When sources disagree:

1. **Current target repository and exact lockfile/manifests**
2. **Official documentation/API for the exact target version**
3. **A reproducible compile/runtime/integration test against that target**
4. **This skill**
5. **Model memory**

This skill is a capability patch, not an oracle.

## 3. Evidence labels

- **VERIFIED API** — current official documentation/API for the named baseline.
- **TESTED BEHAVIOR** — independently reproduced against the named version and environment.
- **CASE-STUDY EVIDENCE** — observed in a real proving repository but not independently minimized here.
- **PROJECT CONVENTION** — a design decision, not a technology requirement.
- **ARCHITECTURAL INTENT** — planned/recommended, not implementation evidence.
- **FALSE / NOT A GENERAL RULE** — explicitly retained to block known over-generalizations.

Never promote a case-study workaround into a universal API rule merely because several reports repeat it.

### Cross-version evidence rule

Case-study evidence from a different patch/minor version is a **lead**, not automatic proof for this baseline. For example, a behavior observed on SurrealDB 3.2.3 may guide investigation for 3.2.4, but it becomes `TESTED BEHAVIOR` for 3.2.4 only after 3.2.4 verification.

## 4. High-value SurrealDB 3.x corrections

### Record constructor

```surql
-- stale pre-3.0
type::thing('person', $id)

-- current 3.x
type::record('person', $id)
```

Official: https://surrealdb.com/docs/reference/query-language/functions/database-functions/type

### Intrinsic record IDs

```rust
use surrealdb::types::{RecordId, SurrealValue};

#[derive(SurrealValue)]
struct Row {
    id: RecordId,
}
```

Do not deserialize intrinsic `id` into `String` unless the query deliberately casts it to text and the transport contract expects text.

Official: https://docs.rs/surrealdb/3.2.4/surrealdb/types/record_id/struct.RecordId.html

### Native Rust conversion

`SurrealValue` is the native SDK value-conversion contract. Serde and `#[surreal(...)]` are distinct systems.

Official:
- https://surrealdb.com/docs/reference/rust/concepts/working-with-types
- https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes

### Embedded SurrealKV

```rust
use surrealdb::{Surreal, engine::local::{Db, SurrealKv}};

let db: Surreal<Db> = Surreal::new::<SurrealKv>(path).await?;
db.use_ns("app").use_db("main").await?;
```

SurrealDB does not inherently mean a separate WS/HTTP server. Current 3.2.4 embedded SurrealKV requires the `kv-surrealkv` feature.

Official:
- https://surrealdb.com/docs/reference/rust
- https://surrealdb.com/docs/reference/rust/embedding
- https://docs.rs/crate/surrealdb/3.2.4

### An exact SDK pin does not pin the engine

`surrealdb = "=3.2.4"` pins only the SDK crate. The SDK reaches `surrealdb-core`, `surrealdb-types` and `surrealdb-types-derive` through caret requirements, and `surrealdb-core` reaches `surrealdb-collections` and `surrealdb-strand` the same way. Since 3.3.0 was published (2026-09-24), a resolution without a lockfile builds SDK 3.2.4 on 3.3.x engine crates, under a 3.2.4 label. Pinning only `surrealdb-core` still lets the two inner crates float. For `Mem` and `SurrealKv`, these crates *are* the database. For a remote connection, check the server's version as well.

```toml
surrealdb = { version = "=3.2.4", default-features = false, features = ["kv-surrealkv"] }
surrealdb-core = { version = "=3.2.4", default-features = false }
surrealdb-types = "=3.2.4"
surrealdb-types-derive = "=3.2.4"
surrealdb-collections = "=3.2.4"
surrealdb-strand = "=3.2.4"
```

Commit `Cargo.lock`, run CI with `--locked`, and read the lock: every `surrealdb-*` crate except `surrealdb-protocol`, which has its own version line, must show the baseline.

**TESTED BEHAVIOR** (dependency resolution, 2026-10-01; corrected 2026-10-08 after the lock showed collections and strand at 3.3.0). See the [ledger](references/verification-status.md).

### SCHEMAFULL nested data

```surql
DEFINE FIELD metadata ON TABLE event TYPE object FLEXIBLE;

-- strict nested alternative
DEFINE FIELD shipping ON TABLE order TYPE object;
DEFINE FIELD shipping.city ON TABLE order TYPE string;
```

Current 3.x errors on undeclared nested fields in a SCHEMAFULL object unless the relevant object-containing field is intentionally `FLEXIBLE`.

Official: https://surrealdb.com/docs/reference/query-language/statements/define/field

### SCHEMAFULL top-level fields and permissive fixtures

A SCHEMAFULL table refuses a top-level field that nobody defined:

```text
Found field 'language_preference', but no such field exists for table 'user'
```

A test that creates rows in a table it never defined gets a schemaless table. The same write passes there, so the test proves nothing about production.

- Add every new field through the migration.
- Build test tables from the production migration script.

**TESTED BEHAVIOR.** Reproducer: `schemafull_refuses_an_undefined_top_level_field_an_undefined_table_hides_it`. Details: [migrations §10](references/surrealdb-3-migrations-and-recovery-evidence.md).

### SCHEMALESS tables still enforce the fields they define

SCHEMALESS accepts unknown fields, but it still applies `TYPE` and `ASSERT` to the fields it defines. An optional enumeration looks like this:

```surql
DEFINE FIELD language ON TABLE user TYPE option<string>
    ASSERT $value = NONE OR $value IN ['pt-br', 'en'];
```

**TESTED BEHAVIOR.** Reproducer: `schemaless_tables_still_enforce_the_fields_they_define`.

### Query response errors

```rust
let mut response = db.query(sql).await?;
let errors = response.take_errors();
```

or:

```rust
let response = db.query(sql).await?.check()?;
```

Outer `Ok` does not prove every statement succeeded.

Official: https://surrealdb.com/docs/reference/rust/concepts/error-handling

### ORDER BY needs its field in the projection

```surql
SELECT text FROM log ORDER BY created_at DESC;              -- parse error
SELECT text, created_at FROM log ORDER BY created_at DESC;  -- works
```

The error, ``Missing order idiom `created_at` in statement selection``, is raised when the request is parsed.

- The whole request fails, and none of its statements run, not even valid ones before the bad one.
- Code that treats an error as "no rows" goes silent for good.

**TESTED BEHAVIOR.** Reproducer: `order_by_needs_its_field_selected_and_the_parse_error_fails_the_whole_request`.

### `NONE` vs `NULL`

```surql
SET field = NONE; -- absence/remove
SET field = NULL; -- stored empty value
```

Do not assume JSON `null` means SurrealQL `NONE`.

### JSON timestamps are strings

JSON has no datetime type. A `chrono::DateTime` passed through `serde_json::to_value` is stored as a **string**. A string never matches `created_at > d'…'`, so those rows silently drop out of the result.

Three fixes:

- Define the field `TYPE datetime`. The JSON write is then refused loudly.
- Write timestamps in SurrealQL (`time::now()`).
- Cast text that is already stored: `<datetime> created_at > d'…'`.

**TESTED BEHAVIOR.** Reproducer: `json_timestamps_are_strings_and_drop_out_of_datetime_comparisons`.

### `UPSERT … MERGE` vs `UPSERT … CONTENT`

- `MERGE` keeps the fields the payload does not mention.
- `CONTENT` replaces the whole record.

Partial writers to one deterministic record need `MERGE`: repeated readings of one document, status patches, webhooks.

**TESTED BEHAVIOR.** Reproducer: `upsert_merge_keeps_absent_fields_while_content_replaces_the_record`.

### Relation tables

```surql
DEFINE TABLE works_at
  TYPE RELATION FROM person TO company
  SCHEMAFULL;
```

### Full-text syntax

```surql
-- stale pre-3.0
SEARCH ANALYZER app_text BM25

-- current 3.x
FULLTEXT ANALYZER app_text BM25
```

Official: https://surrealdb.com/docs/reference/query-language/statements/define/overview

### Hybrid search & HNSW parameter rule

Current 3.x supports BM25 full-text, HNSW KNN vector search, `vector::distance::knn()`, and RRF fusion through `search::rrf()`.

**Critical Pitfall**: In SurrealDB 3.2.4, `<|k, ef|>` strictly requires **unsigned integer literals**. Passing query parameters like `<|$limit, 40|>` throws a runtime parse error. Clamp the limit in Rust and interpolate it as a literal (`<|{limit}, 40|>`).

Official: https://surrealdb.com/docs/reference/query-language/functions/database-functions/search

### Multi-tenant WebSocket session isolation

Do not share or cache cloned `Surreal<Ws>` connections across threads in multi-tenant environments. In SurrealDB 3.2.4, concurrent queries on cloned connections race the SDK's internal authentication replay. Open a fresh dedicated WebSocket connection (`Surreal::new::<Ws>(&endpoint).await`) per tenant session.

### Migration credential security

Avoid running schema migrations with `surreal sql --password ...`, which exposes root credentials in `ps aux` and `/proc/<pid>/cmdline`. Execute migrations over SurrealDB's HTTP REST endpoint (`POST /sql` with HTTP Basic Auth headers).

### Deterministic composite record keys

Avoid auto-increment `SELECT turn_index ... ORDER BY turn_index DESC` queries for hot event streams or conversational memory, which race and collide under concurrent turns. Generate deterministic record keys with SHA-256 (`format!("{digest:x}_{turn_index}")`) and mutate with `UPSERT type::record(...)`.

### Changefeed vs historical versioning

```surql
DEFINE TABLE opportunity CHANGEFEED 30d;
SHOW CHANGES FOR TABLE opportunity SINCE $cursor;
```

is a replayable mutation feed. It is not the same mechanism as historical `SELECT ... VERSION ...`, which requires a supported storage engine with versioning enabled.

For embedded SurrealKV 3.2.4:

```rust
let db = Surreal::new::<SurrealKv>(path)
    .versioned()
    .await?;
```

Official: https://surrealdb.com/docs/reference/rust/methods/new

### Transactions

Sequential application writes are not one all-or-nothing database transaction. When the required invariant is atomic multi-record mutation, use a real transaction and prove rollback with failure injection.

### Multi-line SurrealQL strings in Rust

Do not use escaped backslashes (`\\`) to split query strings across lines in Rust. SurrealQL has no backslash line continuation and errors with `Parse error: Invalid token '\'`. Use a single trailing backslash (`\`) to trim Rust whitespace, or use multi-line strings.

### `db.select()` trait boundary

`db.select((table, id))` requires `R: SurrealValue`. For domain types implementing only Serde, select into `Option<serde_json::Value>` and deserialize explicitly via `serde_json::from_value`.

### Multi-tenant test isolation on shared servers

Avoid hardcoded static tenant slugs (e.g. `astra_company_acme`) in integration tests against shared SurrealDB instances. Use dynamic unique slugs (e.g. `support::unique("cr_acme")`) to prevent cross-test namespace clobbering and state bleed.

## 5. Tauri 2.11.5 path contract

Tauri 2.11.5 exposes application-scoped path APIs including:

```rust
let data = app.path().app_data_dir()?;
let local = app.path().app_local_data_dir()?;
```

`app_data_dir()` resolves to the platform data directory plus the app bundle identifier, while `app_local_data_dir()` resolves to the platform local-data directory plus that identifier. Choosing between them for a product is a project/platform decision.

Official: https://docs.rs/tauri/2.11.5/tauri/path/struct.PathResolver.html

## 6. General technical rules

1. **Use real boundary tests, not mocks alone.**
2. **Match the deployed connection model.** Remote and embedded paths prove different things.
3. **Use process-separated restart evidence for strong embedded durability claims.** Same-handle schema idempotence is not restart proof.
4. **For concurrent mutation invariants, check whether the database can perform the operation atomically before implementing application-side read-modify-write.**
5. **Validate SCHEMAFULL seeds on a fresh database.**
6. **Use exact decimal arithmetic for authoritative money/tax.**
7. **Treat subprocess output as untrusted.**
8. **Use structured error kinds instead of durable substring matching where the SDK exposes them.**
9. **Bind ordinary data; strictly validate any dynamic query identifier/syntax that cannot be bound.**
10. **A resource-killed build is neither pass nor application compile failure.** Report `BLOCKED / INDETERMINATE` until meaningful diagnostics exist.
11. **The strength of a claim must not exceed the strength of its evidence.**
12. **Do not use this skill to make architecture decisions.**
13. **Build test schemas from the production migration.** An undefined table is schemaless and accepts writes that production refuses.
14. **Never treat a query error as an empty result.** A parse error fails every statement in the request.
15. **Filter test suites with naming discipline and execution assertions.** Cargo test filters match by substring; a test omitting the prefix silently skips, reporting a green test run with 0 tests executed.
16. **Verification tools must tolerate `rustfmt` formatting.** Do not use brittle single-line regexes to validate Rust code patterns that `cargo fmt` reflows into multi-line layouts.
17. **`#[serde(default)]` does not skip serialization.** In audited provenance structs, pair `#[serde(default)]` with `skip_serializing_if` to prevent default fields from polluting JSON and corrupting historical cryptographic hashes.

## 7. Reproducers are first-class evidence

The repository contains `reproducers/` for small executable checks of high-value corrections.

A reproducer should be:

- version-pinned, including the engine crates the SDK pulls in, with `Cargo.lock` committed and runs made `--locked`;
- minimal;
- fast enough to run routinely;
- linked from the claim it verifies;
- explicit about engine/transport;
- incapable of silently upgrading a claim for a different baseline.

A reproducer that has not run successfully is **not** `TESTED BEHAVIOR`. Record it as pending until execution evidence exists.

## 8. Topic index

| Topic | Reference | Primary purpose |
|---|---|---|
| SurrealDB stale priors | [surrealdb-3-stale-llm-priors.md](references/surrealdb-3-stale-llm-priors.md) | Fast 1.x/2.x → 3.x correction layer. |
| Core SurrealDB 3.2.4 contract | [surrealdb-3-contract-and-pitfalls.md](references/surrealdb-3-contract-and-pitfalls.md) | RecordId, SurrealValue, atomicity and persistence. |
| Embedded SurrealKV + Tauri | [embedded-surrealkv-tauri-local-first.md](references/embedded-surrealkv-tauri-local-first.md) | Local handle/lifecycle, Tauri paths, versioning, fresh installs. |
| Query/value/response boundaries | [surrealdb-3-query-shapes-and-sdk-binding.md](references/surrealdb-3-query-shapes-and-sdk-binding.md) | `.bind()`, `.check()`, `.take_errors()`, NONE/null, RecordId text, transactions, ORDER BY projection, JSON timestamps, MERGE vs CONTENT. |
| Graph/search/changefeeds | [surrealdb-3-graph-search-and-changefeeds.md](references/surrealdb-3-graph-search-and-changefeeds.md) | Relation schema, BM25/HNSW/RRF, changefeeds, VERSION boundary. |
| Migrations/recovery evidence | [surrealdb-3-migrations-and-recovery-evidence.md](references/surrealdb-3-migrations-and-recovery-evidence.md) | Migration identity/checksums, errors, idempotence vs recovery proof, new SCHEMAFULL fields, SCHEMALESS field checks. |
| Verification ledger | [verification-status.md](references/verification-status.md) | Claim-level evidence and baseline status. |
| Reproducers | [reproducers/README.md](reproducers/README.md) | Executable checks and their verification state. |
| Subprocess sandboxing | [subprocess-sandbox-and-path-containment.md](references/subprocess-sandbox-and-path-containment.md) | Path containment, symlinks, bounded streaming. |
| Multi-agent coordination | [multi-agent-clobbering-and-concurrency.md](references/multi-agent-clobbering-and-concurrency.md) | Worktree clobbering and optimistic concurrency. |
| Structured XML & NF-e | [authoritative-xml-and-fiscal-parsing.md](references/authoritative-xml-and-fiscal-parsing.md) | Deterministic XML/NF-e and exact arithmetic. |
| CI/testing discipline | [rigorous-ci-harness-and-testing-discipline.md](references/rigorous-ci-harness-and-testing-discipline.md) | Real-system gates, restart testing, exact-SHA evidence. |

## 9. Maintenance rule

When a real project exposes a stale-model failure:

```text
model prior
   ↓
identify exact version + engine + connection mode
   ↓
check current official API
   ↓
reduce to the smallest useful reproducer
   ↓
run it against the named baseline
   ↓
fix production code + regression test
   ↓
extract generalized technical correction
   ↓
retain project-specific incident only as evidence
```

If a finding is supported only by a proving repository, keep it `CASE-STUDY EVIDENCE` until a minimal reproducer or official API source justifies promotion.

If a finding is primarily about which architecture a project should choose, leave it in the project's planning/ADR material rather than promoting it into this capability skill.