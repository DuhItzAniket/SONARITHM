// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { SyntaxNode, Tree } from "@lezer/common";
import { parser } from "@lezer/json";

import type { LineColumn, Span } from "./speech.ts";

export type { LineColumn, Span } from "./speech.ts";

/** One step of a JSON pointer: an object key or an array index. */
export type Segment = string | number;

/** The spec text parsed once, for placing several pointers. */
export interface ParsedSpec {
  readonly text: string;
  readonly tree: Tree;
}

/** Parses `text` with CodeMirror's JSON grammar, which keeps going past syntax errors. */
export function parseSpec(text: string): ParsedSpec {
  return { text, tree: parser.parse(text) };
}

/** The segments of a JSON pointer (RFC 6901), with `~1` and `~0` decoded. */
export function segments(pointer: string): readonly string[] {
  if (pointer === "") {
    return [];
  }
  return pointer
    .slice(1)
    .split("/")
    .map((part) => part.replaceAll("~1", "/").replaceAll("~0", "~"));
}

/** A JSON pointer for `path`. */
export function pointerOf(path: readonly Segment[]): string {
  return path
    .map(
      (part) => `/${String(part).replaceAll("~", "~0").replaceAll("/", "~1")}`,
    )
    .join("");
}

/** The value nodes of an object or array, in order (punctuation and errors skipped). */
function children(node: SyntaxNode): SyntaxNode[] {
  const out: SyntaxNode[] = [];
  for (let child = node.firstChild; child !== null; child = child.nextSibling) {
    if (node.name === "Object" ? child.name === "Property" : isValue(child)) {
      out.push(child);
    }
  }
  return out;
}

const VALUES = new Set([
  "Object",
  "Array",
  "String",
  "Number",
  "True",
  "False",
  "Null",
]);

function isValue(node: SyntaxNode): boolean {
  return VALUES.has(node.name);
}

/** A property's key as text, or `null` if it does not read as a JSON string. */
export function keyOf(property: SyntaxNode, text: string): string | null {
  const name = property.getChild("PropertyName");
  if (name === null) {
    return null;
  }
  try {
    const key: unknown = JSON.parse(text.slice(name.from, name.to));
    return typeof key === "string" ? key : null;
  } catch {
    return null;
  }
}

/** A property's value node, if it has one. */
function valueOf(property: SyntaxNode): SyntaxNode | null {
  for (
    let child = property.lastChild;
    child !== null;
    child = child.prevSibling
  ) {
    if (isValue(child)) {
      return child;
    }
    if (child.name === ":") {
      break;
    }
  }
  return null;
}

function rootValue(tree: Tree): SyntaxNode | null {
  for (
    let child = tree.topNode.firstChild;
    child !== null;
    child = child.nextSibling
  ) {
    if (isValue(child)) {
      return child;
    }
  }
  return null;
}

/**
 * Where a diagnostic's JSON pointer is in the text. A member of an object is its whole
 * property, key and value, so an unknown key is marked where it is written; an array item
 * is the item. A pointer into something that is not there (a missing key) is placed on
 * the nearest part that is, and the root on the opening brace. `null` when the text has
 * no value at all.
 */
export function locate(spec: ParsedSpec, pointer: string): Span | null {
  let node = rootValue(spec.tree);
  if (node === null) {
    return null;
  }
  let span: Span = { from: node.from, to: Math.min(node.from + 1, node.to) };
  for (const part of segments(pointer)) {
    if (node === null) break;
    if (node.name === "Object") {
      const property: SyntaxNode | undefined = children(node).find(
        (candidate) => keyOf(candidate, spec.text) === part,
      );
      if (property === undefined) break;
      span = { from: property.from, to: property.to };
      node = valueOf(property);
    } else if (node.name === "Array" && /^\d+$/.test(part)) {
      const item: SyntaxNode | undefined = children(node)[Number(part)];
      if (item === undefined) break;
      span = { from: item.from, to: item.to };
      node = item;
    } else {
      break;
    }
  }
  return span;
}

/** The first syntax error in the text, if any (where `SNR-SPEC-001` points). */
export function firstSyntaxError(spec: ParsedSpec): Span | null {
  let found: Span | null = null;
  spec.tree.iterate({
    enter: (node) => {
      if (found !== null) return false;
      if (node.type.isError) {
        const from = node.from;
        found = {
          from,
          to: Math.max(node.to, Math.min(from + 1, spec.text.length)),
        };
        return false;
      }
      return true;
    },
  });
  return found;
}

/**
 * The line and column of `offset` (a UTF-16 offset, as CodeMirror and the parser give) in
 * `text`. The column counts characters as people see them, by code point, so an emoji
 * before the place counts once, not twice.
 */
export function lineColumn(text: string, offset: number): LineColumn {
  let line = 1;
  let start = 0;
  const end = Math.min(Math.max(offset, 0), text.length);
  for (let i = 0; i < end; i++) {
    if (text.charCodeAt(i) === 10) {
      line += 1;
      start = i + 1;
    }
  }
  return { line, column: Array.from(text.slice(start, end)).length + 1 };
}

/** The path from the root to `node` (a value, or a property's key), as pointer segments. */
export function pathTo(node: SyntaxNode, text: string): Segment[] {
  const path: Segment[] = [];
  let child: SyntaxNode = node;
  for (let parent = node.parent; parent !== null; parent = parent.parent) {
    if (parent.name === "Property") {
      const key = keyOf(parent, text);
      if (key !== null && child.name !== "PropertyName") path.unshift(key);
    } else if (parent.name === "Array") {
      const index = children(parent).findIndex(
        (item) => item.from === child.from,
      );
      if (index >= 0) path.unshift(index);
    }
    child = parent;
  }
  return path;
}

/** The keys an object node already has. */
export function keysOf(object: SyntaxNode, text: string): Set<string> {
  const keys = new Set<string>();
  for (const property of children(object)) {
    const key = keyOf(property, text);
    if (key !== null) keys.add(key);
  }
  return keys;
}

/** A property's value as JSON, when it is a complete constant. */
export function constantOf(
  object: SyntaxNode,
  key: string,
  text: string,
): unknown {
  const property = children(object).find((p) => keyOf(p, text) === key);
  const value = property === undefined ? null : valueOf(property);
  if (value === null) return undefined;
  try {
    return JSON.parse(text.slice(value.from, value.to)) as unknown;
  } catch {
    return undefined;
  }
}
