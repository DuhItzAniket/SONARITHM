// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import type { ReorderableItem } from "./ReorderableList.tsx";
import { ReorderableList } from "./ReorderableList.tsx";

const t = storyLocalizer();

const PARTS: ReorderableItem[] = [
  { id: "value", name: t.format("ui-part-value") },
  { id: "position", name: t.format("ui-part-position") },
  { id: "series", name: t.format("ui-part-series") },
];

function AnnouncementParts() {
  const [items, setItems] = useState(PARTS);
  return (
    <ReorderableList
      label={t.format("ui-announcement-parts")}
      items={items}
      onReorder={(order) => {
        setItems(order.flatMap((id) => PARTS.filter((part) => part.id === id)));
      }}
      labels={{
        reorder: (name) => t.format("ui-reorder", { name }),
        moveUp: (name) => t.format("ui-move-up", { name }),
        moveDown: (name) => t.format("ui-move-down", { name }),
      }}
    />
  );
}

const meta = {
  title: "Components/ReorderableList",
  component: AnnouncementParts,
} satisfies Meta<typeof AnnouncementParts>;

export default meta;

/**
 * The announcement-part order (A11Y-12): the row's "Move down" button, reached with the
 * keyboard alone, moves the item.
 */
export const Parts: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const value = t.format("ui-part-value");
    await expect(
      canvas.getByRole("grid", { name: t.format("ui-announcement-parts") }),
    ).toBeInTheDocument();
    await expect(
      canvas.queryByRole("button", {
        name: t.format("ui-move-up", { name: value }),
      }),
    ).toBeNull();
    await userEvent.tab();
    await expect(canvas.getByRole("row", { name: value })).toHaveFocus();
    const moveDown = canvas.getByRole("button", {
      name: t.format("ui-move-down", { name: value }),
    });
    for (let i = 0; i < 4 && document.activeElement !== moveDown; i++) {
      await userEvent.keyboard("{ArrowRight}");
    }
    await expect(moveDown).toHaveFocus();
    await userEvent.keyboard("{Enter}");
    await expect(
      canvas
        .getAllByRole("row")
        .map((row) => row.getAttribute("aria-label") ?? row.textContent),
    ).toEqual([
      expect.stringContaining(t.format("ui-part-position")),
      expect.stringContaining(value),
      expect.stringContaining(t.format("ui-part-series")),
    ]);
  },
};
