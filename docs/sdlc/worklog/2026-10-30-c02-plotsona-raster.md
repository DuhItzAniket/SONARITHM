# 2026-10-30: C02 PLOTSONA raster digitizer

> **Date(s):** 2026-10-30. **Phase:** C02. **Author:** coding agent (Claude Code), written by an AI agent, reviewed by the coordinator session.
> **Related:** [worklog conventions](README.md) · [C02 tasks](../status.md#c02-tasks) · [implementation plan §4](../../planning/implementation-plan.md#c02--plotsona-raster-digitizer-with-human-correction) · [clean-room register](../../research/clean-room/raster-digitizer-sources.md)

## Context and phase

- Phase: C02, PLOTSONA raster digitizer — [roadmap entry](../../planning/roadmap.md)
- Issue(s) / PR(s): C02-T02 [#304](https://github.com/DuhItzAniket/SONARITHM/issues/304), C02-T03 [#305](https://github.com/DuhItzAniket/SONARITHM/issues/305), C02-T04 [#306](https://github.com/DuhItzAniket/SONARITHM/issues/306), C02-T05 [#307](https://github.com/DuhItzAniket/SONARITHM/issues/307)
- Starting point: C01 built (T01 to T12); ADR-0022 accepted (clean room, documentation only). The same PLOTSONA build session continues on the shared laptop (4 build jobs, no local soaks or benchmarks).

## Goals

- C02-T02 to T06, T08 and T09 in plan order, one PR per task. C02-T07 (OCR) waits on OD-23; T10 and T11 on Gate U.

## What was done

- C02-T02: `raster`, the raster core: PNG and JPEG decoding under a pixel budget, axis lines and the plot area, calibration from hand-entered anchors with the half-pixel uncertainty carried through; `SNR-ENG-327`; fuzz target `raster_image`. CR-01 (WPD's user documentation) read again and dated in the register.
- C02-T03: `raster::colour` (foreground and background filters, box and pen masks, automatic series colours) and `raster::lines` (averaging window, X step with natural-cubic-spline interpolation, custom independents); PNG and JPEG charts with a calibration extract as `cv_estimate` line charts with a confidence per value; category x axes calibrated from their labels; `SNR-ENG-328` without a calibration. The raster corpus: the 12 SVG charts rendered to PNG at 1x and 2x. The 18 raster line series read within 0.05 % to 0.15 % median error. CR-07 (the textbook natural cubic spline) added to the register.
- C02-T04: `raster::components` (8-connected labelling) and `raster::marks` (bars and histogram bins profiled by column, a blob detector with diameter limits, touching markers separated by k-means, fragments of hollow markers joined); each series colour classified as line, bars or points. The 6 bar renders read within 0.11 % median error; the 6 scatter renders count all 60 markers each. CR-08 added to the register.
- C02-T05: the 10 §3.1 checks on raster readings, as diagnostics only (monotonic ticks refused, imprecise fit and log-against-linear warnings from C01-T04, values beyond the outermost ticks, the series count against a caller-given legend, `SNR-ENG-329`); the calibration's confidence folded into each value's, documented as uncalibrated; a raster table taken through filter, sort and aggregate keeps its class and confidences.

## Decisions

| Decision | Why | Record |
|---|---|---|
| `image` with PNG and JPEG decoders only; no `imageproc` yet | Nothing in the raster core needs imageproc's algorithms; fewer crates to fuzz and no `getrandom` on wasm32 | supply-chain C02 notes |
| The pixel budget is checked from the header, then the decoder's allocation limit is set from it | A bomb is refused before any buffer exists, and a decoder that allocates more than the header implies still stops | `raster::decode` docs |
| Axis lines: the lowest and leftmost dark lines spanning half the image | WPD has the user mark the axes (CR-01); this finds them on clean charts and is overridden by a `plotArea` in the calibration. Our design, documented as such | 10 §3 "As built (C02-T02)" |
| Each automatic series colour gets a threshold of at most half its distance to the nearest grey and to the other series | With the default 70, Vega's blue (76, 120, 168) also matched its grey axis line (136, 136, 136) at 69.9, and the line was read along the axis (median error 45 %); with the separating threshold it reads at 0.13 % | `raster::colour::separating_threshold` |
| A category x axis is calibrated from its labels as anchors | The user names the categories (no OCR); sampling at the labels' positions is the documentation's custom independents, and the values land on the categories the exact tiers report | 10 §3 "As built (C02-T03)" |
| Bars are recognised by standing on the baseline, not by filling their box | Touching histogram bins of different heights fill only part of their joint box, and a flat line fills its box too; a bar's base on the value-0 row (clamped into the plot) separates bars from lines and markers | `raster::extract::classify` |
| Pieces of one hollow marker are joined | Anti-aliasing broke one of Vega-Lite's hollow circles into three arcs, which counted as three markers (62 for 60); pieces under half the median marker area within a marker's diameter are one marker | `raster::marks::markers` |
| The legend for the series-count check is an `ExtractOptions` field, not a spec field | Spec v0.2 has no legend; adding one is a contract change (AGENTS rule 11) that the correction UI (C02-T10, T11) is better placed to propose | 10 §3.1 "As built (C02-T05)" |
| Every value carries the range half a pixel either side | A mark drawn into a pixel can lie anywhere in it; the range is the uncertainty a raster cannot remove, and the confidence model (C02-T05) builds on it | `raster::Estimate` |

## Clean room

- Sources used: CR-01 (WPD user documentation), CR-07 (natural cubic spline, textbook), CR-08 (connected components and k-means, textbook). WebPlotDigitizer's and Engauge's source code were not opened. Exposures: none.

## Findings to log

- F-180: raster line charts within 0.15 % median error at 1x and 2x (18 series).
- F-181: a palette colour (Vega's blue) sat inside the default colour distance of the grey axis line; series thresholds now separate them.
- F-182: raster bars within 0.11 % median error, every scatter marker counted, positions within 0.11 %.

## Next steps

- [x] C02-T03: colour masks and line extraction (averaging window, X step with interpolation) from CR-01
- [x] C02-T04: bar, histogram and point detection
- [x] C02-T05: sanity checks and the confidence model
- [ ] C02-T06: RFC-0005 first, then the corrections
