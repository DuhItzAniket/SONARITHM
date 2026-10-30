# A07 phase review: CLI, Python package and first release

> **Status:** Final for the code (2026-10-21; release scope updated 2026-10-30); the release itself waits for the owner. **Phase:** A07. **Author:** A07 agent session (Claude Code) for the project lead. **Reviewers:** coordinator session (owner's delegate), @DuhItzAniket.
> **Related:** [reviews README](README.md) · [roadmap A07](../../planning/roadmap.md#a07--cli-python-package-and-first-release) · [worklog: A07](../worklog/2026-10-21-a07-cli-python-release.md) · [11 bindings, CLI](../../architecture/11-bindings-cli-plugins.md) · [release process](../../engineering/release-process.md)

## Summary

| Item | Value |
|---|---|
| Phase | A07 · CLI, Python package and first release ([roadmap](../../planning/roadmap.md#a07--cli-python-package-and-first-release)) |
| Planned window | after A05 and A06, about 2 weeks |
| Actual window | 2026-10-19 – 2026-10-21 (with a pause for the owner's usage limit and a laptop restart) |
| Outcome | **Code complete; release ready for the owner's go.** Exit criteria 2 and 3 need owner actions |

A07 delivered what people use and cite:

- the `sonarithm` facade (compile, quick, describe, render to files and to memory) as the stable Rust API;
- the `sonarithm` CLI: `render`, `play`, `quick`, `validate`, `inspect`, `lint` (stub), `schema`, `spec migrate` and `devices`, with `--json`, `--plain` and exit codes 0–3;
- the `sonarithm` Python package: abi3 wheels for CPython 3.10 and later, Arrow PyCapsule input from pyarrow, pandas and polars, notebook playback, typed stubs;
- the examples gallery: five synthetic datasets (CC BY 4.0) with specs and a notebook, rendered in CI;
- release tooling (release-plz, PyPI Trusted Publishing, CLI archives with SBOM, notices and attestations), Zenodo metadata, and a quickstart tutorial;
- the v0.1.0-alpha release PR with its gate checklist.

The CLI, Python and the core produce the same Score and the same WAV bytes for the same spec and seed, and this is tested: the parity golden is checked by Rust and by pytest on three OSes, and the CLI's render is compared with the facade's.

## Exit criteria

| # | Criterion (verbatim from the roadmap) | Result | Evidence |
|---|---|---|---|
| 1 | `pip install sonarithm` works on all three OSes. | **Met for the built wheels; publishing waits for the owner** | `wheels.yml` builds the abi3 wheel on Linux (manylinux_2_28), Windows and macOS, installs it alone into a clean venv (no required dependencies), then runs the 19–20 pytest tests on Python 3.10 and 3.14: all green (run 39503116734 on #171). `pip install sonarithm` from PyPI needs the first upload (owner) |
| 2 | A Zenodo DOI is minted. | **Not yet: owner** | `.zenodo.json` and `CITATION.cff` are ready and checked in CI (#172); the DOI is minted when the owner enables the Zenodo–GitHub integration and publishes the tagged release |
| 3 | Quickstart docs take a new user from install to their first FLAC in ≤ 5 minutes (tested with 3 people). | **Docs done; test: owner** | `docs/tutorials/quickstart.md` (every command checked; the CLI path takes 3.6 s of compute on HW-01); the three-person protocol is in the tutorial, and the test is on the owner's checklist (#129) |

Waivers requested: none. Criteria 1 (upload) and 2 depend on the publish, which the owner gates; criterion 3 is a user test the agent cannot run.

## Definition of Done (phase)

- [x] Issues: #156, #77, #78, #79, #80, #81, #82 and #83 closed by their PRs; #84 (release) closes when the owner publishes.
- [x] No open critical bug from the phase.
- [x] Docs match what was built: 11 §1–§3 "as built", 04 and 05 (`dataset` source), release-process (configuration notes, CLI binaries, first-release owner steps, Zenodo), CI/CD inventory, supply chain (release tools and their licences), testing strategy (release independence), datasets.md (DS-001 to DS-005 available), tutorials.
- [x] [status.md](../status.md) updated.
- [x] Findings logged: F-100 to F-103.
- [x] Traceability: FR-004, FR-120, FR-121 Done; FR-023 Done; FR-083 stub; NFR-12 in progress (owner's test).
- [x] Risk register: no change.
- [x] Coverage targets: not measured (no coverage tooling in CI yet); every command has a snapshot or JSON test, the Python package runs against installed wheels, determinism has parity tests.
- [x] Retrospective: folded into "Lessons" below (one agent session ran the phase).

## Metrics

| Metric | Budget | Measured | Conditions | Verdict |
|---|---|---|---|---|
| Quickstart CLI path, compute only | ≤ 5 min including install | 3.6 s for `quick … --spec-out … -o temps.flac` (debug build) | HW-01, Windows 11 | Within (the human test is pending) |
| Wheels | 3 OSes × Python 3.10 and 3.14 | 6 of 6 test jobs green | GitHub-hosted runners | Met |
| Dry-run release | succeeds | `cargo publish --workspace --dry-run --locked` packaged and verify-built all 11 publishable crates | HW-01; CI `release-plz / dry-run` (green from #182) | Met |

### Release scope: what v0.1.0-alpha publishes (2026-10-30)

The roadmap's release table plans `0.1.0-alpha` as the core, NUMISONA, the CLI and the Python package. On 2026-10-30 the owner chose to release from that day's main instead of a Stage A snapshot. That way the release includes the fixes merged since Stage A, such as the fuzzing fixes for untrusted chart and Parquet input (#374). On that main the facade depends on `sonarithm-audaxis`, `sonarithm-sonotrace`, `sonarithm-plotsona` and `sonarithm-i18n`, and crates.io refuses a crate whose dependencies are not published. So the release publishes 16 crates:

| | Crates |
|---|---|
| Published, stable for the alpha | `sonarithm`, `sonarithm-core`, `sonarithm-spec`, `sonarithm-data`, `sonarithm-compiler`, `sonarithm-dsp`, `sonarithm-render`, `sonarithm-audio`, `sonarithm-export`, `sonarithm-numisona`, `sonarithm-echometric` (the CLI's `study` commands need it), `sonarithm-cli` |
| Published, **experimental** (APIs may change in any release before 1.0) | `sonarithm-audaxis`, `sonarithm-sonotrace`, `sonarithm-plotsona`, `sonarithm-i18n` |
| Not published | `sonarithm-app`, `sonarithm-desktop`, `sonarithm-speech`, `sonarithm-stream`, `sonarithm-py` (PyPI wheels instead), `xtask` |

No published crate depends on an unpublished one (`cargo metadata`). The changelog section marks the experimental code, and says that the desktop app is not in the release. A dated note in the roadmap's release table records the deviation.

### Release gate: performance budgets

The owner decided the budgets that A05 left open in the coordinator session on 2026-10-21: "Accept A05's recommendations for v0.1.0-alpha: defer PB-01 to B03, waive PB-03 for alpha with a follow-up before Gate B, accept PB-02 with a 128-frame rerun later, and accept PB-06." The measurements are A05's ([A05 phase review](A05-phase-review.md)).

| Budget | Measured (A05) | Decision for `v0.1.0-alpha` | Follow-up |
|---|---|---|---|
| PB-01 key press → cue onset ≤ 50 ms p95 | 104.0 ms p95 (RAW shared), HW-01 loopback (F-052) | **Deferred to B03** | #186; cable test #123 |
| PB-02 0 xruns in 10 min at 128 frames | 0 underruns at the device's 480- and 144-frame periods; 128 frames not available on HW-01 (F-055) | **Accepted** | 128-frame rerun on capable hardware, #187 |
| PB-03 offline render ≥ 50× real time | 91.4× on HW-01, 43–46× on HW-01e (F-053) | **Waived for the alpha**; since **met**: 60× on HW-01e on 2026-10-25 (#185, F-134) | Done (#185) |
| PB-04, PB-05 | Within budget (A06, A04) | — | — |
| PB-06 null residual ≤ −90 dBFS, ΔLUFS ≤ 0.1 | Bit-identical on three OSes (F-056) | **Accepted** | — |

## Scope

**Delivered**

| Deliverable (from the roadmap) | Where | PRs |
|---|---|---|
| `without_inference()` keeps declared fields (A06 follow-up) | `sonarithm-data` | #159 |
| Facade crate API + rustdoc examples (T01) | `crates/sonarithm` | #162 |
| CLI commands, `--json`, `--plain`, exit codes (T02) | `apps/cli` | #169, #174 |
| Score snapshots independent of the release version (release prerequisite) | NUMISONA tests | #170 |
| Python package, PyCapsule input, notebook audio, stubs, wheels CI (T03) | `bindings/python`, `wheels.yml` | #171 |
| Zenodo metadata, `CITATION.cff` (T06) | `.zenodo.json`, `xtask citation` | #172 |
| Examples gallery (T04) | `examples/`, `xtask gallery`, `ci / gallery` | #175 |
| release-plz, PyPI publishing, CLI archives (T05) | `release-plz.toml`, workflows | #176, #182 (dry-run fix) |
| Quickstart tutorial (T07) | `docs/tutorials` | #180 |
| Release v0.1.0-alpha, gate checklist (T08) | the release PR | #183 |

**Deferred or dropped**

| Item | Reason | New target phase | Issue |
|---|---|---|---|
| The NavTree in `inspect --tree` | Navigation is AUDAXIS's; `--tree` prints the Score's events | B03 | — |
| L2 data statistics and L3 trends in `describe` | ECHOMETRIC's descriptions; A07's text gives L1 facts and Score-level L2 (extremes, estimates) | C05 | — |
| `render --stems`, MIDI output | Not in A05's export | C04 (AXONOTE) | — |
| Data-point steps in `play` | Steps are one second until navigation exists | B03 | — |
| The perceptual lint behind `sonarithm lint` | ECHOMETRIC | C05 | — |
| Wheels for Linux aarch64 and macOS x86_64 | Untestable on hosted runners without emulation; the three tested platforms first | B07 | — |
| OD-11: the Python API docs tool (pdoc or Sphinx) | The alpha documents the API with typed stubs, docstrings and the package README; a generated reference belongs with the documentation site (OD-13) | C06 (deferred 2026-10-21) | — |

## Decisions

| ADR / RFC | Title | Status |
|---|---|---|
| — | No new ADR. Contract-relevant choices stayed additive: the `dataset` source kind exists in spec v0.1 (RFC-0001); `Compiled.provenance` and `CompilerData::provenance_mix` are additive; no IR, manifest or IPC change | — |
| OD-14 | cargo-binstall (GPL-3.0) inside release-plz/action | Closed 2026-10-21: the action was replaced with the checksum-verified download in every job (#184); allowing GPL build tools would be the owner's licensing decision |

Local decisions are in the worklog (PyO3 0.29, in-memory layout normalization, release-plz layout, CLI archives without dist).

## Risks

| Risk ID | Change | Note |
|---|---|---|
| — | None | The hearing-safety incident (worklog) is closed by a structural guard; trusted publishing and the GitHub App are owner set-up, not risks |

## Findings and research hooks

| Item | Status | Link |
|---|---|---|
| F-100 · The same rows from pyarrow, pandas and polars changed the Score hash until layouts and library metadata were normalized | Logged | [findings log](../../research-paper/findings-log.md) |
| F-101 · arrow-pyarrow 60 fixes PyO3 at 0.29 | Logged | as above |
| F-102 · Every Score hash changes with every release, because the header records crate versions | Logged | as above |
| F-103 · The musl build of release-plz cannot render any changelog; the glibc build can | Logged | as above |

## Accessibility

- CLI: results are sentences, one fact per line; diagnostics are miette's narratable reports with `--plain` or off a terminal; no colour carries meaning; `--plain` output is checked for box drawing and escapes in tests; `play` announces every change as a text line; diagnostics come before playback so a screen reader hears them first. A spec report no longer reads a whole spec aloud for a missing top-level key.
- Python: `describe()` is deterministic text; the notebook display is text first with a labelled `<audio>` element.
- Manual checks (owner, #129): NVDA (and VoiceOver or Orca if available) reading `sonarithm --plain inspect --describe`, a `play` session with keys, and the notebook display.

## Lessons

- Shared tables conflict on every rebase when four sessions merge in parallel. A resolver script that keeps main's rows and our A07 rows saved most of the time; its first versions lost rows (change-history lines share a date key; A07 can sit in the third cell), so check every resolved table by eye.
- A test that can make sound must not be able to reach a real device by default. One missing `--backend null` played a test's tones on the owner's headphones; the guard is now an environment variable set by the test helper for every run.
- Determinism needs checks at each language boundary, not only inside the core: pandas metadata and polars string views were invisible until the same data from five routes was hashed (F-100).
- Required checks do not include `wheels`, so auto-merge does not wait for it; read its result before relying on a Python change.
- A failing check that is not required still needs its cause reproduced. The first explanation of the release-plz failure matched the error text, but a run on the CI's own platform (WSL Ubuntu) showed that the binary build was the cause (F-103).
- After the laptop restart, the worktree's git index and `.gitignore` were corrupted (NUL padding); `git reset` rebuilt the index, and a scan for NUL bytes found the one damaged file.

## Follow-up actions

| Action | Owner | Due | Issue |
|---|---|---|---|
| Publish v0.1.0-alpha (release-process "First release: owner steps") | Owner | Gate A | #84, #129 |
| Run the three-person quickstart test; log the median | Owner | Gate A | #129 |
| Screen-reader check of the CLI and the notebook display | Owner | Gate A | #129 |
| Fill in author names and ORCID in `CITATION.cff` and `.zenodo.json` | Owner | before the tag | #129 |
| Add the concept DOI to `CITATION.cff` and the README after the first DOI | Owner or agent | after the release | — |
| Choose the Python API docs tool (OD-11, deferred to C06 on 2026-10-21) | Core lead | C06 | — |

## Sign-off

| Role | Handle | Date | Decision |
|---|---|---|---|
| Project lead | @DuhItzAniket | | |
