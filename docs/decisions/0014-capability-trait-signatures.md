---
status: proposed
date: 2026-10-19
decision-makers: ["@DuhItzAniket"]
consulted: []
informed: []
---

# 0014 · Capability traits are generic over the data type and take spec fragments

## Context and problem statement

[05 §3](../architecture/05-compiler-and-score-ir.md#3-engine-registry-and-capability-traits) sketches the capability traits in `sonarithm-core` with parameters such as `&TrackSpec` (a `sonarithm-spec` type) and `DataBundle`, `DataView` (`sonarithm-data` types, A04). But `sonarithm-core` depends on no other SONARITHM crate ([02 §3](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced) rule 1), and the only crates that see both spec and data are the engines and the compiler. The traits must live in core, because engines implement them and the compiler calls them, yet core cannot name those types. A03-T04 has to finalize the signatures without designing the data layer, which belongs to A04.

## Decision drivers

* 02 §3 rule 1 (core is self-contained) and rule 2 (engines communicate only through core types).
* Do not pre-empt A04's `DataBundle` design or change the spec contract (RFC-0001).
* Plugins (ADR-0009) will cross a WIT boundary, where spec input arrives as JSON.
* Determinism and testability: tests must be able to build a registry with fake engines.

## Considered options

* **A. Generic traits plus spec fragments:** `Source<D>`, `Analyzer<D>`, `Encoder<D>`, `ContextProvider<D>`, `Engine<D>` and `Registry<D>` are generic over the compiler's data type `D`; spec input is a `SpecFragment` (compact JSON plus its JSON pointer) that engines parse into `sonarithm-spec` types.
* **B. Move `DataBundle` and the spec types into core:** core would depend on `arrow-array` and `schemars` and own the spec language.
* **C. A data-access trait object in core:** a `DataView` trait with column accessors that `sonarithm-data` implements.

## Decision outcome

Chosen option: **A**, because it keeps every dependency rule, commits to nothing about the data layer before A04, and matches the plugin boundary. The compiler fixes `D` (its own data type in the A03 skeleton, `sonarithm_data::DataBundle` from A04). `Shaper` works on the `Score` only and is not generic. `Sink` and `VoiceProvider` from the 05 sketch are added with their first implementations (A05, C04), and the navigation tree joins `ContextProvider` in B03; these are additive.

### Consequences

* Good: core stays small, Arrow-free and wasm-friendly; A04 designs `DataBundle` freely.
* Good: native engines and Stage D plugins receive the same JSON fragments.
* Good: `Registry<()>` with fake engines makes registry tests trivial.
* Bad: engines parse their fragment again (cheap: fragments are small, and the spec is already validated).
* Bad: generic traits mean `dyn Encoder<D>` objects are tied to one `D`; a second data type would need a second registry. None is planned.

### Confirmation

A03-T04 tests register fake engines in a `Registry<()>`; A03-T09 compiles a spec end to end through the generic registry; A04 instantiates it with `DataBundle`. Revisit if A04 finds that analyzers and encoders need a data view that cannot be expressed through `D`.

## Pros and cons of the options

### A. Generic traits plus spec fragments

* Good, because no crate boundary moves.
* Neutral, because each engine deserializes its fragment.
* Bad, because trait objects are per data type.

### B. Move `DataBundle` and the spec types into core

* Good, because the 05 sketch would compile as written.
* Bad, because core would carry Arrow arrays and the whole spec language, and the spec crate would shrink to validation, contradicting 02 §2.

### C. A data-access trait object in core

* Good, because engines would be decoupled from Arrow.
* Bad, because it designs data access in A03, before A04 has measured what transforms and encoders need, and adds dynamic dispatch on hot paths.

## More information

Implemented in `sonarithm_core::engine` (A03-T04); 05 §3 links here. Related: [ADR-0007](0007-engines-as-pipeline-capabilities.md), [ADR-0009](0009-wasm-plugins-compile-time.md), RFC-0001 (spec v0.1, in discussion).
