---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0007 · Nine engines as capabilities of one compiler

## Context and problem statement

The product identity has nine named engines (NUMISONA, AUDAXIS, SONOTRACE, PLOTSONA, CURVASONA, AXONOTE, ECHOMETRIC, TONALENS, RESONUM) under the SONARITHM platform. How should they be built: as separate subsystems, or as parts of one pipeline?

## Decision drivers

* Sustainability. Tool abandonment and feature creep are the field's dominant failure modes. Firewheel's author cites feature creep, and Budziszewski & Groß-Vogt (ICAD 2025) name sustainability as a central challenge ([research report](../research/research-report.md#nine-engines-should-compile-through-one-specification-shipped-in-three-waves)).
* Reproducibility: every engine's output must be captured by the spec and Score (SDR-01).
* A small maintainer pool.
* A clear identity for users and papers.

## Considered options

* Nine independent subsystems (each with its own data path, renderer and UI)
* **Nine capability providers** registered in one compiler (Sources, Analyzers, Encoders, Context, Shapers, Voices, Sinks, Interactive)
* No engine identity: a single monolith

## Decision outcome

Chosen option: **capability providers in one compiler** ([05 §3](../architecture/05-compiler-and-score-ir.md#3-engine-registry-and-capability-traits), [07](../architecture/07-engines.md)). Each engine is one crate. Engines never depend on each other, and they communicate only through core types. Registration conflicts fail fast.

### Consequences

* Good: one data model, one Score, one renderer; adding an engine adds capabilities, not a parallel stack.
* Good: the engine names stay meaningful product surfaces (UI groupings, docs, papers) without architectural cost.
* Good: third-party plugins (ADR-0009) use the same capability model.
* Bad: some engines (SONOTRACE live mode, TONALENS lens, RESONUM excitation) need interactive paths beyond static compilation. These are provided as `Interactive` capabilities using live events.
* Rule: **no tenth engine without an ADR.** New capability goes into an existing engine or a plugin.
