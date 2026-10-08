# Rigorous CI Harness & Testing Discipline

This reference extracts reusable testing lessons from an 8-stage Rust/Python/SurrealDB integration gate. The exact stage list is a **proving-project convention**; the transferable capability is knowing which claims require real-system evidence.

---

## 1. Example High-Assurance Gate

Astra's current gate runs:

```text
1. Shell syntax checks (bash -n)
2. Python syntax checks (py_compile)
3. Python unit/security/conversion tests
4. Rust formatting (cargo fmt -- --check)
5. Rust deterministic unit/integration tests (cargo test --locked)
6. Live SurrealDB 3.2.4 tenant + SurrealKV restart tests
7. Clippy with warnings denied
8. RustSec dependency audit
```

This is **PROJECT CONVENTION**. Another repository may need different stages.

The general rule is:

> Every important production claim should have a gate that exercises the boundary responsible for that claim.

Examples:

```text
claim: persists after process restart
→ kill/restart real database

claim: tenant isolation
→ write/read through separate real tenant sessions

claim: worker cannot escape temp directory
→ attack filesystem paths/symlinks

claim: exact fiscal arithmetic
→ test decimal edge cases
```

---

## 2. Unit Tests and Mocks Are Useful, but Boundary Claims Need Boundary Tests

Mocks are excellent for:

- fast feedback;
- deterministic failure injection;
- pure business logic;
- uncommon error branches.

They are insufficient evidence for things the mock does not implement.

A database mock/in-memory engine cannot, by itself, prove:

- remote protocol serialization;
- real record-ID conversion;
- disk durability;
- WAL/recovery behavior;
- process restart;
- reconnection.

A fake downloader cannot, by itself, prove the real Telegram/network adapter aborts an oversized stream at the threshold.

So use both layers deliberately rather than adopting “never mocks” as a slogan.

---

## 3. Persistence Reality: Kill / Restart / Re-read

A proving restart sequence is:

```text
start real SurrealDB 3.2.4 on surrealkv://<test-dir>
→ seed production record types
→ assert seed state
→ terminate server process
→ verify it stopped
→ restart fresh process on same directory
→ reconnect over deployed protocol
→ re-read typed records
→ assert exact values
```

For an embedded desktop architecture, use the analogous reopen test:

```text
open embedded SurrealKV on fixed directory
→ write production record types
→ drop/close handle
→ create fresh handle on same directory
→ re-read typed records
→ assert exact values
```

This caught a real failure:

```text
Expected string, got record
```

that ordinary tests had not exposed.

### Key lesson

Do not let “tests passed” collapse different claims into one bucket. Ask **which boundary the passing test actually exercised**.

---

## 4. Ground Assertions in Fixture Reality

A Phase 5 restart test once expected a total copied from a different NF-e fixture:

```text
actual:   800.00
expected: 1850.00
```

The implementation was not necessarily wrong. The assertion was detached from its source fixture.

Rules:

1. inspect the exact fixture used by the test;
2. derive expected values from that fixture deliberately;
3. avoid copying assertions between fixtures without re-grounding them;
4. give fixtures stable semantic names;
5. for complex fixtures, document the few fields the test treats as canonical;
6. assert representation and semantic value separately when serialization can turn exact decimals into strings;
7. build schema fixtures from the production migration. An undefined SurrealDB table is schemaless and accepts writes that the SCHEMAFULL production table refuses ([migrations §10](surrealdb-3-migrations-and-recovery-evidence.md)).

A failing test is evidence of inconsistency, not automatic proof that production code is the faulty side.

---

## 5. Exact-SHA CI Evidence

When an agent says “CI is green,” record:

```text
repository
branch / PR
exact head SHA
workflow/run ID
conclusion
```

A green run on an older head does not validate a newer commit.

Likewise, a local green table does not substitute for a remote CI claim if the merge policy relies on the remote environment.

This exact-SHA discipline prevented earlier handoff prose from being mistaken for repository state.

---

## 6. Compiler and Linter Discipline

Useful Rust defaults for a locked application repository include:

```bash
cargo fmt -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
```

`--locked` is relevant to Cargo commands that resolve dependencies from `Cargo.lock`.

`cargo audit` already audits the lockfile dependency graph; use the options supported by the installed `cargo-audit` version rather than mechanically adding unrelated Cargo flags from memory.

The reusable lesson is **dependency determinism**, not a ritual command string.

An exact pin on a top-level crate does not pin what that crate depends on.

- The SurrealDB 3.2.4 SDK reaches its engine crates through caret requirements, and `surrealdb-core` reaches `surrealdb-collections` and `surrealdb-strand` the same way. Pinning one level is not enough: read the whole family in the lock.
- So, without a lockfile, a resolution made after 2026-09-24 builds SDK 3.2.4 on 3.3.x engine crates.
- A test or reproducer labelled with a version must pin the crates that implement the behavior, commit `Cargo.lock`, and run with `--locked`.

---

## 7. Dependency Security Claims Need Reachability Humility

A vulnerability appearing in `Cargo.lock` tells you the dependency graph contains an affected package/version. It does not automatically prove your application reaches the vulnerable behavior.

Conversely, dismissing an advisory because “we probably don't use that code path” is also insufficient.

When an advisory is temporarily ignored:

1. record the advisory ID;
2. inspect reverse dependency paths (`cargo tree -i ...` where useful);
3. understand why remediation is blocked;
4. document the actual exposure assumption;
5. remove the ignore when upstream resolution becomes available.

Do not turn an ignore list into permanent wallpaper.

---

## 8. Resource-Constrained Builds: Use a Third State

A native build terminated by infrastructure limits before application diagnostics is **not** a successful build and **not necessarily** an application compile failure.

Report three states separately:

```text
PASS
→ compiler/linker completed successfully

FAIL
→ compiler/linker reached application/dependency diagnostics and reported a code/build error

BLOCKED / INDETERMINATE
→ OOM, sandbox termination, quota/time limit, runner death, or other infrastructure failure stopped verification before meaningful application diagnostics
```

For example, if compiling a large dependency such as `surrealdb-core` is terminated by the sandbox before the target crate is checked, the defensible claim is:

```text
native verification blocked by environment/resource limit before application diagnostics
```

not:

```text
application compiles
```

and not:

```text
application is broken
```

This distinction was important in the Brew & Batch Tauri/embedded-SurrealKV case study.

**CASE-STUDY EVIDENCE.**

---

## 9. Resource-Constrained CI

Limiting parallelism such as `-j 2` can improve reliability on constrained developer machines or runners, but it is not a universal correctness rule.

Choose concurrency based on:

- runner memory;
- CPU count;
- link-time pressure;
- project size;
- action-minute budget.

The principle is to avoid mistaking infrastructure exhaustion for application failure, while still keeping the gate representative enough to catch real concurrency problems.

---

## 10. A Useful Evidence Ladder

From weakest to strongest for a runtime claim:

```text
model says it should work
< static code inspection
< compiles
< unit test
< integration test with fake boundary
< integration test with real dependency
< destructive restart/recovery test
< production observation with monitoring
```

`BLOCKED / INDETERMINATE` is not a rung on the ladder. It means the attempted rung was not reached.

Not every change needs the top rung. But the strength of the claim should not exceed the strength of the evidence.

---

## 11. Cargo Test Substring Filter Trap & Silent Test Omission

When running filtered test suites (such as live database integration tests gated with `#[ignore]`):

```bash
cargo test --locked live_tests -- --ignored
```

Cargo filters tests by **substring matching** against the test name, not by attribute or directory. If a developer or AI adds a new test named:

```rust
#[tokio::test]
#[ignore]
async fn live_contract_gateway_integration() { ... }
```

The filter `live_tests` will **not** match `live_contract_gateway_integration`. Cargo outputs:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Cargo exits with status `0` (success). The CI job or local developer observes a green test run, completely unaware that **zero tests were executed**.

### Rules for Filtered Suites

1. **Enforce naming conventions**: If a filter is `live_tests`, all corresponding tests must begin with `live_tests_` (e.g. `live_tests_contract_gateway_integration`).
2. **Explicit targets**: Prefer targeting tests by test binary or file (`cargo test --test live_tests -- --ignored`) rather than substring matching where feasible.
3. **Assert non-zero test execution**: Test runner scripts and CI steps should parse output or assert that passed test count is greater than zero (`grep -q "[1-9][0-9]* passed"`).

**CASE-STUDY EVIDENCE.**

---

## 12. Brittle Regex Verification Tools vs Formatter (`rustfmt`) Code Reflow

Static check scripts or repo linter harnesses often verify idioms using regular expressions over source code.

For example, checking a constructor pattern with a single-line regex:

```python
re.search(r"Some\(\((.+?), (.+?)\)\)", content)
```

Running `cargo fmt` will reflow multi-argument tuples, long argument lists, or trailing commas across multiple lines:

```rust
Some((
    contract_slug,
    verification_hash,
))
```

The single-line regex immediately fails, even though the Rust source is syntactically standard and functionally identical.

### Hardened Patterns

- Do not use whitespace-sensitive or single-line regular expressions to validate Rust syntax.
- Use multiline-aware, whitespace-tolerant regex patterns:
  ```python
  re.search(r"Some\s*\(\s*\(\s*(.+?)\s*,\s*(.+?)\s*,?\s*\)\s*\)", content, re.DOTALL)
  ```
- Where AST-level semantic verification is needed, use Rust compiler lints or AST parsers (`syn`) rather than textual regex matching.

**CASE-STUDY EVIDENCE.**

---

## 13. Audited Provenance Structs & Serde Serialization Default Trap

When data structures participate in cryptographic provenance chains, audit logs, or content hashing (e.g. SHA-256 over serialized canonical records):

```rust
#[derive(Serialize, Deserialize)]
pub struct ContractEvent {
    pub contract_id: String,
    pub event_type: String,
    #[serde(default)]
    pub notes: String,
}
```

Adding `#[serde(default)]` makes the field optional during **deserialization** (reading older JSON records that omit `"notes"`).

However, `#[serde(default)]` has **no effect on serialization**. When serializing a struct where `notes` is empty (`""`):

```json
{"contract_id":"123","event_type":"verified","notes":""}
```

The emitted JSON includes `"notes": ""`, whereas historical records serialized without the field entirely:

```json
{"contract_id":"123","event_type":"verified"}
```

This difference alters the SHA-256 digest, breaking historical verification, hash chains, and replay checks.

### Prevention

1. **Pair `#[serde(default)]` with `skip_serializing_if`**:
   ```rust
   #[serde(default, skip_serializing_if = "String::is_empty")]
   pub notes: String,
   ```
   Or for `Option<T>`:
   ```rust
   #[serde(default, skip_serializing_if = "Option::is_none")]
   pub notes: Option<String>,
   ```
2. **Provenance canonicalization**: Before computing cryptographic digests, run serialization through an explicit canonicalization routine that strips default/absent fields.

**CASE-STUDY EVIDENCE.**
