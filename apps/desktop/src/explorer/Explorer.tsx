// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Client,
  CursorState,
  DatasetSummary,
  NavQuery,
} from "@sonarithm/ipc";
import { MenuButton, MenuItem } from "@sonarithm/ui";
import type { RefObject } from "react";
import { useId, useState } from "react";

import { useLocalizer } from "../i18n.ts";
import styles from "./explorer.module.css";
import { ValueDialog } from "./ValueDialog.tsx";

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
 *
 * The query menu (Q, `query.menu`; keyboard §7) sits beside the surface. Its answers are
 * announced by the app core (`nav_query`) and never move the cursor; closing it returns
 * the focus to the surface. The app opens it, so the palette can too.
 */
export function Explorer({
  client,
  dataset,
  cursor,
  surface,
  queryOpen,
  onQueryOpenChange,
}: {
  readonly client: Client;
  readonly dataset: DatasetSummary;
  readonly cursor: CursorState | null;
  /** The surface, for `view.focusExplorer`. */
  readonly surface: RefObject<HTMLDivElement | null>;
  /** Whether the query menu is open. */
  readonly queryOpen: boolean;
  readonly onQueryOpenChange: (open: boolean) => void;
}) {
  const t = useLocalizer();
  const hint = useId();
  // Value at x asks for the x first (the go-to dialog's "valueAt" kind).
  const [valueAt, setValueAt] = useState(false);
  const ask = (query: NavQuery) => {
    void client
      .call({ command: "nav_query", args: { query } })
      .catch(() => undefined);
  };
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
      <MenuButton
        label={t.format("ui-explorer-query")}
        isOpen={queryOpen}
        onOpenChange={(open) => {
          onQueryOpenChange(open);
          if (!open) requestAnimationFrame(() => surface.current?.focus());
        }}
        // APG menu button: an opened menu puts the focus on its first item, also when Q
        // opens it.
        // eslint-disable-next-line jsx-a11y/no-autofocus -- see above
        autoFocus="first"
        onAction={(key) => {
          if (key === "valueAt") {
            setValueAt(true);
          } else if (
            key === "summary" ||
            key === "minimum" ||
            key === "maximum" ||
            key === "range"
          ) {
            ask({ query: key });
          }
        }}
      >
        <MenuItem id="summary">{t.format("ui-query-summary")}</MenuItem>
        <MenuItem id="minimum">{t.format("ui-query-minimum")}</MenuItem>
        <MenuItem id="maximum">{t.format("ui-query-maximum")}</MenuItem>
        <MenuItem id="range">{t.format("ui-query-range")}</MenuItem>
        <MenuItem id="valueAt">{t.format("ui-query-value-at")}</MenuItem>
      </MenuButton>
      <ValueDialog
        kind={valueAt ? "valueAt" : null}
        onClose={() => {
          setValueAt(false);
          surface.current?.focus();
        }}
        onGo={(_kind, value) => {
          ask({ query: "valueAt", x: { number: value } });
        }}
      />
    </div>
  );
}
