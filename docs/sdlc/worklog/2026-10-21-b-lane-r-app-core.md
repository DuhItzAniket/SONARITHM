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
- B05-T04: `.sonarithm` project bundles per RFC-0002 (accepted with conditions A and B) and ADR-0018: reader and writer, autosaves and recovery, the `project_*` commands, a memory test for condition B (F-141) and the `project_open` fuzz target.
- B03-T07: speech and sonification sequencing in the app core (`speech::Sequencer` on the control thread), the −20 dB ducker in the render graph, `sonarithm-speech` with the opt-in `tts` engine, sequenced WebVTT cues, `announce_repeat` and the `announcement` event.
- B06-T05: `sonotrace::rate_limit` and the stream commands of the app core (offer, connect with the approval's token, pause, resume, hide, stop, list), with notices from a new `stream.ftl` catalogue.
- B03-T08: profiles per ADR-0020 (accepted with the condition that polarity goes into the effective spec): the document, presets, layers, the effective spec, the profile commands, the project profile, and the hash round-trip test.

## Decisions

| Decision | Why | Record |
|---|---|---|
| One request enum in the app core, dispatched by one function | Thin host (OD-15); tests drive the exact protocol a host exposes | [ADR-0017](../../decisions/0017-ipc-protocol-v1.md) |
| The app core owns the control thread that drives A05's `Player` | `Player` is not `Send` (F-140) | ADR-0017 §1 |
| Settings v0 is the unversioned flat layout | No release wrote settings yet; v0 keeps the migration chain tested from the first version | No ADR needed: settings are not a contract in rfcs/README §1 |
| Export paths from the UI must be absolute `.wav` or `.flac` paths in an existing folder | 09 §4: no path traversal from UI strings | No ADR needed: local choice |
| Project replies carry file names and the recent list opens by index, not by path as RFC-0002 §9 sketched | ADR-0017 §7: no folder crosses the protocol | Reported to the coordinator; RFC-0002 §9 is additive and unchanged in substance |
| The Parquet structural check streams page headers instead of loading each column chunk | Condition B: no copy of a member in memory | F-141 |
| The sequencer runs on the transport's control thread | That thread owns the player, so pausing and ducking need no extra channel; time is passed in, so the rules are tested without a clock | 08 §4 as built |
| Self-voicing is an opt-in `system` feature of `sonarithm-speech` | Headless builds and CI need no speech engine; Linux needs libspeechd-dev only where self-voicing ships | 02 §2 |
| The app core registers AUDAXIS through a facade opt-in, not as a facade default | The default changes every Score's header (F-142); making it the default is B03's decision | F-142 |
| Speech durations are estimated at 180 words per minute unless the listener sets a rate | No documented rate; estimates err long, so speech is not talked over | TODO(verify) in B07 |
| A stream connects only with a one-time token from `stream_offer` | ADR-0012's per-action consent through the protocol: the host can only connect what the user was shown | 07 as built |
| Profiles never reach the compiler; the app core writes them into the effective spec | Scores stay reproducible from the manifest (ADR-0013); the coordinator's condition on ADR-0020 | [ADR-0020](../../decisions/0020-profile-format-v1.md) |
| Marks preference applies through `spec_quick` | A spec always names its mark, so the preference can only choose where a spec is generated | ADR-0020 §3 |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| `Player<B>` cannot live in shared host state | The device stream is not `Send` | A control thread inside the app core (F-140) |

## Findings to log

- F-140: A05's `Player` cannot move between threads, so the app core owns the thread that drives it — [findings log](../../research-paper/findings-log.md)
- F-141: reading an embedded Parquet member in place cuts the peak memory of opening it by 1.6 times the member's size — [findings log](../../research-paper/findings-log.md)
- F-142: registering AUDAXIS changes every Score's header, so the app's Scores differ from the CLI's until AUDAXIS is a default engine — [findings log](../../research-paper/findings-log.md)

## Next steps

- [ ] B01-T02 typed IPC (ts-rs) and B01-T06 network-deny harness (agent).
- [ ] B03-T07, B03-T08 once AUDAXIS's announcement composer (B03-T03) lands (agent).
- [ ] RFC-0002 project format v1 before B05-T04 (agent; the coordinator approves).
