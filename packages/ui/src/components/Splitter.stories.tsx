// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Splitter } from "./Splitter.tsx";

const t = storyLocalizer();

function ProjectSplitter() {
  const [width, setWidth] = useState(264);
  const [collapsed, setCollapsed] = useState(false);
  const name = t.format("ui-project");
  return (
    <div style={{ display: "flex" }}>
      <section id="project" aria-label={name} style={{ inlineSize: width }}>
        {collapsed ? t.format("ui-expand-region", { name }) : name}
      </section>
      <Splitter
        label={t.format("ui-resize-region", { name })}
        controls="project"
        value={width}
        min={200}
        max={400}
        valueText={t.format("ui-width-pixels", { value: { number: width } })}
        onChange={setWidth}
        onCollapse={() => {
          setCollapsed(true);
        }}
        edge="end"
      />
    </div>
  );
}

const meta = {
  title: "Components/Splitter",
  component: ProjectSplitter,
} satisfies Meta<typeof ProjectSplitter>;

export default meta;

/**
 * Tab reaches the splitter; arrows resize by 8 px (Shift by 32), Home and End go to the
 * limits, Enter collapses the region.
 */
export const Keyboard: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const splitter = canvas.getByRole("separator", {
      name: t.format("ui-resize-region", { name: t.format("ui-project") }),
    });
    await userEvent.tab();
    await expect(splitter).toHaveFocus();
    await expect(splitter).toHaveAttribute("aria-valuenow", "264");
    await expect(splitter).toHaveAttribute("aria-valuetext", "264 pixels");
    await userEvent.keyboard("{ArrowRight}");
    await expect(splitter).toHaveAttribute("aria-valuenow", "272");
    await userEvent.keyboard("{Shift>}{ArrowLeft}{/Shift}");
    await expect(splitter).toHaveAttribute("aria-valuenow", "240");
    await userEvent.keyboard("{End}");
    await expect(splitter).toHaveAttribute("aria-valuenow", "400");
    await userEvent.keyboard("{Home}");
    await expect(splitter).toHaveAttribute("aria-valuenow", "200");
    await userEvent.keyboard("{Enter}");
    await expect(canvas.getByRole("region")).toHaveTextContent(
      t.format("ui-expand-region", { name: t.format("ui-project") }),
    );
  },
};
