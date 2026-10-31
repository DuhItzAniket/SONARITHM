// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { Button, DialogActions, ModalDialog, NumberField } from "@sonarithm/ui";
import { useRef, useState } from "react";

import { useLocalizer } from "../i18n.ts";

/**
 * "Go to row…" (B04-T05): a row number from 1 to the table's total. Go loads the window that
 * holds it and focuses that row (`DatasetTable`). The dialog puts the focus in the field
 * when it opens and returns it to the table when it closes.
 */
export function GoToRow({
  isOpen,
  onOpenChange,
  total,
  onGo,
}: {
  readonly isOpen: boolean;
  readonly onOpenChange: (isOpen: boolean) => void;
  readonly total: number;
  /** Called with a valid 1-based row number. */
  readonly onGo: (row: number) => void;
}) {
  const t = useLocalizer();
  const [row, setRow] = useState(1);
  // NumberField commits a typed number during the Enter keydown that also submits the
  // form, before React renders it, so the submit reads the latest value from a ref.
  const latest = useRef(1);
  const isValid = (n: number) => Number.isInteger(n) && n >= 1 && n <= total;
  return (
    <ModalDialog
      title={t.format("ui-go-to-row")}
      isOpen={isOpen}
      onOpenChange={onOpenChange}
    >
      {(close) => (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            if (isValid(latest.current)) {
              close();
              onGo(latest.current);
            }
          }}
        >
          <NumberField
            label={t.format("ui-go-to-row-number")}
            value={row}
            onChange={(value) => {
              latest.current = value;
              setRow(value);
            }}
            minValue={1}
            maxValue={total}
            step={1}
            isInvalid={!isValid(row)}
            errorMessage={t.format("ui-error-out-of-range", {
              low: { number: 1 },
              high: { number: total },
            })}
          />
          <DialogActions>
            <Button onPress={close}>{t.format("ui-cancel")}</Button>
            <Button type="submit">{t.format("ui-go-to-row-action")}</Button>
          </DialogActions>
        </form>
      )}
    </ModalDialog>
  );
}
