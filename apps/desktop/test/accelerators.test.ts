// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Keyboard §12.3 and KB-04: browser accelerators do nothing in the webview, and keys the app
// needs are left alone.
import assert from "node:assert/strict";
import { test } from "node:test";

import type { KeyChord, KeyTarget } from "../src/keyboard/accelerators.ts";
import {
  installAcceleratorGuard,
  isBrowserAccelerator,
} from "../src/keyboard/accelerators.ts";

function chord(
  key: string,
  mods: Partial<Omit<KeyChord, "key">> = {},
): KeyChord {
  return {
    key,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    ...mods,
  };
}

test("kb_04_the_task_s_browser_accelerators_are_blocked", () => {
  for (const blocked of [
    chord("F5"),
    chord("r", { ctrlKey: true }),
    chord("R", { ctrlKey: true, shiftKey: true }),
    chord("f", { ctrlKey: true }),
    chord("F7"),
    chord("F12"),
    chord("ArrowLeft", { altKey: true }),
    chord("ArrowRight", { altKey: true }),
    chord("I", { ctrlKey: true, shiftKey: true }),
    chord("p", { ctrlKey: true }),
    chord("=", { ctrlKey: true }),
    chord("-", { ctrlKey: true }),
    chord("0", { ctrlKey: true }),
    chord("r", { metaKey: true }),
  ]) {
    assert.ok(isBrowserAccelerator(blocked), JSON.stringify(blocked));
  }
});

test("kb_04_navigation_and_app_keys_pass", () => {
  for (const kept of [
    chord("ArrowLeft"),
    chord("ArrowLeft", { ctrlKey: true }),
    chord("F6"),
    chord("F6", { shiftKey: true }),
    chord("F1"),
    chord("Tab"),
    chord("Escape"),
    chord(" "),
    chord("r"),
    chord("f", { ctrlKey: true, altKey: true }),
    chord("k", { ctrlKey: true }),
    chord("P", { ctrlKey: true, shiftKey: true }),
  ]) {
    assert.ok(!isBrowserAccelerator(kept), JSON.stringify(kept));
  }
});

test("the guard prevents the default action and can be removed", () => {
  type Listener = (event: KeyboardEvent) => void;
  const listeners = new Set<Listener>();
  const target: KeyTarget = {
    addEventListener: (_type, listener) => listeners.add(listener),
    removeEventListener: (_type, listener) => listeners.delete(listener),
  };
  const remove = installAcceleratorGuard(target);
  const press = (key: string, mods: Partial<KeyChord> = {}) => {
    let prevented = false;
    const event = {
      ...chord(key, mods),
      preventDefault: () => {
        prevented = true;
      },
    } as unknown as KeyboardEvent;
    for (const listener of listeners) listener(event);
    return prevented;
  };
  assert.equal(press("F5"), true);
  assert.equal(press("ArrowDown"), false);
  remove();
  assert.equal(listeners.size, 0);
});
