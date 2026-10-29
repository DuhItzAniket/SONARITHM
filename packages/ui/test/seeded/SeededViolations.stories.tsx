// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// A seeded accessibility violation for the negative test of the axe gate (B02-T03): a button
// with no accessible name. The story carries the `seeded` tag, so the normal story run skips
// it; `pnpm -C packages/ui test:seeded` runs only seeded stories and passes only if axe
// fails this one with `button-name`.
import type { Meta, StoryObj } from "@storybook/react-vite";

function UnnamedButton() {
  // eslint-disable-next-line jsx-a11y/control-has-associated-label -- the seeded violation
  return <button type="button" />;
}

const meta = {
  title: "Seeded/Axe violation",
  component: UnnamedButton,
  tags: ["seeded"],
} satisfies Meta<typeof UnnamedButton>;

export default meta;

/** Fails axe's `button-name` rule (critical). */
export const UnnamedIconButton: StoryObj<typeof meta> = {};
