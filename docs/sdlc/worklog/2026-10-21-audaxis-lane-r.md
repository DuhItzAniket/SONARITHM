# 2026-10-21: AUDAXIS Lane R and the message catalogues

> **Date(s):** 2026-10-21. **Phase:** B02 (T08), B03 (T01–T03, T05, T06, T11). **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [implementation plan §3](../../planning/implementation-plan.md#3-stage-b--accessible-desktop-task-level) · [12 §5](../../architecture/12-cross-cutting-concerns.md#5-internationalization-and-localization) · [08](../../architecture/08-interaction-and-accessibility.md)

## Context and phase

- Phase: B02 and B03, Lane R (Rust and framework-agnostic work that needs no owner step).
- Issue(s) / PR(s): B02-T08 [#207](https://github.com/DuhItzAniket/SONARITHM/issues/207); B03-T01 [#211](https://github.com/DuhItzAniket/SONARITHM/issues/211), T02 [#212](https://github.com/DuhItzAniket/SONARITHM/issues/212), T03 [#213](https://github.com/DuhItzAniket/SONARITHM/issues/213), T05 [#215](https://github.com/DuhItzAniket/SONARITHM/issues/215), T06 [#216](https://github.com/DuhItzAniket/SONARITHM/issues/216), T11 [#221](https://github.com/DuhItzAniket/SONARITHM/issues/221).
- Starting point: Stage A complete on `main` (`c44dd44`); no message catalogue, no AUDAXIS crate. A second session builds the app core and SONOTRACE (B01, B06) in parallel.

## Goals

- B02-T08: Fluent catalogues shared by Rust and the UI, locale formatting, the pseudo-locale check and the hard-coded string lint.
- B03-T01 to T03: the AUDAXIS crate, NavTree, cursor and verbs, the announcement composer and queue.
- B03-T05, T06, T11: context tracks, the earcon catalogue, stereo placement and the never-pan-alone guard.

## What was done

- B02-T08: `crates/sonarithm-i18n` (Fluent catalogues `common`, `ui` and `units` in `locales/en-US/`; ICU4X numbers and dates; 26 units with symbol, terse and verbose forms; `en-XA` and `check_pseudo`) and `packages/i18n` (`createLocalizer`, the same pseudo-locale, the `sonarithm/no-literal-ui-string` rule). The frontend CI job now runs `pnpm -r test`. Merged as #260.
- B03-T01: `sonarithm_core::nav` (NavTree types and invariant check), `ContextProvider::nav_tree`, `Compiled::nav` in the compiler, and `crates/engines/sonarithm-audaxis` with the tree builder; snapshots for the six NUMISONA templates, property tests and OD-18. Merged as #262.
- B03-T02: `nav::Cursor` with the verbs of keyboard §4.2 and §4.3, outcomes (moved, boundary, invalid, describe, toggled) with their earcon IDs, and where am I; a verb-by-level matrix snapshot reviewed against §4.3 row by row. Merged as #264.
- B03-T11: `spatial::Spatial` shaper and the A11Y-14 guard (`SNR-ENG-100` to `103`, AUDAXIS's first code table), the mono-downmix conformance test and the `stereo-placement` golden case.
- B03-T03: `announce::Composer` (parts as catalogue messages joined by `announce-join`, spoken and display forms, mandatory qualifiers, where am I, pause) and `AnnouncementQueue` (§9.1 classes, supersede, priority, history); `announce.ftl` joins the shared catalogue, with contract cases; `XType::Year` so years are spoken without grouping.

## Decisions

| Decision | Why | Record |
|---|---|---|
| A new crate `sonarithm-i18n` (engines may depend on it) and a new package `@sonarithm/i18n` | Announcements (AUDAXIS), the app core and the UI all need the same catalogues; putting Fluent and ICU4X in `sonarithm-core` would make every crate compile them | 02 §1–§3 updated in the B02-T08 PR (02's rule for new crates); `check-deps` layer added |
| Catalogue files live in the crate (`crates/sonarithm-i18n/locales/`) and the UI reads them from there | `include_str!` must stay inside the crate for `cargo publish`; the UI build can read any workspace path | 12 §5 |
| Numbers follow `Intl.NumberFormat`'s digit rules in Rust (half-expand from shortest round-trip digits, three fraction digits by default), and plural forms use the rounded value | Rust and the UI must print the same digits (DL-P12); fluent-rs ignores maximum fraction digits when choosing plurals | F-130 |
| Bidi isolation marks off on both sides | Full RTL is post-1.0 (12 §5); the marks reach speech synthesizers as characters | 12 §5 |
| Dates are formatted in Rust only (Gregorian), not through Fluent's `DATETIME` | fluent-rs has no `DATETIME`; the UI shows data values from the Rust formatter (DL-P12) | 12 §5 |
| NavTree types in `sonarithm-core`, built through `ContextProvider::nav_tree` | Engines talk through core types (02 §3 rule 2), and ADR-0014 already foresaw the tree joining `ContextProvider` as an additive change | 05 §3, §8 |
| AUDAXIS stays out of the facade's default engines until B03-T05 | Every resolved spec has a `context` section, so registering AUDAXIS before its context tracks exist would drop warning `SNR-COMP-023` from every compile and churn CLI snapshots twice | status.md B03 tasks |
| Points are rows read from the spec and data (filter, time field, x order), not from Score events | Glide events carry only a bounding row range; onsets come from the Score afterwards | 07 AUDAXIS |
| Even segments: at least 8 points, at most 12 per series, provisional | No design rule or source sets a size; recorded as an open decision instead of a default with a rule ID | OD-18 |
| N and Shift+N land on the insight's point, not the insight node | "Next insight in the current series" needs the cursor to stay in the series; the move is flagged as a landmark | keyboard §4.2 |
| The A11Y-14 guard checks the finished Score, not the spec | It then covers every encoder; NUMISONA's spec-level `SNR-ENG-052` already covers its own tracks, so AUDAXIS skips a track NUMISONA warned about | 07 AUDAXIS |
| Pitch extremes use SDR-08's expanded range (65–1480 Hz) as the line | It is the evidence-backed range; no rule gives a separate hearing-loss limit, so no new number was invented | `SNR-ENG-102` |
| Announcement parts joined through a catalogue message (`announce-join`), right to left | Part order is configurable (A11Y-12), so no single message can hold every order; nesting keeps each announcement one message for the pseudo-locale check | 08 §4 |
| Every announcement has a spoken and a display form | Speech uses unit names, the log, readout and braille use symbols (keyboard §8.1); one formatter gives both the same value and precision (DL-P12) | 08 §4 |
| Derived and synthetic values carry qualifiers too | 03 §4 rule 3 states numbers plainly only for exact, vector and user-edited values | 03 §4 |
| B02-T08 built before B01-T02 (its listed dependency) | It needs no generated IPC type: the catalogues are plain files and the contract test reads them directly | status.md B02 tasks |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| ICU4X formatters cannot go in Fluent's thread-safe memoizer | ICU4X data payloads use `Rc` unless `icu_provider`'s `sync` feature is on | `icu_provider` with `sync` |
| The guard saw every track's timing as varying | Note durations and tap counts were added to one range | One range per property; tests with handcrafted Scores |
| A point's x value vanished in a JSON round trip | `NavNode` flattens its kind, so the node's `x` extent and the point's `x` value shared one key | The node field is `span`; a round-trip unit test guards it |
| Bar charts had no x labels in the tree | The bar template orders by row and names no time field | A field with role `category` or `label` names the points when there is no time field |

## Findings to log

- F-130: ICU4X 2.3 and V8's `Intl` agree on all 24 contract cases once plurals use the rounded value — [findings log](../../research-paper/findings-log.md)

## Next steps

- [ ] B03-T05, T06, T11: context tracks, earcon set, stereo placement (agent).
- [ ] Owner: when `sonarithm-i18n` first ships in a release, publish it once by hand so crates.io trusted publishing can be set up for it ([release process](../../engineering/release-process.md)); checklist #129.
