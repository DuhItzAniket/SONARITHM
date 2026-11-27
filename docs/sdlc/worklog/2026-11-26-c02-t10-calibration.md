# 2026-11-26 to 2026-11-28: C02-T10 chart image calibration, and #511

> **Date(s):** 2026-11-26 to 2026-11-28. **Phase:** C02 (C02-T10), plus maintenance (#511). **Author:** AI agent (Claude Code, Claude Opus 5.5) in the PLOTSONA session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [ADR-0017 "Raster calibration"](../../decisions/0017-ipc-protocol-v1.md) · [keyboard §5.11](../../design/keyboard-and-navigation.md) · [F-241](../../research-paper/findings-log.md)

## Context and phase

- **Phase:** C02, task C02-T10, the accessible calibration and point-entry dialog ([roadmap](../../planning/roadmap.md)).
- **Branch:** `feat/c02-t10-calibration`. One PR carries T10 and the #511 fix, following the owner's rule to build big parts locally and push rarely.
- **Starting point:** the app core could not open PNG or JPEG charts, and nothing in the app set a calibration. The coordinator approved the design (ADR-0017 amendment) under the owner's delegation earlier the same day.

## Goals

- **The app core:** raster calibration sessions (`chart_raster_open`, `chart_calibration_check`, `chart_calibration_apply`, `chart_raster_close`, `SNR-APP-096`).
- **The UI:** the calibration dialog in the desktop app, React Aria only.
- **Checks and PR:** all local checks pass, then one PR.

## What was done

- **Rust:**
  - PLOTSONA `raster::preview`: `inspect` and `check`;
  - the facade's `sonarithm::raster`;
  - the app core's `app/raster.rs`, with its IPC types, `SNR-APP-096`, re-extraction through a stored calibration, and the integration and contract tests.
- **UI:** the `@sonarithm/ui` `ImageCrosshair`; the desktop app's `calibrate/` dialog, with its unit tests, mock-host handlers, E2E spec, palette entry and strings.
- **Docs:** the ADR-0017 "Raster calibration" section, keyboard §5.11, the status and traceability rows.
- **Fixture:** `crates/sonarithm-app/tests/fixtures/raster/d3-line@1x.calibration.json`, generated once from PLOTSONA's `svg::hand_calibration` of `d3-line.svg` at 1x through a temporary test that was not kept.
- **The slot run:**
  - merged `origin/main` (after #503); resolved the conflicts with the relink commands in `app.rs`, `codes.rs`, `typescript.rs`, `contract.rs` and ADR-0017 by keeping both;
  - fixed the first build's errors (#503's new `Open::file` field; three clippy or borrow errors in tests);
  - `cargo clippy -p sonarithm-plotsona -p sonarithm -p sonarithm-app --all-targets -- -D warnings` is clean.
  - `cargo test --no-fail-fast -p sonarithm-plotsona -p sonarithm -p sonarithm-app` (2 threads) passes, the contract test included.
- **Corrections found by the first test run:**
  - **No plot area for the line chart:** the d3 line chart's axis lines are not found, so its `plotArea` is `null`. The tests now check `d3-bar@1x.png`, which has one.
  - **Not-image bytes:** text bytes are refused with `SNR-ENG-300`, not `327`. ADR-0017 and `inspect`'s documentation now say so.
  - **A `.svg` file:** the app core refuses it by name (`SNR-APP-060`) before it is read.

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| `c02_t10_a_chart_image_opens…` expected a plot area | `find_axes` finds no plot area on any d3-line, plotly or vl-line/bar/pie/scatter image, only on d3-bar, d3-scatter and vl-area | Assert `null` for d3-line and an area for d3-bar. vl-area's area (`min [419, 33]`) looks like a misdetection; that is for C02-T02, not this task |
| The refusal test expected `SNR-ENG-327` for text bytes | `sniff` refuses bytes no chart format starts with as `SNR-ENG-300` | Test and docs say 300 |
| The refusal test expected `SNR-ENG-327` for `chart.svg` | The app core checks the extension first | Test expects `SNR-APP-060` |

## Resumed 2026-11-28

- **Codes and codegen:** done; the generated TS protocol gains the raster types.
- **Frontend fixes:**
  - lint and type errors in the mock host and the dialog;
  - the axis name in three messages is now `$axisLabel`, because `$axis` is raw data in other messages and the pseudo-locale check caught a Latin letter;
  - the crosshair's click story now expects the pixel under the whole-pixel coordinate it sends.
- **A real announcement bug:** the live region speaks only the latest polite message, so the checks that followed "y3 deleted." replaced it. The checks are now said when a new check result changes them, after the anchor action that caused it.
- **WebKit E2E:** keys were pressed before the app and the palette were ready, and WebKit ignores Primary+A in these fields. The spec now waits for both, and selects a field's text before typing over it.
- **#511 (approved by the coordinator):**
  - the readers drop columns the data fingerprint cannot encode, with an `SNR-DATA-016` warning; a file with nothing else is refused with `SNR-DATA-007`;
  - the compiler stops instead of recording a zero fingerprint;
  - the fuzz check is strict again;
  - 03 §3.1, the CHANGELOG and F-241 are updated.
- **Local checks, all passing:**
  - fmt, check-deps, clippy and the workspace tests;
  - rustdoc, wasm32, cargo deny, golden and replication (91 specs);
  - the fuzz crate builds;
  - frontend lint, typecheck and tests;
  - the crosshair stories with axe in four themes;
  - E2E in Chromium and in WebKit, 61 tests each.

## Next steps

- [ ] An NVDA pass over the dialog on the real host (owner, HP-22); the PR lists what to listen for.
- [ ] C02-T11, the correction view (PLOTSONA, next batch).
- [ ] After T05 merges: the `dialog:allow-open` follow-up (design note in the shared notes).
