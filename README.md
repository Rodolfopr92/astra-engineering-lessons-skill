# Astra Engineering Lessons & Architecture Skill

> An authoritative engineering reference, post-mortem synthesis, and agent skill derived from building the **Astra Analytics Bot** (Rust + SurrealDB 3.2.4 + Python Subprocess Worker + Fiscal NF-e Authoritative Parsing).

## Purpose
This repository hosts the `astra-architecture-and-lessons` skill. It captures every hard bug, security boundary condition, architectural pivot, and validation standard established during Astra's development.

Agents and engineers can integrate this skill into Antigravity or any modern agentic environment to prevent repeating past mistakes and adhere to strict production constraints.

---

## Directory Structure

```text
.
├── SKILL.md                                        # Master skill definition with YAML frontmatter
├── README.md                                       # Repository overview & usage
└── references/
    ├── surrealdb-3-contract-and-pitfalls.md        # SurrealDB 3.2.4 typed records, RecordId trap, restart persistence
    ├── subprocess-sandbox-and-path-containment.md  # Subprocess isolation, symlink defenses, bounded streaming
    ├── multi-agent-clobbering-and-concurrency.md   # Multi-agent edit races, atomic DB operations, TOCTOU
    ├── authoritative-xml-and-fiscal-parsing.md     # Safe XML, NF-e Modulo 11 check digits, Decimal math, SEFAZ protocol
    └── rigorous-ci-harness-and-testing-discipline.md # 8-stage gate, restart testing, fixture grounding
```

---

## How to Install in Antigravity

1. Clone or add this repository to your Antigravity skills path:
   ```bash
   git clone https://github.com/Rodolfopr92/astra-engineering-lessons-skill.git ~/.gemini/antigravity/skills/astra-architecture-and-lessons
   ```
2. The skill will automatically be discovered by Antigravity and loaded into the pairing context.
