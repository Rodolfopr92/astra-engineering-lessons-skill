# Subprocess Sandboxing & Path Containment

This reference captures reusable filesystem/process-boundary lessons proven while hardening a Rust → Python document worker.

The worker process is a **separate execution boundary**, but do not call it an OS sandbox unless the operating system actually enforces isolation (namespaces, seccomp, containers, ACLs, etc.). Environment clearing and path validation are valuable hardening, not magic process confinement.

---

## 1. Threat Model

When external documents are converted by a subprocess, treat all worker-controlled outputs as untrusted.

Possible failures include:

- returning an absolute host path such as `/etc/passwd`;
- returning a relative path that escapes the intended directory;
- returning a symlink to sensitive host data;
- writing through a symlinked ancestor directory;
- returning a directory, FIFO, socket, or device instead of a regular file;
- producing unbounded output;
- inheriting credentials or unrelated environment secrets.

The parent process remains responsible for enforcing the contract.

---

## 2. Dedicated Temporary Directory

A strong baseline is one OS-created temporary directory per task:

```rust
let temp_dir = tempfile::Builder::new()
    .prefix("astra_doc_")
    .tempdir_in(&configured_temp_base)?;

let canonical_sandbox = std::fs::canonicalize(temp_dir.path())?;
```

`TempDir` provides collision-resistant creation and RAII cleanup behavior. Cleanup is still best-effort on drop; code that requires guaranteed cleanup should handle close/removal errors explicitly.

---

## 3. Validate the Raw Path Before Canonicalization

This order matters.

### Incorrect pattern

```rust
let canonical_output = fs::canonicalize(output_path)?;
let metadata = fs::symlink_metadata(&canonical_output)?;
```

`canonicalize()` has already followed the original symlink. Calling `symlink_metadata()` only on the canonical target therefore cannot tell you whether the **worker-returned path itself** was a symlink.

### Proven pattern

```rust
let raw_output = Path::new(output_path_str);

let raw_meta = fs::symlink_metadata(raw_output)?;
if raw_meta.file_type().is_symlink() {
    return Err(SecurityError::SymlinkRejected);
}
if !raw_meta.is_file() {
    return Err(SecurityError::NonRegularFile);
}

let canonical_output = fs::canonicalize(raw_output)?;
if !canonical_output.starts_with(&canonical_sandbox) {
    return Err(SecurityError::PathEscape);
}
```

The parent should also reject empty/malformed output-path values before filesystem access.

### Why both checks exist

- **Raw `symlink_metadata`**: catches a direct symlink returned by the worker.
- **Canonical containment**: resolves path components and catches paths that ultimately land outside the sandbox.
- **Regular-file check**: rejects directories/devices/FIFOs/etc.

For writable persistent stores, ancestor-component symlink checks may also be necessary before create/delete operations, because the target may not exist yet and therefore cannot simply be canonicalized.

---

## 4. Persistent Blob Paths: Verify Ancestors Too

For a content-addressed layout such as:

```text
<root>/<tenant>/<hash-prefix>/<sha256>/original
```

validate the tenant/hash syntax, derive the path internally, and walk existing components from the trusted root using `symlink_metadata`.

If any existing component is a symlink, fail closed.

This matters for:

- `put`
- `exists`
- `open_read`
- `delete`

A proving project originally hardened read/write paths but omitted the same check from delete; a regression test later closed that gap.

---

## 5. Bounded Streaming: Abort During the Transfer

A post-download size check is too late if the process already buffered an oversized response.

The useful abstraction is an `AsyncWrite` wrapper that refuses the write that would cross the limit:

```rust
pub struct BoundedWriter<W> {
    inner: W,
    written: u64,
    max_bytes: u64,
}
```

Conceptually:

```rust
fn would_exceed(written: u64, incoming: usize, limit: u64) -> bool {
    written.saturating_add(incoming as u64) > limit
}
```

If a chunk would exceed the maximum:

1. return an I/O error immediately;
2. abort the download path;
3. remove the partial temporary file;
4. never pass the partial file to the parser.

In the proving implementation the Telegram document limit is **20 MiB**, but the reusable lesson is to make the limit explicit and enforce it **mid-stream**, not after buffering.

---

## 6. Filenames Are Display Metadata, Not Filesystem Authority

Do not create files using a client-provided filename directly.

At minimum reject or neutralize:

- `/` and `\\`;
- `..` traversal sequences;
- absolute/drive-prefixed paths;
- ASCII control characters (`NUL`, CR/LF, ESC, DEL, etc.);
- dangerous executable/script extensions where the product does not support them;
- misleading compound extensions when relevant to the threat model.

Use an application-generated internal filename or path. Preserve the original filename only as bounded display/provenance metadata.

---

## 7. Environment Isolation

A child worker should receive only the environment it needs.

Do not automatically inherit:

- Telegram bot tokens;
- root database credentials;
- owner IDs;
- unrelated API keys;
- cloud credentials.

An explicit environment allowlist reduces accidental secret exposure.

Important: `env_clear()` or an allowlist does **not** enforce network isolation. A metadata field named `NetworkPolicy::NoNetwork` is also not enforcement by itself. If actual no-network execution is a security requirement, implement an OS/runtime control that provides it.

---

## 8. Randomness: Collision Avoidance vs. Security

`fastrand` is not a cryptographically secure random number generator.

It can be appropriate for:

- non-secret collision-avoidance suffixes;
- test IDs;
- inner temporary names when an OS-created exclusive directory is the real isolation boundary.

It is inappropriate for:

- authentication/session tokens;
- API keys;
- password salts;
- cryptographic nonces;
- unguessable security capabilities.

Document randomness according to the guarantee it actually provides.

---

## 9. Minimum Regression Matrix

A hardened worker-output boundary should test at least:

```text
valid regular file inside sandbox             → accept
absolute path outside sandbox                 → reject
../ traversal resolving outside               → reject
direct symlink                                → reject
nested/ancestor symlink escape                → reject
directory                                     → reject
missing output                                → reject
oversized download crossing limit mid-stream  → abort + delete partial
duplicate display filenames                   → isolated internal paths
control-character filename                    → reject/sanitize
```

Do not replace these with mocks alone. The filesystem behavior itself is the boundary being claimed.
