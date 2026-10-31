// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, fn, userEvent, within } from "storybook/test";

import { PlayGlyph } from "../../.storybook/icons.tsx";
import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Button, IconButton } from "./Button.tsx";

const t = storyLocalizer();

const meta = {
  title: "Components/Button",
  component: Button,
  args: { children: t.format("ui-export-audio"), onPress: fn() },
} satisfies Meta<typeof Button>;

export default meta;

type Story = StoryObj<typeof meta>;

/** Tab reaches it with a visible focus ring; Enter and Space press it. */
export const Default: Story = {
  play: async ({ args, canvasElement }) => {
    const button = within(canvasElement).getByRole("button", {
      name: t.format("ui-export-audio"),
    });
    await userEvent.tab();
    await expect(button).toHaveFocus();
    await expect(button).toHaveAttribute("data-focus-visible", "true");
    await userEvent.keyboard("{Enter}");
    await userEvent.keyboard(" ");
    await expect(args.onPress).toHaveBeenCalledTimes(2);
  },
};

export const Quiet: Story = { args: { variant: "quiet" } };

/** Disabled buttons stay in the accessibility tree but leave the Tab order. */
export const Disabled: Story = {
  args: { isDisabled: true },
  play: async ({ canvasElement }) => {
    const button = within(canvasElement).getByRole("button");
    await expect(button).toBeDisabled();
    await userEvent.tab();
    await expect(button).not.toHaveFocus();
  },
};

/** An icon-only button: named, with a tooltip that keyboard focus opens (DL-X05, DL-X11). */
export const IconOnly: StoryObj<typeof IconButton> = {
  render: () => (
    <IconButton
      label={t.format("ui-play")}
      tooltip={t.format("ui-name-with-shortcut", {
        name: t.format("ui-play"),
        key: "Space",
      })}
      icon={<PlayGlyph />}
    />
  ),
  play: async ({ canvasElement }) => {
    const button = within(canvasElement).getByRole("button", {
      name: t.format("ui-play"),
    });
    await userEvent.tab();
    await expect(button).toHaveFocus();
    const tooltip = await within(document.body).findByRole("tooltip");
    await expect(tooltip).toHaveTextContent("Play (Space)");
    // Escape removes this tooltip. Watch for that removal rather than for "no tooltip": after
    // Escape, React Aria still counts the trigger as focused, so any pointer entering it opens
    // a new tooltip at once. A stray pointer event in CI made the absence check fail in every
    // theme, though Escape had worked.
    const removed = new Promise<boolean>((resolve) => {
      const observer = new MutationObserver(() => {
        if (!tooltip.isConnected) {
          observer.disconnect();
          resolve(true);
        }
      });
      observer.observe(document.body, { childList: true, subtree: true });
      setTimeout(() => {
        observer.disconnect();
        resolve(!tooltip.isConnected);
      }, 3000);
    });
    await userEvent.keyboard("{Escape}");
    await expect(await removed, "Escape removes the tooltip").toBe(true);
  },
};
