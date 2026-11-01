// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The command registry, scope precedence and the keymap lint (B02-T07; keyboard §1, §2,
// §12, §13).
import assert from "node:assert/strict";
import { test } from "node:test";

import type { KeyInput, Platform } from "../src/commands/chords.ts";
import { chordOf, displayChord, normalize } from "../src/commands/chords.ts";
import type { CommandSpec } from "../src/commands/keymap.ts";
import {
  bindingsFor,
  DEFAULT_KEYMAP,
  REGION_COMMANDS,
} from "../src/commands/keymap.ts";
import { bindingProblems } from "../src/commands/lint.ts";
import { CommandRegistry } from "../src/commands/registry.ts";

function key(fields: Partial<KeyInput> & { key: string }): KeyInput {
  return {
    code: "",
    ctrlKey: false,
    altKey: false,
    shiftKey: false,
    metaKey: false,
    ...fields,
  };
}

test("kb_08_letters_bind_by_character_and_punctuation_by_physical_key", () => {
  assert.equal(
    chordOf(key({ key: "p", code: "KeyP", ctrlKey: true, shiftKey: true })),
    "Ctrl+Shift+P",
  );
  // AZERTY: the physical key of Period types ";" but binds as Period.
  assert.equal(
    chordOf(key({ key: ";", code: "Period", ctrlKey: true })),
    "Ctrl+Period",
  );
  assert.equal(
    chordOf(key({ key: "F6", code: "F6", shiftKey: true })),
    "Shift+F6",
  );
  assert.equal(
    chordOf(key({ key: "Shift", code: "ShiftLeft", shiftKey: true })),
    null,
  );
  assert.equal(normalize("Shift+Ctrl+P"), "Ctrl+Shift+P");
  assert.equal(displayChord("Shift+Meta+P", "mac"), "Shift+Cmd+P");
});

test("kb_03_kb_04_the_default_keymap_passes_the_lint_on_every_platform", () => {
  for (const platform of ["windows", "linux", "mac"] as Platform[]) {
    for (const spec of DEFAULT_KEYMAP) {
      for (const chord of bindingsFor(spec, platform)) {
        assert.deepEqual(
          bindingProblems(
            chord,
            spec.scope,
            platform,
            REGION_COMMANDS.has(spec.id),
          ),
          [],
          `${spec.id} ${chord} on ${platform}`,
        );
      }
    }
  }
});

test("kb_03_kb_04_the_lint_rejects_screen_reader_and_layout_conflicts", () => {
  const rules = (
    chord: string,
    platform: Platform = "windows",
    scope = "global" as const,
  ) => bindingProblems(chord, scope, platform).map((p) => p.rule);
  assert.deepEqual(rules("Ctrl+Insert"), ["KB-03"]);
  assert.deepEqual(rules("CapsLock"), ["KB-03"]);
  assert.ok(rules("Ctrl+Alt+P", "mac").includes("KB-03"));
  assert.deepEqual(rules("Ctrl+Alt+P"), ["KB-04"]);
  assert.deepEqual(rules("Ctrl+Alt+ArrowLeft"), ["KB-04"]);
  assert.deepEqual(rules("Alt+ArrowLeft"), ["KB-04"]);
  assert.deepEqual(rules("Alt+ArrowLeft", "linux"), ["KB-04"]);
  // KB-07: on macOS, Option+arrows are Jump, not history.
  assert.deepEqual(rules("Alt+ArrowLeft", "mac"), []);
  assert.ok(rules("Ctrl+Alt+ArrowLeft", "mac").includes("KB-03"));
  assert.deepEqual(rules("Ctrl+Space"), ["KB-04"]);
  assert.deepEqual(rules("P"), ["KB-02"]);
  assert.deepEqual(rules("Shift+P"), ["KB-02"]);
  assert.deepEqual(bindingProblems("P", "explorer", "windows"), []);
  assert.deepEqual(rules("Tab"), ["reserved"]);
  assert.deepEqual(rules("Alt+F4"), ["reserved"]);
  assert.deepEqual(rules("F6"), ["reserved"]);
  assert.deepEqual(rules("Meta+Q", "mac"), ["reserved"]);
});

function registry(specs: readonly CommandSpec[]) {
  const reg = new CommandRegistry(specs, "windows");
  const ran: string[] = [];
  for (const spec of specs) {
    reg.register(spec.id, { run: () => ran.push(spec.id) });
  }
  return { reg, ran };
}

const SPECS: CommandSpec[] = [
  { id: "a.global", name: "x", scope: "global", keys: ["Ctrl+F"] },
  { id: "a.table", name: "x", scope: "table", keys: ["Ctrl+F"] },
  {
    id: "a.region",
    name: "x",
    scope: "region",
    keys: ["Ctrl+Shift+Backslash"],
  },
  { id: "a.overlay", name: "x", scope: "overlay", keys: ["Ctrl+F"] },
  { id: "a.f6", name: "x", scope: "global", keys: ["F6"] },
];

test("scope_precedence_widget_over_region_over_global", () => {
  const { reg } = registry(SPECS);
  const inTable = {
    scopes: ["table", "region", "global"] as const,
    typing: false,
  };
  assert.equal(reg.resolve("Ctrl+F", inTable)?.id, "a.table");
  const inRegion = { scopes: ["region", "global"] as const, typing: false };
  assert.equal(reg.resolve("Ctrl+F", inRegion)?.id, "a.global");
  assert.equal(reg.resolve("Ctrl+Shift+Backslash", inRegion)?.id, "a.region");
  assert.equal(
    reg.resolve("Ctrl+Shift+Backslash", { scopes: ["global"], typing: false }),
    null,
  );
});

test("scope_precedence_overlay_first", () => {
  const { reg } = registry(SPECS);
  const inDialog = {
    scopes: ["overlay", "table", "region", "global"] as const,
    typing: false,
  };
  assert.equal(reg.resolve("Ctrl+F", inDialog)?.id, "a.overlay");
  // Outside the overlay, nothing runs while it is open...
  assert.equal(reg.resolve("F6", inDialog), null);
});

test("a11y_15_stop_all_audio_runs_even_in_an_overlay", () => {
  const reg = new CommandRegistry(DEFAULT_KEYMAP, "windows");
  reg.register("transport.stopAll", { run: () => undefined });
  const inDialog = { scopes: ["overlay", "global"] as const, typing: false };
  assert.equal(reg.resolve("Ctrl+Period", inDialog)?.id, "transport.stopAll");
});

test("scope_precedence_a_text_field_keeps_its_editing_keys", () => {
  const reg = new CommandRegistry(
    [{ id: "b.letter", name: "x", scope: "global", keys: ["K"] }, ...SPECS],
    "windows",
  );
  reg.register("b.letter", { run: () => undefined });
  reg.register("a.global", { run: () => undefined });
  reg.register("a.f6", { run: () => undefined });
  const typing = { scopes: ["global"] as const, typing: true };
  assert.equal(reg.resolve("K", typing), null);
  assert.equal(reg.resolve("Ctrl+F", typing)?.id, "a.global");
  assert.equal(reg.resolve("F6", typing)?.id, "a.f6");
});

test("kb_01_only_registered_commands_are_available_and_run", () => {
  const reg = new CommandRegistry(SPECS, "windows");
  assert.deepEqual(reg.available(), []);
  const ran: string[] = [];
  const off = reg.register("a.global", { run: () => ran.push("a.global") });
  assert.deepEqual(
    reg.available().map((c) => c.spec.id),
    ["a.global"],
  );
  assert.equal(reg.run("a.global"), true);
  off();
  assert.equal(reg.run("a.global"), false);
  assert.deepEqual(ran, ["a.global"]);
  assert.throws(() => reg.register("no.such", { run: () => undefined }));
});
