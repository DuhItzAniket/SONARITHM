// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import type { DenseScene, LayerRenderer } from "@sonarithm/viz";
import { DenseLayer, Playhead, PlayheadClock, cssColor } from "@sonarithm/viz";
import { Profiler, useEffect, useMemo, useRef, useState } from "react";
import { expect, waitFor } from "storybook/test";

// The playhead (B04-T04) over a dense layer in every theme. A clock playing at speed 1
// stands in for the app core's anchors. The React Profiler counts commits, and the layer
// counts draws, while the playhead moves for 30 frames.

function waveform(): DenseScene {
  const columns = 600;
  const min = new Float32Array(columns);
  const max = new Float32Array(columns);
  for (let c = 0; c < columns; c += 1) {
    const v = Math.sin(c / 30);
    min[c] = v - 0.2;
    max[c] = v + 0.2;
  }
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue("--series-1")
    .trim();
  const [r, g, b] = cssColor(value);
  return {
    kind: "waveform",
    min,
    max,
    y: { min: -1.5, max: 1.5 },
    color: [r, g, b],
  };
}

function Probe() {
  const scene = useMemo(() => waveform(), []);
  const status = useRef<HTMLDivElement>(null);
  const [clock] = useState(() => new PlayheadClock());
  // The first anchor: playing from 0 at speed 1, received now.
  useEffect(() => {
    clock.update(
      { positionS: 0, speed: 1, playing: true },
      performance.now() / 1000,
    );
  }, [clock]);
  const commits = useRef(0);
  const record = (renderer: LayerRenderer) => {
    if (status.current !== null) {
      status.current.dataset.draws = String(renderer.draws);
    }
  };
  return (
    <div ref={status}>
      <Profiler
        id="playhead"
        onRender={() => {
          commits.current += 1;
          if (status.current !== null) {
            status.current.dataset.commits = String(commits.current);
          }
        }}
      >
        <DenseLayer
          summary={{
            l1: "Waveform with the playhead.",
            l2: "A synthetic sine; the playhead moves at speed 1.",
          }}
          scene={scene}
          height={160}
          onDraw={record}
        >
          <Playhead clock={clock} latencyS={0.05} range={{ min: 0, max: 4 }} />
        </DenseLayer>
      </Profiler>
    </div>
  );
}

const meta = {
  title: "Viz/Playhead",
  component: Probe,
} satisfies Meta<typeof Probe>;

export default meta;

/** Moves every frame with no React render and no redraw of the data (PB-08, notes/05 §6). */
export const MovesWithoutRendering: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const root = canvasElement.querySelector<HTMLElement>("[data-draws]");
    await waitFor(async () => {
      await expect(root?.dataset.draws).toBe("1");
    });
    const line = canvasElement.querySelector<HTMLElement>(
      "[aria-hidden='true'][style*='translateX']",
    );
    const transforms = new Set<string>();
    const commitsBefore = root?.dataset.commits;
    for (let frame = 0; frame < 30; frame += 1) {
      await new Promise((resolve) => requestAnimationFrame(resolve));
      transforms.add(line?.style.transform ?? "");
    }
    // It moved: many distinct positions over 30 frames.
    await expect(transforms.size).toBeGreaterThan(10);
    // Nothing re-rendered and nothing was redrawn while it moved.
    await expect(root?.dataset.commits).toBe(commitsBefore);
    await expect(root?.dataset.draws).toBe("1");
  },
};
