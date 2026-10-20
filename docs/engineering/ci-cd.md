# CI/CD

> **Status:** Baseline v1 (2026-10-18). Nothing described here exists yet; each workflow lands in the phase shown. **Owner:** Project lead (release manager).
> Related: [Git workflow](git-workflow.md) · [Release process](release-process.md) · [Supply chain and licensing](supply-chain-and-licensing.md) · [Testing strategy](testing-strategy.md) · [Tech stack](tech-stack.md) · [SECURITY.md](../../SECURITY.md) · [Roadmap](../planning/roadmap.md)

This document plans the GitHub Actions workflows, the rules that harden them, the required status checks, and the code-signing and key-custody plan. Sources: [research notes 07 §3–§4 and §9](../research/notes/07-engineering-publication.md) and the [research report](../research/research-report.md#cicd-and-signing).

## Principles

1. **Every PR is checked on Linux, Windows and macOS.** Platform differences in audio, file paths and webviews are the main risk.
2. **One gate.** A single aggregate job is the required check for `ci.yml`, so jobs can be added without editing the branch ruleset.
3. **Least privilege.** Workflows start with no token permissions and each job requests only what it needs.
4. **Releases are reproducible and attested.** Pinned toolchain, committed lockfiles, `--locked` builds, build-provenance attestations and SBOMs.
5. **Release credentials live only in protected environments.** Registry publishing uses OIDC trusted publishing, not long-lived tokens.

## Workflow inventory

| Workflow | Trigger | Purpose | Lands in |
|---|---|---|---|
| `ci.yml` | `pull_request`, push to `main`, `merge_group` | Rust checks on three OSes; grows by phase (see below) | A01 |
| `pr-title.yml` | `pull_request` (opened, edited, synchronize) | PR title is a valid Conventional Commit header with an allowed type and scope | A01 |
| `dco.yml` | `pull_request` | Every commit has a `Signed-off-by:` trailer. A DCO GitHub App is the alternative; if it is chosen, this file is not created | A01 |
| `docs.yml` | Every `pull_request` and push to `main` (no path filter: a path-filtered workflow used as a required check would leave PRs that skip it waiting forever) | Markdown lint; internal link check; external link check (not required; advisory on PRs and pushes, where it only warns, and failing in the weekly scheduled run; policy in `lychee.toml`); `schema-diff` from A03: regenerate `schemas/` (and from B01 `packages/ipc/src/generated/`), then `git diff --exit-code` | A01 (`schema-diff` A03) |
| `scorecard.yml` | Push to `main`, weekly schedule, `branch_protection_rule` | OpenSSF Scorecard; results to code scanning and the public badge | A01 |
| `codeql.yml` | `pull_request`, push to `main`, weekly schedule | CodeQL static analysis for Rust, JavaScript/TypeScript and Python as each language appears; confirm CodeQL's Rust support level when enabling | A01 |
| `label-sync.yml` | Push to `main` touching `.github/labels.yml` | Apply the [label taxonomy](../../.github/labels.yml) | A01 |
| `.github/dependabot.yml` (configuration, not a workflow) | Weekly schedule | Version updates for cargo, npm, pip and github-actions, including SHA-pinned actions | A01 |
| `nightly.yml` | Daily schedule, manual dispatch; pull requests touching `sonarithm-core`, `sonarithm-spec` or `sonarithm-data` (two-minute fuzz smoke run) | Fuzzing (spec from A03, data importers from A04), benchmarks and soak tests (from A05), external link check, cargo-deny advisories on `main`, desktop nightly builds (Stage B) | A03, growing |
| `cache-prune.yml` | Completed `ci`, `docs` or `nightly` run on `main`; manual dispatch | Keeps only the newest Actions cache of each job on `main` and deletes superseded copies, so the cache stays well under the 10 GB repository limit (on 2026-10-19 it reached 10.7 GB with 3–4 copies per job) | A05 |
| `ci.yml` `gallery` job | Pull requests, push to `main` | Checks that `examples/data` regenerates from its seeded generators (`cargo xtask gallery --check`) and renders every gallery spec to FLAC with the CLI (artifact `gallery-renders`) | A07 |
| `wheels.yml` | Pull requests touching the crates or `bindings/python`, push to `main`, release tags `v*`, manual dispatch (`publish: testpypi`) | Builds abi3 wheels (3 OSes) and the sdist with maturin-action, installs each wheel into a clean Python 3.10 and 3.14 and runs pytest; on release tags attests everything and publishes to PyPI with Trusted Publishing (TestPyPI on a manual rehearsal) | A07 |
| `release-plz.yml` | Push to `main`; pull requests that change release configuration; manual dispatch | Release PR (GitHub App token), the release itself when a release PR merges (crates.io trusted publishing, tag, GitHub release), and the dry run (`cargo publish --workspace --dry-run`, `release-plz update`) | A07 |
| `release-binaries.yml` | Release tags `v*`; pull requests that change it or `about.toml`/`about.hbs`; manual dispatch | CLI archives for 3 platforms with `cargo auditable`, a CycloneDX SBOM and third-party notices; attested and uploaded to the GitHub release on tags | A07 |
| `release-desktop.yml` | Release tag `v*`, manual dispatch | tauri-action builds for three OSes, signing, updater artifacts and `latest.json` | Stage B previews (roadmap: desktop nightly from B03); signed beta B07 if certificates are in place; full signing and updater C06 |

### What `ci.yml` runs, by phase

This table lists CI jobs. The authoritative list of test types per phase is [testing strategy §19](testing-strategy.md#19-test-types-by-phase).

| Job | Runs on | Command or check | From |
|---|---|---|---|
| `fmt` | ubuntu | `cargo fmt --all --check` | A01 |
| `clippy` | ubuntu, windows, macos | `cargo clippy --workspace --all-targets --locked -- -D warnings` | A01 |
| `test` | ubuntu, windows, macos | `cargo nextest run --workspace --locked`, then `cargo test --workspace --doc --locked` (nextest does not run doctests) | A01 |
| `doc` | ubuntu | `cargo doc --workspace --no-deps --locked` with `RUSTDOCFLAGS="-D warnings"` | A01 |
| `deny` | ubuntu | `cargo deny check` (licences, advisories, bans, sources) | A01 |
| `check-deps` | ubuntu | `cargo xtask check-deps` (crate dependency rules in [crate layout §3](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced)) | A01 |
| `msrv` | ubuntu | Each crate built with its declared `rust-version`: 1.88 for core crates, 1.90 for `sonarithm-desktop`, 1.96 for `sonarithm-plugin-host` ([crate catalogue](../architecture/02-repository-and-crate-layout.md#2-crate-catalogue)); for example with cargo-hack's `--rust-version` option (tool chosen in A01) | A01 |
| `wasm32` | ubuntu | `cargo build --locked --target wasm32-unknown-unknown` for the compiler, engine, render and DSP crates (crate layout rule 4) | A01, as crates appear |
| `golden` | ubuntu, windows, macos | Golden Score snapshots must match exactly; golden audio must match within tolerance (null-test residual at or below −90 dBFS, LUFS difference at most 0.1, per [overview §7](../architecture/01-overview.md#7-quality-attributes-and-budgets)) | A05 |
| `no-alloc` | ubuntu, windows, macos | Debug-build render tests with `assert_no_alloc` around the audio callback | A05 |
| `semver` | ubuntu | `cargo semver-checks` on published library crates (needs a published baseline) | A07 |
| `python` | ubuntu, windows, macos | Build the wheel with maturin, install it, run the pytest smoke suite | A07 |
| `frontend` | ubuntu, windows, macos | `pnpm install --frozen-lockfile`, lint, typecheck, Vitest browser-mode tests, Storybook axe checks and Playwright with Tauri IPC mocking (both from B02) | B01 |
| `licenses-npm` | ubuntu | npm licence allowlist check ([supply chain](supply-chain-and-licensing.md#npm-licence-checking)) | B01 |
| `reuse` | ubuntu | `reuse lint` ([supply chain](supply-chain-and-licensing.md#reuse-and-spdx-headers)); runs in `docs.yml` | A01 |
| `all-checks-pass` | ubuntu | Fails if any job above failed or was cancelled | A01 |

Golden-audio and soak tests on GitHub-hosted runners have no audio device; they exercise the offline renderer. Real-device soak tests (the A05 "0 xruns in a 10-min soak" criterion) run on reference hardware, started manually by a maintainer on a trusted ref. Self-hosted runners are never used for untrusted PR code.

## Required status checks

Configured in the `main` ruleset. Job names are part of the contract; renaming one requires updating the ruleset in the same PR.

| Check | Required from |
|---|---|
| `ci / all-checks-pass` (aggregates every `ci.yml` job, so new jobs are covered automatically) | A01 |
| `pr-title` | A01 |
| `dco` (workflow job or DCO App status) | A01 |
| `docs / markdown` | A01 |
| `docs / internal-links` | A01 |
| `docs / reuse` | A01 |
| `docs / schema-diff` | A03 |

In the ruleset, each check is the job's check-run name (`all-checks-pass`, `pr-title`, `dco`, `markdown`, `internal-links`, `reuse`), bound to the GitHub Actions app (integration ID 15368) so that no other app can satisfy it. The `main` ruleset was created in A01 with these checks before their first run, as the bootstrap exception to the rule below. The A01 pull request is their first run.

Not required: Scorecard, scheduled jobs and external link checks. The external link check is advisory on pull requests (it warns but never fails), because third-party hosts rate-limit, reset connections or block CI clients unpredictably. Its weekly scheduled run fails on errors and is the signal to fix links or update `lychee.toml`. CodeQL alerts are triaged rather than gating merges; requiring code-scanning results in the ruleset is reconsidered at C06.

## Hardening rules

Each rule maps to an [OpenSSF Scorecard](https://github.com/ossf/scorecard/blob/main/docs/checks.md) check where one exists.

| Rule | Detail | Scorecard check |
|---|---|---|
| Empty default permissions | Every workflow sets top-level `permissions: {}`. Each job requests the minimum, usually `contents: read`. | Token-Permissions (High) |
| Pinned actions | Every `uses:` references a full 40-character commit SHA with a version comment (`# vX.Y.Z`). Dependabot updates the SHA and the comment. | Pinned-Dependencies (Medium) |
| No dangerous triggers | No `pull_request_target` or `workflow_run` job checks out or runs code from a pull request. Untrusted values (`github.event.pull_request.title`, branch names) never appear in `${{ }}` inside `run:`; pass them through `env:`. | Dangerous-Workflow (Critical) |
| No persisted credentials | `actions/checkout` runs with `persist-credentials: false` unless a job must push. | |
| OIDC trusted publishing | crates.io through `rust-lang/crates-io-auth-action` (`id-token: write`; the first release of each crate is published manually); PyPI through Trusted Publishing. No registry tokens in secrets. | |
| Protected environments | `release` (crates.io, PyPI, GitHub release) and `desktop-signing` (signing certificates, updater key). Deployments are limited to `main` and release tags; required reviewers are added once a second maintainer exists, following the release-gate pattern in [uv's release workflow](https://github.com/astral-sh/uv/blob/main/.github/workflows/release.yml). | |
| Provenance | `actions/attest-build-provenance` attests every released binary, wheel, installer and SBOM (`id-token: write`, `attestations: write`). Users verify with `gh attestation verify`. | Signed-Releases (High) |
| Immutable releases | Enabled for stable releases. The setting is all-or-nothing, so nightly builds are workflow artifacts, not GitHub Releases in this repository. | |
| Caching | `Swatinem/rust-cache` in PR and `main` workflows. Only `main` saves caches (`save-if`), and `cache-prune.yml` deletes superseded ones. Release workflows build without restoring caches, so a poisoned cache cannot reach a release. | |
| Locked builds | `--locked` (Cargo) and `--frozen-lockfile` (pnpm) everywhere; the toolchain is pinned by `rust-toolchain.toml`. | |
| Concurrency and timeouts | PR runs cancel superseded runs; every job sets `timeout-minutes`. | |
| Secrets | Secrets live only in environments. Secret scanning and repository-level push protection are enabled (push protection is not on by default). | |

## Release pipelines

The [release process](release-process.md) describes the steps and checklists. In summary:

| Artifact | Workflow | Publishing | Signing and provenance |
|---|---|---|---|
| Rust crates | `release-plz.yml` | crates.io, trusted publishing | crates.io does not sign; provenance comes from the tagged source and trusted publishing |
| CLI binaries | `release-plz.yml` release job, or dist-generated jobs (A07 decision) | GitHub Release assets | Attestations; Windows signing when available; built with `cargo auditable` |
| Python wheels | `wheels.yml` | PyPI, Trusted Publishing | Attestations |
| Desktop installers | `release-desktop.yml` | GitHub Release assets, `latest.json` for the updater | Platform signing (below), Tauri updater signatures, attestations |
| SBOMs | Release jobs | GitHub Release assets | Attestations |

dist v0.33.0 (September 2026) added Azure Artifact Signing for Windows binaries and installers, but its maintenance is questioned, so it is used, if at all, as a generator the project can eject from ([notes 07 §3](../research/notes/07-engineering-publication.md)).

## Signing plan by platform

Signing needs paid accounts and, for Windows, an eligible legal identity. Until signing is in place, preview builds ship unsigned with documented warnings. The roadmap schedules full signing in C06; [release process](release-process.md#desktop-release) covers the beta.

| Platform | Mechanism | Requirements and caveats |
|---|---|---|
| **Windows** | **Azure Artifact Signing** (renamed from Trusted Signing, GA January 2026), configured in Tauri through `bundle.windows.signCommand` with `artifact-signing-cli` and `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID` | Available to organisations or individuals in the USA and Canada, and to organisations in the EU or UK. An individual maintainer elsewhere may not be eligible; eligibility depends on the project's legal home ([open decision OD-02](../planning/open-decisions.md)). EV certificates no longer grant instant SmartScreen reputation; reputation builds over time, so every release is signed with the same identity. |
| Windows, alternatives | SignPath Foundation's free signing for open-source projects (eligibility to be verified); an OV certificate held in a cloud HSM; or unsigned builds with a documented SmartScreen workaround | Decided in a signing ADR before B07 |
| **macOS** | **Developer ID Application** certificate, notarization with `notarytool` (the older notarization tooling was retired in November 2023) | Paid Apple Developer Program. Only the Account Holder can create a Developer ID certificate. CI variables: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `KEYCHAIN_PASSWORD`, and for notarization the App Store Connect API key (`APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`, preferred) or an Apple ID with an app-specific password. Ad-hoc signing (`signingIdentity: "-"`) for local builds only. |
| **Linux** | **GPG-signed AppImage**: `SIGN=1`, `SIGN_KEY`, `APPIMAGETOOL_SIGN_PASSPHRASE`, `APPIMAGETOOL_FORCE_SIGN=1` so that a signing failure fails the build | The signature is not verified automatically; the key fingerprint is published through an authenticated channel (SECURITY.md and the release notes). Tauri's docs do not cover deb or rpm signing. Flathub, if used later, signs with its own infrastructure. |

Sources: [Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/), [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/), [Tauri Linux signing](https://v2.tauri.app/distribute/sign/linux/), [notes 07 §3](../research/notes/07-engineering-publication.md).

## Tauri updater key custody

The Tauri updater verifies every update against the public key embedded in installed apps, and verification cannot be disabled. **If the private key is lost, existing installations can never be updated again** ([Tauri updater](https://v2.tauri.app/plugin/updater/)).

1. **Generation.** The project lead generates the key pair with `tauri signer generate`, password-protected, on a trusted machine, never in CI and never in the repository (`.gitignore` excludes key files).
2. **Two custodians.** The project lead and a second custodian, appointed before the first updater-enabled release (C06, see [governance](../../GOVERNANCE.md#continuity)), each hold an encrypted offline backup in a separate physical location. The password is stored separately from the key.
3. **CI use.** The key and password exist in CI only as `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in the `desktop-signing` environment (`.env` files are not supported for these). The public key content goes in `plugins.updater.pubkey`, and `bundle.createUpdaterArtifacts` is `true`.
4. **Drill.** Once a year, and whenever a custodian changes, both custodians confirm their backup decrypts. The result is recorded in the gate or release review, without the key material.
5. **Compromise.** Remove the CI secret, stop publishing updates, publish an advisory, and ship a final update signed with the old key that carries a new public key (planned procedure; rehearse it before C06).

The same custody model (two holders, offline backup, environment-only CI secrets) applies to the Apple Developer ID certificate, the Windows signing credentials and the AppImage GPG key.

## Budget

Figures from [notes 07 §3](../research/notes/07-engineering-publication.md) (October 2026); prices can change.

| Item | Cost | Needed from |
|---|---|---|
| Apple Developer Program (Developer ID, notarization) | US$99 per year | Signed macOS beta (B07) or C06 |
| Azure Artifact Signing, Basic (5,000 signatures, 1 certificate profile) | US$9.99 per month | Signed Windows beta (B07) or C06, if eligible |
| Azure Artifact Signing, Premium (100,000 signatures, 10 profiles) | US$99.99 per month | Not expected to be needed |
| SignPath Foundation OSS signing | Free if eligible (to be verified) | Alternative to Artifact Signing |
| GitHub-hosted runners for a public repository | No cost assumed; verify current plan limits | A01 |
| **Estimated total** with Apple and Artifact Signing Basic | **About US$219 per year** (estimate) | From the first signed release |

## Illustrative `ci.yml` skeleton

**Illustrative — created in A01.** This is not a working file. Replace every `<sha>` with the full commit SHA of the release named in the comment, and adjust package lists as crates appear.

```yaml
# .github/workflows/ci.yml (illustrative — created in A01)
name: ci

on:
  pull_request:
  push:
    branches: [main]
  merge_group:

permissions: {}

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

env:
  CARGO_TERM_COLOR: always

jobs:
  fmt:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@<sha> # vX.Y.Z
        with:
          persist-credentials: false
      # The toolchain (with rustfmt, clippy and the wasm32 target) comes from rust-toolchain.toml.
      - run: cargo fmt --all --check

  clippy-test:
    name: clippy and test (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    permissions:
      contents: read
    timeout-minutes: 45
    steps:
      - uses: actions/checkout@<sha> # vX.Y.Z
        with:
          persist-credentials: false
      - uses: Swatinem/rust-cache@<sha> # vX.Y.Z
      - run: cargo install cargo-nextest --locked # installer action chosen in A01
      - run: cargo clippy --workspace --all-targets --locked -- -D warnings
      - run: cargo nextest run --workspace --locked
      - run: cargo test --workspace --doc --locked

  deny:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@<sha> # vX.Y.Z
        with:
          persist-credentials: false
      - uses: EmbarkStudios/cargo-deny-action@<sha> # vX.Y.Z

  wasm32:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@<sha> # vX.Y.Z
        with:
          persist-credentials: false
      - uses: Swatinem/rust-cache@<sha> # vX.Y.Z
      - run: >-
          cargo build --locked --target wasm32-unknown-unknown
          -p sonarithm-core -p sonarithm-spec -p sonarithm-compiler

  # doc, msrv and check-deps jobs follow the same pattern.

  all-checks-pass:
    if: always()
    needs: [fmt, clippy-test, deny, wasm32]
    runs-on: ubuntu-latest
    permissions: {}
    timeout-minutes: 5
    steps:
      - if: contains(needs.*.result, 'failure') || contains(needs.*.result, 'cancelled')
        run: exit 1
```

## Changing CI

- A workflow change is a normal PR with the `ci` scope. Changes under `.github/` need code-owner review.
- A new required check must be green on `main` before it is added to the ruleset.
- When a phase adds a job, update the tables in this document in the same PR.
