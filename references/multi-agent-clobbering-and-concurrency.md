# Multi-Agent Coordination, File Clobbering & Concurrency

This reference turns a real shared-worktree failure into portable rules for coding agents.

The important lesson is not which agents were involved. It is that **filesystem writes are concurrent state mutations**, and agent tooling often treats them as if only one writer exists.

---

## 1. General Failure Mode: Read → Think → Blind Full-File Write

A common agent loop is:

```text
read file at state A
→ reason for several seconds/minutes
→ another actor changes file to state B
→ first agent writes its remembered full file A′
→ state B is silently lost
```

This is an optimistic-concurrency failure.

It can occur between:

- two AI coding agents;
- an AI agent and a human editor;
- an IDE formatter/code action and an agent;
- two automation jobs sharing a checkout.

---

## 2. Case Study: Astra Shared-Worktree Clobbering

During Astra development, one agent prepared changes to `ingest/repository.rs`. Another agent later wrote its own previously-read file state back without first re-reading the current worktree. The later write removed valid changes.

The consequences were especially confusing because:

1. prose/commit planning still described the intended fixes;
2. local reasoning assumed those fixes existed;
3. repository inspection later showed they were absent;
4. the documentation then temporarily drifted ahead of the code.

This produced a second-order lesson:

> Concurrency bugs do not only corrupt code. They can corrupt the project's shared belief about what code exists.

---

## 3. Defensive File-Edit Protocol

Before modifying an existing file:

```text
1. read current file
2. inspect git status/diff
3. capture current blob/file identity when the tool supports it
4. compute targeted change
5. immediately before write, ensure the source state is still current
6. apply the smallest practical replacement
7. inspect diff after write
```

### Prefer optimistic write guards

Good write APIs require the old content hash/blob SHA/version:

```text
update(path, expected_sha, new_content)
```

If the file changed since it was read, the write fails instead of overwriting newer work.

GitHub's Contents API, for example, uses the current blob SHA for updates. This is safer than an unconditional overwrite.

### `mtime` is a signal, not a transaction

Modification time checks can detect some concurrent edits but are not a complete concurrency-control mechanism. Timestamp granularity, clock behavior, and tools that preserve timestamps can defeat simplistic `mtime` assumptions.

Prefer content hashes, git object IDs, or tool-provided version tokens when available.

---

## 4. Use Targeted Changes Where Possible

Replacing a small anchored block reduces the blast radius compared with reconstructing an entire file from a stale in-memory copy.

But targeted edits are not a substitute for version checking. If the target block itself changed, fail and re-read rather than guessing.

---

## 5. Strong Isolation: Branches and Worktrees

For substantial concurrent work:

```bash
git worktree add ../task-a -b agent/task-a
git worktree add ../task-b -b agent/task-b
```

Each worker gets an independent filesystem tree and branch.

Merge through normal version-control mechanisms where conflicts are visible and CI can validate the combined state.

This is stronger than coordinating multiple writers in one worktree through convention alone.

---

## 6. Database TOCTOU Is the Same Shape of Bug

The same read-then-write race appears in persistence code:

```rust
let existing = lookup_by_unique_key(db, key).await?;
if existing.is_none() {
    create_record(db, key).await?;
}
```

Two requests can both observe absence before either writes.

The durable identity rule should live in the storage engine:

```surql
DEFINE INDEX OVERWRITE idx_source_hash
ON TABLE document_artifact COLUMNS source_hash UNIQUE;
```

Then application code treats a constraint conflict as a real concurrent outcome rather than assuming the preflight check guaranteed uniqueness.

General principle:

> If correctness depends on uniqueness or atomicity, enforce it at the layer that serializes the competing writes.

---

## 7. Counters: Avoid Application Read-Modify-Write

Similarly:

```text
read counter = 10
worker A computes 11
worker B computes 11
A writes 11
B writes 11
```

Two events produced one increment.

Prefer a single server-side atomic mutation and test it under actual concurrency.

For the SurrealDB 3.2.4 example, see `surrealdb-3-contract-and-pitfalls.md`.

---

## 8. Canonicalization Across Case-Insensitive Filesystems

Content-addressed storage should have one canonical textual representation for a digest.

For SHA-256 hex strings:

```rust
let hash = input.trim().to_ascii_lowercase();
```

then validate exactly 64 ASCII hexadecimal characters before using it in a path.

Without canonicalization, the same digest can be represented differently (`ABC...` vs `abc...`), leading to inconsistent paths/caches across case-sensitive and case-insensitive filesystems.

The general rule is broader than filesystems:

> Normalize identifiers once at the boundary, then use the canonical form for identity comparisons and storage.

---

## 9. Handoff Reports Are Not Repository State

Agent reports commonly say:

```text
implemented X
fixed Y
all gates green
```

Treat those as claims to verify.

For code state, prefer:

```text
exact repository
branch
head SHA
actual diff/file contents
exact CI run attached to that SHA
```

This rule exists because prose survived one Astra clobbering incident even when the code it described did not.

---

## 10. Minimum Multi-Agent Safety Checklist

Before parallel agents edit one codebase:

```text
[ ] separate branch/worktree where practical
[ ] no blind full-file overwrites
[ ] expected SHA/version guard on remote writes
[ ] git status/diff before and after edits
[ ] re-read on conflict
[ ] exact-SHA CI evidence before merge
[ ] handoff claims verified against repository state
```
