---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0001 · Rust workspace as the single core

## Context and problem statement

SONARITHM must deliver real-time audio, deterministic offline rendering, a desktop app, a CLI, Python bindings and a WebAssembly build. Which implementation language and structure gives one trustworthy core for all of them?

## Decision drivers

* Real-time safety: no garbage-collector pauses on the audio path.
* One core for desktop, CLI, Python and WASM (the primary differentiator in the research report).
* Memory safety for parsers of untrusted files (charts, PDFs, spreadsheets).
* Ecosystem fit: cpal, fundsp, Arrow, PyO3, wasm-bindgen, Tauri.
* The owner's preference for Rust.

## Considered options

* Rust Cargo workspace (multiple crates)
* C++ (JUCE) core with bindings
* TypeScript/Web Audio core (like most active web sonification tools)
* Python core with a C-extension DSP (like STRAUSS or pyo)

## Decision outcome

Chosen option: **a Rust Cargo workspace**, edition 2024. The core crates (`core`, `spec`, `data`, `compiler`, `dsp`, `render`, engines) have **MSRV 1.88**. Leaf crates may require more (Tauri 1.90, wasmtime 1.96). CI builds with the pinned latest stable (1.99.x as of 2026-10-18).

### Consequences

* Good: the only language that meets real-time safety, memory safety, WASM and Python together with mature tooling. The Rust sonification niche is essentially empty ([notes/01](../research/notes/01-tools-landscape.md)).
* Good: a multi-crate workspace enforces architectural boundaries ([02 §3](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced)).
* Bad: a smaller pool of audio and DSP contributors than C++ or JS. Mitigated by docs, AGENTS.md and good-first-issues.
* Bad: Rust audio crates churn (cpal 0.19 is planned). Mitigated by owning traits ([ADR-0004](0004-audio-backend-and-render-graph.md)).

## Pros and cons of the options

### C++ (JUCE)
* Good: the industry standard for audio plugins, with deep DSP libraries.
* Bad: memory-unsafe parsers; JUCE licensing (GPL or commercial); weaker WASM and Python story; against the owner's preference.

### TypeScript / Web Audio
* Good: where most active tools live (MAIDR, Chart2Music, Erie).
* Bad: no deterministic offline rendering parity across browsers; weak native desktop audio; competes head-on instead of complementing.

### Python core
* Good: the research audience lives there.
* Bad: GIL and garbage-collection pauses on the audio path; heavy desktop packaging. Python is better served as a binding.
