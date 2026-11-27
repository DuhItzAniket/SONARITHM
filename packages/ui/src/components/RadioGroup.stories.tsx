// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Radio, RadioGroup } from "./RadioGroup.tsx";

const t = storyLocalizer();

const meta = {
  title: "Components/RadioGroup",
  component: RadioGroup,
  args: {
    label: t.format("ui-calibrate-area"),
    description: t.format("ui-calibrate-area-hint"),
    defaultValue: "anchors",
    children: [
      <Radio key="anchors" value="anchors">
        {t.format("ui-calibrate-area-anchors")}
      </Radio>,
      <Radio key="detected" value="detected">
        {t.format("ui-calibrate-area-detected")}
      </Radio>,
      <Radio key="image" value="image">
        {t.format("ui-calibrate-area-image")}
      </Radio>,
    ],
  },
} satisfies Meta<typeof RadioGroup>;

export default meta;

type Story = StoryObj<typeof meta>;

/** Tab reaches the chosen radio, the arrows move the choice, and the hint describes it. */
export const Default: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const group = canvas.getByRole("radiogroup", {
      name: t.format("ui-calibrate-area"),
    });
    await expect(group).toHaveAccessibleDescription(
      t.format("ui-calibrate-area-hint"),
    );
    const anchors = canvas.getByRole("radio", {
      name: t.format("ui-calibrate-area-anchors"),
    });
    await userEvent.tab();
    await expect(anchors).toHaveFocus();
    await expect(anchors).toBeChecked();
    await userEvent.keyboard("{ArrowDown}");
    await expect(
      canvas.getByRole("radio", { name: t.format("ui-calibrate-area-detected") }),
    ).toBeChecked();
  },
};

/** A choice that cannot be made is disabled, and the description says why. */
export const DisabledChoice: Story = {
  args: {
    description: t.format("ui-calibrate-area-none-hint"),
    children: [
      <Radio key="anchors" value="anchors">
        {t.format("ui-calibrate-area-anchors")}
      </Radio>,
      <Radio key="detected" value="detected" isDisabled>
        {t.format("ui-calibrate-area-detected")}
      </Radio>,
      <Radio key="image" value="image">
        {t.format("ui-calibrate-area-image")}
      </Radio>,
    ],
  },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(
      canvas.getByRole("radio", { name: t.format("ui-calibrate-area-detected") }),
    ).toBeDisabled();
    await userEvent.tab();
    await userEvent.keyboard("{ArrowDown}");
    await expect(
      canvas.getByRole("radio", { name: t.format("ui-calibrate-area-image") }),
    ).toBeChecked();
  },
};
