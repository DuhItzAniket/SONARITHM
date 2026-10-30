// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import type { DataColumn, DataRow } from "./DataTable.tsx";
import { DataTable } from "./DataTable.tsx";

const t = storyLocalizer();

/** Field names with units are data, not UI text. */
const COLUMNS: DataColumn[] = [
  { id: "time", name: "time (s)", isRowHeader: true, numeric: true },
  { id: "temperature", name: "temperature (°C)", numeric: true },
];

const ROW_COUNT = 1000;

const ROWS: DataRow[] = Array.from({ length: ROW_COUNT }, (_, i) => ({
  id: i,
  cells: {
    time: (i * 0.5).toFixed(1),
    temperature: (20 + 5 * Math.sin(i / 40)).toFixed(2),
  },
}));

const meta = {
  title: "Components/DataTable",
  component: DataTable,
  args: { label: t.format("ui-data-table"), columns: COLUMNS, rows: ROWS },
} satisfies Meta<typeof DataTable>;

export default meta;

/**
 * 1000 rows, virtualized: the headers are in the accessibility tree (A11Y-03), the row count
 * is the full count, and arrows move between rows.
 */
export const Thousand: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const grid = canvas.getByRole("grid", { name: t.format("ui-data-table") });
    // The accessibility snapshot of the headers.
    await expect(
      canvas.getAllByRole("columnheader").map((h) => h.textContent),
    ).toEqual(["time (s)", "temperature (°C)"]);
    await expect(canvas.getAllByRole("rowheader")[0]).toHaveTextContent("0.0");
    await expect(grid).toHaveAttribute("aria-rowcount", String(ROW_COUNT + 1));
    // Only the visible rows are rendered.
    await expect(canvas.getAllByRole("row").length).toBeLessThan(100);
    await userEvent.tab();
    const first = canvas.getAllByRole("row")[1];
    await expect(first).toHaveFocus();
    await userEvent.keyboard("{ArrowDown}");
    await expect(canvas.getAllByRole("row")[2]).toHaveFocus();
  },
};
