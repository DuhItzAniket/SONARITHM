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

**Note (2026-11-01, #434).** "Engine versions" above, and the manifest's `engines`, are the Score header's `engines`: the engines that contributed content, with their versions. That means the engines whose template, source, detector or encoder the spec used, every context provider that added a track, and every shaper that ran ([05 §4](../architecture/05-compiler-and-score-ir.md#4-score-ir)). A registered context provider that adds nothing is not recorded. The decision is unchanged; this states what the record means, and no committed Score changed.

### Consequences

* Good: any result in a paper or bug report can be regenerated exactly at the Score level.
* Good: tests target the strict artifact (Score) and tolerate floating-point variance in audio.
* Bad: discipline is required (no `HashMap` iteration in outputs, no wall clock, seeded RNGs). It is enforced by lints, review checklists and property tests.

### Confirmation

A05: a golden suite runs on all three CI OSes and passes the tolerances. H-08 is evaluated in P07.

## Amendments

| Date | Change | Approved |
|---|---|---|
| 2026-11-26 | The data fingerprint is value-based (fingerprint scheme 2, [RFC-0008](../rfcs/0008-value-based-data-fingerprint.md)), and every record names its scheme: `data_fingerprint_scheme` in the Score header (Score IR version 3), `data.fingerprintScheme` in render manifest version 3, and `datasets[].fingerprintScheme` in project format 1.2 | Coordinator, under the owner's delegation of contract approvals (RFC-0008 accepted 2026-11-26) |

**Amendment 1: what "the data fingerprint" is.** Item 3's data fingerprint was SHA-256 over the Arrow IPC bytes of a canonical copy of the data (scheme 1, A04-T09). arrow-ipc's encoder output is not SONARITHM's to fix, and a minor Arrow release moved every fingerprint without changing any data (F-222), so a record could not be checked by a later build. From RFC-0008 on:

* the fingerprint is SHA-256 over an encoding SONARITHM defines: the field names, logical types and `snr.*` metadata as RFC 8785 canonical JSON, the row count, and every value in row order with a null tag (RFC-0008 §2 and §3);
* it leaves out layout and what no compile path reads: batches, offsets, bytes under nulls, string and list layouts, dictionary key widths, nullability and other metadata;
* floats are hashed bit-exact and dictionaries stay distinct from plain columns, because compile paths order floats with `total_cmp` and NUMISONA takes a category order from a dictionary, so equal fingerprints still imply equal Scores (RFC-0008 §1);
* every record names the scheme beside the fingerprint, and a record without it was written under scheme 1. A scheme-1 fingerprint is never recomputed, since no later build can reproduce it.

The decision is otherwise unchanged. Every Score compiled from the data layer's `DataBundle` names scheme 2 and so records IR version 3 (RFC-0007 §4). A Score whose header names no scheme, such as one built by a compiler test with its own data type, keeps its form and its hash.

**Deviation from RFC-0008 §9.** The RFC expected the canonical JSON to come from `sonarithm-spec`, but `sonarithm-data` may depend only on `sonarithm-core` (`cargo xtask check-deps`). The schema JSON holds only objects, arrays and strings, so `sonarithm-data` writes that subset itself (keys in UTF-16 order, RFC 8785 string escapes), and `crates/sonarithm/tests/fingerprint_scheme.rs` checks it against `sonarithm_spec::canonical` byte for byte.
