# Risk Register

> **Status:** Living document, baseline 2026-10-18. **Owner:** Project lead. Reviewed at every phase review and stage gate.
> **Scale:** Probability (P) and Impact (I) are scored 1–5. **Score = P × I.** 15+ = high (red), 8–14 = medium (amber), < 8 = low (green).

| ID | Risk | P | I | Score | Owner | Mitigation | Trigger / early warning | Status |
|---|---|---|---|---|---|---|---|---|
| R-01 | Tauri on Linux (WebKitGTK) underperforms for 1M-point plots or spectrograms, or Orca support is inadequate | 3 | 4 | 12 | Architecture lead | A02 spikes T06/T07; WebGL2 baseline; Rust-side downsampling; **Electron fallback** behind `IpcTransport` (ADR-0002) | PB-08 missed on WebKitGTK; Orca walkthrough fails | Open |
| R-02 | Screen readers handle the webview badly (browse-mode capture, unreliable live regions) | 3 | 5 | 15 | Accessibility lead | Composite widgets, scoped `role="application"`; native notification plugin (OD-05); early AT walkthroughs; BLV co-designers from B03 | A02-T07 failures; co-design complaints | Open |
| R-03 | Interactive latency exceeds 50 ms on Windows shared mode | 3 | 3 | 9 | Audio lead | Buffer tuning; ASIO opt-in; `wasapi` exclusive backend later; cues pre-rendered | A02-T01 p95 > 50 ms | Open |
| R-04 | Owned render graph or Firewheel brings concurrency bugs or UB | 2 | 4 | 8 | Audio lead | Minimal graph; Miri; shuttle; soak tests; no `unsafe` without review | Miri or shuttle findings; xruns in soak | Open |
| R-05 | cpal API churn (0.19 breaking changes) | 4 | 2 | 8 | Audio lead | Isolate behind `AudioBackend`; pin minor versions; upgrade issue with checklist | cpal release notes | Open |
| R-06 | Cross-platform determinism fails (SIMD, FTZ, libm differences) | 3 | 3 | 9 | Core lead | Score is the strict artifact; audio tolerances; fixed block size; seeded RNG; no wall clock | Golden CI failures on one OS | Open |
| R-07 | A copyleft licence enters through a dependency or a model | 3 | 4 | 12 | Project lead | cargo-deny allowlist in CI; npm licence check; model-licence policy; the avoid list in tech-stack | CI licence failure | Open |
| R-08 | **Scope creep** across nine engines, plus desktop, Python and web | 4 | 5 | **20** | Project lead | Waves (ADR-0007); gates; "no tenth engine without an ADR"; Stage D engines are research-grade | Phase overrunning by > 50%; parallel half-finished engines | Open |
| R-09 | **Bus factor of 1** (single core maintainer) | 4 | 5 | **20** | Project lead | AGENTS.md and thorough docs; small PRs; recruit co-maintainers from Stage B; GOVERNANCE.md path to maintainership; grants (OD-10) | No external reviewer by Gate B | Open |
| R-10 | Cannot recruit or pay BLV co-designers | 3 | 4 | 12 | Accessibility lead | Early outreach (NFB, Program-L, local organizations); budget via grants (OD-10); remote sessions on the participant's own AT | No participants confirmed 6 weeks before B07 | Open |
| R-11 | Ethics approval delayed or unavailable | 2 | 3 | 6 | Research lead | Route identified: RV University (OD-04). Find a faculty mentor by Nov 2026; file by Dec 2026; technical-only papers do not need ethics approval | No mentor or committee contact by end of Nov 2026 | Open |
| R-12 | ICAD 2027 deadline missed | 3 | 3 | 9 | Research lead | A08 scoped to demonstration + technical benchmarks; fallback venues (IEEE VIS short paper, ICAD 2028, JOSS) | A07 not released by mid-Feb 2027 | Open |
| R-13 | Chart extraction accuracy disappoints users | 4 | 3 | 12 | PLOTSONA owner | Exact-first tiers; human correction; honest provenance labels; ML in Stage D only | Corpus targets missed in C01/C02 | Open |
| R-14 | Code-signing eligibility or cost blocks trusted installers | 3 | 3 | 9 | Release lead | Resolve the legal home (OD-02). Azure Artifact Signing is not available to the owner as an individual in India, so use SignPath Foundation's OSS programme or signing through RV University; budget the Apple Developer Program | Unsigned builds at C06 | Open |
| R-15 | Stale but load-bearing crates (hound, midly, tts, assert_no_alloc) break | 2 | 2 | 4 | Core lead | Pin; tests that catch regressions; ready to vendor or fork | New compiler or edition warnings; advisories | Open |
| R-16 | AI-agent contributions are inconsistent or subtly wrong | 3 | 3 | 9 | Project lead | AGENTS.md rules; tests that reference rule IDs; mandatory human review; small PRs; RT-audio review checklist | Rising revert rate; review findings | Open |
| R-17 | Research evidence contradicts defaults or the tagline | 3 | 2 | 6 | Research lead | Tagline discipline; defaults are configurable; findings log updates rules through change control | Study results diverge from SDR defaults | Open |
| R-18 | Hearing-safety incident (sudden loud output) | 1 | 5 | 5 | Audio lead | Gain ramp; true-peak limiter; level tests; low default master | Any user report of loud onsets | Open |
| R-19 | Tauri updater private key lost or leaked | 1 | 5 | 5 | Release lead | Two custodians; offline encrypted backups; documented rotation | Key custody audit fails at a gate | Open |
| R-20 | EU Cyber Resilience Act obligations once SONARITHM is used commercially | 2 | 3 | 6 | Project lead | Steward basics now: security policy, SBOM, vulnerability handling ([supply chain](../engineering/supply-chain-and-licensing.md)) | Commercial redistribution or a legal entity is formed | Open |
| R-21 | Webview graphics limits (no WebGPU on Linux; GPU driver issues) | 3 | 2 | 6 | Frontend lead | WebGL2 baseline; Canvas2D fallback; WebGPU only as an enhancement | Rendering bugs on specific GPUs | Open |
| R-22 | No macOS or Linux hardware for manual audio, webview and screen-reader tests (the owner has one Windows laptop) | 3 | 3 | 9 | Project lead | Linux: an Ubuntu live USB or dual boot on the owner's laptop for A02, B03 and B07 checks (it also covers the NVIDIA WebKitGTK case). macOS: CI runners for build and test; a university lab Mac or volunteer for VoiceOver and real-audio checks by B07 | No Mac access arranged 6 weeks before B07 | Open |

## Review log

| Date | Event | Changes |
|---|---|---|
| 2026-10-18 | Baseline created from the kickoff research | R-01..R-21 opened |
| 2026-10-18 | Owner context recorded (solo, RV University, Windows-only hardware) | R-11 lowered to 6; R-14 mitigation updated; R-22 opened |
