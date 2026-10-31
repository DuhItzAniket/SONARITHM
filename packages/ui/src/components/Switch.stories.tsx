// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Switch } from "./Switch.tsx";

const t = storyLocalizer();

const meta = {
  title: "Components/Switch",
  component: Switch,
  args: {
    children: t.format("ui-gpu-high-performance"),
    description: t.format("ui-gpu-restart-hint"),
  },
} satisfies Meta<typeof Switch>;

export default meta;

type Story = StoryObj<typeof meta>;

/** Tab reaches it, Space toggles it, and the hint is its description. */
export const Default: Story = {
  play: async ({ canvasElement }) => {
    const control = within(canvasElement).getByRole("switch", {
      name: t.format("ui-gpu-high-performance"),
    });
    await expect(control).toHaveAccessibleDescription(
      t.format("ui-gpu-restart-hint"),
    );
    await userEvent.tab();
    await expect(control).toHaveFocus();
    await expect(control).not.toBeChecked();
    await userEvent.keyboard(" ");
    await expect(control).toBeChecked();
  },
};

/** Disabled, with the reason in visible text that describes it (not a tooltip). */
export const DisabledWithReason: Story = {
  args: { isDisabled: true, description: t.format("ui-gpu-unavailable") },
  play: async ({ canvasElement }) => {
    const control = within(canvasElement).getByRole("switch");
    await expect(control).toBeDisabled();
    await expect(control).toHaveAccessibleDescription(
      t.format("ui-gpu-unavailable"),
    );
    await expect(
      within(canvasElement).getByText(t.format("ui-gpu-unavailable")),
    ).toBeVisible();
  },
};
