// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { MenuButton, MenuItem, MenuSeparator, Submenu } from "./Menu.tsx";

const t = storyLocalizer();

/** File names are data, not UI text. */
const RECENT = ["temps.csv", "rainfall.parquet"];

function FileMenu() {
  return (
    <MenuButton label={t.format("ui-menu-file")}>
      <MenuItem id="open" shortcut="Ctrl+O">
        {t.format("ui-open-file")}
      </MenuItem>
      <Submenu label={t.format("ui-recent-files")}>
        {RECENT.map((name) => (
          <MenuItem key={name} id={name}>
            {name}
          </MenuItem>
        ))}
      </Submenu>
      <MenuSeparator />
      <MenuItem id="export" shortcut="Ctrl+E">
        {t.format("ui-export-audio")}
      </MenuItem>
      <MenuItem id="close" shortcut="Ctrl+W">
        {t.format("ui-close-window")}
      </MenuItem>
    </MenuButton>
  );
}

const meta = {
  title: "Components/Menu",
  component: FileMenu,
} satisfies Meta<typeof FileMenu>;

export default meta;

/**
 * Enter opens the menu on its first item, arrows move, ArrowRight opens the submenu, and
 * Escape closes everything and returns focus to the button.
 */
export const File: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const trigger = within(canvasElement).getByRole("button", {
      name: t.format("ui-menu-file"),
    });
    await userEvent.tab();
    await userEvent.keyboard("{Enter}");
    const page = within(document.body);
    const menu = await page.findByRole("menu", {
      name: t.format("ui-menu-file"),
    });
    await expect(menu).toBeInTheDocument();
    await expect(
      page.getByRole("menuitem", { name: /Open file/ }),
    ).toHaveFocus();
    await userEvent.keyboard("{ArrowDown}");
    await expect(
      page.getByRole("menuitem", { name: t.format("ui-recent-files") }),
    ).toHaveFocus();
    await userEvent.keyboard("{ArrowRight}");
    await expect(
      await page.findByRole("menuitem", { name: "temps.csv" }),
    ).toHaveFocus();
    await userEvent.keyboard("{ArrowLeft}");
    await userEvent.keyboard("{Escape}");
    await expect(page.queryByRole("menu")).toBeNull();
    await expect(trigger).toHaveFocus();
  },
};
