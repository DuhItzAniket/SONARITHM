// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { DenseScene, Rgb } from "./scene.ts";
import { normalize } from "./scene.ts";
import type { LayerRenderer } from "./renderer.ts";

const css = (color: Rgb): string =>
  `rgb(${color.map((c) => String(Math.round(c * 255))).join(" ")})`;

/**
 * The Canvas2D fallback (B04-T02) for webviews without WebGL2. A dense line is reduced to
 * its first, minimum, maximum and last point per pixel column (M4, Jugel et al. 2014), which
 * draws the same pixels as the full line with a few thousand segments.
 */
export class CanvasRenderer implements LayerRenderer {
  readonly backend = "canvas2d" as const;
  draws = 0;
  readonly #context: CanvasRenderingContext2D;

  constructor(context: CanvasRenderingContext2D) {
    this.#context = context;
  }

  draw(scene: DenseScene): void {
    const ctx = this.#context;
    const { width, height } = ctx.canvas;
    ctx.clearRect(0, 0, width, height);
    switch (scene.kind) {
      case "lines":
        for (const series of scene.series) {
          const xs = normalize(series.x, scene.x);
          const ys = normalize(series.y, scene.y);
          ctx.strokeStyle = css(series.color);
          ctx.lineWidth = Math.max(1, Math.round(window.devicePixelRatio));
          ctx.beginPath();
          m4Path(ctx, xs, ys, width, height);
          ctx.stroke();
        }
        break;
      case "waveform": {
        const lows = normalize(scene.min, scene.y);
        const highs = normalize(scene.max, scene.y);
        const columns = lows.length;
        const step = width / Math.max(columns, 1);
        ctx.fillStyle = css(scene.color);
        for (let i = 0; i < columns; i += 1) {
          const top = (1 - (highs[i] ?? 0)) * height;
          const bottom = (1 - (lows[i] ?? 0)) * height;
          ctx.fillRect(
            i * step,
            top,
            Math.max(step, 1),
            Math.max(bottom - top, 1),
          );
        }
        break;
      }
      case "spectrogram": {
        const image = ctx.createImageData(scene.columns, scene.rows);
        for (let c = 0; c < scene.columns; c += 1) {
          for (let r = 0; r < scene.rows; r += 1) {
            const value = scene.magnitudes[c * scene.rows + r] ?? 0;
            // Row 0 is the lowest frequency, drawn at the bottom.
            const at = ((scene.rows - 1 - r) * scene.columns + c) * 4;
            image.data[at] = scene.lut[value * 3] ?? 0;
            image.data[at + 1] = scene.lut[value * 3 + 1] ?? 0;
            image.data[at + 2] = scene.lut[value * 3 + 2] ?? 0;
            image.data[at + 3] = 255;
          }
        }
        const buffer = document.createElement("canvas");
        buffer.width = scene.columns;
        buffer.height = scene.rows;
        buffer.getContext("2d")?.putImageData(image, 0, 0);
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(buffer, 0, 0, width, height);
        break;
      }
    }
    this.draws += 1;
  }

  destroy(): void {
    this.#context.clearRect(
      0,
      0,
      this.#context.canvas.width,
      this.#context.canvas.height,
    );
  }
}

/** Adds the M4 reduction of a polyline (normalized coordinates) to the current path. */
function m4Path(
  ctx: CanvasRenderingContext2D,
  xs: Float32Array,
  ys: Float32Array,
  width: number,
  height: number,
): void {
  let column = -1;
  let first = 0;
  let low = 0;
  let high = 0;
  let last = 0;
  let started = false;
  const flush = () => {
    if (column < 0) return;
    const px = column + 0.5;
    const py = (y: number) => (1 - y) * height;
    if (started) {
      ctx.lineTo(px, py(first));
    } else {
      ctx.moveTo(px, py(first));
      started = true;
    }
    ctx.lineTo(px, py(low));
    ctx.lineTo(px, py(high));
    ctx.lineTo(px, py(last));
  };
  for (let i = 0; i < xs.length; i += 1) {
    const x = xs[i] ?? 0;
    const y = ys[i] ?? 0;
    const c = Math.min(width - 1, Math.max(0, Math.floor(x * width)));
    if (c !== column) {
      flush();
      column = c;
      first = y;
      low = y;
      high = y;
    }
    low = Math.min(low, y);
    high = Math.max(high, y);
    last = y;
  }
  flush();
}
