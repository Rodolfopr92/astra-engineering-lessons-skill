---
name: astra-architecture-and-lessons
description: Authoritative guide and engineering lessons learned developing Astra Analytics Bot, covering SurrealDB 3.2.4 typed contracts, subprocess sandboxing, multi-agent coordination, deterministic fiscal XML parsing, and zero-compromise CI gates.
---

# Astra Architecture & Hard-Earned Engineering Lessons

This skill distills the engineering principles, architectural patterns, failure modes, and debugging lessons discovered throughout the development of the **Astra Analytics Bot** across Phases 1 through 5.

Any agent or engineer working on Astra or similar high-assurance systems (Rust + SurrealDB 3.2.4 + Python Subprocesses + Multi-Tenant Telemetry + Brazilian Fiscal Invoicing) must consult this document and its companion references.

---

## 1. Core Engineering Tenets

1. **Test the Real System, Never Mocks**:
   - Astra deploys SurrealDB 3.2.4 over WebSocket with persistent disk storage (`surrealkv://`).
   - Mock engines (`Mem`) hide serialization bugs, record ID mismatches, and disk reload failures. Always run restart-persistence tests that kill the server process, restart it against the exact same data directory, and re-read typed records.
2. **Explicit Logical IDs Over Intrinsic Database Identifiers**:
   - Never model SurrealDB's intrinsic `id` field as a primitive `id: String`. SurrealDB returns a structured `RecordId` (`Thing`).
   - Model your own domain primary keys as explicit strings: `artifact_id`, `checkpoint_id`, `nfe_id`, `dead_letter_id`.
3. **No Float Arithmetic for Money or Tax**:
   - Floating-point arithmetic (`f32`/`f64`) produces binary representation errors (e.g. `0.1 + 0.2 = 0.30000000000000004`).
   - All monetary values, taxes, and rates must use fixed-point arithmetic (`rust_decimal::Decimal`).
4. **Boundary Validation at the Threshold**:
   - Validate and sanitize external input (Telegram attachments, XML strings, file paths) at intake before any persistent state or heavy parsing occurs.
   - Enforce hard limits: 20 MB payload limit, max 32 levels of XML recursion, rejection of DTDs, rejection of control characters.
5. **Fail-Closed Sandbox Security**:
   - External subprocesses (like Python workers) are untrusted. Never trust the paths they return.
   - Canonicalize both sandbox root and output files (`fs::canonicalize`). Reject symlinks, non-regular files, and any path escaping the temporary directory.

---

## 2. Topic Index & Deep References

Refer to the dedicated reference documents in `references/` for full technical breakdowns:

| Topic | Reference File | Key Takeaway |
|---|---|---|
| **SurrealDB 3.2.4 Contract** | [surrealdb-3-contract-and-pitfalls.md](references/surrealdb-3-contract-and-pitfalls.md) | `Expected string, got record` root cause, `SurrealValue` derive, atomic checkpoint counters, silo isolation. |
| **Subprocess Sandboxing** | [subprocess-sandbox-and-path-containment.md](references/subprocess-sandbox-and-path-containment.md) | Sandbox directory containment, symlink rejection, `BoundedWriter` 20 MB streaming, nonce accuracy (`fastrand` vs CSPRNG). |
| **Multi-Agent Coordination** | [multi-agent-clobbering-and-concurrency.md](references/multi-agent-clobbering-and-concurrency.md) | Cross-agent file overwrites, mtime hazards, TOCTOU prevention in database creation, git worktree hygiene. |
| **Authoritative XML & NF-e** | [authoritative-xml-and-fiscal-parsing.md](references/authoritative-xml-and-fiscal-parsing.md) | `roxmltree` DTD immunity, 44-digit Modulo 11 check digit, R$ 0.02 tolerance policy, tenant direction derivation, `NFE_DUPLICATE_CONFLICT`. |
| **CI & Testing Rigor** | [rigorous-ci-harness-and-testing-discipline.md](references/rigorous-ci-harness-and-testing-discipline.md) | 8-stage automated gate, SurrealKV kill/restart harness, fixture accuracy, `-D warnings` and dependency auditing. |

---

## 3. Quick Reference: Common Pitfalls & How to Avoid Them

### Pitfall 1: Deserializing SurrealDB's `id` as `String`
```rust
// ❌ WRONG: Fails at runtime with "Expected string, got record"
#[derive(SurrealValue)]
pub struct MyRecord {
    pub id: String,
    pub title: String,
}

// ✅ CORRECT: Use explicit domain ID, let SurrealDB manage intrinsic id
#[derive(SurrealValue)]
pub struct MyRecord {
    pub record_id: String,
    pub title: String,
}
```

### Pitfall 2: Case-Insensitive Content Addressing
```rust
// ❌ WRONG: Preserves mixed case or uppercase hex strings, breaking on Windows/macOS
pub fn to_blob_path(tenant: &str, hash: &str) -> PathBuf {
    PathBuf::from(tenant).join(&hash[0..2]).join(&hash[2..4]).join(hash)
}

// ✅ CORRECT: Enforce lowercase ASCII hexadecimal normalization
pub fn to_blob_path(tenant: &str, hash: &str) -> PathBuf {
    let lower_hash = hash.to_ascii_lowercase();
    PathBuf::from(tenant).join(&lower_hash[0..2]).join(&lower_hash[2..4]).join(&lower_hash)
}
```

### Pitfall 3: Subprocess Path Escape via Symlinks or Relative Paths
```rust
// ❌ WRONG: Naive containment check vulnerable to symlink bypass
if output_path.starts_with(temp_dir) { ... }

// ✅ CORRECT: Canonicalize both paths to resolve all symlinks and verify regular file
let canonical_sandbox = fs::canonicalize(temp_dir)?;
let canonical_output = fs::canonicalize(output_path)?;
let metadata = fs::symlink_metadata(&canonical_output)?;

if metadata.file_type().is_symlink() || !metadata.is_file() {
    return Err("symlink or non-regular file rejected");
}
if !canonical_output.starts_with(&canonical_sandbox) {
    return Err("path traversal out of sandbox detected");
}
```

### Pitfall 4: XML XXE / Billion Laughs Attacks
```rust
// ❌ WRONG: General XML parser with entity expansion enabled
// Vulnerable to memory exhaustion, Billion Laughs, or local file retrieval

// ✅ CORRECT: roxmltree rejects DTDs by default (Error::DtdDetected)
// Supplement with recursion depth check and control character validator
let doc = parse_safe_xml(xml_bytes)?;
```

### Pitfall 5: Assuming Test Fixture Values
```rust
// ❌ WRONG: Asserting expected numbers from memory or another test file
assert_eq!(totals.v_nf, Decimal::new(185000, 2)); // Fails if fixture has 800.00!

// ✅ CORRECT: Ground assertions in the exact content of the fixture file
assert_eq!(totals.v_nf, Decimal::new(80000, 2));
```
