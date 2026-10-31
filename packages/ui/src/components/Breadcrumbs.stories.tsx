// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, within } from "storybook/test";

import { Breadcrumbs } from "./Breadcrumbs.tsx";

/** Level words with data names, as the desktop app's cursor readout builds them. */
const ITEMS = [
  { id: "dataset", text: "Dataset temps.csv" },
  { id: "series", text: "Series temperature" },
  { id: "point", text: "Point 3" },
];

const meta = {
  title: "Components/Breadcrumbs",
  component: Breadcrumbs,
  args: { label: "Cursor position", items: ITEMS },
} satisfies Meta<typeof Breadcrumbs>;

export default meta;

/** A named `nav` landmark; only the last item is current, and separators are not read. */
export const Trail: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const nav = canvas.getByRole("navigation", { name: "Cursor position" });
    const items = within(nav).getAllByRole("listitem");
    await expect(items.map((item) => item.textContent)).toEqual([
      "Dataset temps.csv",
      "Series temperature",
      "Point 3",
    ]);
    const current = canvasElement.querySelectorAll('[aria-current="page"]');
    await expect(current).toHaveLength(1);
    await expect(current[0]).toHaveTextContent("Point 3");
  },
};
