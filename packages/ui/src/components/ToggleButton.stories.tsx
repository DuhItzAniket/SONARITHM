// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { ToggleButton, ToggleButtonGroup } from "./ToggleButton.tsx";

const t = storyLocalizer();

const meta = {
  title: "Components/ToggleButton",
  component: ToggleButton,
  args: { children: t.format("ui-loop") },
} satisfies Meta<typeof ToggleButton>;

export default meta;

type Story = StoryObj<typeof meta>;

/** Space toggles `aria-pressed`. */
export const Default: Story = {
  play: async ({ canvasElement }) => {
    const toggle = within(canvasElement).getByRole("button", {
      name: t.format("ui-loop"),
    });
    await userEvent.tab();
    await expect(toggle).toHaveAttribute("aria-pressed", "false");
    await userEvent.keyboard(" ");
    await expect(toggle).toHaveAttribute("aria-pressed", "true");
  },
};

export const Selected: Story = { args: { defaultSelected: true } };

export const Disabled: Story = { args: { isDisabled: true } };

/** Modalities: one Tab stop for the group; arrows move between toggles. */
export const Group: StoryObj<typeof ToggleButtonGroup> = {
  render: () => (
    <ToggleButtonGroup
      label={t.format("ui-modalities")}
      selectionMode="multiple"
      defaultSelectedKeys={["sonification"]}
    >
      <ToggleButton id="sonification">
        {t.format("ui-modality-sonification")}
      </ToggleButton>
      <ToggleButton id="speech">{t.format("ui-modality-speech")}</ToggleButton>
      <ToggleButton id="braille">
        {t.format("ui-modality-braille")}
      </ToggleButton>
    </ToggleButtonGroup>
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(
      // A multiple-selection group is a toolbar (one Tab stop, arrows inside).
      canvas.getByRole("toolbar", { name: t.format("ui-modalities") }),
    ).toBeInTheDocument();
    await userEvent.tab();
    await expect(
      canvas.getByRole("button", {
        name: t.format("ui-modality-sonification"),
      }),
    ).toHaveFocus();
    await userEvent.keyboard("{ArrowRight}");
    const speech = canvas.getByRole("button", {
      name: t.format("ui-modality-speech"),
    });
    await expect(speech).toHaveFocus();
    await userEvent.keyboard(" ");
    await expect(speech).toHaveAttribute("aria-pressed", "true");
  },
};
