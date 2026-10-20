# 2026-10-19 – 2026-10-21: A07 CLI, Python package and first release

> **Date(s):** 2026-10-19 – 2026-10-21. **Phase:** A07. **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [11 bindings, CLI](../../architecture/11-bindings-cli-plugins.md) · [release process](../../engineering/release-process.md) · [A07 phase review](../reviews/A07-phase-review.md)

## Context and phase

- Phase: A07, [roadmap entry](../../planning/roadmap.md#a07--cli-python-package-and-first-release); tasks A07-T01 to T08 ([implementation plan](../../planning/implementation-plan.md)), issues #77 to #84 and #156.
- PRs, one per task under the fast-track rule: [#159](https://github.com/DuhItzAniket/SONARITHM/pull/159) (#156), [#162](https://github.com/DuhItzAniket/SONARITHM/pull/162) (T01), [#169](https://github.com/DuhItzAniket/SONARITHM/pull/169) and [#174](https://github.com/DuhItzAniket/SONARITHM/pull/174) (T02), [#170](https://github.com/DuhItzAniket/SONARITHM/pull/170) (version-free snapshots), [#171](https://github.com/DuhItzAniket/SONARITHM/pull/171) (T03), [#172](https://github.com/DuhItzAniket/SONARITHM/pull/172) (T06), [#175](https://github.com/DuhItzAniket/SONARITHM/pull/175) (T04), [#176](https://github.com/DuhItzAniket/SONARITHM/pull/176) (T05), [#180](https://github.com/DuhItzAniket/SONARITHM/pull/180) (T07), [#182](https://github.com/DuhItzAniket/SONARITHM/pull/182) (the release-plz dry-run fix) and [#183](https://github.com/DuhItzAniket/SONARITHM/pull/183), the release PR with this entry (T08).
- Starting point: A03, A04 and A06 done; A05 landing its render, export and audio PRs; the facade and CLI were A01 skeletons.

## Goals

- The stable facade, the CLI, the Python package, the gallery, the release tooling, Zenodo metadata and a quickstart, up to a release PR for v0.1.0-alpha that the owner can publish.

## What was done

- **#156:** `without_inference()` keeps declared fields; NUMISONA's examples declare their roles and read without inference (byte-identical data).
- **T01:** the facade: `Sonarithm`, `Spec`, `compile`/`compile_with` (host datasets through `Platform::with_dataset`, `SNR-COMP-026`), `quick`, `Compiled` with summary and L1/L2 description; later `render_to_file` and `render_wav` on A05's export.
- **T02:** the CLI in two PRs: `render`, `validate`, `inspect`, `lint` (stub), `quick`, `schema`, `spec migrate`, then `play` and `devices` on A05's player; `--json`, `--plain`, exit codes 0–3; assert_cmd + insta tests; render bytes equal the facade's.
- **Release prerequisite:** NUMISONA's Score snapshots no longer pin the crate version (F-102); this PR also refreshed main's lockfile, which #162 and #168 had left stale together.
- **T03:** the Python package (PyO3 0.29, arrow-pyarrow 60, abi3 ≥ 3.10), the data layer's `read_batches` with layout and metadata normalization (F-100), `render`/`wav`/`play`, the parity golden, `wheels.yml` (green on 3 OSes × 2 Pythons).
- **T06:** `.zenodo.json`, `CITATION.cff`, `cargo xtask citation` in CI.
- **T04:** the gallery: DS-001 to DS-005 from seeded generators (`cargo xtask gallery`), five specs, README, notebook; the `ci / gallery` job renders all five.
- **T05:** release-plz (one version group, facade tag), `release-plz.yml` (release PR via a GitHub App, release on merge with crates.io trusted publishing, dry run), PyPI Trusted Publishing in `wheels.yml`, `release-binaries.yml` (auditable CLI, SBOM, notices, attestations); the dry run packaged and verify-built all 11 publishable crates. Its release-plz step failed in CI until #182 (F-103).
- **T07:** the quickstart tutorial with the five-minute test protocol.
- **T08:** the release PR: version 0.1.0-alpha everywhere, citation dates, the changelog section, the gate checklist; this worklog and the phase review.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Host data reaches the compiler through the spec's existing `dataset` source kind (`Platform::with_dataset`) | No spec change; Python DataFrames and the CLI's quick data use one path | 04 §3, 05 §2 notes |
| PyO3 0.29 with arrow-pyarrow 60, not PyO3 0.30 | The Arrow PyCapsule import maintained with arrow-rs is built on 0.29; no `unsafe` in SONARITHM | F-101; 11 §3 note |
| In-memory Arrow drops library schema metadata and uses plain text layouts | Otherwise pandas' version and polars' string views change the fingerprint and the Score hash | F-100 |
| The description gives L1 facts and Score-level L2 facts only | Data statistics and trends belong to ECHOMETRIC (C05) | 11 §1 note |
| Score snapshots write engine versions as `*` | The header records crate versions; a release must not change snapshots | F-102; testing-strategy §4 |
| `sonarithm play` steps one second; `--backend null` and `SONARITHM_BACKEND` for silent runs | No navigation tree before AUDAXIS; tests must never reach a real device | 11 §2 note |
| CLI archives from an own workflow, not cargo-dist | Fewer moving parts; auditable builds, SBOM, notices and attestations are three commands | release-process |
| `publish = false` for xtask and sonarithm-py; sonarithm-echometric published since A08-T03 | Wheels are not crates; the CLI's `study` commands need ECHOMETRIC (#179), so it joined the version group in #182 | release-process |
| No `cliff.toml`: release-plz's `[changelog]` is the git-cliff configuration | One source; release-plz embeds git-cliff | release-process |
| The dry run downloads release-plz's glibc build and checks its SHA-256 instead of using install-action | The musl build cannot render a changelog; cargo-binstall (what release-plz/action uses) is GPL-3.0 | F-103; OD-14 |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| **Hearing safety:** a CLI test played 3.25 s of tones through the development laptop's default output (Headphones, Realtek) during a local test run | One `quick` test call lost its `--backend null` when rustfmt split the line and the edit did not match; locally the default device existed | Found when CI (no device) failed. `--backend` now reads `SONARITHM_BACKEND`, and the CLI test helper sets it to `null` for every run. Level at the time: the player's 500 ms ramp from −20 dB and the −1 dBTP limiter; the device's OEM APO may add gain (F-032) |
| Main's `ci` red after #162 and #168 merged | Each passed alone; together the facade's `thiserror` lock entry needed a version qualifier | Lock refresh in #169 and #170 |
| `wheels` is not a required check, so auto-merge could merge a Python PR before its wheels passed | Required checks are the `ci` aggregate and the docs checks | Read the wheels run before relying on it (it was green for #171) |
| After the laptop restart: "index file corrupt" and a `.gitignore` padded with NUL bytes | The machine ran out of memory while four sessions compiled | Rebuilt the index with `git reset`; scanned changed files for NUL bytes and restored the one damaged file; builds now run with `CARGO_BUILD_JOBS=6` |
| A 53 MB `_core.pdb` entered a pushed WIP commit | `maturin develop` copies debug symbols next to the extension | Rewrote the WIP commit on the feature branch; `*.pdb` under `bindings/python` is ignored |
| The shared-table resolver lost rows | Change-history rows share a date key; A07 can be in a requirement row's third cell | Keyed history rows by the whole line; checked every resolved table by eye |
| A PR title scope was refused | `examples` is not an allowed scope | Retitled `docs:`; the changelog parser does not rely on it |
| `release-plz / dry-run` failed on every release-configuration PR with "stack limit reached" at 1:1 of the changelog header | install-action installs release-plz's musl build. Its template parser checks the free stack, and the musl build reports too little. The first explanation (the changelog header as a template) matched the message but not a reproduction | Reproduced under WSL Ubuntu: musl fails and glibc passes, with or without a `header`. The dry run now downloads the glibc build and checks its SHA-256 (#182, F-103). cargo-binstall, which release-plz/action uses, is GPL-3.0: OD-14 for the owner |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Wheels | 6 of 6 test jobs green (Linux, Windows, macOS × Python 3.10, 3.14) | GitHub-hosted runners, run 39503116734 | — |
| Parity | the same Score (value for value) and WAV SHA-256 from Rust and Python on 3 OSes | `tests/golden/parity` | — |
| Dry-run release | 11 crates packaged and verify-built | `cargo publish --workspace --dry-run --locked`, HW-01 | — |
| release-plz update | musl build fails 2 of 2 runs; glibc build passes 2 of 2 runs (with and without `header`) | release-plz 0.3.172, WSL Ubuntu 24.04, glibc 2.39 | F-103 |
| Quickstart CLI step | 3.6 s wall time, debug build | HW-01, Windows 11 | phase review |

## Findings to log

- F-100: the same rows from pyarrow, pandas and polars changed the Score hash until layouts and library metadata were normalized.
- F-101: arrow-pyarrow 60 fixes PyO3 at 0.29.
- F-102: every Score hash changes with every release, because the header records crate versions.
- F-103: the musl build of release-plz cannot render any changelog; the glibc build can.

## Next steps

- [ ] Owner: the first-release steps in release-process.md (crates.io first publish and trusted publishers, PyPI and TestPyPI pending publishers, the `release` environment, the GitHub App, Zenodo, author names and ORCID), then the tag v0.1.0-alpha (#84, #129).
- [ ] Owner: the three-person quickstart test (NFR-12) and the screen-reader checks of the CLI and the notebook display (#129).
- [ ] After the first DOI: the concept DOI in `CITATION.cff` and the README.
- [ ] Owner: OD-14 (cargo-binstall inside release-plz/action is GPL-3.0) before the first release PR from release-plz.
- [ ] OD-11 (Python API docs tool) was not decided in A07: the package documents its API with typed stubs, docstrings and its README for the alpha.
- [ ] `cargo clean` in this worktree after the phase lands (dev-environment "Disk usage").
