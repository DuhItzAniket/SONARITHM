// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Chord, Platform } from "./chords.ts";
import { normalize, parseChord } from "./chords.ts";
import { isCharacter } from "./lint.ts";
import type { CommandSpec } from "./keymap.ts";
import { bindingsFor } from "./keymap.ts";
import type { ActiveScopes } from "./scopes.ts";

/** What a feature supplies for a command it implements. */
export interface CommandHandler {
  readonly run: () => void;
}

/** A command the palette can list: registered, with its current bindings. */
export interface AvailableCommand {
  readonly spec: CommandSpec;
  readonly bindings: readonly Chord[];
}

/**
 * Commands that run even while a dialog or menu is open: stopping all audio must work
 * from every scope (A11Y-15, B01-T08). Everything else outside the overlay waits until it
 * closes, so no command moves focus out from under a modal.
 */
export const OVERLAY_SAFE: ReadonlySet<string> = new Set(["transport.stopAll"]);

/** Whether a text field keeps this key for typing (keyboard §2: editing keys stay). */
function typedByTextField(chord: Chord): boolean {
  const { mods, key } = parseChord(chord);
  const modified = mods.has("Ctrl") || mods.has("Alt") || mods.has("Meta");
  return !modified && !/^F\d{1,2}$/.test(key);
}

/**
 * The command registry (B02-T07; keyboard-and-navigation §1, §2, §13): stable IDs with
 * names, scopes and per-platform bindings. Features register handlers; key presses resolve
 * to a command by scope precedence (overlay, focused widget, region, global), and the
 * palette lists every registered command (KB-01).
 */
export class CommandRegistry {
  private readonly handlers = new Map<string, CommandHandler>();
  private readonly listeners = new Set<() => void>();
  private readonly ran = new Set<(id: string) => void>();
  private snapshot: readonly AvailableCommand[] = [];
  /** The user's bindings by command (keyboard §13): they replace the defaults. */
  private overrides = new Map<string, readonly Chord[]>();
  /** The group switch of KB-02 (§13 rule 3). */
  private singleKeys = true;
  private readonly specs: readonly CommandSpec[];
  readonly platform: Platform;

  constructor(specs: readonly CommandSpec[], platform: Platform) {
    this.specs = specs;
    this.platform = platform;
  }

  /** Registers the handler for command `id`; returns a function that removes it. */
  register(id: string, handler: CommandHandler): () => void {
    if (!this.specs.some((spec) => spec.id === id)) {
      throw new Error(`no command ${id} in the keymap`);
    }
    this.handlers.set(id, handler);
    this.changed();
    return () => {
      if (this.handlers.get(id) === handler) {
        this.handlers.delete(id);
        this.changed();
      }
    };
  }

  /** The registered commands in keymap order (stable between changes, for React). */
  available(): readonly AvailableCommand[] {
    return this.snapshot;
  }

  /** The current bindings of a command on this platform. */
  bindings(id: string): readonly Chord[] {
    const spec = this.specs.find((s) => s.id === id);
    return spec === undefined ? [] : this.current(spec);
  }

  /**
   * Applies the user's keymap (B03-T09; keyboard §13): `keymap` maps a command ID to its
   * chords, separated by spaces, or the empty string to unbind it. With `singleKeyCommands`
   * off, every single-character binding (letters, Shift+letters, punctuation) is unbound
   * (KB-02, §13 rule 3); arrows, Home, End, Page Up, Page Down, Space and Escape stay.
   */
  configure(
    keymap: Readonly<Record<string, string>>,
    singleKeyCommands: boolean,
  ): void {
    this.overrides = new Map(
      Object.entries(keymap).map(([id, keys]) => [
        id,
        keys.split(" ").filter((chord) => chord !== ""),
      ]),
    );
    this.singleKeys = singleKeyCommands;
    this.changed();
  }

  /** The bindings of `spec` after the user's keymap and the group switch. */
  private current(spec: CommandSpec): readonly Chord[] {
    const chords =
      this.overrides.get(spec.id) ?? bindingsFor(spec, this.platform);
    if (this.singleKeys) {
      return chords;
    }
    return chords.filter((chord) => {
      const { mods, key } = parseChord(chord);
      const onlyShift = [...mods].every((m) => m === "Shift");
      return !(onlyShift && isCharacter(key));
    });
  }

  /**
   * The registered command bound to `chord` in the highest-precedence active scope, or
   * `null`. A text field keeps unmodified keys.
   */
  resolve(chord: Chord, active: ActiveScopes): CommandSpec | null {
    if (active.typing && typedByTextField(chord)) {
      return null;
    }
    const wanted = normalize(chord);
    const inOverlay = active.scopes.includes("overlay");
    for (const scope of active.scopes) {
      const match = this.snapshot.find(
        ({ spec, bindings }) =>
          spec.scope === scope &&
          (!inOverlay || scope === "overlay" || OVERLAY_SAFE.has(spec.id)) &&
          bindings.some((b) => normalize(b) === wanted),
      );
      if (match !== undefined) {
        return match.spec;
      }
    }
    return null;
  }

  /** Runs a registered command; returns whether it ran. */
  run(id: string): boolean {
    const handler = this.handlers.get(id);
    if (handler === undefined) {
      return false;
    }
    handler.run();
    for (const listener of this.ran) {
      listener(id);
    }
    return true;
  }

  /** Calls `listener` when the registered set changes; returns an unsubscribe. */
  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  /** Calls `listener` with the ID of every command that runs. */
  onRun(listener: (id: string) => void): () => void {
    this.ran.add(listener);
    return () => this.ran.delete(listener);
  }

  private changed(): void {
    this.snapshot = this.specs
      .filter((spec) => this.handlers.has(spec.id))
      .map((spec) => ({ spec, bindings: this.current(spec) }));
    for (const listener of this.listeners) {
      listener();
    }
  }
}
