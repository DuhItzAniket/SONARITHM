// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Client,
  CursorState,
  DatasetSummary,
  NavQuery,
  NavVerb,
  Settings,
} from "@sonarithm/ipc";
import { MenuButton, MenuItem } from "@sonarithm/ui";
import { useId, useRef, useState } from "react";

import type { RoutedAnnouncer } from "../announcer/announcer.ts";
import { useCommand, useCommandTable } from "../commands/context.tsx";
import { useLocalizer } from "../i18n.ts";
import { settingsOf } from "../preferences/settings.ts";
import styles from "./explorer.module.css";
import type { ValueKind } from "./ValueDialog.tsx";
import { ValueDialog } from "./ValueDialog.tsx";

/** The explorer's movement commands and the AUDAXIS verb each sends (keyboard §4.2). */
const MOVES: readonly (readonly [string, NavVerb])[] = [
  ["nav.next", { verb: "next" }],
  ["nav.previous", { verb: "previous" }],
  ["nav.jumpForward", { verb: "jumpForward" }],
  ["nav.jumpBack", { verb: "jumpBack" }],
  ["nav.first", { verb: "first" }],
  ["nav.last", { verb: "last" }],
  ["nav.into", { verb: "into" }],
  ["nav.activate", { verb: "activate" }],
  ["nav.out", { verb: "out" }],
  ["nav.nextSeries", { verb: "nextSeries" }],
  ["nav.previousSeries", { verb: "previousSeries" }],
  ["nav.maximum", { verb: "maximum" }],
  ["nav.minimum", { verb: "minimum" }],
  ["nav.nextInsight", { verb: "nextInsight" }],
  ["nav.previousInsight", { verb: "previousInsight" }],
  ["nav.xAxis", { verb: "landmark", landmark: "xAxis" }],
  ["nav.yAxis", { verb: "landmark", landmark: "yAxis" }],
  ["nav.insights", { verb: "landmark", landmark: "insights" }],
  ["nav.filters", { verb: "landmark", landmark: "filters" }],
  ["nav.dataset", { verb: "landmark", landmark: "dataset" }],
];

/** The modality toggles (keyboard §5.5) and the settings field each flips. */
const MODALITIES = [
  ["mod.speech", "speech"],
  ["mod.sonification", "sonification"],
  ["mod.braille", "braille"],
  ["mod.visual", "visual"],
] as const;

type Modality = (typeof MODALITIES)[number][1];

/**
 * The chart explorer (B03-T09; 08 §5, keyboard-and-navigation §4, §5.4, §5.5): one Tab stop
 * inside the Plot tab whose keys move Rust's one cursor through the navigation tree
 * (`nav_move`, ADR-0017 "Navigation"). The table, the plot and the readout follow that
 * cursor; this component holds no position of its own.
 *
 * Only the chart surface has `role="application"`, so NVDA and JAWS switch to focus or
 * forms mode there and pass its keys through. It is named after the dataset and described
 * by a visible hint. It has no `aria-activedescendant`: each move is spoken once, by the
 * app core's announcement, never also as a focus change (OD-30). Its single keys work only
 * while it has the focus (KB-02).
 */
export function Explorer({
  client,
  dataset,
  cursor,
  announcer,
}: {
  readonly client: Client;
  readonly dataset: DatasetSummary;
  readonly cursor: CursorState | null;
  readonly announcer: RoutedAnnouncer;
}) {
  const t = useLocalizer();
  const surface = useRef<HTMLDivElement>(null);
  const hint = useId();
  const [dialog, setDialog] = useState<ValueKind | null>(null);
  const [queryOpen, setQueryOpen] = useState(false);
  const ask = (query: NavQuery) => {
    void client
      .call({ command: "nav_query", args: { query } })
      .catch(() => undefined);
  };

  const move = (verb: NavVerb) => {
    void client
      .call({ command: "nav_move", args: { verb } })
      .catch(() => undefined);
  };
  useCommandTable(
    MOVES.map(([id, verb]) => [
      id,
      () => {
        move(verb);
      },
    ]),
  );
  useCommand("nav.goTo", () => {
    setDialog("goTo");
  });
  useCommand("nav.findValue", () => {
    setDialog("find");
  });
  useCommand("query.menu", () => {
    setQueryOpen(true);
  });
  useCommand("nav.whereAmI", () => {
    void client.call({ command: "nav_where" }).catch(() => undefined);
  });
  useCommand("nav.repeat", () => {
    void client.call({ command: "announce_repeat" }).catch(() => undefined);
  });
  useCommand("view.focusExplorer", () => {
    surface.current?.focus();
  });

  // The toggles change the user's settings, so they persist (keyboard §5.5); each says
  // what it did, because a modality may be the one that was speaking.
  const update = (change: (settings: Settings) => Settings, said: string) => {
    void client
      .call({ command: "settings_get" })
      .then((current) =>
        client.call({
          command: "settings_set",
          args: { settings: change(settingsOf(current)) },
        }),
      )
      .then(() => {
        announcer.announce({ text: said, assertive: false });
      })
      .catch(() => undefined);
  };
  const toggle = (modality: Modality) => {
    void client
      .call({ command: "settings_get" })
      .then(({ modalities }) => {
        const on = !modalities[modality];
        update(
          (settings) => ({
            ...settings,
            modalities: { ...settings.modalities, [modality]: on },
          }),
          t.format("ui-modality-changed", { modality, on: on ? "yes" : "no" }),
        );
      })
      .catch(() => undefined);
  };
  useCommandTable(
    MODALITIES.map(([id, modality]) => [
      id,
      () => {
        toggle(modality);
      },
    ]),
  );
  // Shift+B (keyboard §5.5, B03-T10): the braille line shows the review line or the
  // sparkline; a device setting, so it is saved in the settings, not a profile.
  useCommand("mod.brailleMode", () => {
    void client
      .call({ command: "settings_get" })
      .then(({ braille }) => {
        const mode = braille.mode === "sparkline" ? "review" : "sparkline";
        update(
          (settings) => ({
            ...settings,
            braille: { ...settings.braille, mode },
          }),
          t.format("ui-braille-mode-changed", { mode }),
        );
      })
      .catch(() => undefined);
  });
  useCommand("mod.verbosity", () => {
    void client
      .call({ command: "settings_get" })
      .then(({ verbosity }) => {
        const next = verbosity === "verbose" ? "terse" : "verbose";
        update(
          (settings) => ({ ...settings, verbosity: next }),
          t.format("ui-verbosity-changed", { verbosity: next }),
        );
      })
      .catch(() => undefined);
  });

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
      {/* The query menu (Q; keyboard §7): answers are announced by the app core and never
          move the cursor. Opened from the explorer, it returns the focus there. */}
      <MenuButton
        label={t.format("ui-explorer-query")}
        isOpen={queryOpen}
        onOpenChange={(open) => {
          setQueryOpen(open);
          if (!open) requestAnimationFrame(() => surface.current?.focus());
        }}
        // APG menu button: an opened menu puts the focus on its first item, also when Q
        // opens it.
        // eslint-disable-next-line jsx-a11y/no-autofocus -- see above
        autoFocus="first"
        onAction={(key) => {
          if (key === "valueAt") {
            setDialog("valueAt");
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
        kind={dialog}
        onClose={() => {
          setDialog(null);
          surface.current?.focus();
        }}
        onGo={(kind, value) => {
          if (kind === "valueAt") {
            ask({ query: "valueAt", x: { number: value } });
          } else {
            move(
              kind === "goTo"
                ? { verb: "goTo", x: { number: value } }
                : { verb: "findValue", value },
            );
          }
        }}
      />
    </div>
  );
}
