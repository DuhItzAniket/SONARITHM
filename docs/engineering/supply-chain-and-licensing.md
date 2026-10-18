# Supply chain and licensing

> **Status:** Baseline v1 (2026-10-18). The project licence is pending [ADR-0010](../decisions/0010-licensing.md). **Owner:** Project lead.
> Related: [CI/CD](ci-cd.md) · [Release process](release-process.md) · [SECURITY.md](../../SECURITY.md) · [CONTRIBUTING.md](../../CONTRIBUTING.md#licence-of-contributions) · [Tech stack](tech-stack.md) · [Crate layout](../architecture/02-repository-and-crate-layout.md) · [Risk register](../planning/risk-register.md)

This document sets the licence policy for SONARITHM and its dependencies, models and assets, and the controls that keep the dependency supply chain auditable. Sources: [research notes 07 §4–§5](../research/notes/07-engineering-publication.md), [notes 04 §12–§14](../research/notes/04-rust-core-stack.md), [notes 06](../research/notes/06-chart-extraction-ai.md) and the [research report](../research/research-report.md#security-supply-chain-and-the-cyber-resilience-act).

## Project licence

**Pending ADR-0010.** The proposal is **`MIT OR Apache-2.0`** for all code: the crates, the CLI, the Python bindings and the desktop app. ADR-0010 must be accepted in phase A01 (roadmap exit criterion) before any code is merged.

If accepted:

- Every `Cargo.toml` inherits `license = "MIT OR Apache-2.0"` from `[workspace.package]`. The repository root holds `LICENSE-MIT` and `LICENSE-APACHE`.
- **Inbound equals outbound.** Contributions are licensed under the same terms (Apache-2.0 §5), certified per commit with the **DCO**. There is no CLA.

Why this licence ([notes 07 §5](../research/notes/07-engineering-publication.md)):

- It is the Rust ecosystem norm and matches the Rust project itself, so it gives maximum compatibility (Rust API Guidelines, C-PERMISSIVE).
- Apache-2.0 contributes an explicit patent grant. MIT is kept because Apache-2.0 is incompatible with GPLv2.
- It is OSI-approved, as JOSS requires, and it lets labs, companies and other open-source projects embed SONARITHM.
- A DCO, unlike a CLA, does not deter casual academic contributors and implies no relicensing power.

Documentation, example datasets and audio stimuli are not software and are licensed separately; datasets and study stimuli use CC BY 4.0 ([crate layout §1](../architecture/02-repository-and-crate-layout.md#1-repository-tree-target)). For documentation, ADR-0010 recommends CC BY 4.0 or the code licence, at the owner's choice ([open decision OD-09](../planning/open-decisions.md)).

## Dependency licence policy

Default builds contain only permissive dependencies ([principle P9](../architecture/01-overview.md#2-architectural-principles)). Copyleft code may appear only behind **non-default** features in leaf crates ([crate layout §3 rule 5](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced)). cargo-deny enforces the allowlist in CI from A01.

### Allowed

| SPDX identifier | Notes |
|---|---|
| `MIT` | |
| `Apache-2.0` | |
| `Apache-2.0 WITH LLVM-exception` | |
| `BSD-2-Clause`, `BSD-3-Clause` | |
| `ISC` | |
| `Zlib` | |
| `Unicode-3.0` | Unicode data tables |
| `BSL-1.0` | Boost Software License. Not to be confused with the Business Source License (`BUSL-1.1`), which is denied |
| `CC0-1.0` | |
| `MPL-2.0` | File-level weak copyleft. Allowed, with this condition: if the project modifies an MPL-licensed file, the modified file must be published under MPL-2.0. Known MPL-2.0 dependencies: symphonia, triple_buffer, serialport, audio_thread_priority |
| `OFL-1.1` | Fonts only (for example npm font packages) |

A licence not in this table is denied until a PR adds it with a rationale. Research notes 04 §12 also listed `Unlicense`; it is not on the initial allowlist and can be added by PR if a needed dependency requires it.

### Denied

| Licence | Rule |
|---|---|
| `AGPL-*` | Never |
| `GPL-*` (any version, `-only` or `-or-later`) | Never |
| `SSPL-1.0` | Never |
| `BUSL-1.1` (Business Source License) | Never |
| `LGPL-*` | Never in default features. Allowed only in an opt-in feature of a leaf crate, through a crate-specific exception approved in a PR (for example the `mp3` feature of `sonarithm-export`) |

### Known traps

From the research. The cargo-deny allowlist catches the crates automatically; the other items are design references or model files and need human vigilance.

| Item | Licence | Where it could enter | Policy |
|---|---|---|---|
| `evalexpr` | AGPL-3.0-only since 12.0.0 (2024-10-17) | CURVASONA expressions | Banned. Use exmex behind a facade, and rhai for scripts |
| `anomaly_detection` | GPL-3.0-or-later | ECHOMETRIC detectors | Banned. Use augurs or a port of simple algorithms |
| `mp3lame-encoder` | LGPL-3.0 | MP3 export | Optional `mp3` feature only, never default |
| `oxisynth` | LGPL-2.1 | SoundFont playback (AXONOTE) | Not used. rustysynth (MIT) is the SF2 choice |
| nih-plug VST3 bindings | GPLv3 obligations | A plugin-format target | No VST3 support through nih-plug |
| ChartGemma | Code GPL-3.0 | ML chart extraction (D04) | Do not vendor or link its code. Model weights have separate terms (see [model weights](#model-weights)) |
| Moondream 3 Preview weights | Business Source License 1.1 (converts to Apache-2.0 two years after release) | Local VLM (D04) | Not bundled. Moondream 3.1 uses a custom model licence: also not bundled |
| WebPlotDigitizer | AGPL-3.0 (frontend); its cloud "AI Assist" is closed source | PLOTSONA raster digitizer (C02) | Design reference only. Re-implement from its documentation and published papers; never copy, port or translate its source |
| Engauge Digitizer | GPL | Digitizer reference | Reference only; dormant upstream |
| Slint | GPLv3 or commercial | Native GUI | Not used (Tauri chosen, [ADR-0002](../decisions/0002-desktop-shell-tauri.md)) |
| dockview | Keyboard docking and spatial navigation are Enterprise-only | Docking UI | Not used; fixed regions with F6 cycling instead |

### Illustrative `deny.toml`

**Illustrative — created in A01.** Check key names against the cargo-deny documentation for the version pinned at the time.

```toml
# deny.toml (illustrative — created in A01)
[graph]
# Licence and ban checks run on default features. Optional copyleft features are
# checked by a separate CI job with explicit, crate-scoped exceptions.
all-features = false

[advisories]
# RustSec advisory database
yanked = "deny"
ignore = [
  # { id = "RUSTSEC-YYYY-NNNN", reason = "why, and a link to the tracking issue" },
]

[licenses]
# Anything not listed is denied (AGPL, GPL, SSPL, BUSL-1.1, LGPL in default features).
allow = [
  "MIT",
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "Zlib",
  "Unicode-3.0",
  "BSL-1.0",
  "CC0-1.0",
  "MPL-2.0",
]
confidence-threshold = 0.9

# Used only by the optional-features job, never for default features:
# [[licenses.exceptions]]
# crate = "mp3lame-encoder"
# allow = ["LGPL-3.0-only"]   # use the exact SPDX expression the crate declares

[bans]
multiple-versions = "warn"
wildcards = "deny"
deny = [
  { crate = "evalexpr", reason = "AGPL-3.0 since 12.0.0; use exmex" },
  { crate = "anomaly_detection", reason = "GPL-3.0-or-later; use augurs" },
  { crate = "oxisynth", reason = "LGPL-2.1; use rustysynth" },
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

### Adding a dependency

Before adding a Rust or npm dependency, check:

1. **Licence** passes `cargo deny check licenses` (*available after A01*) or the npm check.
2. **Maintenance:** recent releases, more than one maintainer where possible, and responsiveness to issues. The research found several single-maintainer or dormant crates; wrap those behind a facade.
3. **Placement:** the MSRV fits the crate's declared `rust-version`; core, compiler, engine, render and DSP crates stay `wasm32`-compatible; heavy or exotic dependencies go in leaf crates behind features.
4. **Features:** disable default features that are not needed.
5. **Significance:** a significant dependency (a runtime, framework or core algorithm) needs an ADR ([crate layout](../architecture/02-repository-and-crate-layout.md)).

## npm licence checking

The frontend workspace (pnpm) uses the same allowlist, plus `OFL-1.1` for font packages.

- From B01, a CI job lists production dependency licences (for example with `pnpm licenses list --prod --json`) and fails on any licence outside the allowlist. The exact tool is chosen in B01.
- Build-time-only `devDependencies` that are never shipped are checked against a wider list that also permits data licences such as `CC-BY-4.0`. They are still denied AGPL, GPL and SSPL.
- The npm licence report feeds the third-party notices.

## Model weights

Applies from D04 (ML extraction and AI assist) ([notes 06](../research/notes/06-chart-extraction-ai.md)).

- **Bundled or redistributed** models must carry Apache-2.0, MIT or a similarly permissive OSI licence. The roadmap restricts the local VLM sidecar to Apache- and MIT-licensed models.
- Models under **Gemma terms, BSL or other custom model licences are never bundled.** If supported at all, the user downloads them explicitly, and the licence is shown and accepted first.
- **GPL-licensed model code is never linked** into SONARITHM.
- Each supported model is recorded in a model manifest: name, version, source URL, licence, SHA-256 and model card link. The download is verified against the hash.
- Model use is local by default. Cloud extraction is opt-in per chart ([ADR-0012](../decisions/0012-privacy-and-telemetry.md)).

## Assets

| Asset | Licence | Obligations |
|---|---|---|
| Fonts (IBM Plex Sans and Mono) | `OFL-1.1` | Ship the licence and copyright notice with the fonts; do not sell the fonts alone; check the Reserved Font Name clause before shipping modified or subset files |
| Icons (Lucide) | `ISC`; icons derived from Feather are also `MIT` | Include both the Lucide ISC notice and the Feather MIT notice in the app's About → Licenses view and the third-party notices ([lucide.dev/license](https://lucide.dev/license)) |
| Example datasets (`examples/`) | CC BY 4.0, separately from code | A per-dataset source and attribution file; no personal data |
| Earcons and instrument samples (`assets/`) | Self-made, `CC0-1.0` or CC BY | Record source and licence for each file |
| SoundFonts (AXONOTE) | Varies by file | Check each file's licence before bundling |
| HRTF datasets (SOFA, D03) | Varies by database | Check each database's licence before bundling |
| Research stimuli | CC BY 4.0 or CC0 | Licensed separately from code ([notes 07 §5](../research/notes/07-engineering-publication.md)) |

## REUSE and SPDX headers

Adopted once ADR-0010 is accepted (A01), following the [REUSE specification](https://reuse.software/spec/):

- Source files carry `SPDX-License-Identifier` and `SPDX-FileCopyrightText` headers. The copyright-holder wording (for example "SONARITHM contributors") is fixed in ADR-0010.
- Licence texts live in `LICENSES/`. `REUSE.toml` annotates files that cannot carry headers in bulk: generated `schemas/`, golden audio, snapshots, fonts, icons and datasets.
- The root `LICENSE-MIT` and `LICENSE-APACHE` files stay for GitHub's licence detection and are annotated in `REUSE.toml`.
- `reuse lint` runs in CI.

## Third-party notices

- **Rust:** generated with `cargo-about` from the dependency graph.
- **npm:** generated from the pnpm licence report (B01 onward).
- **Fonts and icons:** the OFL texts and the Lucide ISC and Feather MIT notices.
- The combined notices ship in the desktop app (About → Licenses), in Python wheels and with CLI binaries, and are attached to each release. They are regenerated in every release (see the [release checklist](release-process.md#release-checklist)).

## SBOM, auditable binaries and provenance

| Control | Tool | Applies to | From |
|---|---|---|---|
| SBOM (CycloneDX) | `cargo-cyclonedx` for Rust; a CycloneDX tool for the frontend (chosen in B07) | CLI, Python extension, desktop app | First binary release (A07); desktop from B07 |
| Auditable binaries | `cargo auditable build` embeds the dependency tree so that cargo-audit, Trivy, Syft/Grype and osv-scanner can audit shipped binaries | CLI and desktop binaries | A07 (CLI); check integration with tauri-action in C06 |
| Build provenance | `actions/attest-build-provenance` (SLSA provenance signed with Sigstore; verify with `gh attestation verify`) | Every released binary, wheel, installer and SBOM | A07 |
| Immutable releases | GitHub repository setting | Stable releases | C06, or the first stable release |

**SLSA.** GitHub artifact attestations reach SLSA Build Level 2, and Level 3 when built with reusable workflows ([notes 07 §4](../research/notes/07-engineering-publication.md)). The target is Build L2 at the first binary release (A07) and Build L3 by v1.0 (C06), by moving release builds into a reusable workflow. SLSA v1.2's Source Track is assessed later.

## Advisories and audits

- **cargo-deny advisories** (RustSec) run on every PR and daily on `main` (`nightly.yml`), because new advisories appear without code changes. Ignoring an advisory needs an entry in `deny.toml` with a reason and a tracking issue.
- **cargo-audit** can scan released binaries built with `cargo auditable`.
- **Dependabot security alerts** cover cargo, npm, pip and github-actions.
- **cargo-vet: assess.** cargo-vet records human audits of dependencies and imports audits from other organisations. The cost and benefit for a small team is assessed during C06 hardening and recorded in an ADR.

## Dependency updates: Dependabot or Renovate

The roadmap leaves the choice open for A01. Both satisfy the Scorecard Dependency-Update-Tool check.

| | Dependabot | Renovate |
|---|---|---|
| Setup | Native to GitHub; `.github/dependabot.yml` | GitHub App or self-hosted; `renovate.json` |
| Ecosystems needed (cargo, npm, pip, github-actions with SHA pins) | Yes | Yes |
| Grouping, schedules | Groups and schedules | More configurable grouping, schedules and presets |
| Lockfile maintenance | Limited | Built in |

**Recommendation:** start with **Dependabot** in A01 for the simplest setup: grouped weekly updates per ecosystem, the `dependencies` label and the `deps` commit scope. Switch to Renovate if grouping or lockfile maintenance becomes a burden; record the switch in an ADR.

## OpenSSF Scorecard and Best Practices

Targets ([notes 07 §4](../research/notes/07-engineering-publication.md)):

| Target | When |
|---|---|
| Scorecard workflow running, badge in the README | A01 |
| Scorecard score of 7 or more | Within 3 months of the repository going public |
| OpenSSF Best Practices badge "passing" and OSPS Baseline level 1 (baseline-1) | First public release (A07) |
| OSPS Baseline level 2 | When the project has at least two maintainers |

The OSPS Baseline has 41 requirements across three maturity levels. Compliance is point-in-time and must cite a specific Baseline version.

How the main Scorecard checks are met:

| Check (risk) | Control |
|---|---|
| Dangerous-Workflow (Critical), Token-Permissions (High), Pinned-Dependencies (Medium) | [Hardening rules](ci-cd.md#hardening-rules) |
| Branch-Protection (High) | `main` ruleset ([git workflow](git-workflow.md#trunk-based-development)) |
| Code-Review (High) | One approval; limited while there is a single maintainer ([interim rule](git-workflow.md#review-rules)) |
| Dependency-Update-Tool (High) | Dependabot |
| Signed-Releases (High) | Build-provenance attestations |
| Vulnerabilities (High) | cargo-deny advisories, Dependabot alerts |
| Maintained (High) | Regular activity; triage targets in CONTRIBUTING |
| SAST (Medium) | CodeQL |
| Fuzzing (Medium) | cargo-fuzz on the spec parser and importers (A03), proptest |
| SBOM (Medium) | CycloneDX SBOM on releases |
| Security-Policy (Medium) | [SECURITY.md](../../SECURITY.md) |
| Packaging (Medium) | Publishing workflows |
| Binary-Artifacts (High) | No compiled binaries committed |
| License (Low) | Licence files after ADR-0010 |
| CII-Best-Practices (Low) | Best Practices badge |

## Secrets

- Secret scanning and **repository-level push protection** are enabled in A01. Push protection for users is on by default on GitHub, but repository-level push protection is not.
- Secrets exist only in protected environments ([CI/CD](ci-cd.md#hardening-rules)). `.gitignore` excludes common key and certificate files.

## EU Cyber Resilience Act

Not legal advice. Based on [notes 07 §4](../research/notes/07-engineering-publication.md), which flags that sources disagree on when steward obligations apply.

| Date | Milestone |
|---|---|
| 11 June 2026 | Provisions on conformity assessment bodies apply |
| 27 July 2026 | First Commission guidance published (scope, support periods, reporting) |
| 11 September 2026 | Manufacturer reporting obligations (Art. 14) apply: early warning within 24 hours, notification within 72 hours, through ENISA's Single Reporting Platform |
| 11 December 2027 | The CRA applies in full |

**Position.** SONARITHM is non-commercial and has no legal entity, so it is most likely neither a manufacturer nor an open-source software steward. A steward is a legal person that systematically supports open-source products intended for commercial activities. Companies that ship SONARITHM in commercial products become manufacturers themselves, and they will ask the project for SBOMs and a security contact.

**What SONARITHM does now** (steward-style basics, low cost):

- A documented security policy with a single point of contact and vulnerability handling ([SECURITY.md](../../SECURITY.md)).
- A support-period statement (SECURITY.md; post-1.0 policy at C06).
- SBOMs and build provenance for released binaries.
- Secure development: hardened CI, cargo-deny and CodeQL.
- An OSPS Baseline target.

**Re-assessment triggers:** a legal home (a foundation, a university or a spin-off), monetisation, or commercial distribution by the project. Any of these leads to an ADR informed by legal advice.
