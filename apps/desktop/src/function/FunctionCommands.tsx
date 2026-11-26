// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client } from "@sonarithm/ipc";
import { useState } from "react";

import type { Announcer } from "../announcer/announcer.ts";
import { useCommand } from "../commands/context.tsx";
import type { OpenedFunction } from "./FunctionDialog.tsx";
import { FunctionDialog } from "./FunctionDialog.tsx";

/**
 * The function commands (C03-T08b), registered for the whole app as the explorer's are,
 * so the palette runs them from anywhere (KB-01): "New function…" opens the dialog.
 */
export function FunctionCommands({
  client,
  announcer,
  onOpened,
}: {
  readonly client: Client | null;
  readonly announcer: Announcer;
  readonly onOpened: (opened: OpenedFunction) => void;
}) {
  const [open, setOpen] = useState(false);
  useCommand("data.newFunction", () => {
    setOpen(true);
  });
  return (
    <FunctionDialog
      client={client}
      announcer={announcer}
      isOpen={open}
      onClose={() => {
        setOpen(false);
      }}
      onOpened={onOpened}
    />
  );
}
