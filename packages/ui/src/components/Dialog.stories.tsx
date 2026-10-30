// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, waitFor, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Button } from "./Button.tsx";
import { DialogActions, DialogTrigger, ModalDialog } from "./Dialog.tsx";

const t = storyLocalizer();

function DiscardConfirmation() {
  return (
    <DialogTrigger>
      <Button>{t.format("ui-close-window")}</Button>
      <ModalDialog
        title={t.format("ui-discard-title")}
        role="alertdialog"
        isDismissable={false}
      >
        {(close) => (
          <>
            <p>{t.format("ui-discard-body", { name: "Weather 2026" })}</p>
            <DialogActions>
              <Button onPress={close}>{t.format("ui-cancel")}</Button>
              <Button onPress={close}>{t.format("ui-discard")}</Button>
            </DialogActions>
          </>
        )}
      </ModalDialog>
    </DialogTrigger>
  );
}

const meta = {
  title: "Components/Dialog",
  component: DiscardConfirmation,
} satisfies Meta<typeof DiscardConfirmation>;

export default meta;

/**
 * A confirmation (APG Alert Dialog): opening moves focus inside, Tab stays inside, Escape
 * closes, and focus returns to the button that opened it.
 */
export const Confirmation: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const trigger = within(canvasElement).getByRole("button", {
      name: t.format("ui-close-window"),
    });
    await userEvent.tab();
    await userEvent.keyboard("{Enter}");
    const page = within(document.body);
    const dialog = await page.findByRole("alertdialog", {
      name: t.format("ui-discard-title"),
    });
    await expect(dialog).toContainElement(
      document.activeElement as HTMLElement,
    );
    for (let i = 0; i < 4; i++) {
      await userEvent.tab();
      await expect(dialog).toContainElement(
        document.activeElement as HTMLElement,
      );
    }
    await userEvent.keyboard("{Escape}");
    // Focus returns once the modal has closed, a frame after Escape.
    await waitFor(() => expect(page.queryByRole("alertdialog")).toBeNull());
    await waitFor(() => expect(trigger).toHaveFocus());
  },
};
