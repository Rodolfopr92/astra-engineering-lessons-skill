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
5. for complex fixtures, document the few fields the test treats as canonical.

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

## 8. Resource-Constrained CI

Limiting parallelism such as `-j 2` can improve reliability on constrained developer machines or runners, but it is not a universal correctness rule.

Choose concurrency based on:

- runner memory;
- CPU count;
- link-time pressure;
- project size;
- action-minute budget.

The principle is to avoid mistaking infrastructure exhaustion for application failure, while still keeping the gate representative enough to catch real concurrency problems.

---

## 9. A Useful Evidence Ladder

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

Not every change needs the top rung. But the strength of the claim should not exceed the strength of the evidence.
