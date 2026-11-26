// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import {
  createContext,
  use,
  useEffect,
  useRef,
  useSyncExternalStore,
} from "react";

import { chordOf } from "./chords.ts";
import type { AvailableCommand, CommandRegistry } from "./registry.ts";
import { activeScopes } from "./scopes.ts";

/** The app's command registry. */
export const CommandsContext = createContext<CommandRegistry | null>(null);

/** The command registry; throws outside {@link CommandsContext}. */
export function useCommands(): CommandRegistry {
  const registry = use(CommandsContext);
  if (registry === null) {
    throw new Error("useCommands outside CommandsContext");
  }
  return registry;
}

/** The registered commands, re-rendering when the set changes. */
export function useAvailableCommands(): readonly AvailableCommand[] {
  const registry = useCommands();
  return useSyncExternalStore(registry.subscribe, () => registry.available());
}

/**
 * Registers `run` as the handler of command `id` while the component is mounted; the
 * latest `run` is always the one called.
 */
export function useCommand(id: string, run: () => void): void {
  const registry = useCommands();
  const latest = useRef(run);
  useEffect(() => {
    latest.current = run;
  });
  useEffect(
    () =>
      registry.register(id, {
        run: () => {
          latest.current();
        },
      }),
    [registry, id],
  );
}

/**
 * Registers a table of handlers while the component is mounted, as {@link useCommand} does
 * for one; for a feature that implements many commands (the chart explorer). The IDs must
 * not change between renders; the latest handlers are always the ones called.
 */
export function useCommandTable(
  table: readonly (readonly [string, () => void])[],
): void {
  const registry = useCommands();
  const latest = useRef(table);
  useEffect(() => {
    latest.current = table;
  });
  const ids = table.map(([id]) => id).join(" ");
  useEffect(() => {
    const removers = ids.split(" ").map((id, index) =>
      registry.register(id, {
        run: () => {
          latest.current[index]?.[1]();
        },
      }),
    );
    return () => {
      for (const remove of removers) remove();
    };
  }, [registry, ids]);
}

/**
 * The one keyboard dispatcher (keyboard-and-navigation §2): each key press resolves to the
 * command bound in the highest-precedence active scope. Components never add their own
 * shortcut handlers (coding-standards §12.3); widgets keep their APG keys.
 */
export function useKeymap(registry: CommandRegistry): void {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const chord = chordOf(event);
      if (chord === null) {
        return;
      }
      const command = registry.resolve(
        chord,
        activeScopes(document.activeElement),
      );
      if (command !== null) {
        event.preventDefault();
        registry.run(command.id);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [registry]);
}
