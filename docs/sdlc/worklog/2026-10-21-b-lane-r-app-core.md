# 2026-10-21: Stage B Lane R, app core and SONOTRACE

> **Date(s):** 2026-10-21. **Phase:** B01, B03, B04, B05, B06 (Lane R). **Author:** AI agent (Claude Code) in the app-core and SONOTRACE session, reviewed by the coordinator session.
> **Related:** [worklog conventions](README.md) · [implementation plan §3](../../planning/implementation-plan.md#3-stage-b--accessible-desktop-task-level) · [ADR-0017](../../decisions/0017-ipc-protocol-v1.md)

## Context and phase

- Phase: Stage B, Lane R ([plan §3.1](../../planning/implementation-plan.md#31-how-stage-b-runs)): the Rust-only tasks of B01, B03-T07/T08, B04-T03, B05-T01/T04 and B06.
- Issue(s) / PR(s): [#191](https://github.com/DuhItzAniket/SONARITHM/issues/191) (B01-T01), ADR PR [#259](https://github.com/DuhItzAniket/SONARITHM/pull/259).
- Starting point: Stage A complete on `main`; Stage B planned in #258. OD-15 closed on the condition that the Tauri host stays a thin adapter over `sonarithm-app`.

## Goals

- Build the headless app core first, with the IPC protocol recorded as an ADR before it is used.
- Then the typed IPC codegen, the network-deny harness, and the SONOTRACE and project-format tasks in plan order.

## What was done

- ADR-0017, IPC protocol v1 (#259): one `Request`/`Reply`/`Event` set in `sonarithm-app::ipc`, `App::dispatch` as the only entry point, typed errors with `SNR-APP` codes, the `hello` version handshake, the binary view header, and the layering edge `sonarithm-app → sonarithm`.
- B01-T01: `sonarithm-app` with datasets, the spec and its undo history, compile reports, the transport on its own control thread, export jobs, settings and devices; 16 headless scenarios on the null device, including one that no reply, error or event holds an absolute path (ADR-0017 §7).

## Decisions

| Decision | Why | Record |
|---|---|---|
| One request enum in the app core, dispatched by one function | Thin host (OD-15); tests drive the exact protocol a host exposes | [ADR-0017](../../decisions/0017-ipc-protocol-v1.md) |
| The app core owns the control thread that drives A05's `Player` | `Player` is not `Send` (F-140) | ADR-0017 §1 |
| Settings v0 is the unversioned flat layout | No release wrote settings yet; v0 keeps the migration chain tested from the first version | No ADR needed: settings are not a contract in rfcs/README §1 |
| Export paths from the UI must be absolute `.wav` or `.flac` paths in an existing folder | 09 §4: no path traversal from UI strings | No ADR needed: local choice |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| `Player<B>` cannot live in shared host state | The device stream is not `Send` | A control thread inside the app core (F-140) |

## Findings to log

- F-140: A05's `Player` cannot move between threads, so the app core owns the thread that drives it — [findings log](../../research-paper/findings-log.md)

## Next steps

- [ ] B01-T02 typed IPC (ts-rs) and B01-T06 network-deny harness (agent).
- [ ] B03-T07, B03-T08 once AUDAXIS's announcement composer (B03-T03) lands (agent).
- [ ] RFC-0002 project format v1 before B05-T04 (agent; the coordinator approves).
