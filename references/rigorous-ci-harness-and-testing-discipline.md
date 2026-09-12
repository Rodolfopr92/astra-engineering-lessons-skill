# Rigorous CI Harness & Testing Discipline

## 1. The 8-Stage Zero-Compromise Gate

Astra enforces an automated 8-stage gate in CI (`scripts/test_full.sh`) where a failure in any single stage blocks merge:

```text
1. Shell syntax checks (bash -n)
2. Python syntax checks (py_compile)
3. Python unit, security, and conversion tests (unittest)
4. Rust formatting check (cargo fmt -- --check)
5. Rust deterministic unit & integration tests (cargo test --locked)
6. SurrealDB 3.2.4 destructive isolation & SurrealKV restart persistence tests
7. Clippy linter with warnings as errors (cargo clippy --all-targets -- -D warnings)
8. RustSec dependency vulnerability audit (cargo audit)
```

No pull request is allowed to bypass this gate.

---

## 2. Testing the Persistence Reality: The Kill/Restart Loop

### The Danger of Mocking
In many projects, integration tests use in-memory databases or mock repositories. In Astra, that approach would have completely hidden:
- The SurrealDB `RecordId` vs `id: String` deserialization failure.
- Database index initialization bugs.
- WebSocket session teardown and reconnection leaks.
- WAL write serialization failures on disk.

### The CI Restart Pattern
Stage 6 of the gate performs real destructive testing:
1. Provisions a random ephemeral port and starts `surreal start --log error surrealkv://$TEST_DB_DIR`.
2. Runs `persistence_seed_before_restart`, persisting records and verifying initial state.
3. Kills the SurrealDB process via OS SIGTERM (`kill "$SURREAL_PID"`).
4. Restarts a new SurrealDB instance on the exact same storage directory.
5. Runs `persistence_verify_after_restart`, reconnecting via WebSocket and asserting that all records, relationships, and counters survived intact.

---

## 3. Grounding Assertions in Fixture Realities

### The Mistake
During Phase 5 testing, a test assertion failed with:
```text
assertion `left == right` failed
  left: 800.00
 right: 1850.00
```
The test author had copied assertion code from `valid_nfe_cnpj_cnpj.xml` (which had a total of 1850.00) into the test for `valid_nfe_proc.xml` (which had a total of 800.00).

### The Lesson
- Never copy-paste test assertions blindly.
- Ground every assertion in the actual, inspected content of the fixture file.
- When creating fixtures, document the exact expected totals and key properties in comments or commit notes.

---

## 4. Compiler & Linter Discipline

1. **`-D warnings` in Clippy**:
   Treating warnings as errors prevents code smells, redundant operations (e.g. `unnecessary_unwrap`), and subtle bugs from accumulating.
2. **Deterministic Locking**:
   Always pass `--locked` to all `cargo test`, `cargo clippy`, and `cargo audit` commands in CI to ensure reproducibility and prevent silent dependency upgrades.
3. **Machine Stability Under Load**:
   On resource-constrained developer workstations or shared runners, unbound parallel jobs can cause memory thrashing or CPU starvation. Always pass `-j 2` to control parallelism and maintain system stability.
