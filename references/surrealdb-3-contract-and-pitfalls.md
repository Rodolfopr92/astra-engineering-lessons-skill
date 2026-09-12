# SurrealDB 3.2.4 Contract & Engineering Pitfalls

## 1. The Record ID Deserialization Trap

### The Symptom
During Phase 4 integration testing against SurrealDB 3.2.4, the persistence restart test failed with:
```text
Failed to deserialize field 'id' on type 'ArtifactProbe':
Expected string, got record
```

### The Root Cause
In SurrealDB, every record has an intrinsic `id` field. While SurrealQL syntax displays records as `table:id` (e.g. `document_artifact:doc_123`), the underlying wire protocol returns `id` as a structured **`RecordId`** (composed of `Table` and `Id`), NOT as a primitive UTF-8 string.

When a Rust struct models `id` as:
```rust
// ❌ WRONG
#[derive(SurrealValue)]
pub struct DocumentArtifactRecord {
    pub id: String,
    pub filename: String,
}
```
The SurrealDB deserializer encounters a `RecordId` variant and aborts because it cannot coerce a structured record pointer into a Rust `String`.

### The Solution: Explicit Logical Domain IDs
Never bind your application's domain primary key to SurrealDB's intrinsic `id` field unless you explicitly type it as `surrealdb::RecordId`. 

Instead, model domain IDs as explicit string attributes and let SurrealDB manage its intrinsic table coordinate:
```rust
// ✅ CORRECT
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
pub struct DocumentArtifactRecord {
    pub artifact_id: String,
    pub filename: String,
    pub mime_type: String,
    pub source_hash: String,
    pub canonical_hash: Option<String>,
    pub status: String,
    pub row_count: u64,
    pub metadata_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

When creating or upserting records, specify the record coordinate explicitly in the query or API call:
```rust
db.upsert(("document_artifact", artifact.artifact_id.as_str()))
    .content(artifact.clone())
    .await?;
```

---

## 2. `SurrealValue` vs. Serde Alone

In SurrealDB 3.x, standard Serde `Serialize` and `Deserialize` are necessary for HTTP/JSON payloads, but **insufficient** for the native Rust SDK driver over WebSocket.

- The native SDK requires `#[derive(SurrealValue)]` with `#[surreal(crate = "surrealdb::types")]`.
- Types stored in the database must implement `surrealdb::types::SurrealValue`.
- Supported types:
  - `String`, `bool`, primitive integers (`u32`, `u64`, `i64`), `Option<T>`, `Vec<T>`.
  - `chrono::DateTime<Utc>` (native datetime).
  - `rust_decimal::Decimal` (implements `SurrealValue` natively without precision loss).
- Never serialize typed database structs into intermediate JSON strings to bypass type errors; this breaks indexes, unique constraints, and SurrealQL query filtering.

---

## 3. Atomic Increments vs. TOCTOU in Checkpoints

### The Race Condition
Under concurrent document ingestion, two workers processing attachments at the same time might read the existing checkpoint, increment the in-memory counter, and write it back:
```rust
// ❌ TOCTOU Race Condition:
let current = get_checkpoint(&db, source_code).await?;
let new_count = current.records_seen + 1;
save_checkpoint(&db, new_count).await?;
```
Under 20 concurrent tasks, this race results in lost counts (e.g. 20 updates producing a count of 5).

### The Solution: Atomic SurrealQL Query Increments
Use atomic field increments directly in SurrealQL:
```rust
// ✅ CORRECT:
let sql = "
    UPSERT type::thing('ingest_checkpoint', $source)
    MERGE {
        source_code: $source,
        records_seen: (records_seen || 0) + $seen,
        records_inserted: (records_inserted || 0) + $inserted,
        records_replayed: (records_replayed || 0) + $replayed,
        records_failed: (records_failed || 0) + $failed,
        updated_at: time::now()
    };
";
```
This guarantees that 20 simultaneous concurrent tasks increment the count to exactly 20 without locks or mutexes.

---

## 4. Multi-Tenant Silo Isolation

Astra enforces a strict silo model:
1. **Control Database**: Stores global companies, user subscriptions, and provisioning status in `astra_control`.
2. **Tenant Database**: Each company receives an isolated namespace (`astra_tenant_<slug>`) and database (`operations`).
3. **Session Clones**:
   - The master database client (`Surreal<Ws>`) is cloned.
   - The cloned session executes `.use_ns(tenant_ns).use_db(tenant_db).await`.
   - Data in one tenant silo is completely invisible to any other tenant session.
   - Live tests must verify that concurrent sessions writing to the same table names do not leak or bleed data across namespace boundaries.

---

## 5. The SurrealKV Restart Persistence Discipline

### Why In-Memory Testing Is a False Friend
Testing exclusively against `surrealdb::engine::local::Mem` or mocked repositories gives a false sense of security. In-memory engines:
- Do not serialize data to disk.
- Do not test write-ahead log (WAL) replay or database recovery.
- Do not surface socket connection teardowns or wire protocol deserialization mismatches.

### The Restart Test Pattern
Astra's test harness enforces a 2-phase restart test:
1. **Seed Phase**:
   - Connects to SurrealDB 3.2.4 running on persistent storage (`surrealkv:///path/to/db`).
   - Writes sentinel probes, document artifacts, representations, checkpoints, dead letters, and NF-e records.
2. **Server Kill**:
   - Kills the SurrealDB process with `kill -0` / SIGTERM.
   - Verifies the process is completely terminated.
3. **Restart & Verify Phase**:
   - Starts a fresh SurrealDB process pointing to the exact same storage directory.
   - Reconnects via WebSocket.
   - Queries and asserts that every typed record, child relationship, and numeric total survived restart intact.
