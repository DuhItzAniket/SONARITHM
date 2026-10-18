---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0000 · Record architecture decisions with MADR

## Context and problem statement

SONARITHM is meant to outlive its first contributors. The sonification field's recurring failure mode is abandoned tools whose design rationale is lost ([research report](../research/research-report.md)). AI coding agents also contribute here, and they need durable, discoverable reasons for past decisions, not chat history.

## Decision drivers

* Decisions must be reviewable in pull requests and versioned with the code.
* Low ceremony, so ADRs actually get written.
* A recognized format that agents and humans both understand.

## Considered options

* MADR 4 Markdown files in `docs/decisions/`
* Nygard-style ADRs with adr-tools
* Wiki pages or GitHub Discussions only

## Decision outcome

Chosen option: **MADR 4 in `docs/decisions/`**. It is the most structured lightweight format (considered options, consequences, an explicit *Confirmation* section suited to our prototype gates), it is plain Markdown, and it is the format recommended in the kickoff research ([notes/07 §1](../research/notes/07-engineering-publication.md)).

### Consequences

* Good: decisions are reviewed like code, and AGENTS.md can point agents to them.
* Good: provisional decisions with a gate (Tauri, React Aria, render graph) are explicit.
* Bad: some overhead per decision. Mitigated by keeping ADRs short and using RFCs only for contract changes.
