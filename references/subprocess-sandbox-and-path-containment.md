# Subprocess Sandboxing & Path Containment

## 1. The Threat Model of Subprocess Workers

In Astra, document conversion (PDF, DOCX, XLSX to Markdown) is delegated to an isolated Python worker (`document-worker` utilizing Microsoft's `markitdown`).

Because the documents originate from external Telegram users, the conversion subprocess must be treated as **untrusted**:
- It could be tricked by a malicious PDF or archive into writing files outside its designated directory.
- It could return relative or absolute paths pointing to host system files (e.g. `/etc/passwd` or `C:\Windows\System32\drivers\etc\hosts`).
- It could return symlinks pointing to sensitive host credentials.
- It could generate multi-gigabyte decompression bombs or unconstrained text output leading to out-of-memory (OOM) crashes.

---

## 2. Sandbox Path Containment Architecture

### Step 1: Ephemeral Sandbox Creation
Every document conversion task creates a dedicated, isolated temporary directory using Rust's `tempfile::TempDir`:
```rust
let temp_dir = tempfile::Builder::new()
    .prefix("astra_doc_")
    .tempdir_in(&configured_temp_base)?;
```

### Step 2: Canonicalization of the Sandbox Root
Before passing the path to the worker, canonicalize the sandbox directory to resolve all symlinks, junctions, or relative path components:
```rust
let canonical_sandbox = fs::canonicalize(temp_dir.path())
    .map_err(|e| DocumentIntakeError::Internal(format!("failed to canonicalize sandbox: {e}")))?;
```

### Step 3: Validating Worker Output Paths
When the worker responds with an `output_path`, never open it directly. Apply strict verification:
```rust
pub fn validate_and_read_worker_output(
    canonical_sandbox: &Path,
    output_path_str: &str,
) -> Result<String, DocumentIntakeError> {
    let out_path = Path::new(output_path_str);
    
    // Reject empty, relative, or suspicious paths
    if output_path_str.trim().is_empty() {
        return Err(DocumentIntakeError::SecurityViolation("empty output path".into()));
    }

    // Resolve canonical target path
    let canonical_out = fs::canonicalize(out_path).map_err(|_| {
        DocumentIntakeError::SecurityViolation("worker output path cannot be resolved".into())
    })?;

    // 1. Enforce strict directory containment
    if !canonical_out.starts_with(canonical_sandbox) {
        return Err(DocumentIntakeError::SecurityViolation(
            "worker output path escaped sandbox".into(),
        ));
    }

    // 2. Reject symlinks using symlink_metadata (never follow symlinks)
    let meta = fs::symlink_metadata(&canonical_out).map_err(|_| {
        DocumentIntakeError::SecurityViolation("failed to read output metadata".into())
    })?;

    if meta.file_type().is_symlink() {
        return Err(DocumentIntakeError::SecurityViolation(
            "worker output is a symlink (rejected)".into(),
        ));
    }

    // 3. Enforce regular file (reject directories, FIFOs, devices)
    if !meta.is_file() {
        return Err(DocumentIntakeError::SecurityViolation(
            "worker output is not a regular file".into(),
        ));
    }

    // Read bounded content
    let content = fs::read_to_string(&canonical_out)?;
    Ok(content)
}
```

---

## 3. Bounded Streaming & 20 MB Limit Enforcement

### The OOM Vulnerability
If a malicious user uploads a 500 MB stream or an infinite stream over an open socket, reading to completion via `read_to_end()` will exhaust process heap memory.

### The Bounded Streaming Writer
Astra implements a `BoundedWriter` wrapper around standard `io::Write` / `tokio::io::AsyncWrite`:
```rust
pub struct BoundedWriter<W> {
    inner: W,
    written: u64,
    limit: u64,
}

impl<W: io::Write> io::Write for BoundedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let new_total = self.written + buf.len() as u64;
        if new_total > self.limit {
            return Err(io::Error::new(
                io::ErrorKind::FileTooLarge,
                format!("payload exceeded maximum limit of {} bytes", self.limit),
            ));
        }
        let n = self.inner.write(buf)?;
        self.written += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
```
If incoming bytes exceed 20 MB (`20 * 1024 * 1024`), the writer immediately aborts the stream, drops the connection, and purges the temporary file.

---

## 4. Input Filename Sanitization

Never use untrusted filenames provided by Telegram clients directly on the filesystem.
- Strip path traversal sequences: `/`, `\`, `..`.
- Reject ASCII control characters (`\x00` through `\x1f`, `\x7f`).
- Enforce allowable extensions: reject `.exe`, `.dll`, `.bat`, `.cmd`, `.sh`, `.ps1`, `.vbs`, `.so`, `.dylib`.
- Disambiguate identical filenames: generate an internal synthetic filename combining a short content hash and a pseudo-random nonce:
  ```rust
  let safe_filename = format!("{short_hash}_{nonce}_{sanitized_basename}");
  ```

---

## 5. Nonce Classification: `fastrand` vs CSPRNG

A subtle documentation and architectural trap is confusing collision-avoidance nonces with cryptographic security tokens:
- **`fastrand`**: Fast, small-state pseudo-random number generator (PRNG).
  - **Appropriate use**: Generating temporary directory prefixes, local test IDs, or file disambiguation nonces where the security guarantee comes from OS filesystem permissions (such as `0700` `TempDir`), not randomness.
  - **Inappropriate use**: Session tokens, API keys, password salts, cryptographic nonces.
- **Rule**: Never document an inner `fastrand` nonce as "cryptographically secure." Be honest and explicit about security properties.
