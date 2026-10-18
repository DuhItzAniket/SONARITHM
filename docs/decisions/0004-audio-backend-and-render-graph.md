---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0004 · Own render graph over a cpal-backed `AudioBackend` trait

## Context and problem statement

SONARITHM needs low-latency interactive playback and deterministic offline export from the **same** code. No Rust audio-I/O crate renders offline. The graph engines are either pre-1.0 with reported undefined behaviour (Firewheel 0.14) or stale (glicol, knyst, oddio), and synthizer-rs is archived ([notes/04 §§1–2](../research/notes/04-rust-core-stack.md)).

## Decision drivers

* One `process()` path for real time and offline (P4, SDR-01).
* Real-time safety: no allocation or locks on the audio thread.
* Isolation from upstream churn (cpal 0.19 breaking changes; Firewheel pre-1.0).
* Determinism (seeded randomness, fixed block size).

## Considered options

* An own small render graph (fixed buses + voice pool) behind a `RenderGraph` trait, with cpal behind an `AudioBackend` trait
* Firewheel 0.14 as the graph, behind the same trait
* kira or rodio as the engine
* Write directly against cpal with no abstraction

## Decision outcome

Chosen option: **an own render graph plus our own `AudioBackend` trait**, with cpal 0.18 as the first backend and an `OfflineBackend` for export. fundsp 0.23 supplies the synthesis building blocks; granular, wavetable and modal nodes are custom. **Firewheel is trialled behind the same trait in A02** and adopted only if it beats the owned graph without safety concerns.

### Consequences

* Good: deterministic export through the shipping code; small API surface to maintain; cpal and Firewheel are swappable.
* Good: sonification needs a modest graph (buses, voices, spatializer, limiter), not a general modular DAW engine.
* Bad: we own scheduler and voice-management code. Mitigated by Miri, shuttle tests and soak tests ([testing-strategy](../engineering/testing-strategy.md)).
* Bad: no WASAPI or CoreAudio exclusive mode in cpal yet, so shared-mode latency applies on Windows. ASIO is an opt-in feature, and the `wasapi` crate is a later option (risk R-03).

### Confirmation

A02 graph spike: both implementations render the benchmark Score, and we compare CPU per block, xruns in a soak test, code size, Miri results and API ergonomics. A05 exit criteria (0 xruns at 128 frames, ≥ 50× real-time offline, no allocations on the RT thread) confirm the final choice.

## Pros and cons of the options

### Firewheel 0.14
* Good: DAG compiler, real-time-safe event scheduling, sample and musical clocks, sampler, filters.
* Bad: pre-1.0; a Bevy audit reported "nontrivial UB"; HRTF and CLAP not done; its API may reshape as it moves toward Bevy.

### kira / rodio
* Good: easy, maintained.
* Bad: game- and playback-oriented; no deterministic offline render path; limited control over scheduling.

### No abstraction
* Bad: cpal API churn leaks everywhere, and offline export would need a second code path.
