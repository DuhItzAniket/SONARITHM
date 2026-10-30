// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { Button, ToggleButton, Toolbar } from "@sonarithm/ui";

import { useCommand } from "../commands/context.tsx";
import { useLocalizer } from "../i18n.ts";
import type { TransportIntent } from "./machine.ts";
import type { TransportView } from "./useTransport.ts";

/** The intent of Play/Pause in this status: pause while playing, play otherwise. */
export function playPauseIntent(
  status: TransportView["status"],
): TransportIntent {
  return status === "playing" ? { kind: "pause" } : { kind: "play" };
}

/** Looping covers the whole Score; the loop editor (B05) sets other ranges. */
export function loopIntent(
  state: TransportView["state"],
  on: boolean,
): TransportIntent {
  return {
    kind: "loop",
    range: on && state !== null ? { startS: 0, endS: state.durationS } : null,
  };
}

/**
 * The transport toolbar (keyboard §5.3): Play/Pause, Stop and Loop, one Tab stop with
 * arrows inside (APG Toolbar). The buttons show what the app core reports, never what was
 * just pressed (B01-T05), and stay disabled until a Score is loaded. Play/Pause and Loop
 * are also commands, so the palette and the keymap reach them (DL-P06).
 */
export function TransportBar({
  transport,
}: {
  readonly transport: TransportView;
}) {
  const t = useLocalizer();
  const { status, state, intend } = transport;
  const loaded = state?.loaded === true;
  const looping = state?.loopRange !== null && state?.loopRange !== undefined;

  useCommand("transport.playPause", () => {
    if (loaded) intend(playPauseIntent(status));
  });
  useCommand("transport.stop", () => {
    if (loaded) intend({ kind: "stop" });
  });
  useCommand("transport.loop", () => {
    if (loaded) intend(loopIntent(state, !looping));
  });

  return (
    <Toolbar label={t.format("ui-transport")}>
      <Button
        variant="quiet"
        isDisabled={!loaded}
        onPress={() => {
          intend(playPauseIntent(status));
        }}
      >
        {t.format(status === "playing" ? "ui-pause" : "ui-play")}
      </Button>
      <Button
        variant="quiet"
        // Enabled even when stopped (it then returns to the start): disabling the focused
        // button after a press would drop the keyboard focus.
        isDisabled={!loaded}
        onPress={() => {
          intend({ kind: "stop" });
        }}
      >
        {t.format("ui-stop")}
      </Button>
      <ToggleButton
        isSelected={looping}
        isDisabled={!loaded}
        onChange={(on) => {
          intend(loopIntent(state, on));
        }}
      >
        {t.format("ui-loop")}
      </ToggleButton>
    </Toolbar>
  );
}
