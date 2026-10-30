// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Foundations } from "./Foundations.tsx";

const t = storyLocalizer();

const meta = {
  title: "Foundations/Surfaces and text",
  component: Foundations,
  args: {
    sample: t.format("ui-workspace-empty"),
    readout: t.format("unit-hertz", {
      style: "symbol",
      value: { number: 440.25, minimumFractionDigits: 2 },
    }),
  },
} satisfies Meta<typeof Foundations>;

export default meta;

type Story = StoryObj<typeof meta>;

/** Every surface and text role in the theme chosen in the toolbar. */
export const Default: Story = {};

/** The same in each theme, so the axe contrast check covers all four. */
export const Light: Story = { globals: { theme: "light" } };
export const HighContrastDark: Story = {
  globals: { theme: "highContrastDark" },
};
export const HighContrastLight: Story = {
  globals: { theme: "highContrastLight" },
};
