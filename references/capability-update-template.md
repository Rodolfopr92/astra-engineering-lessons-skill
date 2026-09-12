# Capability Update Template

Use this template when adding a new technology/version correction to the skill.

The goal is to prevent confident but weakly grounded “current API” claims from accumulating over time.

---

# `<Technology>` — `<Capability / Breaking Change>`

## Baseline

```text
technology:
server/runtime version:
client/SDK version:
language/toolchain version:
connection mode / storage engine:
verified date:
```

## Evidence classification

Choose one or more:

```text
[ ] VERIFIED API
[ ] TESTED BEHAVIOR
[ ] CASE-STUDY EVIDENCE
[ ] PROJECT CONVENTION
[ ] ARCHITECTURAL INTENT
```

Definitions:

- **VERIFIED API** — confirmed in current official documentation/API for the named version.
- **TESTED BEHAVIOR** — independently reproduced with a minimal compile/runtime/integration test against the named version.
- **CASE-STUDY EVIDENCE** — observed in a real proving project, but not yet independently reduced/reproduced by this skill repository.
- **PROJECT CONVENTION** — an application design choice, not a technology requirement.
- **ARCHITECTURAL INTENT** — proposed/recommended behavior, not implementation evidence.

## Stale model prior

What older pattern is an AI model likely to generate?

```text
<old API / old syntax / wrong assumption>
```

Why is it stale?

```text
<version boundary, renamed API, changed type system, removed behavior, etc.>
```

## Current verified pattern

```text
<minimal current code or API example>
```

## Official sources

Prefer versioned/current primary sources.

```text
- <official docs URL>
- <API reference URL>
- <release/migration notes URL if relevant>
```

## Runtime / compile evidence

Record what was actually executed.

```text
repository:
commit SHA:
command/test:
connection mode / storage engine:
environment:
result:
```

If there is no independent runtime evidence, say so. If evidence comes from an external proving project, classify it as CASE-STUDY EVIDENCE until reduced/reproduced independently.

## Scope limits

Explicitly say what this correction does **not** prove.

Examples:

```text
- remote WebSocket only; embedded engine not tested
- embedded SurrealKV only; remote transport not tested
- Linux tested; Windows behavior unknown
- parser extracts protocol fields but does not contact external authority
- API verified for 3.2.4; do not assume future major versions
```

## General rule

State the reusable lesson without project-specific names.

```text
<one or two sentences>
```

## Proving case study (optional)

If a real project exposed the stale prior:

```text
project:
symptom:
root cause:
fix:
regression test:
```

The case study is supporting evidence, not the rule itself.

## Reverification trigger

When should this entry be checked again?

```text
[ ] dependency major/minor upgrade
[ ] official API deprecation
[ ] compiler/toolchain upgrade
[ ] contradictory runtime result
[ ] new security advisory
[ ] migration to different transport/storage engine
[ ] migration to different desktop/runtime framework
```

---

## Acceptance checklist for a new capability entry

```text
[ ] exact versions named
[ ] connection/storage mode named where relevant
[ ] stale prior shown explicitly
[ ] current pattern shown explicitly
[ ] official source linked where available
[ ] tested behavior distinguished from documented API
[ ] external case-study evidence not promoted to independently tested fact
[ ] project convention not presented as universal rule
[ ] scope limits stated
[ ] no roadmap item presented as completed capability
[ ] old entry superseded or marked stale if needed
```
