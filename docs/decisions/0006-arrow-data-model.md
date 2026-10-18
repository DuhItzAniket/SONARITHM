---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0006 · Apache Arrow as the internal data model

## Context and problem statement

Data arrives from files, Python DataFrames, charts, functions and live streams. It must carry semantics (type, unit, concept) and per-value provenance, and cross into Python and later JavaScript efficiently.

## Decision drivers

* Zero-copy interop with pyarrow, pandas and polars (the research audience).
* Columnar performance for 1M+ rows.
* Metadata that survives file round trips.
* Rust API stability.

## Considered options

* Apache Arrow (arrow-rs 60) + Parquet as the internal contract
* Polars DataFrame as the internal model
* A custom columnar structure
* ndarray matrices

## Decision outcome

Chosen option: **Arrow**, with our semantic metadata in Arrow field metadata (`snr.*`) and reserved provenance columns (`__snr_prov`, `__snr_conf`) ([03](../architecture/03-data-model.md)). **Polars is an optional adapter only.**

### Consequences

* Good: zero-copy to Python via the Arrow C Data Interface; Parquet and Arrow IPC preserve metadata; a stable, widely used format.
* Good: a contract independent of DataFrame libraries.
* Bad: Arrow's Rust API is lower-level than Polars, so transforms are written by hand. That is acceptable, since our transform set is small ([03 §6](../architecture/03-data-model.md#6-transforms)).
* Rejected Polars-internal: its versioning policy says Rust API breaking changes "are not deprecated first" ([notes/04 §6](../research/notes/04-rust-core-stack.md)).
