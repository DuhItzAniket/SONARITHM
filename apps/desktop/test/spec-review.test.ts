// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// B05-T03 review fixes (#505): F8 at one offset, Format by tokens, Apply outcomes, the
// cleared check and the coalesced outside-change notice.
import assert from "node:assert/strict";
import { mock, test } from "node:test";

import { IpcFailure, connect, createMockTransport } from "@sonarithm/ipc";

import { coalesce } from "../src/spec/coalesce.ts";
import { formatJson } from "../src/spec/format.ts";
import { stepDiagnostic } from "../src/spec/speech.ts";
import { SpecStore, needsCheck } from "../src/spec/store.ts";

test("f8 reaches every diagnostic at one offset, then follows the caret once it moves", () => {
  // Two at the whole spec (offset 0), two at one pointer (offset 40), one later.
  const sorted = [
    { from: 0 },
    { from: 0 },
    { from: 40 },
    { from: 40 },
    { from: 90 },
  ];
  let last = null;
  const visited: number[] = [];
  let caret = 0;
  for (let i = 0; i < 6; i += 1) {
    const index = stepDiagnostic(sorted, caret, last, "next");
    assert.ok(index !== null);
    visited.push(index);
    caret = sorted[index]?.from ?? 0;
    last = { index, caret };
  }
  // From the caret at 0 the first step goes past 0, then each index in turn, wrapping.
  assert.deepEqual(visited, [2, 3, 4, 0, 1, 2]);
  // Backwards from index 1 reaches index 0, at the same offset.
  assert.equal(
    stepDiagnostic(sorted, 0, { index: 1, caret: 0 }, "previous"),
    0,
  );
  // The caret moved since: the step starts from the caret.
  assert.equal(stepDiagnostic(sorted, 50, { index: 0, caret: 0 }, "next"), 4);
  assert.equal(
    stepDiagnostic(sorted, 50, { index: 0, caret: 0 }, "previous"),
    3,
  );
  assert.equal(stepDiagnostic([], 0, null, "next"), null);
});

test("format keeps every digit of a large integer", () => {
  const text = '{"seed":9007199254740993,"tracks":[]}';
  const formatted = formatJson(text);
  assert.ok(formatted.ok);
  assert.equal(
    formatted.text,
    '{\n  "seed": 9007199254740993,\n  "tracks": []\n}\n',
  );
  // JSON.parse would have rounded it.
  const parsed = JSON.parse(text) as { seed: number };
  assert.notEqual(String(parsed.seed), "9007199254740993");
});

test("format keeps duplicate keys for the validator to report", () => {
  const formatted = formatJson('{"mode":"analysis","mode":"presentation"}');
  assert.ok(formatted.ok);
  assert.equal(
    formatted.text,
    '{\n  "mode": "analysis",\n  "mode": "presentation"\n}\n',
  );
});

test("format nests and leaves text with a syntax error alone", () => {
  const formatted = formatJson('{"a":[1,{"b":true}],"c":{}}');
  assert.ok(formatted.ok);
  assert.equal(
    formatted.text,
    '{\n  "a": [\n    1,\n    {\n      "b": true\n    }\n  ],\n  "c": {}\n}\n',
  );
  const broken = formatJson('{"a": tru}');
  assert.deepEqual(broken, { ok: false, offset: 6 });
});

const SPEC = '{"specVersion":"0.4","tracks":[]}';
const COUNTS = { errors: 0, warnings: 0, info: 0 };

async function storeWith(compile: () => never | object) {
  const client = await connect(
    createMockTransport({
      spec_set: (request) => ({
        reply: "spec" as const,
        text: request.args.text,
        undoToken: 1,
      }),
      compile: compile as never,
    }),
  );
  return new SpecStore(client);
}

test("apply says when the spec was set but could not be compiled", async () => {
  const failing = await storeWith(() => {
    throw new IpcFailure({
      code: "SNR-APP-003",
      message: "No dataset is open.",
      hint: null,
      pointer: null,
    });
  });
  const outcome = await failing.apply(SPEC);
  assert.equal(outcome.kind, "appliedNotCompiled");
  assert.equal(failing.snapshot().text, SPEC);

  const working = await storeWith(() => ({
    reply: "compile" as const,
    ok: true,
    diagnostics: [],
    counts: COUNTS,
    score: null,
  }));
  assert.equal((await working.apply(SPEC)).kind, "applied");
});

test("an empty editor clears the latest check", async () => {
  const store = await storeWith(() => ({
    reply: "compile" as const,
    ok: true,
    diagnostics: [],
    counts: COUNTS,
    score: null,
  }));
  store.clearChecked("");
  assert.deepEqual(store.snapshot().checked, {
    text: "",
    diagnostics: [],
    counts: COUNTS,
  });
});

test("outside changes are each said, and a burst is said once more", () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  try {
    let said = 0;
    const notice = coalesce(1500, () => {
      said += 1;
    });
    notice.trigger();
    assert.equal(said, 1);
    notice.trigger();
    notice.trigger();
    notice.trigger();
    assert.equal(said, 1);
    mock.timers.tick(1500);
    assert.equal(said, 2);
    mock.timers.tick(1500);
    assert.equal(said, 2);
    // A change after a quiet time is said at once.
    mock.timers.tick(5000);
    notice.trigger();
    assert.equal(said, 3);
    notice.cancel();
  } finally {
    mock.timers.reset();
  }
});

const CHECK = {
  reply: "validation" as const,
  diagnostics: [],
  counts: COUNTS,
};

test("a patch while edits are unapplied leaves the draft to be checked again", async () => {
  const draft = '{"specVersion":"0.4","tracks":[],"name":"draft"}';
  const client = await connect(
    createMockTransport({
      spec_validate: () => CHECK,
      spec_apply: () => ({
        reply: "spec_patch" as const,
        undoToken: 2,
        text: SPEC,
        diagnostics: [],
        counts: COUNTS,
      }),
    }),
  );
  const store = new SpecStore(client);
  await store.validate(draft);
  assert.equal(needsCheck(store.snapshot(), draft), false);
  // The inspector's patch replaces the check with one of the patched spec …
  await store.patch([{ op: "replace", path: "/name", value: "T" }], null);
  assert.equal(store.snapshot().checked?.text, SPEC);
  // … so the editor's draft needs a check of its own.
  assert.equal(needsCheck(store.snapshot(), draft), true);
  await store.validate(draft);
  assert.equal(needsCheck(store.snapshot(), draft), false);
});

test("a check that fails is said, and the next change checks again", async () => {
  let down = true;
  const client = await connect(
    createMockTransport({
      spec_validate: () => {
        if (down) throw new Error("The transport is down.");
        return CHECK;
      },
    }),
  );
  const store = new SpecStore(client);
  await store.validate(SPEC);
  assert.equal(store.snapshot().checking, false);
  assert.equal(store.snapshot().failed, SPEC);
  // Not again for the same text: that would loop while the host is down.
  assert.equal(needsCheck(store.snapshot(), SPEC), false);
  const changed = `${SPEC} `;
  assert.equal(needsCheck(store.snapshot(), changed), true);
  down = false;
  await store.validate(changed);
  assert.equal(store.snapshot().failed, null);
  assert.equal(store.snapshot().checked?.text, changed);
});

test("apply resets the text history as soon as the spec is set, before the compile", async () => {
  let release = (): void => undefined;
  const compiled = new Promise<void>((resolve) => {
    release = resolve;
  });
  const order: string[] = [];
  const client = await connect(
    createMockTransport({
      spec_set: (request) => ({
        reply: "spec" as const,
        text: request.args.text,
        undoToken: 1,
      }),
      compile: async () => {
        await compiled;
        order.push("compiled");
        return {
          reply: "compile" as const,
          ok: true,
          diagnostics: [],
          counts: COUNTS,
          score: null,
        };
      },
    }),
  );
  const store = new SpecStore(client);
  const outcome = store.apply(SPEC, () => order.push("reset"));
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.deepEqual(order, ["reset"]);
  release();
  assert.equal((await outcome).kind, "applied");
  assert.deepEqual(order, ["reset", "compiled"]);
});
