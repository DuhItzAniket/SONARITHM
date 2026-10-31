// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * `@sonarithm/viz`: plots for the SONARITHM desktop app (docs/architecture/09-desktop-app.md
 * §3): the uPlot line plot as a named, described object (B04-T01), the series encoding and
 * the perceptual colormaps, and the dense layer for waveforms, spectrograms and million-point
 * lines on WebGL2 with a Canvas2D fallback (B04-T02), and the playhead extrapolated from
 * clock anchors outside React (B04-T04).
 */
export { type ColormapName, colormapColor, colormapLut } from "./colormaps.ts";
export { DenseLayer, type DenseLayerProps } from "./DenseLayer.tsx";
export {
  type LayerRenderer,
  createRenderer,
  cssColor,
} from "./dense/renderer.ts";
export {
  type DenseScene,
  type DenseSeries,
  type Range,
  type Rgb,
  normalize,
} from "./dense/scene.ts";
export {
  Plot,
  type PlotCursor,
  type PlotProps,
  type PlotRange,
  type PlotSeries,
  type PlotSummary,
} from "./Plot.tsx";
export { Playhead, type PlayheadProps } from "./Playhead.tsx";
export { type PlayheadAnchor, PlayheadClock } from "./playhead.ts";
export { DASHES, dashFor, seriesToken } from "./series.ts";
