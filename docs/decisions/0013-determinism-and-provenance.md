---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0013 · Determinism and render provenance

## Context and problem statement

Sonification research suffers from unreproducible stimuli. Hermann's definition requires that the same data and interactions produce structurally identical sound. How do we make every SONARITHM output reproducible and attributable?

## Decision drivers

* SDR-01 and SDR-02 (spec + data determine the render; declared, seeded randomness).
* Golden testing across Windows, macOS and Linux.
* Publishable artifacts (papers cite release DOIs and spec hashes).

## Considered options

* Strict determinism of the Score IR + tolerance-based audio comparison + a mandatory render manifest
* Bit-exact audio everywhere
* Best effort (no guarantees)

## Decision outcome

Chosen option:

1. **The Score IR is byte-identical** for the same (spec, data, seed, engine versions) on every platform. Every value that reaches the Score is computed with **platform-independent maths**: the pure-Rust `libm` crate for transcendental functions (`exp`, `ln`, `log2`, `pow`, `sin`…). The `std` float functions may differ in the last bits between platforms. Byte-identity is checked on the full-precision serialization by hash. Human-readable snapshots are rounded ([05 §4.3](../architecture/05-compiler-and-score-ir.md#43-snapshot-format)).
2. **Audio is equal within tolerance:** a peak null-test residual ≤ −90 dBFS (RMS reported alongside) and ΔLUFS ≤ 0.1. Bit-exactness across CPUs and SIMD paths is not promised.
3. Every export writes a **render manifest** (`*.render.json`) containing:
   * engine and engine-crate versions and the git commit;
   * the RFC 8785-canonical spec with defaults applied, and its SHA-256;
   * the data fingerprint and the provenance mix;
   * seed, sample rate, block size and mode;
   * output hash and loudness;
   * the OS and architecture.

   Paths are relative or redacted ([06 §7](../architecture/06-audio-engine.md#7-export)).

### Consequences

* Good: any result in a paper or bug report can be regenerated exactly at the Score level.
* Good: tests target the strict artifact (Score) and tolerate floating-point variance in audio.
* Bad: discipline is required (no `HashMap` iteration in outputs, no wall clock, seeded RNGs). It is enforced by lints, review checklists and property tests.

### Confirmation

A05: a golden suite runs on all three CI OSes and passes the tolerances. H-08 is evaluated in P07.
