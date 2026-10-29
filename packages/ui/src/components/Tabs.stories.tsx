// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Tab, TabList, TabPanel, Tabs } from "./Tabs.tsx";

const t = storyLocalizer();

function WorkspaceTabs() {
  return (
    <Tabs>
      <TabList label={t.format("ui-region-workspace")}>
        <Tab id="plot">{t.format("ui-tab-plot")}</Tab>
        <Tab id="table">{t.format("ui-tab-table")}</Tab>
        <Tab id="spec">{t.format("ui-tab-spec")}</Tab>
      </TabList>
      <TabPanel id="plot">{t.format("ui-workspace-empty")}</TabPanel>
      <TabPanel id="table">{t.format("ui-workspace-empty")}</TabPanel>
      <TabPanel id="spec">{t.format("ui-workspace-empty")}</TabPanel>
    </Tabs>
  );
}

const meta = {
  title: "Components/Tabs",
  component: WorkspaceTabs,
} satisfies Meta<typeof WorkspaceTabs>;

export default meta;

/** Tab reaches the selected tab, arrows select the next, and Tab moves into the panel. */
export const Workspace: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.tab();
    const plot = canvas.getByRole("tab", { name: t.format("ui-tab-plot") });
    await expect(plot).toHaveFocus();
    await expect(plot).toHaveAttribute("aria-selected", "true");
    await userEvent.keyboard("{ArrowRight}");
    const table = canvas.getByRole("tab", { name: t.format("ui-tab-table") });
    await expect(table).toHaveFocus();
    await expect(table).toHaveAttribute("aria-selected", "true");
    await userEvent.tab();
    await expect(canvas.getByRole("tabpanel")).toHaveFocus();
  },
};
