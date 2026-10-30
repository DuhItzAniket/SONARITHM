// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Tree, TreeItem } from "./Tree.tsx";

const t = storyLocalizer();

/** Dataset and field names are data, not UI text. */
const DATASET = "temps.csv";
const FIELDS = ["time", "temperature"];

function ProjectTree() {
  return (
    <Tree label={t.format("ui-project")}>
      <TreeItem id="dataset" title={DATASET}>
        <TreeItem id="fields" title={t.format("ui-tree-fields")}>
          {FIELDS.map((field) => (
            <TreeItem key={field} id={field} title={field} />
          ))}
        </TreeItem>
        <TreeItem id="tracks" title={t.format("ui-tree-tracks")} />
      </TreeItem>
    </Tree>
  );
}

const meta = {
  title: "Components/Tree",
  component: ProjectTree,
} satisfies Meta<typeof ProjectTree>;

export default meta;

/**
 * The project tree: ArrowRight expands, ArrowDown moves. Each row's cell is named by its
 * title alone, not "Expand temps.csv temps.csv" (#383, F-045), and no button is exposed.
 */
export const Project: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(
      canvas.getByRole("treegrid", { name: t.format("ui-project") }),
    ).toBeInTheDocument();
    await expect(canvas.queryAllByRole("button")).toHaveLength(0);
    await userEvent.tab();
    const dataset = canvas.getByRole("row", { name: DATASET });
    await expect(dataset).toHaveFocus();
    await expect(within(dataset).getByRole("gridcell")).toHaveAccessibleName(
      DATASET,
    );
    await expect(dataset).toHaveAttribute("aria-expanded", "false");
    await userEvent.keyboard("{ArrowRight}");
    await expect(dataset).toHaveAttribute("aria-expanded", "true");
    await userEvent.keyboard("{ArrowDown}");
    const fields = canvas.getByRole("row", {
      name: t.format("ui-tree-fields"),
    });
    await expect(fields).toHaveFocus();
    await expect(within(fields).getByRole("gridcell")).toHaveAccessibleName(
      t.format("ui-tree-fields"),
    );
  },
};
