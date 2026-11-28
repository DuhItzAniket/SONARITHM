// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { expect, fn, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { ActionList } from "./ActionList.tsx";
import { Button } from "./Button.tsx";

const t = storyLocalizer();

// Critical points as the app core words them: data, so shown as they are.
const ITEMS = [
  { id: 0, text: "zero at x = -1.7321" },
  { id: 1, text: "local maximum at x = -1, y = 2" },
  { id: 2, text: "zero at x = 0" },
  { id: 3, text: "local minimum at x = 1, y = -2" },
];

const meta = {
  title: "Components/ActionList",
  component: ActionList,
  args: {
    label: t.format("ui-function-points"),
    items: ITEMS,
    current: 1,
    onAction: fn(),
  },
} satisfies Meta<typeof ActionList>;

export default meta;

type Story = StoryObj<typeof meta>;

/**
 * One Tab stop; the arrows move between rows and Enter acts on the focused one. Moving
 * changes nothing: the current row is the caller's, marked `aria-current` and with a bar.
 */
export const Points: Story = {
  play: async ({ args, canvasElement }) => {
    const canvas = within(canvasElement);
    const grid = canvas.getByRole("grid", {
      name: t.format("ui-function-points"),
    });
    await expect(grid).toBeInTheDocument();
    const rows = canvas.getAllByRole("row");
    await expect(rows[1]).toHaveAttribute("aria-current", "true");
    await expect(rows[0]).not.toHaveAttribute("aria-current");
    await userEvent.tab();
    await expect(rows[0]).toHaveFocus();
    await userEvent.keyboard("{ArrowDown}{ArrowDown}");
    await expect(rows[2]).toHaveFocus();
    await expect(rows[1]).toHaveAttribute("aria-current", "true");
    await expect(rows[2]).not.toHaveAttribute("aria-selected", "true");
    await userEvent.keyboard("{Enter}");
    await expect(args.onAction).toHaveBeenCalledWith(2);
  },
};

/** With no current row, no row is marked. */
export const NoneCurrent: Story = {
  args: { current: null },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    for (const row of canvas.getAllByRole("row")) {
      await expect(row).not.toHaveAttribute("aria-current");
    }
  },
};

/** The list over stable items, with a button that moves the current row, as the cursor does. */
function MovingCurrent() {
  const [current, setCurrent] = useState(0);
  return (
    <>
      <ActionList
        label={t.format("ui-function-points")}
        items={ITEMS}
        current={current}
        onAction={() => undefined}
      />
      <Button
        onPress={() => {
          setCurrent((at) => (at + 1) % ITEMS.length);
        }}
      >
        {t.format("command-nav-next")}
      </Button>
    </>
  );
}

/**
 * The items stay the same objects while the current row moves (the cursor moved elsewhere):
 * the mark follows, and only one row has it.
 */
export const CurrentMoves: StoryObj<typeof MovingCurrent> = {
  render: () => <MovingCurrent />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const rows = canvas.getAllByRole("row");
    await expect(rows[0]).toHaveAttribute("aria-current", "true");
    await userEvent.click(
      canvas.getByRole("button", { name: t.format("command-nav-next") }),
    );
    await expect(rows[1]).toHaveAttribute("aria-current", "true");
    await expect(rows[0]).not.toHaveAttribute("aria-current");
    await expect(
      canvasElement.querySelectorAll('[aria-current="true"]'),
    ).toHaveLength(1);
  },
};
