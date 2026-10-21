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

- B02-T08: `crates/sonarithm-i18n` (Fluent catalogues `common`, `ui` and `units` in `locales/en-US/`; ICU4X numbers and dates; 26 units with symbol, terse and verbose forms; `en-XA` and `check_pseudo`) and `packages/i18n` (`createLocalizer`, the same pseudo-locale, the `sonarithm/no-literal-ui-string` rule). The frontend CI job now runs `pnpm -r test`.

## Decisions

| Decision | Why | Record |
|---|---|---|
| A new crate `sonarithm-i18n` (engines may depend on it) and a new package `@sonarithm/i18n` | Announcements (AUDAXIS), the app core and the UI all need the same catalogues; putting Fluent and ICU4X in `sonarithm-core` would make every crate compile them | 02 §1–§3 updated in the B02-T08 PR (02's rule for new crates); `check-deps` layer added |
| Catalogue files live in the crate (`crates/sonarithm-i18n/locales/`) and the UI reads them from there | `include_str!` must stay inside the crate for `cargo publish`; the UI build can read any workspace path | 12 §5 |
| Numbers follow `Intl.NumberFormat`'s digit rules in Rust (half-expand from shortest round-trip digits, three fraction digits by default), and plural forms use the rounded value | Rust and the UI must print the same digits (DL-P12); fluent-rs ignores maximum fraction digits when choosing plurals | F-130 |
| Bidi isolation marks off on both sides | Full RTL is post-1.0 (12 §5); the marks reach speech synthesizers as characters | 12 §5 |
| Dates are formatted in Rust only (Gregorian), not through Fluent's `DATETIME` | fluent-rs has no `DATETIME`; the UI shows data values from the Rust formatter (DL-P12) | 12 §5 |
| B02-T08 built before B01-T02 (its listed dependency) | It needs no generated IPC type: the catalogues are plain files and the contract test reads them directly | status.md B02 tasks |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| ICU4X formatters cannot go in Fluent's thread-safe memoizer | ICU4X data payloads use `Rc` unless `icu_provider`'s `sync` feature is on | `icu_provider` with `sync` |

## Findings to log

- F-130: ICU4X 2.3 and V8's `Intl` agree on all 24 contract cases once plurals use the rounded value — [findings log](../../research-paper/findings-log.md)

## Next steps

- [ ] B03-T01: `sonarithm-audaxis` and the NavTree (agent).
- [ ] Owner: when `sonarithm-i18n` first ships in a release, publish it once by hand so crates.io trusted publishing can be set up for it ([release process](../../engineering/release-process.md)); checklist #129.
