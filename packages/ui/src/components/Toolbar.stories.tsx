// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { LoopGlyph, PlayGlyph, StopGlyph } from "../../.storybook/icons.tsx";
import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Button, IconButton } from "./Button.tsx";
import { ToggleButton } from "./ToggleButton.tsx";
import { Toolbar } from "./Toolbar.tsx";

const t = storyLocalizer();

const meta = {
  title: "Components/Toolbar",
  component: Toolbar,
  args: { label: t.format("ui-transport") },
} satisfies Meta<typeof Toolbar>;

export default meta;

/**
 * The transport toolbar (keyboard §5.3): Tab enters on the first control, arrows move
 * inside, and Tab leaves for the next control after the toolbar.
 */
export const Transport: StoryObj<typeof meta> = {
  render: (args) => (
    <>
      <Toolbar {...args}>
        <IconButton label={t.format("ui-play")} icon={<PlayGlyph />} />
        <IconButton label={t.format("ui-stop")} icon={<StopGlyph />} />
        <ToggleButton aria-label={t.format("ui-loop")}>
          <LoopGlyph />
        </ToggleButton>
      </Toolbar>
      <Button>{t.format("ui-export-audio")}</Button>
    </>
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(
      canvas.getByRole("toolbar", { name: t.format("ui-transport") }),
    ).toBeInTheDocument();
    await userEvent.tab();
    await expect(
      canvas.getByRole("button", { name: t.format("ui-play") }),
    ).toHaveFocus();
    await userEvent.keyboard("{ArrowRight}");
    await expect(
      canvas.getByRole("button", { name: t.format("ui-stop") }),
    ).toHaveFocus();
    await userEvent.keyboard("{ArrowRight}");
    await expect(
      canvas.getByRole("button", { name: t.format("ui-loop") }),
    ).toHaveFocus();
    await userEvent.tab();
    await expect(
      canvas.getByRole("button", { name: t.format("ui-export-audio") }),
    ).toHaveFocus();
  },
};
