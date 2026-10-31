// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import type { DenseScene, LayerRenderer } from "@sonarithm/viz";
import { DenseLayer, colormapLut, cssColor } from "@sonarithm/viz";
import { useEffect, useMemo, useRef, useState } from "react";
import { expect, waitFor, within } from "storybook/test";

// The dense layer (B04-T02) in every theme: WebGL2 where the browser has it, the Canvas2D
// fallback when asked, a waveform, a spectrogram and a million-point line. The summaries
// are data (from the NavTree), not UI strings.

/** A deterministic signal: two sines and seeded noise (xorshift32), as in the A02 spike. */
function signal(n: number): Float32Array {
  const y = new Float32Array(n);
  let s = 1729;
  for (let i = 0; i < n; i += 1) {
    s ^= s << 13;
    s ^= s >>> 17;
    s ^= s << 5;
    y[i] =
      0.6 * Math.sin(i / 900) +
      0.25 * Math.sin(i / 37) +
      0.1 * ((s >>> 0) / 2 ** 32 - 0.5);
  }
  return y;
}

/** First token colour of the series palette, as the layer needs it (sRGB floats). */
function seriesColor(): [number, number, number] {
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue("--series-1")
    .trim();
  const [r, g, b] = cssColor(value);
  return [r, g, b];
}

type Kind = "waveform" | "spectrogram" | "lines";

function scene(kind: Kind, points: number): DenseScene {
  const color = seriesColor();
  if (kind === "spectrogram") {
    const columns = 256;
    const rows = 128;
    const magnitudes = new Uint8Array(columns * rows);
    for (let c = 0; c < columns; c += 1) {
      for (let r = 0; r < rows; r += 1) {
        magnitudes[c * rows + r] = Math.round(
          127.5 + 127.5 * Math.sin(c / 20 + r / 9),
        );
      }
    }
    return { kind, columns, rows, magnitudes, lut: colormapLut("viridis") };
  }
  const y = signal(points);
  if (kind === "waveform") {
    const columns = 600;
    const per = points / columns;
    const min = new Float32Array(columns);
    const max = new Float32Array(columns);
    for (let c = 0; c < columns; c += 1) {
      let lo = Infinity;
      let hi = -Infinity;
      for (let i = Math.floor(c * per); i < Math.floor((c + 1) * per); i += 1) {
        lo = Math.min(lo, y[i] ?? 0);
        hi = Math.max(hi, y[i] ?? 0);
      }
      min[c] = lo;
      max[c] = hi;
    }
    return { kind, min, max, y: { min: -1, max: 1 }, color };
  }
  const x = new Float32Array(points);
  for (let i = 0; i < points; i += 1) x[i] = i;
  return {
    kind,
    series: [{ x, y, color }],
    x: { min: 0, max: points - 1 },
    y: { min: -1, max: 1 },
  };
}

/**
 * The layer, with its backend and draw count on data attributes, and an overlay that
 * moves `frames` times (a stand-in for the playhead) to show the data is not redrawn.
 */
function Probe({
  kind,
  points,
  forceCanvas,
  frames,
}: {
  readonly kind: Kind;
  readonly points: number;
  readonly forceCanvas: boolean;
  readonly frames: number;
}) {
  const data = useMemo(() => scene(kind, points), [kind, points]);
  const status = useRef<HTMLDivElement>(null);
  const [frame, setFrame] = useState(0);
  useEffect(() => {
    if (frame >= frames) return;
    const id = requestAnimationFrame(() => {
      setFrame((f) => f + 1);
    });
    return () => {
      cancelAnimationFrame(id);
    };
  }, [frame, frames]);
  const record = (renderer: LayerRenderer) => {
    if (status.current !== null) {
      status.current.dataset.backend = renderer.backend;
      status.current.dataset.draws = String(renderer.draws);
    }
  };
  return (
    <div ref={status} data-frames={frame}>
      <DenseLayer
        summary={{
          l1: `Dense ${kind} view of ${String(points)} points.`,
          l2: "A synthetic signal: two sines and noise.",
        }}
        scene={data}
        height={200}
        forceCanvas={forceCanvas}
        onDraw={record}
      >
        <div
          style={{
            position: "absolute",
            insetBlock: 0,
            insetInlineStart: `${String((frame / Math.max(frames, 1)) * 100)}%`,
            borderInlineStart: "2px solid var(--signal-playhead)",
          }}
        />
      </DenseLayer>
    </div>
  );
}

/** How many pixels of the data canvas are drawn (alpha above zero). */
function inkedPixels(root: HTMLElement): number {
  const source = root.querySelector("canvas");
  if (source === null) return 0;
  const copy = document.createElement("canvas");
  copy.width = source.width;
  copy.height = source.height;
  const ctx = copy.getContext("2d");
  if (ctx === null) return 0;
  ctx.drawImage(source, 0, 0);
  const pixels = ctx.getImageData(0, 0, copy.width, copy.height).data;
  let inked = 0;
  for (let i = 3; i < pixels.length; i += 4) {
    if ((pixels[i] ?? 0) > 0) inked += 1;
  }
  return inked;
}

const meta = {
  title: "Viz/DenseLayer",
  component: Probe,
  args: { kind: "waveform", points: 100_000, forceCanvas: false, frames: 0 },
} satisfies Meta<typeof Probe>;

export default meta;

type Story = StoryObj<typeof meta>;

/** Checks the backend, that pixels were drawn, and the A11Y-01 name. */
async function drawn(canvasElement: HTMLElement, backend: string) {
  const root = canvasElement.querySelector<HTMLElement>("[data-backend]");
  await waitFor(async () => {
    await expect(root?.dataset.backend).toBe(backend);
  });
  await expect(inkedPixels(canvasElement)).toBeGreaterThan(0);
  const image = within(canvasElement).getByRole("img");
  await expect(image).toHaveAccessibleDescription(
    "A synthetic signal: two sines and noise.",
  );
  await expect(image).not.toHaveAttribute("tabindex");
}

/** WebGL2 in Chromium (SwiftShader in CI): a waveform of 600 min/max columns. */
export const Waveform: Story = {
  play: async ({ canvasElement }) => {
    await drawn(canvasElement, "webgl2");
  },
};

/** The fallback renders when WebGL2 is not used (B04-T02 acceptance). */
export const CanvasFallback: Story = {
  args: { forceCanvas: true },
  play: async ({ canvasElement }) => {
    await drawn(canvasElement, "canvas2d");
  },
};

/** A spectrogram through the viridis colormap (design-language §6). */
export const Spectrogram: Story = {
  args: { kind: "spectrogram" },
  play: async ({ canvasElement }) => {
    await drawn(canvasElement, "webgl2");
  },
};

/** The spectrogram with the fallback. */
export const SpectrogramFallback: Story = {
  args: { kind: "spectrogram", forceCanvas: true },
  play: async ({ canvasElement }) => {
    await drawn(canvasElement, "canvas2d");
  },
};

/**
 * A million-point line, and an overlay that moves for 30 frames: the data is drawn once
 * (notes/05 §6; PB-08's playhead never redraws the data).
 */
export const MillionPointsDrawnOnce: Story = {
  args: { kind: "lines", points: 1_000_000, frames: 30 },
  play: async ({ canvasElement }) => {
    await drawn(canvasElement, "webgl2");
    const root = canvasElement.querySelector<HTMLElement>("[data-backend]");
    await waitFor(
      async () => {
        await expect(root?.dataset.frames).toBe("30");
      },
      { timeout: 5000 },
    );
    await expect(root?.dataset.draws).toBe("1");
  },
};
