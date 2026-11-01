// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { BrailleState, Client } from "@sonarithm/ipc";
import { useEffect, useId, useRef } from "react";

import { useLocalizer } from "../i18n.ts";
import styles from "./braille.module.css";

/**
 * The braille line (B03-T10; keyboard-and-navigation §10, §14; ADR-0017 "Braille"): a
 * read-only text field holding the one cursor's review line or sparkline, which the
 * screen reader shows on the braille display while the field has the focus. The caret is
 * placed on the cursor's cell. A routing-key press moves the caret; the field sends that
 * cell as `braille_route`, and on a sparkline the cursor moves there (origin `braille`,
 * not announced, as for the table: OD-30). The field never takes the focus by itself.
 */
export function BrailleField({
  client,
  braille,
}: {
  readonly client: Client;
  readonly braille: BrailleState;
}) {
  const t = useLocalizer();
  const field = useRef<HTMLInputElement>(null);
  const hint = useId();
  // The caret this component last placed, so its own moves are not sent back as presses.
  const placed = useRef(braille.caret);

  useEffect(() => {
    const element = field.current;
    if (element === null) return;
    placed.current = braille.caret;
    element.setSelectionRange(braille.caret, braille.caret);
  }, [braille.text, braille.caret]);

  // A routing key sets the caret through the accessibility API, with no key or mouse
  // event, so the field listens for the selection itself (`selectionchange`, at the
  // field in current engines and at the document in older ones). The cell under the caret
  // is where the listener wants to go.
  useEffect(() => {
    const element = field.current;
    if (element === null) return;
    const onChange = () => {
      if (document.activeElement !== element) return;
      const cell = element.selectionStart;
      if (cell === null || cell === placed.current) return;
      placed.current = cell;
      void client
        .call({ command: "braille_route", args: { cell } })
        .catch(() => undefined);
    };
    element.addEventListener("selectionchange", onChange);
    document.addEventListener("selectionchange", onChange);
    return () => {
      element.removeEventListener("selectionchange", onChange);
      document.removeEventListener("selectionchange", onChange);
    };
  }, [client]);

  return (
    <div className={styles.braille} data-braille-line="">
      <input
        ref={field}
        className={styles.field}
        readOnly
        aria-label={t.format("ui-braille-line", { mode: braille.mode })}
        aria-describedby={hint}
        value={braille.text}
      />
      <p id={hint} className={styles.hint}>
        {t.format("ui-braille-hint", { mode: braille.mode })}
      </p>
    </div>
  );
}
