// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { ComboBox, ListBox, NumberField, Option, Select } from "./Fields.tsx";

const t = storyLocalizer();

const meta = {
  title: "Components/Fields",
  component: NumberField,
  args: {
    label: t.format("ui-duration"),
    formatOptions: { style: "unit", unit: "second" },
    minValue: 0.5,
    maxValue: 60,
    step: 0.1,
    defaultValue: 2.5,
  },
} satisfies Meta<typeof NumberField>;

export default meta;

type Story = StoryObj<typeof meta>;

/** The value reads with its unit; ArrowUp steps it. */
export const NumberWithUnit: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.tab();
    const input = canvas.getByRole("textbox", {
      name: t.format("ui-duration"),
    });
    await expect(input).toHaveFocus();
    // Intl writes the unit; React Aria may pluralise it ("2.5 secs").
    await expect((input as HTMLInputElement).value).toMatch(/^2\.5 \p{L}+$/u);
    await userEvent.keyboard("{ArrowUp}");
    await expect((input as HTMLInputElement).value).toMatch(/^2\.6 \p{L}+$/u);
  },
};

/** The error is text linked to the field, never colour alone (DL-X08). */
export const NumberError: Story = {
  args: {
    isInvalid: true,
    errorMessage: t.format("ui-error-out-of-range", {
      low: "0.5 s",
      high: "60 s",
    }),
  },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const input = canvas.getByRole("textbox", {
      name: t.format("ui-duration"),
    });
    await expect(input).toHaveAttribute("aria-invalid", "true");
    await expect(input).toHaveAccessibleDescription(
      t.format("ui-error-out-of-range", { low: "0.5 s", high: "60 s" }),
    );
  },
};

export const NumberDisabled: Story = { args: { isDisabled: true } };

/** Enter opens the list, arrows move, Enter chooses and closes it. */
export const ScaleSelect: StoryObj<typeof Select> = {
  render: () => (
    <Select label={t.format("ui-scale")} defaultValue="linear">
      <Option id="linear">{t.format("ui-scale-linear")}</Option>
      <Option id="log">{t.format("ui-scale-log")}</Option>
    </Select>
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.tab();
    const trigger = canvas.getByRole("button");
    await expect(trigger).toHaveFocus();
    await expect(trigger).toHaveTextContent(t.format("ui-scale-linear"));
    await userEvent.keyboard("{Enter}");
    const page = within(document.body);
    await expect(await page.findByRole("listbox")).toBeInTheDocument();
    await userEvent.keyboard("{ArrowDown}");
    await userEvent.keyboard("{Enter}");
    await expect(page.queryByRole("listbox")).toBeNull();
    await expect(trigger).toHaveTextContent(t.format("ui-scale-log"));
  },
};

/** Typing filters the options; ArrowDown and Enter choose one. */
export const ConceptComboBox: StoryObj<typeof ComboBox> = {
  render: () => (
    <ComboBox label={t.format("ui-concept")} openLabel={t.format("ui-concept")}>
      <Option id="temperature">{t.format("ui-concept-temperature")}</Option>
      <Option id="price">{t.format("ui-concept-price")}</Option>
      <Option id="count">{t.format("ui-concept-count")}</Option>
    </ComboBox>
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.tab();
    const input = canvas.getByRole("combobox", {
      name: t.format("ui-concept"),
    });
    await expect(input).toHaveFocus();
    await userEvent.keyboard("Pr");
    const page = within(document.body);
    const options = await page.findAllByRole("option");
    await expect(options.map((o) => o.textContent)).toEqual([
      t.format("ui-concept-price"),
    ]);
    await userEvent.keyboard("{ArrowDown}");
    await userEvent.keyboard("{Enter}");
    await expect(input).toHaveValue(t.format("ui-concept-price"));
  },
};

/** A list in place: arrows move the focus between options. */
export const ScaleListBox: StoryObj<typeof ListBox> = {
  render: () => (
    <ListBox label={t.format("ui-scale")} selectionMode="single">
      <Option id="linear">{t.format("ui-scale-linear")}</Option>
      <Option id="log">{t.format("ui-scale-log")}</Option>
    </ListBox>
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.tab();
    await userEvent.keyboard("{ArrowDown}");
    await expect(
      canvas.getByRole("option", { name: t.format("ui-scale-log") }),
    ).toHaveFocus();
  },
};
