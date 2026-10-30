// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * `@sonarithm/viz`: plots for the SONARITHM desktop app (docs/architecture/09-desktop-app.md
 * §3): the uPlot line plot as a named, described object (B04-T01), the series encoding and
 * the perceptual colormaps. The WebGL2 layer (B04-T02) and the playhead (B04-T04) follow.
 */
export { type ColormapName, colormapColor, colormapLut } from "./colormaps.ts";
export {
  Plot,
  type PlotProps,
  type PlotSeries,
  type PlotSummary,
} from "./Plot.tsx";
export { DASHES, dashFor, seriesToken } from "./series.ts";
