# SurrealDB 3.x Graph, Search, Changefeeds, and Temporal Boundaries

**Primary baseline:** SurrealDB 3.2.4  
**Case-study baselines:** ARGOS on 3.2.3; Alexandria / Omphalos / Saturno on 3.x embedded SurrealKV  
**Last verified:** 2026-09-12

This reference exists because AI models often remember SurrealDB as either a document database with graph syntax or as a generic vector database. Current 3.x exposes distinct graph, full-text, vector, changefeed, and versioned-storage capabilities. They compose well, but they are not interchangeable.

## Evidence labels

- **VERIFIED API** — current official SurrealDB documentation.
- **CASE-STUDY EVIDENCE** — exercised in one of the proving repositories but not independently reduced by this skill repository.
- **PROJECT CONVENTION** — an architecture choice, not a SurrealDB requirement.

---

## 1. Relation tables are first-class typed schema

Current SurrealQL can constrain a table to relation records and optionally constrain its endpoints:

```surql
DEFINE TABLE works_at
    TYPE RELATION FROM person TO company
    SCHEMAFULL;

DEFINE FIELD role ON TABLE works_at TYPE option<string>;
DEFINE FIELD primary ON TABLE works_at TYPE bool DEFAULT false;
```

`IN ... OUT ...` is also supported as an alternative spelling to `FROM ... TO ...`.

You do not need to redefine the intrinsic `in` and `out` edge endpoints just to make a relation table usable. Define application edge fields that carry domain meaning.

**VERIFIED API.**

Official source:
- https://surrealdb.com/docs/reference/query-language/statements/define/table

### Case-study evidence

Omphalos, Saturno, Alexandria, and ARGOS all use SCHEMAFULL relation tables for different domains: causality, ontology, workflow, CRM, scheduling, and context graphs.

A recurring integrity pattern is a unique edge index when only one logical edge should exist between a pair:

```surql
DEFINE INDEX ux_works_at_pair
    ON TABLE works_at COLUMNS in, out UNIQUE;
```

That uniqueness rule is a **PROJECT CONVENTION**. Some domains legitimately allow several edges between the same endpoints.

---

## 2. Full-text syntax changed in SurrealDB 3.0

A model trained heavily on 2.x material may emit the old index clause:

```surql
-- STALE PRE-3.0 SPELLING
SEARCH ANALYZER my_analyzer BM25
```

Current 3.x syntax is:

```surql
DEFINE ANALYZER app_text
    TOKENIZERS class, blank, camel, punct
    FILTERS lowercase, ascii;

DEFINE INDEX ft_knowledge_search
    ON TABLE knowledge_chunk
    FIELDS search_text
    FULLTEXT ANALYZER app_text BM25(1.2, 0.75);
```

The current search functions include `search::score()`, `search::rrf()`, and `search::linear()`.

**VERIFIED API.**

Official source:
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/search

---

## 3. HNSW vector search and hybrid RRF are current 3.x capabilities

Representative current schema:

```surql
DEFINE INDEX hnsw_knowledge_embedding
    ON TABLE knowledge_chunk
    FIELDS embedding
    HNSW DIMENSION 384 DIST COSINE TYPE F32;
```

Representative nearest-neighbour query:

```surql
SELECT id, vector::distance::knn() AS distance
FROM knowledge_chunk
WHERE embedding <|50,100|> $embedding
ORDER BY distance ASC;
```

A hybrid lexical/vector query can rank each branch independently and fuse the lists with reciprocal rank fusion:

```surql
LET $vector = SELECT id, vector::distance::knn() AS distance
    FROM knowledge_chunk
    WHERE embedding <|50,100|> $embedding
    ORDER BY distance ASC
    LIMIT 20;

LET $lexical = SELECT id, search::score(1) AS score
    FROM knowledge_chunk
    WHERE search_text @1@ $query
    ORDER BY score DESC
    LIMIT 20;

RETURN search::rrf([$vector, $lexical], 20, 60);
```

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search
- https://surrealdb.com/docs/learn/data-models/vector-search/similarity-search
- https://surrealdb.com/docs/reference/query-language/functions/database-functions/vector

### Case-study evidence

ARGOS 3.2.3 defines both BM25 and HNSW indexes and implements an RRF search using two subqueries. This closely matches the current official 3.2.x pattern and is useful implementation evidence, but its exact constants (`K`, `EF`, dimensions, limits, RRF `k`) are workload choices rather than universal defaults.

---

## 4. Changefeeds are replay cursors, not automatic bitemporal history

Changefeeds record database mutations for a configured retention period and are consumed with `SHOW CHANGES`:

```surql
DEFINE TABLE opportunity CHANGEFEED 30d;

SHOW CHANGES FOR TABLE opportunity
    SINCE $last_versionstamp
    LIMIT 100;
```

Current 3.2 supports datetime or versionstamp cursors. `INCLUDE ORIGINAL` can preserve additional pre-change information, including delete pre-images in 3.2.

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/learn/querying/real-time/changefeeds
- https://surrealdb.com/docs/reference/query-language/statements/show

Useful applications include:

- external search/index projections;
- cache or warehouse catch-up;
- audit/event replication;
- rebuilding derived services after downtime.

But a changefeed is not the same feature as time-travel querying.

---

## 5. `VERSION` time-travel requires versioning-enabled storage

Do not teach the model:

> "SurrealKV means every table automatically supports `SELECT ... VERSION ...`."

Current documentation requires a storage engine with versioning enabled. For embedded Rust + SurrealKV, the SDK exposes a versioned constructor flow:

```rust
let db = Surreal::new::<SurrealKv>("path/to/database")
    .versioned()
    .await?;
```

Then a compatible query can read historical state:

```surql
SELECT * FROM opportunity:abc VERSION d"2026-09-01T12:00:00Z";
```

**VERIFIED API.**

Official sources:
- https://surrealdb.com/docs/reference/rust/methods/new
- https://surrealdb.com/docs/reference/query-language/statements/select

### Why this correction exists

An older architecture document in a proving project described bitemporal/versioned reads too broadly. The current capability skill must preserve the tighter rule: **changefeeds, domain valid-time fields, and storage-engine `VERSION` history are separate mechanisms.**

---

## 6. A graph can be authoritative or reconstructible

SurrealDB's graph capability does not determine its authority role.

Two valid architectures found in the proving repositories are:

### Pattern A — SurrealDB is authoritative

```text
commands
   ↓
SurrealDB transaction
   ↓
nodes + relations + events/outbox
```

ARGOS uses this model for its local operational domain.

### Pattern B — SurrealDB is a derived graph projection

```text
authoritative transactional store
   ↓ idempotent outbox
SurrealDB graph projection
   ↓
relationship expansion / context retrieval
```

Alexandria uses this pattern. Omphalos/Saturno also articulate a derived "Platinum" graph tier, though parts of the rebuild implementation are architectural intent rather than completed evidence.

These are **PROJECT ARCHITECTURE PATTERNS**, not competing SurrealDB APIs.

The reusable lesson is:

> Decide whether a graph record is operational truth or a rebuildable projection before writing schema and recovery code.

---

## 7. Do not dual-write one fact to independent authorities

When SurrealDB is a projection, a robust pattern is:

```text
single authoritative transaction
  ├── domain mutation
  └── projection_outbox record
          ↓
    idempotent projector
          ↓
    SurrealDB node/edge
```

Use stable external identity, revision, content hash, and checkpoint fields so projection retries are deterministic.

Alexandria's context schema is a useful case-study shape:

```text
tenant_id
external_id
revision
content_hash
properties FLEXIBLE
```

with a unique `(tenant_id, external_id)` identity and a separate projection checkpoint.

**CASE-STUDY EVIDENCE / GENERAL DISTRIBUTED-SYSTEMS PATTERN.**

---

## 8. Scope limits

This reference does not claim:

- SurrealDB should replace a dedicated search/vector engine in every system;
- HNSW parameters from one repository are correct for another dataset;
- every graph edge should be unique by `(in, out)`;
- `CHANGEFEED` is a permanent audit log beyond its configured retention;
- `VERSION` works unless the selected storage engine has versioning enabled;
- every application should make SurrealDB authoritative;
- every multi-store system should make it derived.

Choose the authority model and capability surface from the application's recovery and workload requirements, then verify the exact 3.x feature against the target engine.