// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { expect, fireEvent, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import type { ImageMark } from "./ImageCrosshair.tsx";
import { ImageCrosshair } from "./ImageCrosshair.tsx";

const t = storyLocalizer();

/** A small line chart drawn as SVG, standing in for a chart image. */
const CHART = `data:image/svg+xml;utf8,${encodeURIComponent(
  '<svg xmlns="http://www.w3.org/2000/svg" width="200" height="120">' +
    '<rect width="200" height="120" fill="white"/>' +
    '<path d="M20 10V100H190" fill="none" stroke="black"/>' +
    '<polyline points="20,90 60,70 100,40 140,55 180,20" fill="none" stroke="steelblue" stroke-width="2"/>' +
    "</svg>",
)}`;

const MARKS: readonly ImageMark[] = [
  { id: "1", axis: "x", at: [20, 100], label: "x1" },
  { id: "2", axis: "x", at: [180, 100], label: "x2" },
  { id: "3", axis: "y", at: [20, 100], label: "y1" },
  { id: "4", axis: "y", at: [20, 10], label: "y2" },
];

const pixel = (value: number) =>
  t.format("ui-calibrate-pixel", { index: { number: value } });

/** The crosshair with its own state, as the calibration dialog holds it. */
function Calibrating({ marks }: { readonly marks: readonly ImageMark[] }) {
  const [at, setAt] = useState<readonly [number, number]>([100, 60]);
  return (
    <ImageCrosshair
      src={CHART}
      width={200}
      height={120}
      imageLabel={t.format("ui-calibrate-image", {
        name: "rain.png",
        x: { number: 200 },
        y: { number: 120 },
      })}
      x={at[0]}
      y={at[1]}
      onMove={(x, y) => {
        setAt([x, y]);
      }}
      xLabel={t.format("ui-calibrate-crosshair-x")}
      yLabel={t.format("ui-calibrate-crosshair-y")}
      formatX={pixel}
      formatY={pixel}
      plotArea={{ min: [20, 10], max: [190, 100] }}
      marks={marks}
    />
  );
}

const meta = {
  title: "Components/ImageCrosshair",
  component: Calibrating,
  args: { marks: MARKS },
} satisfies Meta<typeof Calibrating>;

export default meta;

type Story = StoryObj<typeof meta>;

/**
 * Two sliders move the crosshair a pixel per arrow and say where it is; Home and End go to
 * the edges. The image is named and is not a Tab stop; x and y anchors differ by shape and
 * label, not by colour alone.
 */
export const KeyboardOnly: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(
      canvas.getByRole("img", {
        name: t.format("ui-calibrate-image", {
          name: "rain.png",
          x: { number: 200 },
          y: { number: 120 },
        }),
      }),
    ).toBeVisible();
    await userEvent.tab();
    const x = canvas.getByRole("slider", {
      name: t.format("ui-calibrate-crosshair-x"),
    });
    await expect(x).toHaveFocus();
    await expect(x).toHaveAttribute("aria-valuetext", pixel(100));
    await userEvent.keyboard("{ArrowRight}");
    await expect(x).toHaveAttribute("aria-valuetext", pixel(101));
    await userEvent.keyboard("{End}");
    await expect(x).toHaveAttribute("aria-valuetext", pixel(199));
    await userEvent.tab();
    const y = canvas.getByRole("slider", {
      name: t.format("ui-calibrate-crosshair-y"),
    });
    await expect(y).toHaveFocus();
    await userEvent.keyboard("{Home}");
    await expect(y).toHaveAttribute("aria-valuetext", pixel(0));
    // Squares for x, circles for y.
    const svg = canvasElement.querySelector("svg");
    await expect(svg?.querySelectorAll('[data-axis="x"] rect')).toHaveLength(2);
    await expect(svg?.querySelectorAll('[data-axis="y"] circle')).toHaveLength(
      2,
    );
  },
};

/** A click on the image moves the crosshair there: the single-pointer alternative. */
export const ClickToPlace: Story = {
  args: { marks: [] },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const image = canvas.getByRole("img");
    const box = image.getBoundingClientRect();
    await fireEvent.click(image, {
      clientX: box.left + box.width / 4,
      clientY: box.top + box.height / 2,
    });
    await expect(
      canvas.getByRole("slider", {
        name: t.format("ui-calibrate-crosshair-x"),
      }),
    ).toHaveAttribute("aria-valuetext", pixel(50));
  },
};
