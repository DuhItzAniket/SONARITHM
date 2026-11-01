// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Remapping (B03-T09; keyboard-and-navigation §13): the user's keymap replaces defaults,
// the group switch unbinds single keys, and the recorder refuses reserved combinations.
import assert from "node:assert/strict";
import { test } from "node:test";

import { DEFAULT_KEYMAP } from "../src/commands/keymap.ts";
import { record } from "../src/commands/recorder.ts";
import { CommandRegistry } from "../src/commands/registry.ts";

const explorer = {
  scopes: ["explorer", "region", "global"],
  typing: false,
} as const;

function registry() {
  const reg = new CommandRegistry(DEFAULT_KEYMAP, "windows");
  for (const spec of DEFAULT_KEYMAP) {
    reg.register(spec.id, { run: () => undefined });
  }
  return reg;
}

test("keyboard_13_the_users_keymap_replaces_the_defaults", () => {
  const reg = registry();
  reg.configure({ "nav.whereAmI": "Ctrl+Shift+W L", "nav.repeat": "" }, true);
  assert.deepEqual(reg.bindings("nav.whereAmI"), ["Ctrl+Shift+W", "L"]);
  assert.deepEqual(reg.bindings("nav.repeat"), [], "an empty value unbinds");
  assert.equal(reg.resolve("L", explorer)?.id, "nav.whereAmI");
  assert.equal(reg.resolve("W", explorer), null, "the default is gone");
  assert.equal(reg.resolve("R", explorer), null);
});

test("kb_02_with_single_keys_off_letters_do_nothing", () => {
  const reg = registry();
  reg.configure({}, false);
  for (const chord of ["N", "Shift+N", "X", "W", "S", "Shift+T", "Q", "H"]) {
    assert.equal(reg.resolve(chord, explorer), null, chord);
  }
  // Arrows, Home, End, Page keys and chords with Ctrl stay.
  assert.equal(reg.resolve("ArrowRight", explorer)?.id, "nav.next");
  assert.equal(reg.resolve("Home", explorer)?.id, "nav.first");
  assert.equal(reg.resolve("PageDown", explorer)?.id, "nav.nextSeries");
  assert.equal(reg.resolve("Ctrl+ArrowRight", explorer)?.id, "nav.jumpForward");
  assert.equal(reg.resolve("Ctrl+F", explorer)?.id, "nav.findValue");
  // A user-assigned chord with Ctrl stays too.
  reg.configure({ "nav.whereAmI": "Ctrl+Shift+W" }, false);
  assert.equal(reg.resolve("Ctrl+Shift+W", explorer)?.id, "nav.whereAmI");
});

test("keyboard_13_rule_6_the_recorder_refuses_reserved_combinations", () => {
  const reg = registry();
  const spec = DEFAULT_KEYMAP.find((s) => s.id === "nav.whereAmI");
  assert.ok(spec !== undefined);
  const check = (chord: string, platform: "windows" | "mac" = "windows") =>
    record(chord, spec, platform, (id) => reg.bindings(id), DEFAULT_KEYMAP);
  for (const chord of [
    "Tab",
    "Shift+Tab",
    "F6",
    "Shift+F6",
    "Ctrl+Insert",
    "CapsLock",
    "Alt+F4",
    "Alt+Space",
    "Meta+K",
  ]) {
    assert.equal(check(chord).ok, false, chord);
  }
  assert.equal(check("Meta+Q", "mac").ok, false);
  assert.equal(check("Ctrl+Alt+K", "mac").ok, false, "VoiceOver keys");
  assert.equal(
    check("Ctrl+Alt+ArrowLeft").ok,
    false,
    "screen-reader table keys",
  );
  const shadow = check("N");
  assert.ok(shadow.ok);
  assert.deepEqual(shadow.shadows, ["nav.nextInsight"]);
  const fresh = check("Shift+W");
  assert.ok(fresh.ok);
  assert.deepEqual(fresh.shadows, []);
});
