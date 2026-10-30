# Integrating staged code that never compiled

Repair classes met while integrating a block of SurrealQL + Rust SDK + Tauri code that had been
written, statically "refined" and packaged without ever being compiled or executed. The proving
case is DLF Block 04 (2026-10-29): fourteen repairs before the first test passed, then 61 tests,
strict Clippy, rustfmt and the frontend build green, merged through PR #7 on 2026-10-30. Blocks
05–10 of the same package are expected to fail in the same classes.

## Baseline

```text
technology:                       SurrealDB 3.2.4 (embedded SurrealKV in the proving repository;
                                  in-memory engine in the reproducers)
client/SDK version:               surrealdb 3.2.4 (engine crates pinned =3.2.4, Cargo.lock, --locked)
language/toolchain version:       Rust 1.97.0 local (proving repository), 1.96.0 reproducer CI
desktop/runtime framework:        Tauri 2.11.5
verified date:                    2026-10-30
reverify by:                      2027-01-28, or at once on a target-version mismatch
```

If the target uses another SurrealDB engine, SDK or Tauri version, every `TESTED BEHAVIOR` and
`VERIFIED API` claim below is **UNVERIFIED FOR THAT TARGET** until re-checked (SKILL §1).

Reproducers: `reproducers/surrealdb-3.2.4/tests/staged_code_pitfalls.rs` (7 tests). The file's
imports also pin two compile-time facts (§5).

---

## 1. SurrealQL parse errors

Parse errors fail the whole request before any statement runs
(`order_by_needs_its_field_selected_and_the_parse_error_fails_the_whole_request`). Two shapes
the staged code produced:

### 1.1 `IF … ELSE IF … ELSE … END` closes with one `END`

```surql
-- staged (parse error: Unexpected token `END`, expected Eof)
blocked_reason = IF a THEN 'x' ELSE IF b THEN 'y' ELSE 'z' END END

-- correct
blocked_reason = IF a THEN 'x' ELSE IF b THEN 'y' ELSE 'z' END
```

Every transition in the staged kernel would have failed; nothing in the package's static
"preflight" caught it.

**TESTED BEHAVIOR.** Reproducer: `an_if_else_if_chain_closes_with_one_end`.

### 1.2 `ORDER BY` takes projected idioms, not expressions

```surql
-- staged (parse error)
SELECT * FROM bucket ORDER BY IF status = 'AVAILABLE' THEN 0 ELSE 1 END, qty DESC;

-- correct: project the rank under an alias, order by the alias
SELECT record::id(id) AS key,
       IF status = 'AVAILABLE' THEN 0 ELSE 1 END AS availability_rank,
       qty
FROM bucket ORDER BY availability_rank, qty DESC;
```

**TESTED BEHAVIOR.** Reproducer: `order_by_takes_projected_idioms_not_expressions`.

---

## 2. Exactly four parameter names are protected

`PROTECTED_PARAM_NAMES` in `surrealdb-core-3.2.4/src/cnf/mod.rs` is `access`, `auth`, `token`,
`session`. Setting one with `LET` or binding it from the SDK fails:

```text
'session' is a protected variable and cannot be set
```

Block 04's count-session SQL bound `$session`; the repair renamed it to `$count_session` on both
sides. Names the engine defines itself in some contexts (`$value`, `$this`, `$before`, `$after`,
`$parent`, `$event`, `$input`) are **not** protected: `LET $value = 3; RETURN $value;` is accepted,
so a collision there shadows silently instead of failing. Avoid them by convention; the engine
will not tell you.

**TESTED BEHAVIOR** for the four names (via `LET` and via `.bind()`) and for `$value` / `$this`;
**VERIFIED API** for the list (crate source). Reproducer:
`the_four_protected_parameter_names_are_refused_by_let_and_by_bind`.

---

## 3. A required record link refuses `NONE`

```surql
DEFINE FIELD transaction_id ON TABLE handling_unit_movement TYPE record<handling_unit_transaction>;
```

is required. The staged code wrote `transaction_id = NONE` for HU-backed moves; a SCHEMAFULL
table refuses it, and so does a `CREATE` that omits the field:

```text
Couldn't coerce value for field `transaction_id` of `movement:one`: Expected `record<tx>` but found `NONE`
```

Either create the linked record first (Block 04 creates the execution transaction and links the
movement to it) or declare `option<record<…>>` in a migration when absence is legitimate. Check
every `CREATE` the staged code issues against the production DDL (SKILL rule 13).

**TESTED BEHAVIOR.** Reproducer: `a_schemafull_record_link_field_refuses_none_unless_declared_option`.

---

## 4. UNIQUE indexes already in force, and absent values

### 4.1 Pick the written field against existing indexes

Migration 0007 had `UNIQUE (warehouse_id, reference)` on `warehouse_task`; the staged
`create_task` wrote `reference = $source_id`, so two tasks from one source would have collided.
Before choosing a field to write, read every `DEFINE INDEX` in the migration history for that
table.

**CASE-STUDY EVIDENCE** (Block 04 patch B04-P001).

### 4.2 Absent values do not collide

The Block 04 register assumed that defining a UNIQUE index over legacy rows that lack the field
would fail. It does not in 3.2.4: the definition is accepted, and rows that keep lacking the field
are accepted afterwards. The real hazard of a migration that adds such an index is **duplicates
among the values the migration writes**. Normalize the rows first and define the index after:
a duplicate then fails at the index definition, with the data still consistent, instead of
failing mid-normalization.

```text
Database index `task2_code_unique` already contains ['w1', 'a'], with record `task2:a`
```

**TESTED BEHAVIOR.** Reproducer:
`a_unique_index_ignores_absent_values_so_normalize_rows_before_defining_it`.
**FALSE / NOT A GENERAL RULE:** "a UNIQUE index cannot be defined while rows lack the field".

---

## 5. Rust SDK 3.2.4 shapes the staged code got wrong

| Staged assumption | 3.2.4 fact | Evidence |
|---|---|---|
| `surrealdb::Response` | The response type is `surrealdb::IndexedResults` (`src/lib.rs` re-export); the 2.x name does not exist | VERIFIED API (crate source); the reproducer file imports it |
| `q.check()?; q.take(0)` | `check(self) -> Result<Self>` consumes the response (E0382); rebind: `let mut q = q.check()?;` | VERIFIED API (`src/method/query.rs:496`) |
| `take_errors().into_iter().next()` is the cause | `take_errors(&mut self) -> HashMap<usize, Error>`: unordered. Inside a failed transaction every statement carries an error; see below | VERIFIED API + TESTED BEHAVIOR |
| `record_id.to_string()` | `RecordId { table, key }` implements `ToSql` and `Debug`, not `Display` (E0599). `to_sql()` gives `item:sku1`, `key.to_sql()` gives `sku1`, `record::id(id)` in SQL gives the key | VERIFIED API (no impl in `surrealdb-types-3.2.4`) + TESTED BEHAVIOR |

### Cascade texts in a failed transaction

For `BEGIN; CREATE …; THROW 'BUSINESS_RULE_X'; CREATE …; COMMIT;` the map holds:

```text
1 -> The query was not executed due to a failed transaction
2 -> An error occurred: BUSINESS_RULE_X
3 -> The query was not executed due to a cancelled transaction
4 -> Cannot COMMIT: the transaction was aborted due to a prior error
```

`BEGIN` is statement 0 and carries no error. The lowest index is a cascade text, so neither
"first entry" nor "lowest index" finds the cause: report the lowest index whose message is not
one of the three cascade texts. Business error codes only reach callers and tests once this is
done.

**TESTED BEHAVIOR.** Reproducers: `take_errors_is_unordered_and_the_cause_hides_among_cascade_errors`,
`record_id_text_comes_from_record_id_in_sql_or_to_sql_in_rust`.

---

## 6. Migrations that rewrite data

- Run them through the transactional path together with the APPLIED receipt. Through a
  non-transactional path the row normalization persisted while a later statement failed,
  leaving half-applied rows (observed in Block 04's first populated-upgrade test run).
- Put the normalization `UPDATE` before any new UNIQUE index (§4.2).
- Add a populated-upgrade test that injects a failure after every real statement and asserts the
  rows and the migration table are untouched, then reopens and performs the genuine upgrade.
- A test that asserts a literal registry size ("15 migrations after reopen") breaks when the
  next migration lands; compare with `MIGRATIONS.len()`.

**CASE-STUDY EVIDENCE** (Block 04 patches B04-P006, B04-P011, B04-P014). The transactional
requirement is consistent with [migrations and recovery evidence](surrealdb-3-migrations-and-recovery-evidence.md).

---

## 7. A guard that exists in two languages must be one expression

Block 01's Rust guard treated a warehouse as active when `lifecycle_status ?? status`, compared
case-insensitively, was `ACTIVE`; seeded warehouses carried no `lifecycle_status`. The staged SQL
required `lifecycle_status = 'ACTIVE'`, so every `create_task` threw `WAREHOUSE_NOT_ACTIVE` inside
its transaction. The repair mirrors the Rust rule exactly:

```surql
string::uppercase(lifecycle_status ?? status ?? 'DRAFT') = 'ACTIVE'
```

When porting a guard, port its null-coalescing and case rules, and test it against the seed data,
which is where the older rows live.

**CASE-STUDY EVIDENCE** (Block 04 patch B04-P012).

---

## 8. `-D warnings` gates

- Clippy's `too_many_arguments` fires above seven parameters (its default `too-many-arguments-threshold`
  is 7). Group them into a request struct (`ReceiptDescription`, `TransferRequest`); do not add
  `#[allow]`. **VERIFIED API** (Clippy lint documentation) + CASE-STUDY (B04-P013).
- Futures passed to `tokio::join!` must borrow `let`-bound values, not temporaries (E0716).
  **CASE-STUDY EVIDENCE.**
- Unused `mut`, unused imports and unused reads are build failures, not warnings.
- `cargo fmt --check` and `git diff --check` run before every push.

---

## 9. Package "cumulative corrections" may already exist

The package's sections D (IMS → WMS demand projection) and E (wave release creates PICK tasks)
were already provided by the previously integrated block; applying E would have created a second
PICK task per allocation. Read the integrated tree before applying any patch section; a package
patch is an advisory mapping that may target an obsolete baseline.

**CASE-STUDY EVIDENCE** (Block 04 register, sections D and E).

---

## 10. Working method that held

1. Prepare the block from the archive with its own hash verification, outside the checkout.
2. Integrate by hand into current source, one block at a time; never copy folders over the tree.
3. First gate is `cargo test` on a focused filter (the block's own tests); every repair is logged
   as diagnosis / supplied / patched / reason / effect / tests.
4. Then the full suite, strict Clippy, rustfmt, frontend build; then a PR with the required checks.
5. Record the merge in the living status files through a small follow-up PR, and write every new
   repair class back into this skill.

## Proving case study

```text
project:                  DLF Merchanting Operations System (SurrealDB 3.2.4 embedded, Tauri 2.11.5)
exact dependency version: surrealdb 3.2.4 with engine crates 3.2.4 (lockfile matches this skill)
symptom:                  fourteen repairs before the first Block 04 test passed
root cause:               nine blocks written and statically refined without compiling or executing
fix:                      docs/block04/DELIVERY/PATCH_REGISTER.md on main after PR #7 (41fd8db2)
regression test:          61 Rust tests (8 for Block 04, including a populated 15→16 upgrade with
                          injected rollback), Clippy -D warnings, rustfmt, pnpm build
```
