// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Disclosure, DisclosureGroup } from "./Disclosure.tsx";

const t = storyLocalizer();

function InspectorSections() {
  return (
    <DisclosureGroup>
      <Disclosure id="channels" title={t.format("ui-section-channels")}>
        {t.format("ui-workspace-empty")}
      </Disclosure>
      <Disclosure id="context" title={t.format("ui-section-context")}>
        {t.format("ui-workspace-empty")}
      </Disclosure>
    </DisclosureGroup>
  );
}

const meta = {
  title: "Components/Disclosure",
  component: InspectorSections,
} satisfies Meta<typeof InspectorSections>;

export default meta;

/**
 * Each title is a heading holding a button with `aria-expanded`; Enter toggles it, and
 * opening one section closes the other.
 */
export const Inspector: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const channels = canvas.getByRole("button", {
      name: t.format("ui-section-channels"),
    });
    await expect(
      canvas.getByRole("heading", {
        level: 3,
        name: t.format("ui-section-channels"),
      }),
    ).toContainElement(channels);
    await userEvent.tab();
    await expect(channels).toHaveFocus();
    await expect(channels).toHaveAttribute("aria-expanded", "false");
    await userEvent.keyboard("{Enter}");
    await expect(channels).toHaveAttribute("aria-expanded", "true");
    await userEvent.tab();
    const context = canvas.getByRole("button", {
      name: t.format("ui-section-context"),
    });
    await expect(context).toHaveFocus();
    await userEvent.keyboard(" ");
    await expect(context).toHaveAttribute("aria-expanded", "true");
    await expect(channels).toHaveAttribute("aria-expanded", "false");
  },
};
