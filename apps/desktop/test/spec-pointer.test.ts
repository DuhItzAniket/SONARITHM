// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// B05-T03: diagnostics carry JSON pointers (ADR-0017, DiagnosticInfo); the editor places
// them in the text, and completion reads the generated spec schema.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";

import type { Schema } from "../src/spec/completion.ts";
import { completeAt, schemasAt } from "../src/spec/completion.ts";
import {
  firstSyntaxError,
  lineColumn,
  locate,
  parseSpec,
  pointerOf,
  segments,
} from "../src/spec/pointer.ts";
import { SPEC_SCHEMA_FILE } from "../src/spec/schema-file.ts";

const SPEC = `{
  "specVersion": "0.4",
  "data": { "source": { "kind": "dataset", "id": "d1" } },
  "tracks": [
    { "id": "t", "mark": "tone", "channels": { "pitch": { "field": "tempx" } } }
  ]
}`;

const slice = (text: string, span: { from: number; to: number } | null) =>
  span === null ? null : text.slice(span.from, span.to);

test("a pointer to a member marks its key and value", () => {
  const spec = parseSpec(SPEC);
  assert.equal(
    slice(SPEC, locate(spec, "/tracks/0/channels/pitch/field")),
    '"field": "tempx"',
  );
  assert.equal(
    slice(SPEC, locate(spec, "/specVersion")),
    '"specVersion": "0.4"',
  );
});

test("a pointer to an array item marks the item", () => {
  const spec = parseSpec('{"a": [1, {"b": 2}]}');
  assert.equal(slice(spec.text, locate(spec, "/a/1")), '{"b": 2}');
});

test("a pointer to a missing key falls back to the nearest part that exists", () => {
  const spec = parseSpec(SPEC);
  assert.equal(
    slice(SPEC, locate(spec, "/tracks/0/channels/duration")),
    '"channels": { "pitch": { "field": "tempx" } }',
  );
  assert.equal(slice(SPEC, locate(spec, "")), "{");
  assert.equal(locate(parseSpec(""), ""), null);
});

test("pointer segments decode and encode RFC 6901 escapes", () => {
  assert.deepEqual(segments("/a~1b/c~0d/0"), ["a/b", "c~d", "0"]);
  assert.equal(pointerOf(["a/b", "c~d", 0]), "/a~1b/c~0d/0");
  assert.deepEqual(segments(""), []);
});

test("the first syntax error is found", () => {
  const text = '{"a": tru, "b": 1}';
  assert.equal(firstSyntaxError(parseSpec(text))?.from, 6);
  assert.equal(firstSyntaxError(parseSpec(SPEC)), null);
});

test("line and column count from one", () => {
  assert.deepEqual(lineColumn("ab\ncd", 0), { line: 1, column: 1 });
  assert.deepEqual(lineColumn("ab\ncd", 4), { line: 2, column: 2 });
});

const schema = JSON.parse(
  readFileSync(
    new URL(`../../../schemas/${SPEC_SCHEMA_FILE}`, import.meta.url),
    "utf8",
  ),
) as Schema;

test("the editor uses the newest generated spec schema", () => {
  const versions = readdirSync(new URL("../../../schemas/", import.meta.url))
    .map((name) => /^sonarithm-spec-(\d+)\.(\d+)\.schema\.json$/.exec(name))
    .filter((match) => match !== null)
    .map((match) => [Number(match[1]), Number(match[2]), match[0]] as const)
    .sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  assert.equal(versions.at(-1)?.[2], SPEC_SCHEMA_FILE);
  const editor = readFileSync(
    new URL("../src/spec/SpecEditor.tsx", import.meta.url),
    "utf8",
  );
  assert.ok(editor.includes(`schemas/${SPEC_SCHEMA_FILE}`));
});

const labels = (text: string, pos = text.indexOf("|")) =>
  completeAt(schema, text.replace("|", ""), pos)?.options.map((o) => o.label) ??
  [];

test("completion offers the top-level keys not yet written", () => {
  const offered = labels('{ "specVersion": "0.4", | }');
  assert.ok(offered.includes('"tracks"'));
  assert.ok(offered.includes('"mode"'));
  assert.ok(!offered.includes('"specVersion"'));
});

test("completion of a key being typed replaces the typed part and adds the colon", () => {
  const text = '{ "mo';
  const result = completeAt(schema, text, text.length);
  assert.ok(result !== null);
  assert.equal(result.from, 2);
  const mode = result.options.find((o) => o.label === '"mode"');
  assert.ok(mode !== undefined);
  assert.equal(mode.apply, '"mode": ');
  assert.ok((mode.info ?? "").length > 0);
});

test("completion offers a key's allowed values", () => {
  assert.deepEqual(labels('{ "mode": | }').sort(), [
    '"analysis"',
    '"monitoring"',
    '"presentation"',
  ]);
});

test("completion follows arrays and references into nested objects", () => {
  const offered = labels('{ "tracks": [ { "id": "t", | } ] }');
  assert.ok(offered.includes('"mark"'));
  assert.ok(offered.includes('"channels"'));
  assert.ok(schemasAt(schema, ["tracks", 0, "channels", "pitch"]).length > 0);
});

test("completion keeps to the alternative a kind tag chose", () => {
  const offered = labels('{ "data": { "source": { "kind": "dataset", | } } }');
  assert.ok(offered.includes('"id"'));
  assert.ok(!offered.includes('"path"'));
});

test("completion offers nothing where the schema knows nothing", () => {
  assert.deepEqual(labels('{ "x-note": { | } }'), []);
});

test("completion adds the colon when the next key's colon follows", () => {
  const text = '{\n  "na"\n  "specVersion": "0.4"\n}';
  const result = completeAt(schema, text, text.indexOf('na"') + 2);
  assert.ok(result !== null);
  const name = result.options.find((o) => o.label === '"name"');
  assert.equal(name?.apply, '"name": ');
});
