# 2026-10-26: Stage C docs groundwork (RFC-0004, clean room, AXONOTE licences)

> **Date(s):** 2026-10-26. **Phase:** C01, C02, C04. **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [implementation plan §4](../../planning/implementation-plan.md#4-stage-c--bring-any-chart-task-level) · [Stage C planning](2026-10-26-stage-c-planning.md)

## Context and phase

- Phase: the docs-only roots of Stage C's Lane R: C01-T01 ([#290](https://github.com/DuhItzAniket/SONARITHM/issues/290)), C02-T01 ([#303](https://github.com/DuhItzAniket/SONARITHM/issues/303)), C04-T01 ([#323](https://github.com/DuhItzAniket/SONARITHM/issues/323)).
- PRs: [#360](https://github.com/DuhItzAniket/SONARITHM/pull/360) (RFC-0004), [#361](https://github.com/DuhItzAniket/SONARITHM/pull/361) (clean room, ADR-0022), and the C04-T01 PR that carries this entry.
- Starting point: the Stage C plan had just merged (#359). Docs only: no cargo, pnpm or maturin command ran, because the owner was using the laptop for other heavy work. Crate licences were read from the crates.io API.

## Goals

- Write RFC-0004 (spec v0.2) and send it to the coordinator for a decision.
- Write the clean-room procedure for the raster digitizer, with a way to record sources and check PRs.
- Record the licences of AXONOTE's candidate crates and decide where its live sinks live; lay out OD-21 and OD-22 for the owner without resolving them.

## What was done

- **RFC-0004** ([docs/rfcs/0004-spec-v0.2.md](../../rfcs/0004-spec-v0.2.md)), accepted by the coordinator with three amendments and recorded in ADR-0021: shapes for `template`, `instruments` and `lint`; a typed `chart.calibration`; `chart.corrections`; AXONOTE's scale and tuning vocabularies; 14 proposed diagnostic codes; a 0.1 → 0.2 migration that changes only `specVersion`; 15 unresolved questions with proposals.
- **Clean room** ([ADR-0022](../../decisions/0022-clean-room-raster-digitizer.md), proposed): the procedure in the supply-chain policy, the [sources register](../../research/clean-room/raster-digitizer-sources.md) (CR-01 to CR-06, two licence-fact entries), a "Clean-room sources" line in the PR template, and the image and imageproc licence check.
- **AXONOTE:** licence tables for rustysynth, midir, rosc and the Standard MIDI File candidates; OD-21 and OD-22 laid out neutrally; live MIDI and the OSC socket placed in a new leaf crate `sonarithm-axonote-live` (docs 02 and 07).

## Decisions

| Decision | Why | Record |
|---|---|---|
| Propose that a v0.2 reader reads `"0.1"` documents as they are, without migration or warning | Keeps every v0.1 spec hash valid (render manifests, A08 study logs) and follows 04 §7's "a document declares the minimum version it needs" | RFC-0004 Q1, for the coordinator |
| Hold RFC-0004's PR (no auto-merge) until the coordinator decides | The RFC process merges an RFC with its final status | #360, merged as Accepted |
| Coordinator's amendments: every committed spec stays valid under v0.2 reading; hearing safety is not lint; `corrections.path` reserved for RFC-0005 | Protects existing specs and hearing safety; keeps RFC-0005's format open | RFC-0004 resolution, ADR-0021 |
| Documentation-only clean room with a register, not a two-team clean room | The project has one maintainer; the documented WPD workflow covers what C02 needs | ADR-0022 (proposed) |
| Live MIDI and OSC sockets in `sonarithm-axonote-live`, encoding in `sonarithm-axonote` | Engine crates stay pure and build for `wasm32` (02 §3 rule 4); follows the `sonarithm-plotsona-ml` leaf-crate pattern; `check-deps` can restrict `midir` like `cpal` | 02 §2–§4; no ADR needed: placement within the existing rules |
| The MIDI file writer belongs to AXONOTE, not `sonarithm-export` | Glide policy is engine logic; export only writes bytes to disk. 02 listed midly under export, 07 under AXONOTE | 02 §2, 07 AXONOTE |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| ADR-0021 is reserved twice: by the plan for OD-25's static-parsing ADR and by the coordinator's brief for RFC-0004's ADR | Two numbering plans written on the same day | The coordinator chose: RFC-0004's ADR is ADR-0021, the clean-room ADR keeps ADR-0022, and OD-25's static-parsing ADR becomes ADR-0023 (plan, OD-25 and issue #295 updated) |
| C01-T01's acceptance needs code (schema, migration, doc 04 at v0.2, tests) | The task mixes the RFC with its implementation | The docs half is done here; #290 stays open for a builder after acceptance |
| Docs 02 and 07 disagreed on where the MIDI file writer lives | Baseline drift | Resolved in C04-T01 (above) |

## Next steps

- [x] Coordinator: RFC-0004 decided (accepted with three amendments); ADR-0021 written in #360.
- [ ] Coordinator: accept ADR-0022 before C02-T03's code.
- [ ] Owner (#129): OD-21 (MIDI file library) and OD-22 (SoundFont bundling), laid out in the [supply-chain policy](../../engineering/supply-chain-and-licensing.md#owner-decisions-for-c04).
- [ ] Builder: C01-T01's code half (spec types, the 0.2 schema, the migration step, doc 04 at v0.2, the tests in RFC-0004 §11) after acceptance.
- [ ] C04-T06: add `midir` to `check-deps`' restricted crates for `sonarithm-axonote-live`. C02-T02: check imageproc's `wasm32` build (TODO(verify)).
