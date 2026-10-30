// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Slider } from "./Slider.tsx";

const t = storyLocalizer();

const meta = {
  title: "Components/Slider",
  component: Slider,
  args: {
    label: t.format("ui-reference-frequency"),
    // The shared unit message, as the app formats every frequency (DL-P12).
    formatValue: (value: number) =>
      t.format("unit-hertz", { style: "symbol", value: { number: value } }),
    minValue: 100,
    maxValue: 1000,
    step: 10,
    defaultValue: 440,
  },
} satisfies Meta<typeof Slider>;

export default meta;

type Story = StoryObj<typeof meta>;

/**
 * `aria-valuetext` carries the value with its unit and the range is on the thumb; arrows
 * step, End and Home go to the limits.
 */
export const WithUnit: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.tab();
    const thumb = canvas.getByRole("slider", {
      name: t.format("ui-reference-frequency"),
    });
    await expect(thumb).toHaveFocus();
    await expect(thumb).toHaveAttribute("aria-valuetext", "440 Hz");
    // The thumb is a native range input: its range is `min` and `max`.
    await expect(thumb).toHaveAttribute("min", "100");
    await expect(thumb).toHaveAttribute("max", "1000");
    await userEvent.keyboard("{ArrowRight}");
    await expect(thumb).toHaveAttribute("aria-valuetext", "450 Hz");
    await userEvent.keyboard("{End}");
    await expect(thumb).toHaveAttribute("aria-valuetext", "1,000 Hz");
    await userEvent.keyboard("{Home}");
    await expect(thumb).toHaveAttribute("aria-valuetext", "100 Hz");
    // F-045: no live region beside the thumb.
    await expect(canvas.queryByRole("status")).toBeNull();
  },
};

export const Disabled: Story = { args: { isDisabled: true } };
