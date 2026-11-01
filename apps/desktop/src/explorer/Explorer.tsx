// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { CursorState, DatasetSummary } from "@sonarithm/ipc";
import type { RefObject } from "react";
import { useId } from "react";

import { useLocalizer } from "../i18n.ts";
import styles from "./explorer.module.css";

/**
 * The chart explorer's surface (B03-T09; 08 §5, keyboard-and-navigation §4, §5.4): one Tab
 * stop inside the Plot tab. Its commands live with the app (`ExplorerCommands`), so the
 * palette reaches them from anywhere; its keys work while this surface has the focus.
 *
 * Only the surface has `role="application"`, so NVDA and JAWS switch to focus or forms
 * mode there and pass its keys through. It is named after the dataset and described by a
 * visible hint. It has no `aria-activedescendant`: each move is spoken once, by the app
 * core's announcement, never also as a focus change (OD-30). It holds no position of its
 * own: it shows the level of Rust's one cursor.
 */
export function Explorer({
  dataset,
  cursor,
  surface,
}: {
  readonly dataset: DatasetSummary;
  readonly cursor: CursorState | null;
  /** The surface, for `view.focusExplorer`. */
  readonly surface: RefObject<HTMLDivElement | null>;
}) {
  const t = useLocalizer();
  const hint = useId();
  const position = cursor?.path.at(-1);
  return (
    <div className={styles.explorer}>
      <div
        ref={surface}
        role="application"
        // 08 §5: the explorer is one focusable application surface, so screen readers pass
        // its keys through; jsx-a11y does not count "application" as interactive.
        // eslint-disable-next-line jsx-a11y/no-noninteractive-tabindex -- see above
        tabIndex={0}
        data-scope="explorer"
        aria-label={t.format("ui-explorer-label", { name: dataset.name })}
        aria-describedby={hint}
        className={styles.surface}
      >
        <span className={styles.position} aria-hidden="true">
          {position === undefined
            ? t.format("ui-explorer-start")
            : t.format("ui-cursor-step", {
                kind: position.level,
                name: position.label,
              })}
        </span>
      </div>
      <p id={hint} className={styles.hint}>
        {t.format("ui-explorer-hint")}
      </p>
    </div>
  );
}
