// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { BrailleState, Client } from "@sonarithm/ipc";
import { useEffect, useId, useRef } from "react";

import { useLocalizer } from "../i18n.ts";
import styles from "./braille.module.css";

/**
 * How long after an arrow, Home or End keydown a caret change counts as review. Engines
 * deliver `selectionchange` as a queued task, after the key's own events, so the window
 * is measured from the keydown; key repeat extends it.
 */
const REVIEW_WINDOW_MS = 150;

/** Keys that move the caret to review the line, never to route (keyboard §14). */
const REVIEW_KEYS: ReadonlySet<string> = new Set([
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "ArrowDown",
  "Home",
  "End",
]);

/**
 * The braille line (B03-T10; keyboard-and-navigation §10, §14; ADR-0017 "Braille"): a
 * read-only text field holding the one cursor's review line or sparkline, which the
 * screen reader shows on the braille display while the field has the focus. The caret is
 * placed on the cursor's cell. A routing-key press moves the caret; the field sends that
 * cell as `braille_route`, and on a sparkline the cursor moves there (origin `braille`,
 * not announced, as for the table: OD-30). Arrow keys, Home and End in the field review
 * the line instead: a caret change right after one of them sends nothing (keyboard §14).
 * The field never takes the focus by itself.
 *
 * It stays mounted when there is no line (braille off, or no point yet): empty, marked
 * unavailable and out of the Tab order, but still focusable, so a dialog that opened from
 * it can return the focus here instead of to the page (#513 review).
 */
export function BrailleField({
  client,
  braille,
  on,
}: {
  readonly client: Client;
  /** The cursor's line, or `null` when braille is off or the cursor is on nothing. */
  readonly braille: BrailleState | null;
  /** Whether braille output is on (the effective modality, B). */
  readonly on: boolean;
}) {
  const t = useLocalizer();
  const field = useRef<HTMLInputElement>(null);
  const hint = useId();
  // The caret this component last placed, so its own moves are not sent back as presses.
  const placed = useRef(braille?.caret ?? 0);
  // Until when a caret change is review: set by an arrow, Home or End keydown.
  const reviewUntil = useRef(0);
  // Whether there is a line to route on; read by the listener below.
  const live = useRef(braille !== null);

  useEffect(() => {
    live.current = braille !== null;
    const element = field.current;
    if (element === null || braille === null) return;
    placed.current = braille.caret;
    element.setSelectionRange(braille.caret, braille.caret);
  }, [braille]);

  // A routing key sets the caret through the accessibility API, with no key or mouse
  // event, so the field listens for the selection itself (`selectionchange`, at the
  // field in current engines and at the document in older ones). The cell under the caret
  // is where the listener wants to go, unless an arrow, Home or End just moved it.
  useEffect(() => {
    const element = field.current;
    if (element === null) return;
    const onChange = () => {
      if (document.activeElement !== element || !live.current) return;
      const cell = element.selectionStart;
      if (cell === null || cell === placed.current) return;
      placed.current = cell;
      if (performance.now() < reviewUntil.current) return;
      void client
        .call({ command: "braille_route", args: { cell } })
        .catch(() => undefined);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (REVIEW_KEYS.has(event.key)) {
        reviewUntil.current = performance.now() + REVIEW_WINDOW_MS;
      }
    };
    element.addEventListener("selectionchange", onChange);
    document.addEventListener("selectionchange", onChange);
    element.addEventListener("keydown", onKeyDown);
    return () => {
      element.removeEventListener("selectionchange", onChange);
      document.removeEventListener("selectionchange", onChange);
      element.removeEventListener("keydown", onKeyDown);
    };
  }, [client]);

  const mode = braille?.mode ?? "review";
  const description =
    braille !== null
      ? t.format("ui-braille-hint", { mode })
      : t.format(on ? "ui-braille-waiting" : "ui-braille-off");
  return (
    <div className={styles.braille} data-braille-line="">
      <input
        ref={field}
        className={styles.field}
        readOnly
        aria-label={t.format("ui-braille-line", { mode })}
        aria-describedby={hint}
        value={braille?.text ?? ""}
        {...(braille === null ? { "aria-disabled": true, tabIndex: -1 } : {})}
      />
      <p id={hint} className={styles.hint}>
        {description}
      </p>
    </div>
  );
}
