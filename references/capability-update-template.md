# Capability Update Template

Use this template when adding or revising a version-sensitive correction.

The goal is to prevent the skill itself from becoming a confident stale prior.

---

# `<Technology>` — `<Capability / Breaking Change>`

## Baseline

```text
technology:
server/runtime version:
client/SDK version:
language/toolchain version:
connection mode / storage engine:
desktop/runtime framework version:
verified date:
reverify by:
```

## Baseline mismatch rule

State explicitly what happens if the consuming project uses a different version.

```text
If target version != verified version:
- mark this claim UNVERIFIED FOR TARGET
- check official target-version docs
- run reproducer if available
- do not silently transfer TESTED BEHAVIOR across versions
```

## Evidence classification

Choose one or more:

```text
[ ] VERIFIED API
[ ] TESTED BEHAVIOR
[ ] CASE-STUDY EVIDENCE
[ ] PROJECT CONVENTION
[ ] ARCHITECTURAL INTENT
[ ] FALSE / NOT A GENERAL RULE
```

Definitions:

- **VERIFIED API** — confirmed in current official documentation/API for the named baseline.
- **TESTED BEHAVIOR** — independently reproduced against the named baseline.
- **CASE-STUDY EVIDENCE** — observed in a real proving project, but not independently minimized here.
- **PROJECT CONVENTION** — an application choice, not a technology requirement.
- **ARCHITECTURAL INTENT** — proposed/recommended behavior, not implementation evidence.
- **FALSE / NOT A GENERAL RULE** — retained to block a known over-generalization.

Case-study evidence from another patch/minor version is a lead, not automatic proof for the current baseline.

## Stale model prior

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

## Claim-level official source

Attach the primary source directly to the claim, not only to a section bibliography.

```text
claim:
official source URL:
source version/currentness:
```

## Reproducer

```text
path:
command:
expected result:
last successful run:
exact dependency versions:
engine/transport:
```

If no reproducer exists, say `NONE YET`.

A reproducer file that has never passed does **not** create TESTED BEHAVIOR.

## Runtime / compile evidence

```text
repository:
commit SHA:
command/test:
connection mode / storage engine:
environment:
result:
```

If evidence comes only from a proving project, classify it as CASE-STUDY EVIDENCE until independently minimized or supported by official API documentation.

## Scope limits

Examples:

```text
- remote WebSocket only; embedded engine not tested
- embedded SurrealKV only; remote transport not tested
- Linux tested; Windows behavior unknown
- API verified for 3.2.4; future versions unverified
```

## General technical correction

State what an agent should know about the technology. Do not turn a project architecture preference into a skill rule.

```text
<one or two technical sentences>
```

## Proving case study (optional)

```text
project:
exact dependency version:
symptom:
root cause:
fix:
regression test:
```

## Reverification triggers

```text
[ ] target dependency version differs from baseline
[ ] dependency major/minor upgrade
[ ] official API deprecation/change
[ ] compiler/toolchain upgrade
[ ] contradictory runtime result
[ ] new security advisory
[ ] different transport/storage engine
[ ] different desktop/runtime framework
[ ] 90 days since last full verification
```

---

## Acceptance checklist

```text
[ ] exact versions named
[ ] version mismatch behavior stated
[ ] connection/storage mode named where relevant
[ ] stale prior shown explicitly
[ ] current pattern shown explicitly
[ ] claim-level official source linked where available
[ ] reproducer linked or explicitly marked absent
[ ] tested behavior distinguished from documented API
[ ] cross-version case-study evidence not promoted automatically
[ ] project convention not presented as universal rule
[ ] architecture-planning doctrine excluded
[ ] scope limits stated
[ ] old entry superseded or marked stale if needed
[ ] reverification date/trigger recorded
```
