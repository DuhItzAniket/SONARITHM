# 2026-10-30: C02 PLOTSONA raster digitizer

> **Date(s):** 2026-10-30. **Phase:** C02. **Author:** coding agent (Claude Code), written by an AI agent, reviewed by the coordinator session.
> **Related:** [worklog conventions](README.md) · [C02 tasks](../status.md#c02-tasks) · [implementation plan §4](../../planning/implementation-plan.md#c02--plotsona-raster-digitizer-with-human-correction) · [clean-room register](../../research/clean-room/raster-digitizer-sources.md)

## Context and phase

- Phase: C02, PLOTSONA raster digitizer — [roadmap entry](../../planning/roadmap.md)
- Issue(s) / PR(s): C02-T02 [#304](https://github.com/DuhItzAniket/SONARITHM/issues/304), C02-T03 [#305](https://github.com/DuhItzAniket/SONARITHM/issues/305)
- Starting point: C01 built (T01 to T12); ADR-0022 accepted (clean room, documentation only). The same PLOTSONA build session continues on the shared laptop (4 build jobs, no local soaks or benchmarks).

## Goals

- C02-T02 to T06, T08 and T09 in plan order, one PR per task. C02-T07 (OCR) waits on OD-23; T10 and T11 on Gate U.

## What was done

- C02-T02: `raster`, the raster core: PNG and JPEG decoding under a pixel budget, axis lines and the plot area, calibration from hand-entered anchors with the half-pixel uncertainty carried through; `SNR-ENG-327`; fuzz target `raster_image`. CR-01 (WPD's user documentation) read again and dated in the register.
- C02-T03: `raster::colour` (foreground and background filters, box and pen masks, automatic series colours) and `raster::lines` (averaging window, X step with natural-cubic-spline interpolation, custom independents); PNG and JPEG charts with a calibration extract as `cv_estimate` line charts with a confidence per value; category x axes calibrated from their labels; `SNR-ENG-328` without a calibration. The raster corpus: the 12 SVG charts rendered to PNG at 1x and 2x. The 18 raster line series read within 0.05 % to 0.15 % median error. CR-07 (the textbook natural cubic spline) added to the register.

## Decisions

| Decision | Why | Record |
|---|---|---|
| `image` with PNG and JPEG decoders only; no `imageproc` yet | Nothing in the raster core needs imageproc's algorithms; fewer crates to fuzz and no `getrandom` on wasm32 | supply-chain C02 notes |
| The pixel budget is checked from the header, then the decoder's allocation limit is set from it | A bomb is refused before any buffer exists, and a decoder that allocates more than the header implies still stops | `raster::decode` docs |
| Axis lines: the lowest and leftmost dark lines spanning half the image | WPD has the user mark the axes (CR-01); this finds them on clean charts and is overridden by a `plotArea` in the calibration. Our design, documented as such | 10 §3 "As built (C02-T02)" |
| Each automatic series colour gets a threshold of at most half its distance to the nearest grey and to the other series | With the default 70, Vega's blue (76, 120, 168) also matched its grey axis line (136, 136, 136) at 69.9, and the line was read along the axis (median error 45 %); with the separating threshold it reads at 0.13 % | `raster::colour::separating_threshold` |
| A category x axis is calibrated from its labels as anchors | The user names the categories (no OCR); sampling at the labels' positions is the documentation's custom independents, and the values land on the categories the exact tiers report | 10 §3 "As built (C02-T03)" |
| Every value carries the range half a pixel either side | A mark drawn into a pixel can lie anywhere in it; the range is the uncertainty a raster cannot remove, and the confidence model (C02-T05) builds on it | `raster::Estimate` |

## Clean room

- Sources used: CR-01 (WPD user documentation), CR-07 (natural cubic spline, textbook). WebPlotDigitizer's and Engauge's source code were not opened. Exposures: none.

## Findings to log

- F-180: raster line charts within 0.15 % median error at 1x and 2x (18 series).
- F-181: a palette colour (Vega's blue) sat inside the default colour distance of the grey axis line; series thresholds now separate them.

## Next steps

- [x] C02-T03: colour masks and line extraction (averaging window, X step with interpolation) from CR-01
- [ ] C02-T04: bar, histogram and point detection
