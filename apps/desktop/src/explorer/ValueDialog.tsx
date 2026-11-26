// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { Button, DialogActions, ModalDialog, NumberField } from "@sonarithm/ui";
import { useRef, useState } from "react";

import { useLocalizer } from "../i18n.ts";

/** What the dialog asks for: an x value to go to (G), or a value to find (Primary+F). */
export type ValueKind = "goTo" | "find";

/**
 * The explorer's value field (B03-T09; keyboard-and-navigation §4.2): "Go to x" for G and
 * "Find a value" for Primary+F. The dialog puts the focus in the field when it opens, and
 * `onClose` returns it to the explorer. Numbers only in v0: a category x is a later
 * addition.
 */
export function ValueDialog({
  kind,
  onClose,
  onGo,
}: {
  readonly kind: ValueKind | null;
  readonly onClose: () => void;
  readonly onGo: (kind: ValueKind, value: number) => void;
}) {
  const t = useLocalizer();
  const [value, setValue] = useState(Number.NaN);
  // NumberField commits a typed number during the Enter keydown that also submits the
  // form, before React renders it (as in "Go to row…").
  const latest = useRef(Number.NaN);
  const shown = kind ?? "goTo";
  return (
    <ModalDialog
      title={t.format(
        shown === "goTo" ? "ui-explorer-go-to" : "ui-explorer-find",
      )}
      isOpen={kind !== null}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      {(close) => (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            if (Number.isFinite(latest.current)) {
              close();
              onGo(shown, latest.current);
            }
          }}
        >
          <NumberField
            label={t.format(
              shown === "goTo"
                ? "ui-explorer-go-to-value"
                : "ui-explorer-find-value",
            )}
            value={value}
            onChange={(next) => {
              latest.current = next;
              setValue(next);
            }}
          />
          <DialogActions>
            <Button onPress={close}>{t.format("ui-cancel")}</Button>
            <Button type="submit">
              {t.format(
                shown === "goTo"
                  ? "ui-explorer-go-to-action"
                  : "ui-explorer-find-action",
              )}
            </Button>
          </DialogActions>
        </form>
      )}
    </ModalDialog>
  );
}
