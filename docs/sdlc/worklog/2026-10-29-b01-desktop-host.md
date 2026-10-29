# 2026-10-29: Lane U starts with the Tauri host

> **Date(s):** 2026-10-29. **Phase:** B01. **Author:** written by an AI agent (Claude Code, Lane U session), for review by @DuhItzAniket.
> **Related:** [worklog conventions](README.md)

## Context and phase

- Phase: B01 ([roadmap](../../planning/roadmap.md)); task B01-T03 ([#193](https://github.com/DuhItzAniket/SONARITHM/issues/193)).
- Starting point: Stage B's Rust lane was complete. The owner opened Gate U provisionally on 2026-10-29: Lane U builds now, the NVDA and Narrator walkthroughs (HP-01) follow, and no component is signed off before them.

## Goals

- The Tauri 2 host as a thin adapter over `sonarithm-app` (OD-15, ADR-0017), with CI on three OSes.

## What was done

- `apps/desktop/src-tauri` (crate `sonarithm-desktop`): four generic commands (`ipc_dispatch`, `ipc_binary`, `ipc_subscribe`, `ipc_unsubscribe`) and an event router; a navigation guard; WebView2's browser accelerator keys turned off; a `--smoke` mode for CI.
- `apps/desktop/src`: Vite + React 19 shell, `createTauriTransport` (the `IpcTransport` over Tauri), the page-level accelerator guard, the Fluent catalogues bundled from `sonarithm-i18n`, and the empty workspace.
- `cargo xtask check-deps`: rule 6 (thin host). CI: the `desktop` job; the other Rust jobs skip the host, which needs WebKitGTK on Linux.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Four generic commands instead of one per request | ADR-0017 left the shape to B01-T03; generic handlers keep the command list in the app core, so an Electron host repeats nothing | 09 §2 |
| `unsafe` COM calls in the host for the accelerator setting | Tauri 2.12 does not expose it (F-170) | [ADR-0025](../../decisions/0025-unsafe-register-desktop-host.md), accepted by the coordinator |
| The host stays out of the existing Rust CI jobs and gets its own | WebKitGTK and the UI build are only needed there; the other jobs stay as fast as before | ci.yml comments |

## Findings to log

- F-170: Tauri 2.12 cannot turn off WebView2's browser accelerator keys, although wry can.

## Next steps

- [x] Coordinator: ADR-0025 accepted (2026-10-29; numbered 0025, since 0023 and 0024 are reserved).
- [ ] B02-T03 (frontend test infrastructure), then B02-T04 and T05, then B03-T04.
- [ ] Owner: the NVDA and Narrator walkthroughs on a nightly build (B01-T09) before any component sign-off.
