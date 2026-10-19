# A03 phase review: Core types, Spec v0.1 and Score IR

> **Status:** Final (2026-10-19). **Phase:** A03. **Author:** A03 agent session (Claude Code) for the project lead. **Reviewers:** coordinator session (owner's delegate), @DuhItzAniket.
> **Related:** [reviews README](README.md) · [roadmap A03](../../planning/roadmap.md#a03--core-types-spec-v01-and-score-ir) · [A03 retrospective](../retrospectives/A03-retro.md) · [worklog](../worklog/2026-10-19-a03-core-types-spec-score.md) · [RFC-0001](../../rfcs/0001-spec-v0.1.md)

## Summary

| Item | Value |
|---|---|
| Phase | A03 · Core types, Spec v0.1 and Score IR ([roadmap](../../planning/roadmap.md#a03--core-types-spec-v01-and-score-ir)) |
| Planned window | 2026-11-15 – 2026-12-06 |
| Actual window | 2026-10-19 (one day, in parallel with A02, A04 and A05) |
| Outcome | **Done** |

A03 delivered the project's shared vocabulary:

- `sonarithm-core`: unit newtypes over a `libm` maths module, IDs and row references, provenance, diagnostics with a compile-time-checked code registry, the Score IR with invariants, a canonical hash and rounded review snapshots, and the engine capability traits with the registry and `CompileContext`;
- `sonarithm-spec`: spec v0.1 as frozen by RFC-0001, with serde and schemars types, the generated JSON Schema, layered validation with JSON pointers and miette reports, RFC 8785 canonicalization, the spec hash, defaults reported as `Info`, and a migration framework;
- `sonarithm-compiler`: the pipeline skeleton that compiles a spec end to end through whatever engines are registered;
- fuzz targets, proptest generators, and a test that validates every spec example in the docs.

Two decisions were recorded on the way: [ADR-0015](../../decisions/0015-spec-v0.1-frozen.md) (spec v0.1 frozen per RFC-0001, all 21 resolutions accepted) and [ADR-0014](../../decisions/0014-capability-trait-signatures.md) (capability traits generic over the data type, with spec input as JSON fragments, accepted by the coordinator under the owner's delegation).

## Exit criteria

| # | Criterion (verbatim from the roadmap) | Result | Evidence |
|---|---|---|---|
| 1 | 100% of the spec examples in the docs validate. | Met | `crates/sonarithm-spec/tests/doc_examples.rs` (#97) reads 04 at build time: the 4 complete examples and the 9 section fragments parse and validate without errors, and the §2 skeleton's keys are checked. The fuzz seeds (the same examples plus one spec using every section) are kept valid by `tests/fuzz_corpus.rs` (#105) |
| 2 | The schema in `schemas/` is regenerated and diffed in CI. | Met | `cargo xtask codegen` writes `schemas/sonarithm-spec-0.1.schema.json` (#66) and the code registry page (#6); the `docs / schema-diff` job regenerates both and fails on any diff. It ran green on every A03 PR and on `main`. Making it a required check is the follow-up below |
| 3 | A fuzz run of ≥1 h finds no panic. | Met | `spec_parse` ran 3 721 s on the owner's Windows machine: 7 755 932 executions, no crash or panic, peak RSS 623 MB ([worklog](../worklog/2026-10-19-a03-core-types-spec-score.md#measurements)). Linux CI: `nightly.yml` run [39250910610](https://github.com/DuhItzAniket/SONARITHM/actions/runs/39250910610), 3 601 s per target: `spec_parse` 27 825 673 executions and `spec_migrate` 82 703 844, no crash in either |
| 4 | Public types are documented (`#![deny(missing_docs)]` on core and spec). | Met | `#![deny(missing_docs)]` in `sonarithm-core`, `sonarithm-spec` and `sonarithm-compiler`; CI builds docs with `RUSTDOCFLAGS="-D warnings"` |

Waivers: none.

## Definition of Done (phase)

- [x] All issues in the phase are closed. #35 (A03-T02) closes with this review's PR, #121 with #128.
- [x] No open critical bug from the phase.
- [x] Docs match what was built. Recorded deviations and additions:
  - 05 §3 and §4 carry implementation notes: the trait signatures of ADR-0014, the event `type` tag, `pitch_range` on tracks and `shapers` in the header;
  - 05 §4.3 defines the canonical Score form;
  - 04 §7 names the canonical spec pipeline;
  - coding-standards §4.1 (miette only in `sonarithm-spec`) and §10 (`sonarithm_core::math`, per-crate clippy configuration);
  - testing-strategy §3 (sharing `test-support` strategies) and §7 (fuzzing locally and on Windows);
  - codes added beyond RFC-0001 §12: `SNR-SPEC-080` (default applied), `SNR-COMP-020` … `023`.
- [x] [status.md](../status.md) updated.
- [x] Findings logged: F-019 (serde_json float parsing) and F-020 (Rust float formatting ties versus RFC 8785).
- [x] Traceability matrix: FR-020 … FR-023 point at their tests (#105).
- [x] Risk register: no A03 risk changed. The determinism risk is reduced by the three-OS hash tests; it stays tracked under ADR-0013 until A05's golden audio.
- [ ] Coverage targets (≥ 85% for core, spec and compiler): not measured, because the coverage tool is chosen in A05 ([testing-strategy §16](../../engineering/testing-strategy.md#16-coverage-targets)). Every validation rule, invariant and conversion has a direct test.
- [x] Retrospective held: [A03-retro.md](../retrospectives/A03-retro.md).

## Metrics against budget

| Metric | Budget | Measured | Conditions | Evidence | Verdict |
|---|---|---|---|---|---|
| CI wall time on a PR | ≤ 15 min (A01-T07) | About 2–4 min per `ci.yml` run | GitHub-hosted runners, 3-OS matrix, rust-cache warm | `ci.yml` runs on `main` after #105 and #127 | Within |
| Spec fuzzing | ≥ 1 h without a panic (A03 exit) | 3 721 s, 7 755 932 executions, 2 084 per second | Windows 11, i7-14650HX, libFuzzer with ASan | Worklog, measurements | Within |
| Spec fuzzing on Linux CI | ≥ 1 h without a panic (A03 exit) | 3 601 s per target: `spec_parse` 27 825 673 executions (peak RSS 709 MB), `spec_migrate` 82 703 844 | `ubuntu-latest`, nightly toolchain, libFuzzer with ASan | `nightly.yml` run 39250910610 | Within |
| Workspace tests | — | 189 tests pass in about 2 s (nextest) | `cargo xtask ci`, debug build, after #128 | #128 | — |

## Scope

**Delivered**

| Deliverable (from the roadmap) | Where | PRs |
|---|---|---|
| Units as newtypes, IDs, `RowRef`, `Provenance`, `Diagnostic`, the code registry | `sonarithm-core`, `xtask codes`, [error codes](../../reference/error-codes.md) | #4, #12, #6 |
| `Score`, `ScoreTrack`, `ScoreEvent`; the insta snapshot format and canonical hash | `sonarithm-core::score` | #106, #111 |
| Capability traits, engine registry, `CompileContext` | `sonarithm-core::engine`, `sonarithm-compiler` | #10 (ADR-0014; accepted in #117) |
| Spec v0.1: RFC, serde and schemars types, JSON Schema in `schemas/` | `docs/rfcs`, `sonarithm-spec`, `schemas/` | #3 (ADR-0015), #66 |
| Semantic validation with miette diagnostics and JSON pointers | `sonarithm-spec::{parse, validate, report}` | #96, #97 |
| Migration framework, RFC 8785 canonical form, spec hash | `sonarithm-spec::{migrate, canonical, defaults, prepare}` | #99 |
| Compiler skeleton | `sonarithm-compiler::pipeline` | #100 |
| Fuzz target for the spec parser; proptest generators; doc-example test | `crates/sonarithm-spec/fuzz`, `nightly.yml`, `sonarithm-spec::testing` | #105, #97 |
| Sharing core's test strategies with other crates | Workspace `proptest` features, compiler tests | #128 |

**Deferred or dropped**

| Item | Reason | Where it goes |
|---|---|---|
| `sonarithm schema` CLI command | The CLI arrives in A07; `cargo xtask codegen` writes the schema until then | A07 |
| Field checks (`SNR-SPEC-070`, `071`) for file sources | The field list of a file is known only after loading | A04 data layer (`FieldCatalog` from a `DataBundle`) |
| `Sink`, `VoiceProvider` and navigation-tree capabilities | Added with their first implementations (ADR-0014) | A05, B03, C04 |
| Transforms in the compiler | They run in `sonarithm-data`; the skeleton warns (`SNR-COMP-022`) | A04 |
| The flagged-concept list of `SNR-SPEC-042` | A small constant until NUMISONA's concept table exists | A06 |
| Shipped migrations | v0.1 is the first spec version | With the first version change |

## Lessons for the next phases

1. **Squash merges and stacks.** GitHub closes a PR whose base branch is deleted (#5, #7, #8), and squash merges hide history, so every PR touching shared lines (the lockfile, code-table lists) conflicts after its parent merges. Retarget the child to `main` before merging the parent, and rebase the stack in one burst with `git rebase --update-refs`.
2. **Parse exactly, format exactly.** Byte-identical artifacts need exact parsing (`serde_json/float_roundtrip`, F-019) as well as exact formatting, and RFC 8785 needs ECMAScript's ties-to-even digits, which Rust's own formatting does not give (F-020).
3. **cargo metadata unifies dev-dependency features.** Dependency rules computed from it see test-only features as normal edges (#121); keep `test-support` dependencies clear of forbidden crates.
4. **Fuzzing on Windows works** with the MSVC AddressSanitizer runtime on `PATH`; it also gave a local 1 h run without waiting for CI.
5. **Cancelled duplicate CI runs** report a failing `all-checks-pass` for a commit whose real run is still going; check the run, not only the rollup, before calling a PR red.

## Follow-ups

| Item | Owner | Tracked in |
|---|---|---|
| Make `docs / schema-diff` a required check | Coordinator | Ruleset |
| Switch A05's render tests to `sonarithm_core::testing::valid_score` | A05 | #121 note in #128 |
| Implement `CompilerData` for `DataBundle`, apply transforms, add the file source | A04 | A04 issues |
| Merge the Dependabot bumps of `sha2` (#124, safe as is) and `rand_chacha` (#125, safe with the import fix pushed to its branch); both need a lower-case PR title to pass `pr-title` | Coordinator | #124, #125 |
