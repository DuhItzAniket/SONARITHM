// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import type { DiagnosticRow } from "./DiagnosticList.tsx";
import { DiagnosticList } from "./DiagnosticList.tsx";

const t = storyLocalizer();

const location = (line: number, column: number) =>
  t.format("ui-diagnostic-location", {
    line: { number: line },
    column: { number: column },
  });

const ROWS: readonly DiagnosticRow[] = [
  {
    id: "0",
    severity: "error",
    severityLabel: t.format("ui-severity-error"),
    code: "SNR-SPEC-070",
    message:
      'The data has no field "tempx". Name a field of the data, or declare it in `data.fields`.',
    location: location(4, 63),
  },
  {
    id: "1",
    severity: "warning",
    severityLabel: t.format("ui-severity-warning"),
    code: "SNR-SPEC-006",
    message: "The spec was migrated from version 0.1.",
    location: location(2, 18),
  },
  {
    id: "2",
    severity: "info",
    severityLabel: t.format("ui-severity-info"),
    code: "SNR-SPEC-090",
    message: "The pitch range defaults to 65 to 1480 Hz (SDR-01).",
    location: t.format("ui-diagnostic-location-none"),
  },
];

function Diagnostics({ rows }: { readonly rows: readonly DiagnosticRow[] }) {
  const [activated, setActivated] = useState("");
  return (
    <>
      <DiagnosticList
        label={t.format("ui-tab-diagnostics")}
        rows={rows}
        empty={t.format("ui-no-diagnostics")}
        onActivate={setActivated}
      />
      <output data-testid="activated">{activated}</output>
    </>
  );
}

const meta = {
  title: "Components/DiagnosticList",
  component: Diagnostics,
  args: { rows: ROWS },
} satisfies Meta<typeof Diagnostics>;

export default meta;

/**
 * A grid list: arrows and Home/End move, Enter goes to the source (keyboard §5.10). Each
 * row says its severity in words, then its code, message and place.
 */
export const Rows: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const grid = canvas.getByRole("grid", {
      name: t.format("ui-tab-diagnostics"),
    });
    const rows = within(grid).getAllByRole("row");
    await expect(rows).toHaveLength(3);
    await expect(rows[0]).toHaveTextContent(t.format("ui-severity-error"));
    await expect(rows[0]).toHaveTextContent("SNR-SPEC-070");
    await userEvent.tab();
    await expect(rows[0]).toHaveFocus();
    await userEvent.keyboard("{ArrowDown}");
    await expect(rows[1]).toHaveFocus();
    await userEvent.keyboard("{End}");
    await expect(rows[2]).toHaveFocus();
    await userEvent.keyboard("{Home}{Enter}");
    await expect(canvas.getByTestId("activated")).toHaveTextContent("0");
    // The severity shape is decorative: the word names it.
    await expect(rows[0]?.querySelector("svg")).toHaveAttribute(
      "aria-hidden",
      "true",
    );
  },
};

/** No rows: one line saying so. */
export const Empty: StoryObj<typeof meta> = {
  args: { rows: [] },
  play: async ({ canvasElement }) => {
    await expect(
      within(canvasElement).getByText(t.format("ui-no-diagnostics")),
    ).toBeInTheDocument();
  },
};
