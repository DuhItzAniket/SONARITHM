# 2026-10-19: A03 core types, Spec v0.1 and Score IR

> **Date(s):** 2026-10-19. **Phase:** A03. **Author:** project lead, written by an AI agent (Claude Code) in a side session started by the coordinator session; review by @DuhItzAniket pending.
> **Related:** [worklog conventions](README.md) · [status](../status.md) · [RFC-0001](../../rfcs/0001-spec-v0.1.md) · [findings log](../../research-paper/findings-log.md)

## Context and phase

- Phase: A03 — [roadmap entry](../../planning/roadmap.md#a03--core-types-spec-v01-and-score-ir)
- PRs, in merge order: [#3](https://github.com/DuhItzAniket/SONARITHM/pull/3) (RFC-0001, merged), [#4](https://github.com/DuhItzAniket/SONARITHM/pull/4) (units, merged), [#12](https://github.com/DuhItzAniket/SONARITHM/pull/12) (IDs, provenance, diagnostics, merged), then the stack [#6](https://github.com/DuhItzAniket/SONARITHM/pull/6) → [#7](https://github.com/DuhItzAniket/SONARITHM/pull/7) → [#8](https://github.com/DuhItzAniket/SONARITHM/pull/8) → [#10](https://github.com/DuhItzAniket/SONARITHM/pull/10) → [#66](https://github.com/DuhItzAniket/SONARITHM/pull/66) → [#96](https://github.com/DuhItzAniket/SONARITHM/pull/96) → [#97](https://github.com/DuhItzAniket/SONARITHM/pull/97) → [#99](https://github.com/DuhItzAniket/SONARITHM/pull/99) → [#100](https://github.com/DuhItzAniket/SONARITHM/pull/100) → the T10 PR. [#5](https://github.com/DuhItzAniket/SONARITHM/pull/5) was closed by GitHub when its base branch was deleted and replaced by #12.
- Starting point: A01 merged; the core, spec and compiler crates were empty skeletons; A02 (spikes) ran in parallel in another session.

## Goals

- Write RFC-0001 from the 04 draft and open it (T05).
- Implement T01–T04 in `sonarithm-core`, T06–T08 and T10–T11 in `sonarithm-spec`, and the T09 compiler skeleton, each in small stacked PRs with CI green.

## What was done

- **T05:** RFC-0001 with 21 open questions and proposals; the owner accepted all of them (ADR-0015).
- **T01:** unit newtypes; `sonarithm_core::math` over pure-Rust `libm`; a per-crate `clippy.toml` denies the `std` float functions in core.
- **T02:** IDs, `RowRef`/`RowRange`, provenance with combination rules, `Diagnostic` and RFC 6901 pointers, code tables checked at compile time, `cargo xtask codes` and the generated [code registry](../../reference/error-codes.md).
- **T03:** Score IR types, invariants 1–3 and 5 with proptest, the canonical full-precision form and its SHA-256 pinned on three OSes, and the rounded insta review form.
- **T04:** capability traits generic over the data type with spec input as JSON fragments (ADR-0014, accepted in #117 by the coordinator under the owner's delegation), `Registry` conflict rules, `CompileContext` (per-track ChaCha8 streams, diagnostics, cancellation, budget).
- **T06–T08, T11:** serde + schemars spec types and the generated JSON Schema; layered validation with a failing and a passing test per rule; miette reports with spans; RFC 8785 canonicalization, defaults reported as Info, the spec hash and a migration framework; a test that validates every JSON example in 04.
- **T09:** `Compiler<D>` running every stage with no-op engines; an inline-source spec compiles end to end to an empty Score.
- **T10:** `spec_parse` and `spec_migrate` cargo-fuzz targets with a seed corpus, a `nightly.yml` fuzz job, and valid-spec proptest generators behind `test-support`.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Capability traits generic over the data type; spec input as `SpecFragment` | Core may depend on neither spec nor data (02 §3 rule 1), and A04 owns `DataBundle` | [ADR-0014](../../decisions/0014-capability-trait-signatures.md) (accepted, #117) |
| Score canonical form = compact serde_json output, not RFC 8785 | Exact `u64` seeds and no dependence on a JSON reimplementation; pinned by `ir_version` | 05 §4.3 |
| Spec canonical form = RFC 8785 with `ryu` digits | RFC 8785 is mandated by ADR-0013; Rust's own float formatting breaks ties the wrong way | F-020, RFC-0001 §13 |
| Only `sonarithm-spec` depends on miette, without `fancy` | Core stays free of rendering; apps pick the handler (the narratable handler suits screen readers) | coding-standards §4.1 |
| New codes `SNR-SPEC-080` (default applied) and `SNR-COMP-020` … `023` | "Explain what you assumed" (05 §7); never drop transforms or context silently | [code registry](../../reference/error-codes.md) |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| A generated Score did not round-trip byte for byte | serde_json's default float parser can be one ULP off | `float_roundtrip` feature (F-019) |
| RFC 8785 vector `1424953923781206.2` failed with `{}` formatting | Rust breaks exact decimal ties upward | digits from `ryu` plus the ECMAScript layout (F-020) |
| #5 closed when #4 merged | GitHub closes PRs whose base branch is deleted, unless they are retargeted first | Replaced by #12; merge procedure in the coordinator's notes: retarget the next PR to `main` before merging |
| `cargo fuzz run` exited with `STATUS_DLL_NOT_FOUND` on Windows | The MSVC AddressSanitizer runtime DLL is not on `PATH`; `-s none` does not link with MSVC | Put the MSVC `bin/Hostx64/x64` directory on `PATH` (testing-strategy §7) |
| `status.md` conflicted on every rebase of the stack | The coordinator updates A03 rows on `main` as PRs merge | Keep `main`'s version during rebases; the newest PR refreshes the A03 rows |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| `spec_parse` fuzzing | 7 755 932 runs in 3 721 s (≥ 1 h), no crash or panic; 2 084 runs/s, peak RSS 623 MB, 8 248 new corpus units, coverage 10 228 edges | Windows 11, i7-14650HX, nightly 2026-10-12, libFuzzer with ASan, seed corpus of 5 specs | this entry (fuzzing is a test, not a benchmark) |
| `spec_migrate` fuzzing | 6 291 176 runs in 301 s, no crash | same machine and toolchain | this entry |
| Workspace tests | 147 tests pass in about 1.4 s (nextest) | `cargo xtask ci`, debug build | — |

## Findings to log

- F-019: serde_json's default float parser can be one ULP off — [findings log](../../research-paper/findings-log.md)
- F-020: Rust's float formatting breaks exact decimal ties upward, which RFC 8785 forbids — [findings log](../../research-paper/findings-log.md)

## Next steps

- [x] ADR-0014 accepted (#117) by the coordinator under the owner's delegation.
- [ ] Coordinator: merge the stack in order; make `docs / schema-diff` a required check once green on `main`.
- [ ] Coordinator or owner: run `nightly.yml` by hand with `seconds: 3600` once it is on `main`, so the 1 h run also exists on Linux CI.
- [ ] A04: implement `CompilerData` for `DataBundle`, apply transforms, add file sources, and replace the `FLAGGED_CONCEPTS` constant with NUMISONA's concept table (A06).
