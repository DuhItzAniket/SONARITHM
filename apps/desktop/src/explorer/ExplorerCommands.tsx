// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, NavVerb } from "@sonarithm/ipc";
import { useState } from "react";

import { useCommand, useCommandTable } from "../commands/context.tsx";
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

/**
 * The chart explorer's commands (B03-T09; keyboard-and-navigation §4.2, §5.4), registered
 * for the whole app rather than with the Plot tab, so the palette lists and runs them from
 * anywhere (KB-01); their keys still work only while the explorer has the focus (KB-02).
 * Each sends a verb that moves Rust's one cursor (`nav_move`, ADR-0017 "Navigation"); the
 * table, the plot and the readout follow it. G and Primary+F ask for a value first; the
 * dialog returns the focus to wherever it was opened from.
 */
export function ExplorerCommands({
  client,
}: {
  readonly client: Client | null;
}) {
  const [dialog, setDialog] = useState<ValueKind | null>(null);
  const move = (verb: NavVerb) => {
    void client
      ?.call({ command: "nav_move", args: { verb } })
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
  useCommand("nav.whereAmI", () => {
    void client?.call({ command: "nav_where" }).catch(() => undefined);
  });
  useCommand("nav.repeat", () => {
    void client?.call({ command: "announce_repeat" }).catch(() => undefined);
  });
  return (
    <ValueDialog
      kind={dialog}
      onClose={() => {
        setDialog(null);
      }}
      onGo={(kind, value) => {
        move(
          kind === "goTo"
            ? { verb: "goTo", x: { number: value } }
            : { verb: "findValue", value },
        );
      }}
    />
  );
}
