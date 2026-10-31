// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// PB-08 for the dense layer (B04-T02): the script time of redrawing a million-point line
// on zoom, frame by frame, written to `.perf/dense-frames.json` (the trace). CI records it in the
// frontend-browser job; it is not gated there, because CI renders WebGL with SwiftShader
// on shared runners. The webview runs on HW-01 and the other webviews are B04-T08's.
import { createRenderer } from "@sonarithm/viz";
import type { DenseScene } from "@sonarithm/viz";
import { expect, test } from "vitest";
import { commands } from "vitest/browser";

const POINTS = 1_000_000;
const FRAMES = 60;
/** PB-08: script time per frame with a 1M-point series (12 §8). */
const BUDGET_MS = 8;
/**
 * Measuring stops after this much wall time, however many frames it took. CI draws WebGL in
 * software (SwiftShader) on shared runners, where 60 redraws of a million points ran past
 * the test timeout and no trace was written (#419). The report says how many frames it got.
 */
const MEASURE_MS = 8000;

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

function percentile(sorted: readonly number[], q: number): number {
  return (
    sorted[Math.min(sorted.length - 1, Math.ceil(q * sorted.length) - 1)] ?? 0
  );
}

test(
  "pb_08 dense layer frame times with a million points are recorded",
  {
    // A backstop well above MEASURE_MS plus the setup; the test records, it does not gate.
    timeout: 120_000,
  },
  async () => {
    const canvas = document.createElement("canvas");
    canvas.width = 1440;
    canvas.height = 320;
    document.body.append(canvas);
    const renderer = createRenderer(canvas);
    expect(renderer).not.toBeNull();
    if (renderer === null) return;

    const x = new Float32Array(POINTS);
    for (let i = 0; i < POINTS; i += 1) x[i] = i;
    const y = signal(POINTS);
    const scene = (zoom: number): DenseScene => ({
      kind: "lines",
      series: [{ x, y, color: [0.9, 0.62, 0] }],
      x: { min: 0, max: (POINTS - 1) / zoom },
      y: { min: -1, max: 1 },
    });

    // Each frame zooms, so the data layer is redrawn: the worst case. A playhead frame
    // redraws nothing (the overlay), so it is not timed here.
    const times: number[] = [];
    const began = performance.now();
    for (let frame = 0; frame < FRAMES; frame += 1) {
      if (frame > 0 && performance.now() - began > MEASURE_MS) break;
      const start = performance.now();
      renderer.draw(scene(1 + frame / FRAMES));
      times.push(performance.now() - start);
      await new Promise((resolve) => requestAnimationFrame(resolve));
    }
    renderer.destroy();
    canvas.remove();

    const sorted = [...times].sort((a, b) => a - b);
    const report = {
      test: "dense layer, 1M-point line, redrawn on zoom",
      backend: renderer.backend,
      userAgent: navigator.userAgent,
      points: POINTS,
      frames: times.length,
      framesPlanned: FRAMES,
      // Fewer frames than planned: the runner was too slow to finish in MEASURE_MS.
      cutShort: times.length < FRAMES,
      budgetMs: BUDGET_MS,
      p50Ms: percentile(sorted, 0.5),
      p95Ms: percentile(sorted, 0.95),
      maxMs: sorted.at(-1) ?? 0,
      withinBudget: percentile(sorted, 0.95) <= BUDGET_MS,
      frameTimesMs: times.map((t) => Math.round(t * 100) / 100),
    };
    await commands.writeFile(
      ".perf/dense-frames.json",
      JSON.stringify(report, null, 2),
    );
    // At least one frame drew; how fast is reported, never asserted here (B04-T08 gates).
    expect(times.length).toBeGreaterThan(0);
  },
);
