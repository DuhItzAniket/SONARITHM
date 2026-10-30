// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Button } from "./Button.tsx";
import { Tooltip, TooltipTrigger } from "./Tooltip.tsx";

const t = storyLocalizer();

function ExportWithTooltip() {
  return (
    <TooltipTrigger>
      <Button>{t.format("ui-export-audio")}</Button>
      <Tooltip>
        {t.format("ui-name-with-shortcut", {
          name: t.format("ui-export-audio"),
          key: "Ctrl+E",
        })}
      </Tooltip>
    </TooltipTrigger>
  );
}

const meta = {
  title: "Components/Tooltip",
  component: ExportWithTooltip,
} satisfies Meta<typeof ExportWithTooltip>;

export default meta;

/** Keyboard focus opens the tooltip as hover does, and Escape closes it (DL-X11). */
export const OnFocus: StoryObj<typeof meta> = {
  play: async () => {
    await userEvent.tab();
    const page = within(document.body);
    const tooltip = await page.findByRole("tooltip");
    await expect(tooltip).toHaveTextContent("Ctrl+E");
    await expect(
      page.getByRole("button", { name: t.format("ui-export-audio") }),
    ).toHaveAttribute("aria-describedby", tooltip.id);
    await userEvent.keyboard("{Escape}");
    await expect(page.queryByRole("tooltip")).toBeNull();
  },
};
