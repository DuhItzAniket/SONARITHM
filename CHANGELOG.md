# Changelog

All notable changes to SONARITHM are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). Versions follow [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) with the pre-1.0 rules in the [release process](docs/engineering/release-process.md#versioning-policy). The spec version (`specVersion`) is versioned separately.

From phase A07, release-plz maintains this file: it generates each release's entries with git-cliff from Conventional Commit messages and proposes them in the release pull request. Edit wording in that pull request, not after the release. Until A07, entries are written by hand.

## [Unreleased]

### Added

- Documentation baseline (2026-10-18), written before any code:
  - Research: the kickoff research report, seven research notes, and the design-rules catalogue (`SDR-##`, `A11Y-##`).
  - Product: the engine catalogue (nine engines on one platform).
  - Architecture: the overview, the repository and crate layout, and the component documents.
  - Decisions: initial architecture decision records in MADR 4 format, and the RFC process.
  - Planning: the roadmap (stages A to D, phases A01 to D06), implementation plan, risk register and open decisions.
  - SDLC: the process overview, live status, Definition of Ready and Done, and traceability matrix.
  - Engineering: tech stack, coding standards, testing strategy, development environment, git workflow, CI/CD, release process, and supply chain and licensing policy.
  - Design: the accessibility conformance plan and design documentation.
  - Research-paper track: the findings log and publication planning.
  - Community and governance: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (Contributor Covenant 3.0 by reference), `SECURITY.md`, `GOVERNANCE.md`, `CITATION.cff`, issue forms, pull request template, `CODEOWNERS` and the label taxonomy.
  - Agent instructions: `AGENTS.md` and `CLAUDE.md`.
  - Repository hygiene: `.gitignore`, `.gitattributes` and `.editorconfig`.
- Repository bootstrap (phase A01):
  - Cargo workspace with 11 skeleton crates, pinned toolchain, workspace lints, `rustfmt`, `clippy` and `cargo-deny` configuration, and `cargo xtask` automation (`ci`, `check-deps`, `codegen`, `wasm-check`).
  - pnpm workspace with placeholder frontend packages, ESLint and Prettier.
  - GitHub Actions workflows for CI, documentation checks, PR titles, DCO, CodeQL and OpenSSF Scorecard, plus Dependabot configuration.
- Licensing ([ADR-0010](docs/decisions/0010-licensing.md)): code under `MIT OR Apache-2.0`; documentation, example datasets and study stimuli under `CC-BY-4.0`. Adds `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSES/`, `REUSE.toml`, SPDX headers and a REUSE check in CI.
- Render engine (phase A05):
  - `sonarithm-dsp` oscillators: sine, triangle, and band-limited (PolyBLEP) saw and square, computed with IEEE 754 arithmetic only so renders stay bit-identical across platforms.
  - `sonarithm-dsp` envelopes: ADSR with raised-cosine onset and offset ramps of at least 5 ms, about 40 ms for notes (SDR-22); retriggers and releases never step.
  - `sonarithm-dsp` filters (one-pole low-pass; RBJ low-, high- and band-pass), a constant-power panner and a linear parameter smoother (10 ms).
- NUMISONA, the parameter-mapping engine (phase A06):
  - `sonarithm-numisona` scales: `linear`, `log`, `symlog`, `sqrt`, `pow`, `quantile`, `band` and `ordinal`, with `auto`, `zero-based` and explicit domains, `nice` rounding, and off-scale reporting for clamped values.
  - `sonarithm-numisona` pitch spaces and ranges: log-frequency (the default, SDR-09), linear frequency, mel, ERB-rate and Bark; the 400–1400 Hz default range and the 65–1480 Hz expanded range (SDR-08); ranges in hertz, MIDI notes or cents.
  - `sonarithm-render` plan builder: a Score becomes a `RenderPlan` in frames, with resolved frequencies, gains, pan gains, instruments and voice-pool sizes; unknown instruments raise `SNR-RENDER-001` and fall back to the mallet preset.
  - `sonarithm-render` render graph: preallocated voice pools, data, context, earcon and monitoring buses, a sample-accurate scheduler and a transport (play, pause, stop, seek, loop, speed and live cues) with crossfades instead of cuts. Earcon sounds are provisional until B03.

[Unreleased]: https://github.com/DuhItzAniket/SONARITHM/commits/main
