# Security policy

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead (security contact).
> Related: [Supply chain and licensing](docs/engineering/supply-chain-and-licensing.md) · [CI/CD](docs/engineering/ci-cd.md) · [Release process](docs/engineering/release-process.md) · [ADR-0012 privacy and telemetry](docs/decisions/0012-privacy-and-telemetry.md) · [Governance](GOVERNANCE.md)

## Supported versions

SONARITHM has no release yet. The first release, `v0.1.0-alpha`, is planned for roadmap phase A07.

Before 1.0, only the latest minor release line receives security fixes.

| Version | Security fixes |
|---|---|
| Latest `0.x` minor (for example `0.1.x` once released) | Yes |
| Older `0.x` minors | No. Upgrade to the latest minor. |
| Pre-releases (`-alpha`, `-beta`, `-rc`) | Only the most recent pre-release of the current line |
| Nightly builds and unreleased `main` | Fixes land on `main` first; nightlies are not supported |

The post-1.0 support period will be published here at Gate C (roadmap phase C06).

## Reporting a vulnerability

**Do not report vulnerabilities in public issues, discussions or pull requests.**

Use GitHub Private Vulnerability Reporting:

1. Open the repository's **Security** tab.
2. Select **Report a vulnerability** (direct link: [new private report](https://github.com/DuhItzAniket/SONARITHM/security/advisories/new)).
3. Fill in the advisory form. Only you and the maintainers can see it.

Private vulnerability reporting must be enabled in the repository settings; this is a phase A01 task. If the link does not work yet, open a minimal public issue that says only that you need a private security channel, with no details, and the project lead will respond.

Please include:

- The affected component (crate, CLI, Python package, desktop app, workflow) and version or commit.
- OS, architecture and, where relevant, the audio backend or webview.
- Steps or a proof of concept. Attach crafted files to the private advisory, never publicly.
- The impact as you understand it.
- Whether and how you want to be credited, and any disclosure deadline you are working to.

There is no bug bounty.

## Response targets

These are targets. The project currently has one maintainer, so they are best effort.

| Step | Target |
|---|---|
| Acknowledge the report | 3 business days |
| Triage: confirm, assess severity, identify affected versions | 10 business days |
| Status updates to the reporter | At least every 14 days until resolved |
| Fix or mitigation released, advisory published: **Critical** | 14 days after triage |
| **High** | 30 days after triage |
| **Medium** | 90 days after triage |
| **Low** | Next scheduled release, at most 180 days |

Severity uses the CVSS scoring available in GitHub Security Advisories, adjusted for SONARITHM's context. For example, most file-parser issues need the user to open a crafted file.

## How a report is handled

1. Acknowledge, then reproduce and triage in the private advisory.
2. Develop the fix in the advisory's temporary private fork, or privately elsewhere if CI is needed and unavailable there.
3. Request a CVE through the GitHub advisory where warranted.
4. Release the fix (see the hotfix procedure in the [release process](docs/engineering/release-process.md#hotfix-procedure)).
5. Publish the GitHub Security Advisory. For Rust crates, also submit an advisory to the RustSec Advisory Database. Note the fix in the `Security` section of [CHANGELOG.md](CHANGELOG.md).
6. Credit the reporter as agreed.

## Coordinated disclosure

- The default embargo ends when the fix is released or 90 days after the report, whichever comes first.
- The reporter and the project can agree to extend the embargo, for example when a fix needs an upstream release.
- If a vulnerability is actively exploited or already public, the project may publish an advisory with mitigations before a fix is ready.
- Please do not disclose details before the advisory is published or the embargo ends.

## Scope

In scope: code in this repository and the artifacts the project releases.

| Area | Examples of in-scope issues |
|---|---|
| File parsers and importers: CSV, TSV, JSON/NDJSON, Parquet, Arrow IPC, XLSX/ODS (`sonarithm-data`); chart inputs such as Office chart XML, SVG, PDF and raster images (PLOTSONA); spec JSON and the Erie and Vega-Lite importers (`sonarithm-spec`); `.sonarithm` project bundles | Memory corruption, code execution, path traversal when unpacking bundles, writes outside the chosen output location, resource exhaustion from small inputs (decompression bombs) |
| Expression and script evaluation (exmex facade, rhai) | Sandbox escape, unbounded CPU or memory despite configured limits |
| Desktop IPC surface: Tauri commands, capabilities, channels, webview content | Untrusted content (a data file, spec or SVG chart) executing script in the webview; reaching commands that the capability configuration does not grant |
| Plugin sandbox (`sonarithm-plugin-host`, wasmtime with WIT, Stage D) | Escaping the sandbox, abusing host calls, bypassing fuel or epoch limits, running plugin code outside compile time |
| Updater (Tauri updater, `latest.json`, update signatures) | Installing an update that is unsigned, signed with the wrong key, or a downgrade |
| Network behaviour | Any network request made without the user's explicit consent ([principle P8](docs/architecture/01-overview.md#2-architectural-principles)) |
| Live sources (`sonarithm-stream`: WebSocket, MQTT, serial, OSC) | Crafted network or device input causing memory corruption or unbounded resource use |
| Python bindings (`sonarithm-py`) | Memory-safety errors across the FFI and Arrow C Data Interface boundary |
| ML model loading (Stage D) | Crafted model files causing code execution |
| Release pipeline: workflows, signing, provenance | Workflow injection, excessive token permissions, tampering with release artifacts or attestations |
| Hearing safety | A crafted spec or data file that bypasses the start-up ramp, the master limiter or the output volume control ([audio engine §10](docs/architecture/06-audio-engine.md#10-hearing-safety-and-levels)) |

Out of scope:

- Vulnerabilities in dependencies with no demonstrated impact on SONARITHM. Report those upstream; the project tracks advisories through cargo-deny, RustSec and Dependabot.
- Attacks that need an already-compromised machine or account, or physical access.
- Social engineering of maintainers or users.
- Issues in GitHub's platform. Report those to GitHub.
- Throwaway code in `spikes/` (phase A02) that is never shipped.
- Slow processing that is proportional to an input's size (a very large CSV is slow; a small file that expands without bound is in scope).
- A plain panic on malformed input with no further impact. That is a bug; report it publicly.
- Automated scanner output without a demonstrated impact.

## Safe harbour

If you research and report in good faith under this policy, the project considers your research authorised, will not pursue or support legal action against you for it, and will work with you to understand and fix the issue. Good faith means:

- Test only on your own installations and data.
- Do not access, modify or keep other people's data.
- Do not degrade other people's systems. SONARITHM runs no hosted services; GitHub's infrastructure is covered by GitHub's own policies.
- Give the project reasonable time to fix the issue before disclosure, as described above.

This statement cannot bind third parties, and it is not legal advice.

## Security-relevant design

| Property | Status |
|---|---|
| **No network by default.** No network access without explicit, per-action consent and no default telemetry ([P8](docs/architecture/01-overview.md#2-architectural-principles), [ADR-0012](docs/decisions/0012-privacy-and-telemetry.md)). A network-deny integration test enforces this. | Planned, test in B01 |
| **Consent-gated update check.** Update checks run only after the user's explicit consent, stay visible in settings and can be turned off ([ADR-0012](docs/decisions/0012-privacy-and-telemetry.md)). | Planned, C06 |
| **Signed updates.** Tauri updater signature verification cannot be disabled. The updater private key has two custodians and an offline backup ([CI/CD](docs/engineering/ci-cd.md#tauri-updater-key-custody)). | Planned, C06 |
| **Capability-scoped IPC.** The webview can call only the Tauri commands its capabilities grant. | Planned, B01 |
| **I/O-free core.** `sonarithm-core` depends on no file-system, network, async-runtime or audio-device crate ([crate layout §3](docs/architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced)). | Enforced by `cargo xtask check-deps` from A01 |
| **Compile-time-only plugins.** Plugins run sandboxed with fuel and epoch limits, never in the audio callback. | Planned, D05 |
| **Supply chain.** cargo-deny (licences, advisories, bans, sources), SHA-pinned actions, least-privilege workflow tokens, trusted publishing, SBOMs, build-provenance attestations and immutable stable releases ([supply chain](docs/engineering/supply-chain-and-licensing.md)). | CI checks from A01; release controls from A07 and C06 |
| **Secrets.** Secret scanning with repository-level push protection, which is not on by default and must be enabled. | A01 |

## EU Cyber Resilience Act

This section records the project's position. It is not legal advice.

- **Dates.** The CRA's reporting obligations for manufacturers apply from 11 September 2026, and the Act applies in full from 11 December 2027 ([research notes 07 §4](docs/research/notes/07-engineering-publication.md)).
- **Position.** SONARITHM is a non-commercial open-source project with no legal entity. It is most likely neither a "manufacturer" nor an "open-source software steward", because a steward must be a legal person that systematically supports open-source products intended for commercial activities ([OpenSSF stewards one-pager](https://policy.openssf.org/CRA/stewards-one-pager.html)).
- **What the project does anyway.** It follows the steward-style basics, because they cost little and downstream manufacturers will ask for them: this documented policy, a single point of contact (private vulnerability reporting), documented vulnerability handling, a support-period statement, SBOMs and provenance for released binaries, and the OSPS Baseline ([supply chain](docs/engineering/supply-chain-and-licensing.md#eu-cyber-resilience-act)).
- **Re-assessment.** If SONARITHM gains a legal home, is monetised, or is distributed commercially by the project, the position is re-assessed through an ADR, with legal advice.
