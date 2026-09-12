# Multi-Agent Coordination, Clobbering & Concurrency

## 1. The Multi-Agent Clobbering Incident

### The Event
During development, two AI models (Claude and Antigravity) were asked to work on Astra's codebase around the same time. 

Claude inspected the tree, worked out SurrealQL queries via `curl`, and applied edits to `ingest/repository.rs`. Twelve seconds later, Antigravity wrote files back to disk without verifying the filesystem's modification time (`mtime`) or git working tree status. Antigravity's write completely overwritten Claude's edits without realizing it, reverting the bug fixes and leaving the repository in a broken state.

When reviewing the code later, Claude reported that several critical fixes had vanished, mistakenly believing they were never implemented.

### Root Causes
1. **Blind Overwrites Without `mtime` Verification**:
   Writing an entire file back without checking whether the file on disk has changed since it was read.
2. **Lack of Agent State Synchronization**:
   Multiple agents operating on the same worktree simultaneously without branch isolation or git lock guards.
3. **Optimistic False Assumptions**:
   Assuming that "nothing else is writing to the disk right now."

### Defensive Rules for Multi-Agent Pairing
1. **Always Check Git Status Before & After**:
   Before modifying any file, inspect `git status` or file modification timestamp. If an unexpected diff exists, stop and rebase/re-read immediately.
2. **Use Targeted Line-Range Replacements**:
   Instead of rewriting full files from memory (which wipes out concurrent edits in other sections), use contiguous block replacements (`replace_file_content`) anchored to exact matching target lines.
3. **Branch Isolation**:
   When dispatching subagents or delegating tasks to external assistants, assign each agent its own git branch or worktree (`git worktree add`). Merge only through PRs with automated CI checks.

---

## 2. Concurrency & TOCTOU in Database State

### The Preflight Check Trap
In multi-tenant systems, a common pattern is:
```rust
// ❌ TOCTOU Vulnerability:
let existing = db.select(("company", slug)).await?;
if existing.is_none() {
    // A concurrent thread creates the company right here!
    db.create(("company", slug)).content(company).await?;
}
```
Between the `SELECT` and the `CREATE`, a concurrent request with the same company slug can slip in, causing race conditions, split-brain silo provisioning, or runtime errors.

### The Solution: Deterministic Atomic Creation
Rely on atomic constraints enforced by the storage engine:
```rust
// ✅ CORRECT:
// Ensure UNIQUE index exists on company slug
let created: Option<CompanyTenant> = master_db
    .create(("company", slug.clone()))
    .content(company.clone())
    .await
    .map_err(|error| {
        format!("company '{slug}' could not be created atomically (already exists): {error}")
    })?;

if created.is_none() {
    return Err(format!("company '{slug}' was not created"));
}
```
If two requests race to create `acme`, the database's unique constraint atomically rejects the second request without any TOCTOU window.

---

## 3. Case-Insensitive Filesystems & Content Addressing

A subtle multi-platform concurrency hazard exists when dealing with SHA-256 hashes on Windows/macOS vs. Linux:
- Linux filesystems (ext4) are case-sensitive: `E3B0...` and `e3b0...` are two different directories.
- Windows (NTFS) and macOS (APFS) are case-insensitive by default: `E3B0...` and `e3b0...` resolve to the same folder on disk.

If an intake handler or agent does not strictly normalize hashes to lowercase ASCII hex before constructing blob paths:
```rust
// ❌ HAZARDOUS
let path = base_dir.join(&hash[0..2]).join(&hash[2..4]).join(hash);

// ✅ SAFE & CANONICAL
let lower_hash = hash.to_ascii_lowercase();
let path = base_dir.join(&lower_hash[0..2]).join(&lower_hash[2..4]).join(&lower_hash);
```
Un-normalized hashing leads to path duplication, cache misses, and potential replay bypasses.
