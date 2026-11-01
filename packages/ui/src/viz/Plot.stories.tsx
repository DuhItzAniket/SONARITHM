// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { Plot } from "@sonarithm/viz";
import type { PlotSeries } from "@sonarithm/viz";
import { expect, fn, userEvent, waitFor, within } from "storybook/test";

// The `@sonarithm/viz` plot (B04-T01) in every theme: its series colours come from the
// theme's tokens. The summary and series names are data (from the NavTree and the dataset),
// not UI strings.
const DAYS = Array.from({ length: 365 }, (_, i) => i + 1);

function wave(phase: number, scale: number): number[] {
  return DAYS.map(
    (day) => scale * Math.sin((day / 365) * 2 * Math.PI + phase) + 10 * phase,
  );
}

const THREE: readonly PlotSeries[] = [
  { label: "Oslo", values: wave(0, 12) },
  { label: "Lisbon", values: wave(1, 6) },
  { label: "Nairobi", values: wave(2, 2) },
];

const EIGHT: readonly PlotSeries[] = Array.from({ length: 8 }, (_, i) => ({
  label: `Series ${String(i + 1)}`,
  values: wave(i * 0.6, 5 + i),
}));

const SUMMARY = {
  l1: "Line chart of daily mean temperature in 2026 for Oslo, Lisbon and Nairobi, 365 days.",
  l2: "Oslo rises from −4 to 17 °C and falls back; Lisbon stays between 11 and 23 °C; Nairobi stays near 20 °C.",
};

const meta = {
  title: "Viz/Plot",
  component: Plot,
  args: { summary: SUMMARY, x: DAYS, series: THREE, height: 240 },
} satisfies Meta<typeof Plot>;

export default meta;

/**
 * A11Y-01: the plot is one image named by the summary's L1 and described by its L2; it is
 * not a Tab stop, and its canvas is presentational.
 */
export const NamedAndDescribed: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const plot = canvas.getByRole("img", { name: SUMMARY.l1 });
    await expect(plot).toHaveAccessibleDescription(SUMMARY.l2);
    await expect(plot).not.toHaveAttribute("tabindex");
    await expect(plot.querySelector("canvas")).not.toBeNull();
    const focusable = plot.querySelectorAll(
      "button, [href], input, select, textarea, [tabindex]",
    );
    await expect(focusable.length).toBe(0);
  },
};

/** All eight series: each has its token colour and its own dash pattern (DL-X08). */
export const EightSeries: StoryObj<typeof meta> = {
  args: {
    series: EIGHT,
    summary: {
      l1: "Line chart of eight synthetic series over 365 days.",
      l2: "Eight phase-shifted waves, one per palette colour.",
    },
  },
  play: async ({ canvasElement }) => {
    const lines = canvasElement.querySelectorAll("li svg line");
    await expect(lines.length).toBe(8);
    const dashes = new Set(
      [...lines].map((line) => line.getAttribute("stroke-dasharray")),
    );
    await expect(dashes.size).toBe(8);
  },
};

/**
 * B04-T06: the shared cursor as a dashed crosshair and a ring at its point, placed from data
 * units; a click reports the x value under it, so the caller can move the cursor there.
 */
export const WithCursor: StoryObj<typeof meta> = {
  args: { cursor: { x: 180, y: 2 }, onPick: fn() },
  play: async ({ args, canvasElement }) => {
    const line = await waitFor(() => {
      const found =
        canvasElement.querySelector<HTMLElement>("[data-plot-cursor]");
      if (found === null || found.hidden) throw new Error("no crosshair yet");
      return found;
    });
    await expect(line.dataset.cursorX).toBe("180");
    // The plot stays one image, not a control: the crosshair adds no focus stop.
    await expect(within(canvasElement).queryAllByRole("button")).toHaveLength(
      0,
    );
    const over = canvasElement.querySelector<HTMLElement>(".u-over");
    if (over === null) throw new Error("no plotting area");
    await userEvent.pointer({ keys: "[MouseLeft]", target: over });
    await expect(args.onPick).toHaveBeenCalledTimes(1);
  },
};
