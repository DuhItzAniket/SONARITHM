# Engineering, Release & Governance Practices and the Research-Publication Pathway for SONARITHM (as of 18 October 2026)

How to read these notes. Every finding has an inline source. A source marked **(nf)** is a canonical or official URL I cited from established knowledge but did not re-fetch in this session (October 2026). Treat (nf) items as "very likely still correct, verify before quoting". Unmarked sources were fetched or seen in search results during this session. Deadlines marked **[EST]** are estimates based on past cycles, not announced dates.

---

## 1. Documentation and process conventions (Diátaxis, ADRs/MADR, RFCs, Conventional Commits, SemVer + cargo-semver-checks, changelog/release automation, branching, CODEOWNERS, issue forms/PR templates, DoR/DoD, GitHub Projects)

### Takeaway
The 2026 Rust-ecosystem default stack is: Diátaxis-structured docs, MADR-format ADRs in `docs/decisions/`, a lightweight Rust-style RFC folder for big cross-cutting changes, Conventional Commits 1.0.0 driving **release-plz** (which wraps git-cliff for Keep-a-Changelog output and cargo-semver-checks for API-break detection), trunk-based development with short-lived branches and squash merges, and GitHub-native governance files (CODEOWNERS, YAML issue forms, PR template, Projects roadmap view).

### Cited Findings
**Documentation (Diátaxis)**
- Diátaxis splits documentation into four distinct kinds by user need: tutorials (learning-oriented), how-to guides (task-oriented), reference (information-oriented), and explanation (understanding-oriented). It is a framework, not a tool, and is meant to be applied iteratively — [diataxis.fr](https://diataxis.fr/) (nf)

**Architecture Decision Records**
- The ADR community hub lists templates and tooling (MADR, Nygard format, adr-tools, log4brains, etc.) — [adr.github.io](https://adr.github.io/) (nf)
- MADR's current major line is 4.0.0 (tagged in `adr/madr`); adopting projects describe it as a lean Markdown template that captures any decision (not only architecture), with **optional** sections "Decision Drivers", "Pros and Cons of the Options", "Confirmation", and "More Information" so short decisions stay short — [pyvista-wasm ADR-0000](https://pyvista-wasm.readthedocs.io/en/latest/decisions/0000-use-markdown-architectural-decision-records.html); [JabRef ADR-0000](https://devdocs.jabref.org/decisions/0000-use-markdown-architectural-decision-records.html); template home [adr.github.io/madr](https://adr.github.io/madr/) (nf)
- MADR 4.x uses YAML front matter for lifecycle metadata (`status`, `date`, `decision-makers`, `consulted`, `informed`) and files are conventionally stored as `docs/decisions/NNNN-title-with-dashes.md`; status values used in practice: proposed / accepted / rejected / deprecated / superseded — [SAIF Forge ADR-0000](https://docs.platformdev.saif.com/forge/decisions/0000-use-markdown-architectural-decision-records/); [capturing-decisions skill description](https://smithery.ai/skills/claylo/capturing-decisions)
- Tooling: `adr-tools` (shell, Nygard-format, `adr new …`) — [github.com/npryce/adr-tools](https://github.com/npryce/adr-tools) (nf); log4brains (ADR management + static-site knowledge base, supports MADR) — [github.com/thomvaill/log4brains](https://github.com/thomvaill/log4brains) (nf)

**RFCs**
- The Rust RFC process: a Markdown file from `0000-template.md` submitted as a PR to `rust-lang/rfcs`, discussed, then entering Final Comment Period (FCP) before merge/close — [github.com/rust-lang/rfcs](https://github.com/rust-lang/rfcs) (nf). Example of a recent RFC in rendered-book form: RFC 3691 trusted publishing — [rust-lang.github.io/rfcs/3691](https://rust-lang.github.io/rfcs/3691-trusted-publishing-cratesio.html)

**Commits, versioning, changelogs, release automation**
- Conventional Commits 1.0.0: `type(scope)!: description`, with `feat` → MINOR, `fix` → PATCH, `BREAKING CHANGE:` footer or `!` → MAJOR — [conventionalcommits.org/en/v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) (nf)
- SemVer 2.0.0 (MAJOR.MINOR.PATCH; 0.y.z is "anything may change") — [semver.org](https://semver.org/) (nf); Cargo applies its own caret rules where for 0.x the *minor* number is treated as breaking — [Cargo SemVer compatibility guide](https://doc.rust-lang.org/cargo/reference/semver.html) (nf)
- cargo-semver-checks lints a crate's public API against the last published version to detect SemVer violations — [github.com/obi1kenobi/cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) (nf)
- **release-plz**: for every push to main, CI opens/updates a "release PR" listing unpublished changes; merging it releases the packages. It generates changelogs with **git-cliff** in **Keep a Changelog** format by default, integrates **cargo-semver-checks**, supports Cargo workspaces, bumps versions from Conventional Commits, creates per-package git tags and GitHub/Gitea/GitLab releases, publishes to any Cargo registry, and needs no config to start — [release-plz.dev/docs](https://release-plz.dev/docs)
- git-cliff: changelog generator driven by `cliff.toml` templates over Conventional Commits — [git-cliff.org](https://git-cliff.org/) (nf). Keep a Changelog 1.1.0 sections: Added / Changed / Deprecated / Removed / Fixed / Security, plus an "Unreleased" section — [keepachangelog.com/en/1.1.0](https://keepachangelog.com/en/1.1.0/) (nf)
- Changesets is a JS/monorepo-oriented alternative (per-PR "change files" instead of commit parsing) — [github.com/changesets/changesets](https://github.com/changesets/changesets) (nf). Tauri itself uses its own change-file tool, covector — [github.com/tauri-apps/covector](https://github.com/tauri-apps/covector) (nf)

**Branching**
- Trunk-based development: everyone integrates to a single trunk with short-lived feature branches, relying on CI and feature flags — [trunkbaseddevelopment.com](https://trunkbaseddevelopment.com/) (nf)
- GitFlow's author added a 2020 "note of reflection" saying GitFlow is not the best fit for continuously delivered software (e.g., web apps) and suggesting simpler workflows such as GitHub flow — [nvie.com: A successful Git branching model](https://nvie.com/posts/a-successful-git-branching-model/) (nf)

**GitHub governance files**
- `CODEOWNERS` (in `.github/`, root, or `docs/`) auto-requests reviews from owners of matching paths; with branch protection/rulesets it can require code-owner approval — [GitHub Docs: About code owners](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners) (nf)
- Issue forms are YAML files in `.github/ISSUE_TEMPLATE/*.yml` with typed inputs (`input`, `textarea`, `dropdown`, `checkboxes`, `markdown`) and `validations: required`; `config.yml` can disable blank issues and add contact links; PR template lives at `.github/pull_request_template.md` — [GitHub Docs: Syntax for issue forms](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms) (nf)
- GitHub Projects supports table, board and **roadmap** layouts with date/iteration fields for timelines — [GitHub Docs: Customizing the roadmap layout](https://docs.github.com/en/issues/planning-and-tracking-with-projects/customizing-views-in-your-project/customizing-the-roadmap-layout) (nf)
- The Scrum Guide 2020 defines the **Definition of Done** as a formal commitment for the Increment; "Definition of Ready" is not part of the Scrum Guide (it is a team practice) — [scrumguides.org](https://scrumguides.org/scrum-guide.html) (nf)

**Exemplar: how a top Rust OSS project ships (astral-sh/uv)**
- uv's release workflow header says it was "Originally generated by cargo-dist" and is "now maintained manually"; it runs only on `workflow_dispatch` with a `dry-run` input, has a `release-gate` job using a protected environment requiring a second approval, pins all actions to commit SHAs, sets top-level `permissions: {}` with per-job least privilege, runs `dist plan`/`dist host`, signs artifacts in a reusable workflow, generates build-provenance attestations with `actions/attest-build-provenance`, and publishes to PyPI, crates.io (jobs request `id-token: write`), Docker and GitHub Releases — [uv release.yml](https://github.com/astral-sh/uv/blob/main/.github/workflows/release.yml)

### Inferences
- For a multi-crate workspace with a desktop app and Python bindings, release-plz is the lowest-friction choice for the *library crates and CLI* (crates.io + changelog + semver checks), while the desktop app and wheels need their own tag-triggered workflows (tauri-action, maturin-action). Configure release-plz to not publish the app crate (`publish = false`).
- Use MADR 4.x for ADRs (one decision per file) and reserve an `rfcs/` folder (Rust-style template + FCP-like 7-day comment window) for cross-cutting changes such as the sonification mapping DSL, plugin ABI, or file formats. The RFC/ADR distinction: RFC = proposal under discussion; ADR = the recorded outcome.
- Map Diátaxis to SONARITHM: Tutorials ("sonify your first CSV"), How-to (e.g., "export to WAV with loudness normalization"), Reference (CLI flags, Rust API via rustdoc, Python API), Explanation (perceptual mapping theory, design rationale). Research-paper material naturally lives in "Explanation".
- Trunk-based + squash-merge + Conventional-Commit PR titles (enforced by a PR-title lint action) gives a clean history that release-plz can parse; avoid GitFlow's long-lived `develop` branch.

### Gaps
- I did not fetch the MADR 4.0.0 template text itself or confirm whether a MADR 4.1/5.0 exists as of Oct 2026; I also did not confirm log4brains' current maintenance status.
- release-plz's exact handling of crates.io Trusted Publishing and its current action version (`release-plz/action@v0.5`?) were not confirmed from its docs page.

---

## 2. AGENTS.md — stewardship, agent support, nesting, sections, relation to CLAUDE.md

### Takeaway
AGENTS.md is a plain-Markdown, no-required-fields instruction file for coding agents, now stewarded by the **Agentic AI Foundation under the Linux Foundation**; it is read by Codex, Jules, Cursor, Copilot's coding agent, Gemini CLI (via config), Aider (via config), Zed, Windsurf, Devin, Junie and others, and — since **Claude Code v2.1.277 (Sept 2026)** — natively by Claude Code when no CLAUDE.md is present. Nearest file wins; for Claude Code, keep a `CLAUDE.md` containing `@AGENTS.md` (not a symlink, because of Windows contributors).

### Cited Findings
- Stewardship: agents.md states the format is "now stewarded by the Agentic AI Foundation under the Linux Foundation"; copyright credits "AGENTS.md a Series of LF Projects, LLC"; it emerged from collaboration among OpenAI Codex, Amp, Google Jules, Cursor and Factory — [agents.md](https://agents.md/)
- Supported tools listed on agents.md (Oct 2026): Codex (OpenAI), Jules (Google), Factory, Aider, goose, opencode, Zed, Warp, VS Code, Devin (Cognition), UiPath Autopilot & Coded Agents, Junie (JetBrains), Amp, Cursor, RooCode, Gemini CLI, Kilo Code, Phoenix, Semgrep, GitHub Copilot coding agent, Ona, Windsurf (Cognition), Augment Code; it claims use by "over 60k open-source projects" and notes the OpenAI main repo had 88 AGENTS.md files — [agents.md](https://agents.md/)
- Configuration for tools that don't auto-detect: Aider uses `read: AGENTS.md` in `.aider.conf.yml`; Gemini CLI sets `context.fileName` to `AGENTS.md` in `.gemini/settings.json` — [agents.md](https://agents.md/)
- Nesting: in monorepos place an AGENTS.md in each subproject; "Agents automatically read the nearest file in the directory tree, so the closest one takes precedence"; FAQ: "The closest AGENTS.md to the edited file wins; explicit user chat prompts override everything" — [agents.md](https://agents.md/)
- Recommended sections: project overview; build and test commands; code style guidelines; testing instructions; security considerations; plus commit/PR guidelines, security gotchas, large datasets, deployment steps. No required fields ("just standard Markdown"). If test commands are listed, agents "will attempt to execute relevant programmatic checks and fix failures before finishing the task." Migration advice: rename to AGENTS.md and symlink old names — [agents.md](https://agents.md/)
- **Claude Code (official docs):** "Claude Code can read AGENTS.md as your project instructions… without adding a CLAUDE.md, an import, or a setting." Default table: AGENTS.md with no CLAUDE.md/CLAUDE.local.md in cwd or above → reads AGENTS.md; AGENTS.md **and** a CLAUDE.md/CLAUDE.local.md → reads CLAUDE.md files only; a CLAUDE.md that imports AGENTS.md → both via import. "Reading AGENTS.md directly requires Claude Code v2.1.277 or later." — [code.claude.com/docs/en/memory](https://code.claude.com/docs/en/memory)
- Claude Code loading details: at session start it reads every `AGENTS.md` and `.claude/AGENTS.md` in the working directory and above; subdirectory AGENTS.md files load on demand when Claude reads a file there (if that subdirectory has no CLAUDE.md); `@path` imports inside AGENTS.md are expanded; it does **not** read `AGENTS.local.md`, `AGENTS.override.md`, or anything under `.agents/` — [code.claude.com/docs/en/memory](https://code.claude.com/docs/en/memory)
- A "Project instructions" setting (`/config`) accepts `claude-md-or-agents-md` (default), `claude-md-and-agents-md`, `claude-md`, `managed-only`; it is implemented by a built-in plugin `cc-plugin-agents-md@builtin`. Gotcha: adding a personal `CLAUDE.local.md` stops AGENTS.md from loading under the default — [code.claude.com/docs/en/memory](https://code.claude.com/docs/en/memory)
- Recommended sharing pattern: a `CLAUDE.md` whose first line is `@AGENTS.md`, with Claude-specific notes below; keeping the import "never makes Claude read AGENTS.md twice." The docs advise **against a symlink if anyone works on Windows** (symlink creation needs Administrator/Developer Mode and Git checks symlinks out as plain text unless `core.symlinks` is enabled) — [code.claude.com/docs/en/memory](https://code.claude.com/docs/en/memory)
- Some sessions can't load AGENTS.md natively (pre-2.1.277, plugin disabled, first session after upgrade; before v2.1.281 also Bedrock/telemetry-disabled sessions) — hence the `@AGENTS.md` import remains the robust choice — [code.claude.com/docs/en/memory](https://code.claude.com/docs/en/memory)
- `/doctor prompt-audit` (Claude Code v2.1.283+) audits CLAUDE.md, CLAUDE.local.md and AGENTS.md files for stale or conflicting instructions — [code.claude.com/docs/en/memory](https://code.claude.com/docs/en/memory)
- Secondary coverage of the Sept 2026 change (Claude Code v2.1.277 released 18 Sept 2026 per one write-up) — [devops.com](https://devops.com/claude-code-adds-agents-md-fallback-cutting-instruction-file-sprawl); older (mid-2026) articles claiming Claude Code ignores AGENTS.md are now outdated — [gist (outdated claim)](https://gist.github.com/yurukusa/d36197848911f025add142abefcde685)

### Inferences
- SONARITHM layout: root `AGENTS.md` (overview, workspace map, `cargo xtask`/`just` commands, MSRV, test/lint commands, commit conventions, "never edit golden audio files without regenerating via X", security notes), plus nested `crates/sonarithm-core/AGENTS.md`, `apps/desktop/AGENTS.md` (Tauri/frontend commands), `bindings/python/AGENTS.md` (maturin dev, pytest). Root `CLAUDE.md` = `@AGENTS.md` + Claude-only notes. Add `CLAUDE.local.md` to `.gitignore` but document the gotcha.
- Keep AGENTS.md short and command-centric; long prose belongs in Diátaxis docs that AGENTS.md links to.
- Given the project plans to publish papers, add an AGENTS.md "AI usage" note pointing contributors to the project's AI-disclosure policy (JOSS now requires an AI usage disclosure — see §6).

### Gaps
- agents.md does not list Claude Code among supporting tools as of my fetch, even though Claude Code's own docs confirm support; the list may lag.
- I did not verify per-tool nesting semantics for every agent (e.g., whether Copilot/Cursor merge parent+child files or take only the nearest).

---

## 3. CI/CD for a Rust workspace + Tauri desktop app (Windows/macOS/Linux), wheels, wasm, signing, auto-update, reproducibility

### Takeaway
Use a GitHub Actions matrix (ubuntu/windows/macos) with `Swatinem/rust-cache`, separate workflows for (a) PR checks, (b) library/CLI release via release-plz (+ dist for CLI binaries), (c) desktop release via `tauri-apps/tauri-action`, and (d) Python wheels via `PyO3/maturin-action`. Windows signing is now realistically **Azure Artifact Signing** (renamed from Trusted Signing, GA Jan 2026, ~$9.99/month) — EV certs no longer buy instant SmartScreen reputation; macOS needs a Developer ID Application certificate + notarization (App Store Connect API key preferred in CI); Linux AppImages can be GPG-signed but signatures aren't auto-verified; Tauri updater signatures are mandatory and keys must never be lost.

### Cited Findings
**Build/caching/packaging tools**
- `Swatinem/rust-cache` (v2) caches `~/.cargo` and `target/` keyed on toolchain + lockfile — [github.com/Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) (nf); `sccache` is Mozilla's compiler cache with local/S3/GCS/GHA backends — [github.com/mozilla/sccache](https://github.com/mozilla/sccache) (nf)
- `tauri-apps/tauri-action` builds Tauri bundles per OS and can create GitHub Releases and generate the updater's static `latest.json` — [github.com/tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) (nf); Tauri docs confirm "Tauri Action can generate the static JSON for you" and an endpoint like `https://github.com/user/repo/releases/latest/download/latest.json` — [v2.tauri.app/plugin/updater](https://v2.tauri.app/plugin/updater/)
- **dist (formerly cargo-dist)** is actively shipping: v0.33.0 released 2026-09-10, adding **Azure Artifact Signing support for Windows binaries and installers**; repo activity through 14 Oct 2026 — [cargo-dist releases](https://github.com/axodotdev/cargo-dist/releases); [v0.33.0-prerelease.1 notes](https://github.com/axodotdev/cargo-dist/releases/tag/v0.33.0-prerelease.1). Caveat: many open issues lack maintainer responses and at least one downstream team publicly questioned whether betting on dist maintenance is safe — [vig-os/devkit issue #1774](https://github.com/vig-os/devkit/issues/1774)
- uv originally generated its release workflow with cargo-dist and now maintains it manually (still invoking `dist`) — [uv release.yml](https://github.com/astral-sh/uv/blob/main/.github/workflows/release.yml)
- `PyO3/maturin-action` builds manylinux/musllinux/macOS/Windows wheels; `maturin generate-ci github` scaffolds a workflow — [github.com/PyO3/maturin-action](https://github.com/PyO3/maturin-action) (nf); `abi3` (stable ABI) wheels let one wheel per platform cover many CPython versions — [PyO3 guide: building and distribution](https://pyo3.rs/latest/building-and-distribution.html) (nf)
- **crates.io Trusted Publishing** (launched July 2025): crates.io issues short-lived tokens via OIDC; the job needs `permissions: id-token: write` and calls `rust-lang/crates-io-auth-action@v1` before `cargo publish`; the first release must be published manually; config per crate = owner, repo, workflow filename, optional environment — [crates.io development update, Jul 2025](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07); [rust-lang/crates-io-auth-action](https://github.com/rust-lang/crates-io-auth-action); [Rust forge: trusted publishing](https://forge.rust-lang.org/infra/docs/trusted-publishing.html)
- **wasm**: the Rust project archived the Rust & WebAssembly WG and announced sunsetting the `rustwasm` GitHub org (full archive planned Sept 2025); wasm-bindgen moved to a new org with new maintainers; other repos archived or transferred — [Inside Rust blog, 21 Jul 2025](https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org). wasm-pack's status is reported inconsistently (transferred to a personal maintainer vs. archived) — [users.rust-lang.org thread](https://users.rust-lang.org/t/future-of-rust-wasm/133089)

**Windows signing**
- Microsoft renamed **Trusted Signing → Artifact Signing** and declared it **GA (blog dated 12 Jan 2026)**; existing accounts/profiles/APIs unchanged — [Microsoft Security blog](https://techcommunity.microsoft.com/blog/microsoft-security-blog/-/4482789); [Azure Artifact Signing product page](https://azure.microsoft.com/en-us/products/artifact-signing)
- Availability: organizations or individuals in the USA and Canada, or organizations in the EU or UK; pricing $9.99/month (5,000 signatures, 1 certificate profile) or $99.99/month (100,000 signatures, 10 profiles) — [DevClass, 14 Jan 2026](https://devclass.com/2026/01/14/code-signing-windows-apps-may-be-easier-and-more-secure-with-new-azure-artifact-service/)
- Tauri v2 Windows signing options: OV cert via `bundle.windows.certificateThumbprint` / `digestAlgorithm` / `timestampUrl`; Azure Key Vault via `relic` in `signCommand`; **Azure Artifact Signing** via `cargo install artifact-signing-cli` and `"signCommand": "artifact-signing-cli -e https://<region>.codesigning.azure.net -a <Account> -c <Profile> -d <AppName> %1"` with `AZURE_CLIENT_ID/SECRET/TENANT_ID` env vars; cross-compiling Windows installers from Linux/macOS requires a custom sign command — [v2.tauri.app/distribute/sign/windows](https://v2.tauri.app/distribute/sign/windows/)
- "Since 2024, EV certificates no longer grant instant SmartScreen reputation"; OV and EV both build reputation over time; sign every release with the same certificate — [v2.tauri.app/distribute/sign/windows](https://v2.tauri.app/distribute/sign/windows/)

**macOS signing/notarization**
- Requires paid Apple Developer Program (US$99/yr) for notarization; use a **Developer ID Application** certificate (only the Account Holder can create it) for outside-App-Store distribution; notarization is required with Developer ID. CI env vars: `APPLE_CERTIFICATE` (base64 .p12), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `KEYCHAIN_PASSWORD`; notarization via App Store Connect API (`APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`) or Apple ID (`APPLE_ID`, app-specific `APPLE_PASSWORD`, `APPLE_TEAM_ID`); ad-hoc signing with `signingIdentity: "-"` — [v2.tauri.app/distribute/sign/macos](https://v2.tauri.app/distribute/sign/macos/)
- Apple moved notarization to `notarytool` (altool/Xcode 13-era notarization retired in Nov 2023) — [Apple TN3147: Migrating to the latest notarization tool](https://developer.apple.com/documentation/technotes/tn3147-migrating-to-the-latest-notarization-tool) (nf)

**Linux signing**
- Tauri documents only AppImage signing: GPG key + `SIGN=1`, optional `SIGN_KEY`, `APPIMAGETOOL_SIGN_PASSPHRASE`, `APPIMAGETOOL_FORCE_SIGN=1` (fail build on signing error); inspect with `--appimage-signature`; validate with `validate-$PLATFORM.AppImage`. "The signature is not verified automatically" — publish the key ID via an authenticated channel. deb/rpm/Flatpak/Snap signing are not covered — [v2.tauri.app/distribute/sign/linux](https://v2.tauri.app/distribute/sign/linux/)
- Flatpak distribution via Flathub uses Flathub's own build infrastructure and repo signing (apps submit a manifest) — [docs.flathub.org](https://docs.flathub.org/docs/for-app-authors/submission) (nf)

**Auto-update signing (Tauri updater)**
- Generate keys with `tauri signer generate -w ~/.tauri/app.key`; put the **public key content** in `plugins.updater.pubkey`; set `TAURI_SIGNING_PRIVATE_KEY` (+ optional `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`) at build (`.env` not supported); `bundle.createUpdaterArtifacts: true` produces `.sig` files (`"v1Compatible"` slated for removal in v3). Signature verification "cannot be disabled". Losing the private key means you can't ship updates to existing installs. `latest.json` needs `version`, `platforms.<os-arch>.url`, `platforms.<os-arch>.signature` (the .sig *content*), and every listed platform must be complete — [v2.tauri.app/plugin/updater](https://v2.tauri.app/plugin/updater/)

**Reproducible builds**
- Reproducible-builds definition: identical bit-for-bit output from the same source, environment and instructions; `SOURCE_DATE_EPOCH` is the standard timestamp convention — [reproducible-builds.org/docs/definition](https://reproducible-builds.org/docs/definition/) (nf); [SOURCE_DATE_EPOCH spec](https://reproducible-builds.org/docs/source-date-epoch/) (nf)
- Rust levers: `rust-toolchain.toml` pinning, committed `Cargo.lock` + `--locked`, and `--remap-path-prefix` to strip absolute paths — [rustc docs: remap-path-prefix](https://doc.rust-lang.org/rustc/command-line-arguments.html#--remap-path-prefix-remap-source-names-in-output) (nf)

### Inferences
- Recommended workflow split: `ci.yml` (PRs: fmt, clippy `-D warnings`, `cargo test --workspace` / `cargo nextest`, `cargo deny check`, docs build, MSRV job, Python `maturin develop && pytest`, wasm target `cargo build --target wasm32-unknown-unknown -p sonarithm-core`); `release-plz.yml` (release PR + crates.io via trusted publishing); `release-desktop.yml` (on `app-v*` tags: tauri-action matrix `macos-latest` [aarch64 + x86_64 or universal], `windows-latest`, `ubuntu-22.04`, signing secrets, updater JSON, attestations); `wheels.yml` (maturin-action matrix → PyPI trusted publishing); optional `dist` for standalone CLI installers (shell/PowerShell installers, Homebrew).
- Because dist's long-term maintenance is uncertain and uv itself forked its generated workflow, treat dist as a *generator* you can eject from, not a hard dependency.
- For wasm, depend directly on `wasm-bindgen`/`wasm-bindgen-cli` rather than wasm-pack given the rustwasm sunset ambiguity.
- For an early-stage academic project without a legal entity, Artifact Signing's eligibility (individuals only in US/Canada; organizations in EU/UK) may block an individual maintainer based outside the US/Canada; alternatives are an OV certificate on a cloud HSM, SignPath Foundation's free OSS signing program (nf: [signpath.org](https://signpath.org/)), or shipping unsigned initially with documented SmartScreen workaround.

### Gaps
- Current exact versions of `tauri-apps/tauri-action`, `Swatinem/rust-cache`, `PyO3/maturin-action`, `actions/attest-build-provenance` were not verified this session; pin by commit SHA anyway.
- GitHub-hosted runner image availability (e.g., Intel macOS images) changes frequently and was not verified.
- No confirmed official guidance found for Debian/RPM repository signing inside Tauri's docs.

---

## 4. Supply-chain security (Scorecard, Best Practices/Baseline badges, SLSA + attestations, SBOM, cargo-deny/audit/vet, Dependabot vs Renovate, secret scanning, SECURITY.md/PVR, EU CRA)

### Takeaway
A credible 2026 baseline is: OpenSSF Scorecard action + Best Practices badge (now with a three-level **OSPS Baseline** series), SLSA v1.2 Build-track provenance via GitHub artifact attestations and **immutable releases** (GA Oct 2025), CycloneDX SBOMs plus `cargo-auditable` binaries, `cargo-deny` in CI (licenses, advisories, bans, sources), Dependabot or Renovate, push protection, and a SECURITY.md with GitHub private vulnerability reporting. The EU CRA's reporting obligations started **11 Sept 2026**, full application **11 Dec 2027**; a non-commercial academic project is generally not a "manufacturer", but stewards (legal persons supporting OSS intended for commercial use) get a light-touch regime.

### Cited Findings
**OpenSSF**
- Scorecard checks and risk levels: Binary-Artifacts (High), Branch-Protection (High), CI-Tests (Low), CII-Best-Practices (Low), Code-Review (High), Contributors (Low), Dangerous-Workflow (Critical), Dependency-Update-Tool (High; Dependabot or Renovate), Fuzzing (Medium; OSS-Fuzz/ClusterFuzzLite/property-based), License (Low), Maintained (High), Packaging (Medium), Pinned-Dependencies (Medium), SAST (Medium; e.g., CodeQL), SBOM (Medium), Security-Policy (Medium), Signed-Releases (High; signatures or SLSA provenance), Token-Permissions (High), Vulnerabilities (High; via OSV), Webhooks (Critical) — [ossf/scorecard checks.md](https://github.com/ossf/scorecard/blob/main/docs/checks.md)
- OSPS Baseline: 41 security requirements across three maturity levels and six lifecycle stages; the site designates a "current" version (v2026.02.19 listed as current; a 2026.08.28 checklist also exists) and says compliance should cite a specific version and is point-in-time — [baseline.openssf.org](https://baseline.openssf.org/); [2026.08.28 checklist](https://baseline.openssf.org/versions/2026-08-28-checklist.md); [OpenSSF guide, Jan 2026](https://openssf.org/blog/2026/01/07/your-guide-to-the-openssf-osps-baseline-for-more-secure-open-source-projects/)
- Level targeting: L1 any project; L2 ≥2 maintainers and small user base; L3 large user base — [InfoQ, Mar 2025](https://www.infoq.com/news/2025/03/openssf-security-baseline)
- The OpenSSF Best Practices Badge program now offers a "baseline" series (baseline-1/2/3, MUST-only security criteria) alongside the "metal" series (passing/silver/gold) — [OpenSSF blog, 25 Feb 2026](https://openssf.org/blog/2026/02/25/getting-an-openssf-baseline-badge-with-the-best-practices-badge); [bestpractices.dev example](https://www.bestpractices.dev/projects/1/baseline-1)

**SLSA / attestations / immutable releases**
- SLSA v1.2 (Nov 2025) adds a **Source Track** alongside the Build Track, backwards compatible with v1.1; Build Environment and Dependency tracks are in development — [slsa.dev announcement](https://slsa.dev/blog/2025/11/announce-slsa-v1.2); [What's new in v1.2](https://slsa.dev/spec/v1.2/whats-new)
- GitHub artifact attestations (`actions/attest-build-provenance`) produce Sigstore-signed SLSA provenance verifiable with `gh attestation verify`; GitHub states this reaches SLSA Build L2, and L3 when built with reusable workflows — [GitHub Docs: artifact attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds) (nf)
- **Immutable releases GA (28 Oct 2025):** assets can't be added/modified/deleted after publishing, tags are protected, and releases receive signed (Sigstore-bundle) release attestations; enable at repo or org level; existing releases stay mutable. Community reports: it is all-or-nothing (awkward for rolling pre-releases) and manual asset uploads can break the attestation — [GitHub changelog](https://github.blog/changelog/2025-10-28-immutable-releases-are-now-generally-available/); [GitHub Docs: immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases); [community discussion #178351](https://github.com/orgs/community/discussions/178351)
- uv attests JSON/shell/PowerShell/zip/tar.gz release artifacts with `actions/attest-build-provenance` — [uv release.yml](https://github.com/astral-sh/uv/blob/main/.github/workflows/release.yml)

**Rust SBOM and dependency tooling**
- `cargo-cyclonedx` generates CycloneDX SBOMs per crate — [github.com/CycloneDX/cyclonedx-rust-cargo](https://github.com/CycloneDX/cyclonedx-rust-cargo) (nf); `cargo-auditable` embeds the dependency tree into the compiled binary so scanners (cargo-audit, Trivy, Syft/Grype, osv-scanner) can audit shipped binaries — [github.com/rust-secure-code/cargo-auditable](https://github.com/rust-secure-code/cargo-auditable) (nf)
- `cargo-deny` checks advisories, licenses, bans (duplicates/forbidden crates) and sources via `deny.toml` — [github.com/EmbarkStudios/cargo-deny](https://github.com/EmbarkStudios/cargo-deny) (nf); `cargo-audit` checks `Cargo.lock` against the RustSec Advisory DB — [rustsec.org](https://rustsec.org/) (nf); `cargo-vet` (Mozilla) records human audits of dependencies and imports audits from other orgs — [mozilla.github.io/cargo-vet](https://mozilla.github.io/cargo-vet/) (nf)
- Dependabot (native, `.github/dependabot.yml`, supports cargo/pip/npm/github-actions) vs Renovate (more configurable grouping, schedules, lockfile maintenance, presets) — both satisfy Scorecard's Dependency-Update-Tool — [GitHub Docs: Dependabot options](https://docs.github.com/en/code-security/dependabot/working-with-dependabot/dependabot-options-reference) (nf); [docs.renovatebot.com](https://docs.renovatebot.com/) (nf); [Scorecard checks](https://github.com/ossf/scorecard/blob/main/docs/checks.md)

**Secrets and vulnerability disclosure**
- Push protection *for users* is on by default on GitHub.com and blocks pushing secrets to public repos; repository-level push protection is **not** on by default and must be enabled by an admin — [GitHub Docs: About push protection](https://docs.github.com/en/code-security/secret-scanning/introduction/about-push-protection)
- GitHub private vulnerability reporting lets reporters submit advisories privately from the Security tab; maintainers enable it per repo and can collaborate on a draft GHSA and request a CVE — [GitHub Docs: private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/working-with-repository-security-advisories/configuring-private-vulnerability-reporting-for-a-repository) (nf)

**EU Cyber Resilience Act**
- Timeline: manufacturer reporting obligations (Art. 14: actively exploited vulnerabilities and severe incidents; early warning within 24h, notification within 72h, final report later; via ENISA Single Reporting Platform) apply from **11 Sept 2026**; the CRA applies in full from **11 Dec 2027** — [noze.it CRA/SBOM](https://www.noze.it/en/insights/cyber-resilience-act-sbom/); [OpenSSF stewards one-pager](https://policy.openssf.org/CRA/stewards-one-pager.html)
- The Commission published its first CRA guidance on **27 July 2026** (scope, support periods, reporting) — [noze.it: practical guide after Commission clarifications](https://www.noze.it/en/insights/cra-commission-guidance/)
- Open-source software steward (Art. 3(14)): a *legal person*, not a manufacturer, that systematically supports development of FOSS products with digital elements **intended for commercial activities**. Art. 24 obligations: put in place and document a verifiable cybersecurity policy (covering vulnerability handling, encouraging voluntary reporting); cooperate with market surveillance authorities; Art. 14(1) reporting to the extent the steward is involved in development, and Art. 14(3)/(8) incident reporting for infrastructure it provides. Recommendations: SECURITY.md + documented vulnerability handling, single point of contact, incident response, MSA contact, secure development incl. SBOMs, OSPS Baseline — [OpenSSF stewards one-pager](https://policy.openssf.org/CRA/stewards-one-pager.html)
- Sources differ on framing of when steward duties bite: one law-firm note says steward obligations (policy + cooperation) apply from 11 Dec 2027 — [Goodwin, Sept 2026](https://www.goodwinlaw.com/en/insights/publications/2026/09/alerts-lifesciences-technology-preparing-for-eu-cyber-resilience-act); a vendor claims stewards are affected by Art. 14 from Sept 2026 — [Patchstack](https://patchstack.com/articles/launching-managed-cra-article-14-reporting-for-open-source-maintainers/) (vendor, treat cautiously)
- The provisions on conformity assessment bodies apply from 11 June 2026 — [Regulation (EU) 2024/2847, Art. 71](https://eur-lex.europa.eu/eli/reg/2024/2847/oj) (nf)

### Inferences
- A university-hosted, non-monetized SONARITHM is most likely outside "manufacturer" scope; it would only become relevant if a legal entity (e.g., a foundation or a spin-off) monetizes it or if companies ship it in commercial products (those companies become manufacturers and will ask SONARITHM for SBOMs/security contacts). Doing the steward-style basics now (SECURITY.md, PVR, SBOM, documented vuln-handling, support-period statement) costs little and anticipates downstream requests.
- Target: Scorecard ≥7 within 3 months, Best Practices "passing" + OSPS baseline-1 at first public release, baseline-2 once ≥2 maintainers.
- Enable immutable releases only for the stable release channel or accept that nightlies go elsewhere (e.g., a separate repo or artifacts-only), given the all-or-nothing limitation.

### Gaps
- Not legal advice; I did not read the CRA text or the 27 July 2026 Commission guidance directly to settle the steward-reporting timing conflict.
- Whether secret-scanning *alerts* and repo-level push protection remain free for public repos in 2026 (they were historically) was not confirmed in the fetched doc.

---

## 5. Licensing and contribution (MIT/Apache-2.0, alternatives, DCO vs CLA, REUSE/SPDX, OFL fonts, Lucide ISC, Contributor Covenant)

### Takeaway
Dual **MIT OR Apache-2.0** is the Rust ecosystem norm because it matches the Rust project itself (maximal compatibility), Apache-2.0 contributes an explicit patent grant, and MIT preserves GPLv2 compatibility. For an academic project seeking wide adoption this is the best default; use a DCO (not a CLA), REUSE/SPDX headers, keep OFL font and ISC/MIT icon notices, and adopt Contributor Covenant 3.0 (released July 2025) or 2.1.

### Cited Findings
- Rust API Guidelines (C-PERMISSIVE): "The software produced by the Rust project is dual-licensed, under either the MIT or Apache 2.0 licenses. Crates that simply need the maximum compatibility with the Rust ecosystem are recommended to do the same" — [rust-lang/api-guidelines](https://github.com/rust-lang/api-guidelines); [necessities chapter mirror](https://www.rustmax.net/library/api-guidelines/necessities)
- Rationale from Rust contributors: requiring both licenses for inbound contributions means every contributor provides the Apache-2.0 patent grant; MIT is retained mainly because Apache-2.0 is incompatible with GPLv2 ("the one weird case that we needed to retain the nearly-redundant MIT for"); BSD-2-Clause-Patent would fit today but didn't exist then — [internals.rust-lang.org: Rationale of Apache dual licensing](https://internals.rust-lang.org/t/rationale-of-apache-dual-licensing/8952)
- MPL-2.0 is file-level weak copyleft (modified MPL files must stay open; can combine with proprietary code) — [mozilla.org MPL 2.0 FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/) (nf); community discussion of MIT/Apache vs MPL for Rust crates — [users.rust-lang.org](https://users.rust-lang.org/t/licensing-mit-apache-2-vs-mpl-2-0/46250)
- Apache-2.0 §5 makes contributions submitted to the work licensed under the same terms by default ("inbound = outbound") unless stated otherwise — [apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0) (nf)
- The Developer Certificate of Origin (v1.1) is a per-commit `Signed-off-by:` attestation, used by the Linux kernel — [developercertificate.org](https://developercertificate.org/) (nf)
- REUSE: per-file `SPDX-License-Identifier` and `SPDX-FileCopyrightText` headers, `LICENSES/` directory, and `REUSE.toml` for bulk annotations; `reuse lint` checks compliance — [reuse.software/spec](https://reuse.software/spec/) (nf)
- Lucide is ISC-licensed ("Copyright (c) 2026 Lucide Icons and Contributors"); the only condition is retaining the copyright and permission notice in all copies; icons derived from Feather are additionally MIT-licensed ("Copyright (c) 2013-present Cole Bemis"), whose notice must also be included — [lucide.dev/license](https://lucide.dev/license)
- SIL Open Font License 1.1 permits bundling/embedding fonts with software; fonts cannot be sold by themselves; the license and copyright must accompany the fonts; Reserved Font Names restrict naming of modified versions — [openfontlicense.org](https://openfontlicense.org/) (nf)
- Contributor Covenant 3.0 was announced by the Organization for Ethical Source on 28 July 2025 as a major rewrite: adaptable to online/offline communities, plainer and less US-centric language, enforcement section reframed as "Addressing and Repairing Harm" (restorative) — [ethicalsource.dev: Contributor Covenant 3](https://ethicalsource.dev/blog/contributor-covenant-3); adoption example: Hanami (Sept 2025) — [hanamirb.org](https://hanamirb.org/blog/2025/09/02/hanami-adopts-contributor-covenant-3-0); Electron PR updating to v3.0 — [releases.electronjs.org PR #52384](https://releases.electronjs.org/pr/52384)

### Inferences
- License choice for SONARITHM: `MIT OR Apache-2.0` for all crates, CLI, Python bindings and desktop app; this maximizes uptake by labs, companies and other OSS (including GPLv2 projects), and is accepted by JOSS (OSI-approved). GPL-3.0 would block embedding in proprietary or permissively licensed tools (e.g., plugins in commercial DAWs), and MPL-2.0 adds friction without much benefit for a research toolkit. If example datasets or audio stimuli are distributed, license them separately (e.g., CC BY 4.0 or CC0) — data is not software.
- DCO over CLA: CLAs deter casual academic contributors and imply future relicensing power; DCO + inbound=outbound is the norm in Rust projects. Enforce with a DCO check app/action.
- Desktop app "About → Licenses" screen must list third-party notices: generate with `cargo-about` or `cargo-deny`'s license data for Rust deps, `license-checker`-style tooling for npm deps, and include OFL texts for bundled fonts and the Lucide ISC + Feather MIT notices.
- Choose Contributor Covenant 3.0 for a new project (restorative framing suits a research community); 2.1 remains acceptable and widely recognized if GitHub's community template still defaults to it.

### Gaps
- I did not verify which Contributor Covenant version GitHub's "Add code of conduct" wizard offers in Oct 2026.
- I did not find an official Rust FAQ page restating the licensing rationale (forum posts and API guidelines only).

---

## 6. Research software practices (CITATION.cff, Zenodo DOI, JOSS, FAIR4RS, SMPs, reproducible audio outputs, ACM artifact badges)

### Takeaway
Add `CITATION.cff` (schema 1.2.0) and enable the Zenodo–GitHub integration on day one so every release gets a DOI; plan a JOSS paper **no earlier than ~6 months after the repo goes public**, because JOSS now requires six months of public development history, demonstrated research impact, a 750–1,750-word paper with mandated sections (incl. "State of the field", "Software design", "Research impact statement") and an **AI usage disclosure**. For audio reproducibility, combine deterministic rendering with golden-file tests using tolerance-based comparison.

### Cited Findings
- `CITATION.cff` (example `cff-version: 1.2.0`) on the default branch is rendered on the GitHub repo page with a copyable citation/BibTeX; Zenodo uses it to populate the record when releasing via the Zenodo–GitHub integration; Zotero uses it on import; `cffinit` web form creates one; `cff-converter-python` exports to BibTeX/RIS/CodeMeta — [citation-file-format.github.io](https://citation-file-format.github.io/)
- JOSS scope/requirements: research software with "clear research impact", feature-complete and packaged to community standards; minor utilities, thin API clients and single-function packages are not acceptable; projects developed privately need "at least six months of public history prior to submission" with releases, public issues and PRs ("A repo dump is not a history"); pre-review screening gates with desk rejection; automated repo-history checks (commit distribution, burst commits); evidence of research use required; OSI-approved license; issue tracker open without registration; paper files in the same repo; submitting author must be a major contributor; respond to reviewers within 2 weeks and finish changes within 4–6 weeks; after acceptance make a tagged release and archive (Zenodo/figshare) for a DOI — [JOSS: Submitting](https://joss.readthedocs.io/en/latest/submitting.html)
- JOSS AI policy: AI use permitted but must be disclosed; authors confirm human review and human core design decisions; reviewers may not let AI determine verdicts — [JOSS: Submitting](https://joss.readthedocs.io/en/latest/submitting.html)
- JOSS paper format: 750–1,750 words (longer may be asked to shorten); required sections **Summary**, **Statement of need**, **State of the field** (incl. "build vs. contribute" justification), **Software design**, **Research impact statement**, **AI usage disclosure**; plus authors/affiliations (optional ROR), key references with full venue names, funding acknowledgement; `paper.md` with YAML metadata — [JOSS: Paper format](https://joss.readthedocs.io/en/latest/paper.html)
- FAIR4RS: Barker et al., "Introducing the FAIR Principles for research software", *Scientific Data* 9:622 (2022), DOI 10.1038/s41597-022-01710-x; canonical principles v1.0 published by RDA, DOI 10.15497/RDA00068, endorsed by RDA Council May 2022; software-specific adaptation needed because of executability, composite nature, continuous evolution/versioning — [ResearchSoft blog](https://www.researchsoft.org/blog/2022-06/); [RDA FAIR4RS v1.0 PDF](https://archive.rd-alliance.org/system/files/FAIR4RS%20principles%20v1.0.pdf); [DOAJ record](https://doaj.org/article/86cd4f4221c545bab339e6d2aceac8c6)
- Software Management Plans: the Software Sustainability Institute publishes an SMP checklist/guidance used by funders — [software.ac.uk SMP](https://www.software.ac.uk/software-management-plans) (nf)
- ACM Artifact Review and Badging (v1.1) badges: **Artifacts Available**, **Artifacts Evaluated – Functional**, **Artifacts Evaluated – Reusable**, **Results Reproduced**, **Results Replicated** — [ACM policy page](https://www.acm.org/publications/policies/artifact-review-and-badging-current) (nf; page returned HTTP 403 to my fetcher)
- CHI 2027 explicitly encourages supplementary materials such as source code, data and protocols for reproducibility — [CHI 2027 Papers](https://chi2027.acm.org/authors/papers/)

### Inferences
- Reproducible audio outputs for SONARITHM:
  - Determinism: fix sample rate, block size, seeds for any stochastic synthesis (`rand_chacha` seeded), avoid wall-clock time and thread-order-dependent mixing, and document float behaviour; floating-point results may still differ across CPUs/compilers (FMA/SIMD), so golden tests should compare with tolerances (max-abs error, RMS error / SNR threshold) rather than byte equality; additionally store exact SHA-256 hashes for same-platform regression detection.
  - Golden files: keep short reference renders (WAV, 32-bit float) under `tests/golden/` via Git LFS, plus a manifest (`input`, `mapping spec`, `engine version`, `hash`, `tolerance`); provide an explicit `--bless`/`UPDATE_GOLDEN=1` workflow; snapshot non-audio outputs (mapping JSON, event lists) with a snapshot-test crate.
  - Version the *mapping specification* format with SemVer and record engine version + spec hash in rendered file metadata (e.g., WAV `LIST/INFO` or a sidecar JSON) so a paper figure/sound can be regenerated exactly.
- Pair every paper with a tagged release + Zenodo DOI + an archived "paper artifact" repo/folder (scripts, stimuli, analysis notebooks) to qualify for ACM "Available"/"Functional" badges where offered.
- An SMP (2–4 pages) is cheap to write in month 1 and doubles as grant-proposal material; it can live in `docs/governance/software-management-plan.md`.

### Gaps
- ACM badging page could not be fetched (403); badge names are from prior knowledge of policy v1.1.
- I did not find whether JOSS imposes a specific minimum effort (it explicitly gives no LOC/person-month threshold per the submitting page) beyond the 6-month history and impact rules; the exact effective date of the stricter 2025–2026 JOSS rules was not stated on the pages.
- No authoritative community standard for "golden-file audio regression testing" was found; the recommendations above are engineering inference.

---

## 7. Publication venues and timing; toolkit evaluation expectations; research ethics

### Takeaway
ICAD is the home venue (2026 was a distributed three-hub conference, 28–31 July 2026; **ICAD 2027 is in Athens, Greece**, dates/deadline TBA; expect a full-paper deadline around **March 2027 [EST]**). CHI 2027's deadline (10 Sept 2026) has passed, so CHI 2028 (~Sept 2027 [EST]) is the realistic CHI target. Near-term options: EuroVis 2027 full papers (~early Dec 2026 [EST]), NIME 2027 (Paris, 22–25 June 2027; CfP due Oct 2026), IEEE VIS 2027 (~late March 2027 [EST]), ASSETS 2027 (~late April 2027 [EST]), DAFx27 (Cremona, 24–27 Aug 2027), and JOSS once the 6-month history rule is met. Toolkit papers should follow Ledo et al. (CHI 2018): choose evaluation strategies matched to claims rather than defaulting to a usability study.

### Cited Findings
**ICAD**
- ICAD 2026 (31st): 28–31 July 2026, distributed across three hubs — Bielefeld University/CITEC (Thomas Hermann), ESMUC Barcelona (Adrian Garcia Riber), Rensselaer Polytechnic Institute, Troy NY (Jonas Braasch); deadlines (as updated 2026-03-05): full papers **2026-03-20**, all other categories **2026-03-27**, via EasyChair; Office/LaTeX templates; early-bird registration before 2026-06-22 — [ICAD 2026 site](https://interactive-sonification.org/icad2026/); [icad.org](https://icad.org/)
- ICAD 2027 (32nd) will be held in **Athens, Greece**; no dates/deadlines published as of my fetch — [icad.org](https://icad.org/); [ICAD conferences page](https://icad.org/conferences/)

**ACM CHI 2027**
- Pittsburgh, 10–14 May 2027; submission site opened 13 Aug 2026; paper deadline **10 Sept 2026 AoE** (no separate abstract deadline); reviews 5 Nov 2026; revise-and-resubmit until 3 Dec 2026; final notification 17 Dec 2026; publication-ready 18 Feb 2027. Length: 5,000–8,000 words encouraged (<5,000 = short paper; >12,000 desk-rejected without justification); abstract ≤150 words. All papers published under ACM Open (ACM fully OA since 1 Jan 2026); 2027 APC $500 (ACM/SIG members) / $750 (non-members) unless the institution participates in ACM Open; accessibility: follow SIGCHI accessible submission guide; ethics: human-subjects research must meet the authors' local ethics review requirements, with a short note to reviewers; ACM's 2021 policy on research involving humans applies — [CHI 2027 Papers](https://chi2027.acm.org/authors/papers/); [How the CHI 2027 review process will work](https://chi2027.acm.org/2026/08/17/how-the-chi-2027-papers-review-process-will-work/)

**ACM ASSETS**
- ASSETS 2026 technical papers: deadline **22 Apr 2026 AoE**; reviews 3 June; rebuttal until 10 June; notification 24 June; camera-ready 22 July 2026; tracks include Technical Papers, Experience Reports, Posters & Demos, Workshops, Doctoral Consortium, SRC; ORCID required on acceptance; in-person presenter required — [ASSETS 2026 CfP](https://assets26.sigaccess.org/call_for_papers.html). ASSETS 2026 conference listed for 25 Oct 2026 in Porto, Portugal (secondary listing) — [myhuiban ASSETS](https://www.myhuiban.com/conference/893.md)
- No ASSETS 2027 call found as of 18 Oct 2026 — [search result summary; ASSETS 2025 CfP for pattern](https://assets25.sigaccess.org/call_for_papers.html)

**IEEE VIS / EuroVis**
- IEEE VIS 2026 call: mandatory abstract **21 Mar 2026**, full paper **31 Mar 2026** (AoE); same dates in 2024 — [IEEE VIS 2026 CfP](https://aa.ieeevis.org/year/2026/info/call-participation/call-for-participation); [VIS 2024 CfP](https://aa.ieeevis.org/year/2024/info/call-participation/call-for-participation)
- IEEE VIS 2027: one third-party listing gives Chicago, 7–12 Nov 2027 (unconfirmed) — [showsbee listing](https://www.showsbee.com/fairs/104666-IEEE-VISWeek-2027.html)
- EuroVis 2027: Stuttgart, Germany, 31 May–4 June 2027 (aggregator + Eurographics events list); workshop-proposal deadline was 12 Oct 2026; EuroVis 2026 full-paper deadline was 3 Dec 2025, with mandatory abstracts beforehand; full papers appear in Computer Graphics Forum — [EuroVis 2027 timeline (unofficial)](https://velitchko.github.io/eurovis-timeline/); [Eurographics events](https://www.eg.org/wp/events/list/); [EuroVis 2027 call for workshops](http://dmatheorynet.blogspot.com/2026/09/dmanet-eurovis-2027-call-for-workshops.html); [EuroVis full papers page (past cycle)](https://eurovis.org.uk/full-papers/)

**Audio/music venues**
- DAFx27 (30th): Cremona Campus of Politecnico di Milano, Cremona, Italy, 24–27 Aug 2027 — [dafx27.deib.polimi.it](https://dafx27.deib.polimi.it/); [dafx.de](https://www.dafx.de/). DAFx26 hosted at MIT — [dafx26.mit.edu](https://dafx26.mit.edu/)
- NIME 2027: Paris, France, 22–25 June 2027; call for papers to be published October 2026 — [nime2027.org](https://www.nime2027.org/)
- SMC 2026: Zagreb, Croatia (listed for 5 Nov 2026); no SMC 2027 info found — [SMC network conferences](https://smcnetwork.org/conferences.html)
- Audio Mostly: no 2026/2027 edition details found; most recent confirmed in results was AM'24 (Univ. of Milan, 18–20 Sept 2024); proceedings in ACM DL; the host-call page seeking hosts for 2026 and 2027 appears stale — [audiomostly.com 2024](https://audiomostly.com/2024/global-hosting/); [dblp Audio Mostly](https://dblp.dagstuhl.de/db/conf/audio/index.html)
- Journals (rolling submissions): ACM TACCESS — [dl.acm.org/journal/taccess](https://dl.acm.org/journal/taccess) (nf); *Organised Sound* (Cambridge UP; themed issues with calls) — [cambridge.org/core/journals/organised-sound](https://www.cambridge.org/core/journals/organised-sound) (nf); *Journal of the Audio Engineering Society* — [aes.org/journal](https://www.aes.org/journal/) (nf); JOSS (rolling, see §6) — [joss.theoj.org](https://joss.theoj.org/) (nf)

**Toolkit evaluation and ethics**
- Ledo, Houben, Vermeulen, Marquardt, Oehlberg, Greenberg, "Evaluation Strategies for HCI Toolkit Research", CHI '18, DOI 10.1145/3173574.3173610: analysed 68 toolkit papers and identified four evaluation strategies with associated techniques, value, limitations and trade-offs — [UCL Discovery (OA accepted version)](https://discovery.ucl.ac.uk/id/eprint/10053385/); [iLab Calgary](https://ilab.ucalgary.ca/publications/chi-2018-ledo/); [doi.org/10.1145/3173574.3173610](https://www.doi.org/10.1145/3173574.3173610)
- CHI requires that human-subjects research meet the ethics-review requirements of the authors' environment and that authors explain that context to reviewers; ACM's 2021 policy on research involving human participants applies — [CHI 2027 Papers](https://chi2027.acm.org/authors/papers/); [ACM Publications Policy on Research Involving Human Participants and Subjects](https://www.acm.org/publications/policies/research-involving-human-participants-and-subjects) (nf)

### Inferences
- Paper-type fit:
  - **ICAD**: systems/toolkit papers, design-space/taxonomy papers, empirical perception studies, and demos — the most receptive audience for a sonification framework.
  - **CHI / UIST-style**: toolkit papers need strong framing + evaluation; CHI also accepts design-space and empirical studies (e.g., a study of how blind/low-vision analysts use SONARITHM).
  - **ASSETS / TACCESS**: accessibility-centred contributions — studies with blind and low-vision (BLV) users, co-design, accessible data analysis workflows; expect participants from the target population and accessibility of the paper itself.
  - **IEEE VIS / EuroVis (CGF)**: sonification as data representation — design spaces, multimodal (audio+visual) analysis systems, perceptual evaluations; VIS has short-paper and full-paper tracks.
  - **DAFx / JAES**: signal-processing/synthesis engine contributions (efficient real-time rendering, parameter mapping algorithms, spatial audio).
  - **NIME / SMC / Audio Mostly / Organised Sound**: interaction design, artistic/compositional use of data, sonic interaction, aesthetics.
  - **JOSS**: the software paper itself (citable, short), once the 6-month public-history and impact evidence exist.
- Ledo et al.'s four strategies are commonly summarized as demonstration (example applications/novel use cases/replicating prior systems), usage (user studies, case studies, walkthroughs), technical performance/benchmarks, and heuristic evaluation; best practice is to combine a demonstration gallery (e.g., replicating classic sonifications in N lines of code) with benchmarks (latency, real-time factor) and a targeted usage study, avoiding the "usability trap" of a lab usability test that doesn't support the paper's claims. (Strategy names not re-verified from full text this session — see Gaps.)
- Ethics: get IRB/ethics board approval *before* any study (including telemetry-based or in-the-wild studies); for BLV participants provide consent forms in accessible formats (tagged PDF/HTML/plain text compatible with screen readers, audio/verbal consent options, plain-language summaries), budget fair compensation, and avoid sonification stimuli at unsafe loudness levels (state calibrated playback levels).

### Gaps
- No official ICAD 2027 dates/deadline, ASSETS 2027 call, IEEE VIS 2027 CfP, EuroVis 2027 full-paper deadline, SMC 2027, or Audio Mostly 2026/2027 details were found as of 18 Oct 2026.
- The four Ledo et al. strategy names come from prior knowledge; the PDF could not be text-extracted in this session.
- I did not verify TACCESS/Organised Sound/JAES current review times or APCs.

---

## 8. Telemetry and privacy norms for desktop OSS

### Takeaway
The prevailing norm for developer/research OSS is **off-by-default or local-only telemetry with explicit opt-in upload** (Go's 2024 model), transparent documentation of every collected field, and crash reporting that asks before sending. Under EU law, reading/storing information on a user's device for non-essential analytics generally needs consent (ePrivacy Art. 5(3), clarified by EDPB Guidelines 2/2023), and IP addresses/device IDs can be personal data under GDPR.

### Cited Findings
- Go toolchain telemetry (since Go 1.23): modes `local` (default — collected locally, never uploaded), `on` (uploads approved, aggregated counters, sampled), `off`; controlled with `go telemetry on|off|local`; data in `os.UserConfigDir()/go/telemetry`; weekly reports are aggregated counter summaries — [go.dev/doc/telemetry](https://go.dev/doc/telemetry)
- Go switched to opt-in upload after developer feedback; critics noted opt-in samples may be non-representative — [DevClass, 14 Aug 2024](https://devclass.com/2024/08/14/go-1-23-released-with-telemetry-uploaded-to-google-but-opt-in-after-developer-feedback)
- EDPB Guidelines 2/2023 on the technical scope of ePrivacy Art. 5(3) (final adopted 7 Oct 2024) clarify that Art. 5(3) covers more than cookies — including URL/pixel tracking, local processing, IP-only tracking, and intermittent/mediated IoT reporting — via three elements: "information", "terminal equipment", and "gaining access/storage" — [EDPB guidelines page](https://www.edpb.europa.eu/documents/guideline/guidelines-22023-on-technical-scope-of-art-53-of-eprivacy-directive_en); [final PDF](https://edpb.europa.eu:443/system/files/2024-10/edpb_guidelines_202302_technical_scope_art_53_eprivacydirective_v2_en_0.pdf); [Hunton summary](https://hunton.com/privacy-and-information-security-law/edpb-publishes-guidelines-to-clarify-scope-of-eu-cookie-notice-and-consent-requirements)
- Dynamic IP addresses can be personal data (CJEU, *Breyer*, C-582/14, 2016) — [EUR-Lex judgment](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:62014CJ0582) (nf)
- GDPR Art. 25 requires data protection by design and by default — [gdpr-info.eu Art. 25](https://gdpr-info.eu/art-25-gdpr/) (nf)

### Inferences
- SONARITHM policy: no network calls on first launch except an *optional* update check that the user can disable (and that sends no identifiers beyond what an HTTPS GET needs); analytics strictly opt-in with a first-run dialog defaulting to "No"; crash reports kept locally and shown to the user, uploaded only per-crash with consent; document every field in `docs/explanation/telemetry.md`; never collect user data files or sonified datasets; offer an env var/CLI flag (e.g., `SONARITHM_TELEMETRY=off`) honoured by CLI, desktop app and Python bindings; prefer self-hostable/EU-hosted backends.
- Research-use telemetry (e.g., logging interaction for a study) must be a separate, study-specific build or mode covered by ethics approval and informed consent — not general product analytics.
- Tauri's updater check itself is a network request to GitHub; disclose it in the privacy notice.

### Gaps
- I did not survey 2026 telemetry practices of specific Rust desktop apps (e.g., Zed) or verify Tauri-specific crash-reporting plugins; no authoritative "OSS telemetry standard" exists to cite.

---

## 9. Synthesis — Day-one SDLC checklist for SONARITHM and Publication plan

### Takeaway
Everything below is achievable in the first 1–2 weeks with free GitHub features and OSS tooling; signing and JOSS/major-venue submissions are scheduled for later milestones because they depend on budget/legal entity and on six months of public history.

### Cited Findings

#### Day-one SDLC checklist for SONARITHM
(Each item cites the source supporting the practice; see §§1–8 for detail.)

**Repository & governance**
- [ ] Public GitHub repo, default branch `main`, ruleset: require PR, ≥1 review (code-owner review for core), required status checks, linear history/squash merge, no force-push — [Scorecard Branch-Protection/Code-Review](https://github.com/ossf/scorecard/blob/main/docs/checks.md)
- [ ] Root files: `README.md`, `LICENSE-MIT` + `LICENSE-APACHE` (`license = "MIT OR Apache-2.0"` in every `Cargo.toml`), `CONTRIBUTING.md` (DCO sign-off, Conventional Commits, dev setup), `CODE_OF_CONDUCT.md` (Contributor Covenant 3.0), `SECURITY.md`, `GOVERNANCE.md` (maintainers, decision process, RFC/ADR flow), `CITATION.cff`, `CHANGELOG.md` — [Rust API guidelines](https://github.com/rust-lang/api-guidelines); [Contributor Covenant 3](https://ethicalsource.dev/blog/contributor-covenant-3); [CFF](https://citation-file-format.github.io/)
- [ ] `.github/CODEOWNERS`, `.github/ISSUE_TEMPLATE/{bug.yml,feature.yml,research-question.yml,config.yml}`, `.github/pull_request_template.md` (checklist = Definition of Done: tests, docs, changelog-worthy title, ADR link if architectural, golden files updated intentionally) — [GitHub code owners](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners) (nf); [issue forms](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms) (nf)
- [ ] `AGENTS.md` (root + per crate/app) and `CLAUDE.md` = `@AGENTS.md` + Claude notes; `.gitignore` `CLAUDE.local.md` — [agents.md](https://agents.md/); [Claude Code memory docs](https://code.claude.com/docs/en/memory)
- [ ] `docs/` structured by Diátaxis; `docs/decisions/0000-use-madr.md` (MADR 4.x) + ADRs for: workspace layout, Tauri vs alternatives, audio backend, mapping-spec format, license; `rfcs/0000-template.md` — [diataxis.fr](https://diataxis.fr/) (nf); [adr.github.io/madr](https://adr.github.io/madr/) (nf); [rust-lang/rfcs](https://github.com/rust-lang/rfcs) (nf)
- [ ] GitHub Project with Roadmap view (fields: Status, Milestone, Iteration, Area, Paper-link); labels `area/*`, `kind/*`, `good first issue`; written Definition of Ready (problem statement, acceptance criteria, design/ADR link if needed) — [GitHub roadmap layout](https://docs.github.com/en/issues/planning-and-tracking-with-projects/customizing-views-in-your-project/customizing-the-roadmap-layout) (nf)

**Code quality & CI**
- [ ] Cargo workspace with `rust-toolchain.toml`, declared MSRV (`rust-version`), committed `Cargo.lock`, `[workspace.lints]`, `rustfmt.toml`, `clippy.toml` — [rustc remap-path-prefix / reproducibility](https://doc.rust-lang.org/rustc/command-line-arguments.html#--remap-path-prefix-remap-source-names-in-output) (nf)
- [ ] `ci.yml`: matrix ubuntu/windows/macos; `Swatinem/rust-cache`; fmt, clippy `-D warnings`, tests (incl. golden-audio tests with tolerance), docs, MSRV job, `cargo deny check`, `cargo semver-checks` (on library crates), Python wheel smoke test, wasm32 build of core; top-level `permissions: {}`, actions pinned to SHAs — [uv release.yml (pattern)](https://github.com/astral-sh/uv/blob/main/.github/workflows/release.yml); [rust-cache](https://github.com/Swatinem/rust-cache) (nf); [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) (nf)
- [ ] PR-title Conventional Commit lint + DCO check — [conventionalcommits.org](https://www.conventionalcommits.org/en/v1.0.0/) (nf); [developercertificate.org](https://developercertificate.org/) (nf)
- [ ] `release-plz.yml` (release PR, git-cliff changelog, semver checks) with crates.io Trusted Publishing (`id-token: write`, `rust-lang/crates-io-auth-action@v1`) after first manual publish — [release-plz docs](https://release-plz.dev/docs); [crates.io trusted publishing](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07)
- [ ] `wheels.yml` with `PyO3/maturin-action` (abi3) → PyPI Trusted Publishing — [maturin-action](https://github.com/PyO3/maturin-action) (nf)
- [ ] `release-desktop.yml` with `tauri-apps/tauri-action`; updater keypair generated and backed up offline (two custodians); `createUpdaterArtifacts: true`; `latest.json` on GitHub Releases — [Tauri updater](https://v2.tauri.app/plugin/updater/)

**Supply chain & security**
- [ ] Enable: Dependabot (cargo, pip, npm, github-actions) *or* Renovate; secret scanning + repo-level push protection; private vulnerability reporting; CodeQL (Rust/JS/Python) — [push protection docs](https://docs.github.com/en/code-security/secret-scanning/introduction/about-push-protection); [Scorecard SAST/Dependency-Update-Tool](https://github.com/ossf/scorecard/blob/main/docs/checks.md)
- [ ] OpenSSF Scorecard action + badge; register on bestpractices.dev (target passing + baseline-1) — [OSPS Baseline](https://baseline.openssf.org/); [Baseline badge blog](https://openssf.org/blog/2026/02/25/getting-an-openssf-baseline-badge-with-the-best-practices-badge)
- [ ] Release artifacts: `actions/attest-build-provenance`, CycloneDX SBOM (`cargo-cyclonedx`) attached to releases, binaries built with `cargo auditable`, immutable releases enabled for stable channel — [GitHub immutable releases](https://github.blog/changelog/2025-10-28-immutable-releases-are-now-generally-available/); [SLSA v1.2](https://slsa.dev/spec/v1.2/whats-new)
- [ ] `SECURITY.md`: supported versions/support period, PVR link, response SLAs, embargo policy; documented vulnerability-handling process (CRA-steward-style) — [OpenSSF CRA stewards one-pager](https://policy.openssf.org/CRA/stewards-one-pager.html)
- [ ] Signing plan ADR: macOS Developer ID + notarization (API key) when Apple account available; Windows via Azure Artifact Signing (if eligible) or SignPath/OV; AppImage GPG signing with published key fingerprint — [Tauri macOS](https://v2.tauri.app/distribute/sign/macos/); [Tauri Windows](https://v2.tauri.app/distribute/sign/windows/); [Tauri Linux](https://v2.tauri.app/distribute/sign/linux/)

**Licensing hygiene**
- [ ] REUSE-compliant SPDX headers + `REUSE.toml`; `reuse lint` in CI; third-party notices generated for Rust and npm deps; OFL texts for bundled fonts; Lucide ISC + Feather MIT notices in the app's About/Licenses view — [reuse.software](https://reuse.software/spec/) (nf); [lucide.dev/license](https://lucide.dev/license); [openfontlicense.org](https://openfontlicense.org/) (nf)

**Research readiness**
- [ ] Enable Zenodo–GitHub integration before the first tagged release; `CITATION.cff` with ORCIDs; `docs/governance/software-management-plan.md`; `paper/` folder reserved for JOSS `paper.md` + `paper.bib`; AI-usage disclosure policy in CONTRIBUTING — [CFF/Zenodo](https://citation-file-format.github.io/); [JOSS paper format](https://joss.readthedocs.io/en/latest/paper.html); [FAIR4RS](https://www.researchsoft.org/blog/2022-06/)
- [ ] Golden-audio test harness + deterministic render mode; renders embed engine version + mapping-spec hash — (engineering inference, §6)
- [ ] Ethics: contact institutional IRB/ethics board early; prepare accessible consent templates — [CHI 2027 ethics requirement](https://chi2027.acm.org/authors/papers/)
- [ ] Privacy: no default telemetry; opt-in analytics/crash upload; documented update-check — [Go telemetry model](https://go.dev/doc/telemetry); [EDPB 2/2023](https://www.edpb.europa.eu/documents/guideline/guidelines-22023-on-technical-scope-of-art-53-of-eprivacy-directive_en)

#### Publication plan
Deadlines: **confirmed** = officially published; **[EST]** = estimate from previous cycles.

| Venue | Paper type for SONARITHM | Typical / known deadline | Event / timing | Fit | Source |
|---|---|---|---|---|---|
| **EuroVis 2027** (CGF full papers; also short papers/posters) | Design-space or multimodal (audio+visual) analysis system | Full papers ~early Dec 2026 **[EST]** (2026 cycle: 3 Dec 2025, abstract required beforehand) | Stuttgart, 31 May–4 Jun 2027 (unofficial listing) | Medium; requires strong vis framing; probably too soon for a full system paper — consider short paper/poster (deadlines typically later, ~Jan–Feb [EST]) | [timeline](https://velitchko.github.io/eurovis-timeline/); [EG events](https://www.eg.org/wp/events/list/) |
| **NIME 2027** | Interaction/instrument-style use of data sonification; demo | CfP published Oct 2026; deadline typically late Jan–Feb 2027 **[EST]** | Paris, 22–25 Jun 2027 | Medium (if expressive/interactive angle) | [nime2027.org](https://www.nime2027.org/) |
| **ICAD 2027** | Systems/toolkit paper (framework architecture, mapping DSL), design-space/taxonomy paper, demo | ~Mar 2027 **[EST]** (2026: full papers 20 Mar, others 27 Mar) | Athens, Greece; dates TBA (2026 was late July) | **Very high** — primary community | [icad.org](https://icad.org/); [ICAD 2026](https://interactive-sonification.org/icad2026/) |
| **IEEE VIS 2027** (TVCG full / short papers) | Sonification-as-representation design space; empirical perception study; multimodal system | Abstract ~21 Mar, full ~31 Mar 2027 **[EST]** (same dates in 2024 and 2026) | Chicago, 7–12 Nov 2027 (unconfirmed) | Medium–high for a design-space or evaluation paper | [VIS 2026 CfP](https://aa.ieeevis.org/year/2026/info/call-participation/call-for-participation) |
| **ACM ASSETS 2027** | Empirical study / co-design with BLV users using SONARITHM; experience report | ~late Apr 2027 **[EST]** (2026: 22 Apr) | ~Oct 2027 [EST] | High for accessibility contributions (needs BLV participants) | [ASSETS 2026 CfP](https://assets26.sigaccess.org/call_for_papers.html) |
| **DAFx27** | Rendering-engine / synthesis / mapping-algorithm paper | ~Mar–Apr 2027 **[EST]** | Cremona, Italy, 24–27 Aug 2027 (confirmed) | Medium (engine-level novelty needed) | [dafx27](https://dafx27.deib.polimi.it/) |
| **Audio Mostly 2027** | Sonic interaction / auditory interface paper | ~Apr–May 2027 **[EST]** (low confidence; no 2026/27 info found) | ~Aug–Sept **[EST]** | Medium | [audiomostly.com](https://audiomostly.com/2024/global-hosting/) |
| **SMC 2027** | Sound & music computing: system or perception study | ~Feb–Apr 2027 **[EST]** (no 2027 info found) | TBA (2026: Zagreb, Nov) | Medium | [smcnetwork.org](https://smcnetwork.org/conferences.html) |
| **JOSS** | Software paper (750–1,750 words) | Rolling; earliest ≈ 6 months after repo goes public + evidence of research use | Review typically weeks–months | **Very high** — establishes citable DOI'd software paper | [JOSS submitting](https://joss.readthedocs.io/en/latest/submitting.html) |
| **ACM CHI 2028** | Toolkit paper with Ledo-style evaluation, or empirical/design-space paper | ~early–mid Sept 2027 **[EST]** (CHI 2027: 10 Sept 2026, passed) | ~Apr–May 2028 [EST] | High but competitive; needs mature system + multi-strategy evaluation | [CHI 2027 dates](https://chi2027.acm.org/authors/papers/) |
| **ACM TACCESS** | Extended accessibility study / journal version of ASSETS work | Rolling | — | High for accessibility work | [TACCESS](https://dl.acm.org/journal/taccess) (nf) |
| **Organised Sound** | Aesthetic/compositional/theoretical sonification work | Themed-issue calls (rolling, check CUP) | — | Medium (arts/theory angle) | [Organised Sound](https://www.cambridge.org/core/journals/organised-sound) (nf) |
| **Journal of the AES** | Engineering/perceptual audio contribution (rendering, loudness, spatialization of sonifications) | Rolling | — | Medium | [AES Journal](https://www.aes.org/journal/) (nf) |

### Inferences
- Suggested sequence: (1) public repo + Zenodo DOIs from first release (Q4 2026); (2) ICAD 2027 systems/toolkit paper (~Mar 2027) — the natural debut; (3) JOSS submission once ≥6 months of public history + first external research use (~mid 2027); (4) ASSETS 2027 (~Apr 2027) or IEEE VIS 2027 (~Mar 2027) if a BLV study or design-space study can be completed in time — otherwise target CHI 2028 (~Sept 2027) with a mature toolkit + multi-strategy evaluation; (5) journal extensions (TACCESS, JAES, Organised Sound) afterwards.
- Start IRB/ethics applications by Dec 2026 to make spring 2027 study-based deadlines realistic.
- Each paper should cite a specific SONARITHM release DOI and ship a reproducibility package (scripts, stimuli, golden renders).

### Gaps
- All 2027 deadlines other than CHI 2027 (passed) are estimates; ICAD 2027, ASSETS 2027, VIS 2027, EuroVis 2027, NIME 2027, DAFx27, SMC 2027 and Audio Mostly 2027 calls should be re-checked monthly from their official sites.
- Legal eligibility for Azure Artifact Signing depends on SONARITHM's legal/organizational home (individual vs university vs foundation), which is not specified.

## Errata

Dated corrections. The text above is kept as the kickoff snapshot ([research README](../README.md)).

- **2026-10-18, link check.**
  - **MADR conventions (§1).** The SAIF Forge example page (`docs.platformdev.saif.com/…`) returned HTTP 404. The MADR conventions it illustrated (files in `docs/decisions/NNNN-title-with-dashes.md`; statuses proposed, accepted, rejected, deprecated, superseded) are documented by MADR itself: <https://adr.github.io/madr/>.
  - **covector (§1).** `github.com/tauri-apps/covector` returned HTTP 404. covector is maintained at <https://github.com/jbolda/covector>, verified 2026-10-18 (not archived).
  - **FAIR4RS (§6).** The RDA archive PDF of the FAIR4RS principles returned HTTP 404. The canonical reference is the DOI <https://doi.org/10.15497/RDA00068>, which resolves to Zenodo record 6623556 (verified 2026-10-18).
- **2026-10-19, citation check for P01 (A08).** Two records from these notes were incomplete and are completed in `references.bib` from Crossref and the author version: `ledo2018evaluation` (David Ledo, Steven Houben, Jo Vermeulen, Nicolai Marquardt, Lora Oehlberg, Saul Greenberg; CHI '18, paper 36, pp. 36:1–36:17; the four strategy types are Demonstration, Usage, Technical Performance and Heuristics) and `barker2022introducing` (eleven authors from Michelle Barker to Tom Honeyman; Scientific Data 9(1), 622).
