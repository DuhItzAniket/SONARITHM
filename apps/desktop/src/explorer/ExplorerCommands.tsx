// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Client,
  EffectiveSetting,
  NavVerb,
  ToggleTarget,
} from "@sonarithm/ipc";
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

/** The toggles (keyboard §5.5) and what each flips in the app core. */
const TOGGLES: readonly (readonly [string, ToggleTarget])[] = [
  ["mod.speech", "speech"],
  ["mod.sonification", "sonification"],
  ["mod.braille", "braille"],
  ["mod.visual", "visual"],
  ["mod.verbosity", "verbosity"],
];

/**
 * The chart explorer's commands (B03-T09; keyboard-and-navigation §4.2, §5.4), registered
 * for the whole app rather than with the Plot tab, so the palette lists and runs them from
 * anywhere (KB-01); their keys still work only while the explorer has the focus (KB-02).
 * Each sends a verb that moves Rust's one cursor (`nav_move`, ADR-0017 "Navigation"); the
 * table, the plot and the readout follow it. G and Primary+F ask for a value first; the
 * dialog returns the focus to wherever it was opened from.
 *
 * The toggles (T, S, B, V, Shift+T) flip the effective value in the app core
 * (`modality_toggle`), which saves the choice over the profile, acts on it and announces
 * it; `onEffective` gets the effective settings it replies with. A refusal (the last
 * non-visual output, SNR-APP-094) is announced by the app core.
 */
export function ExplorerCommands({
  client,
  onEffective,
}: {
  readonly client: Client | null;
  readonly onEffective: (settings: readonly EffectiveSetting[]) => void;
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
  useCommandTable(
    TOGGLES.map(([id, modality]) => [
      id,
      () => {
        void client
          ?.call({ command: "modality_toggle", args: { modality } })
          .then((reply) => {
            onEffective(reply.settings);
          })
          .catch(() => undefined);
      },
    ]),
  );
  useCommand("mod.reset", () => {
    void client
      ?.call({ command: "choices_reset" })
      .then((reply) => {
        onEffective(reply.settings);
      })
      .catch(() => undefined);
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
