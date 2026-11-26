// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { SyntaxNode } from "@lezer/common";

import type { Segment } from "./pointer.ts";
import { constantOf, keysOf, parseSpec, pathTo } from "./pointer.ts";

/** The parts of a JSON Schema (2020-12) that completion reads; the rest is ignored. */
export interface Schema {
  readonly $ref?: string;
  readonly $defs?: Readonly<Record<string, Schema>>;
  readonly description?: string;
  readonly type?: string | readonly string[];
  readonly properties?: Readonly<Record<string, Schema>>;
  readonly additionalProperties?: Schema | boolean;
  readonly items?: Schema | boolean;
  readonly prefixItems?: readonly Schema[];
  readonly enum?: readonly unknown[];
  readonly const?: unknown;
  readonly oneOf?: readonly Schema[];
  readonly anyOf?: readonly Schema[];
  readonly allOf?: readonly Schema[];
}

/** One completion, independent of CodeMirror so it can be tested in Node. */
export interface SpecCompletion {
  /** What the list shows, as JSON: `"mode"`, `"analysis"`, `true`. */
  readonly label: string;
  /** The text inserted, when it differs from the label (`"mode": `). */
  readonly apply?: string;
  /** `property` for a key, `enum` for a value. */
  readonly type: "property" | "enum";
  /** The schema's description, first paragraph. */
  readonly info?: string;
}

/** The completions at a caret: the span they replace and the options. */
export interface SpecCompletions {
  readonly from: number;
  readonly to: number;
  readonly options: readonly SpecCompletion[];
}

const MAX_DEPTH = 32;

/** The schema `$ref` names, inside the root's `$defs`. */
function deref(root: Schema, schema: Schema, depth = 0): Schema {
  if (schema.$ref === undefined || depth > MAX_DEPTH) {
    return schema;
  }
  const name = /^#\/\$defs\/(.+)$/.exec(schema.$ref)?.[1];
  const target = name === undefined ? undefined : root.$defs?.[name];
  return target === undefined ? schema : deref(root, target, depth + 1);
}

/** The alternatives a value may match: `oneOf`, `anyOf` and `allOf` flattened. */
function alternatives(root: Schema, schema: Schema, depth = 0): Schema[] {
  const resolved = deref(root, schema);
  if (depth > MAX_DEPTH) {
    return [resolved];
  }
  const nested = [
    ...(resolved.oneOf ?? []),
    ...(resolved.anyOf ?? []),
    ...(resolved.allOf ?? []),
  ];
  const own =
    resolved.properties !== undefined ||
    resolved.enum !== undefined ||
    resolved.const !== undefined ||
    resolved.items !== undefined ||
    resolved.additionalProperties !== undefined ||
    resolved.type !== undefined;
  return [
    ...(own ? [resolved] : []),
    ...nested.flatMap((alt) => alternatives(root, alt, depth + 1)),
  ];
}

/** The schemas a value at `path` may match. */
export function schemasAt(root: Schema, path: readonly Segment[]): Schema[] {
  let current = alternatives(root, root);
  for (const part of path) {
    current = current.flatMap((schema): Schema[] => {
      if (typeof part === "number") {
        const item = schema.prefixItems?.[part] ?? schema.items;
        return typeof item === "object" ? alternatives(root, item) : [];
      }
      const property = schema.properties?.[part];
      if (property !== undefined) {
        return alternatives(root, property);
      }
      return typeof schema.additionalProperties === "object"
        ? alternatives(root, schema.additionalProperties)
        : [];
    });
  }
  return current;
}

function firstParagraph(text: string | undefined): string | undefined {
  const paragraph = text
    ?.split(/\n\s*\n/)[0]
    ?.replaceAll(/\s*\n\s*/g, " ")
    .trim();
  return paragraph === "" ? undefined : paragraph;
}

function types(schema: Schema): readonly string[] {
  return typeof schema.type === "string" ? [schema.type] : (schema.type ?? []);
}

/**
 * Whether an object alternative still fits what is written: a key whose schema is a
 * constant (a `kind` tag) must not already hold another value.
 */
function fits(
  root: Schema,
  schema: Schema,
  object: SyntaxNode,
  text: string,
): boolean {
  for (const [key, property] of Object.entries(schema.properties ?? {})) {
    const resolved = deref(root, property);
    if (resolved.const !== undefined) {
      const written = constantOf(object, key, text);
      if (written !== undefined && written !== resolved.const) {
        return false;
      }
    }
  }
  return true;
}

function keyOptions(
  root: Schema,
  object: SyntaxNode,
  text: string,
  withColon: boolean,
): SpecCompletion[] {
  const path = pathTo(object, text);
  const present = keysOf(object, text);
  const seen = new Map<string, SpecCompletion>();
  for (const schema of schemasAt(root, path)) {
    if (!fits(root, schema, object, text)) continue;
    for (const [key, property] of Object.entries(schema.properties ?? {})) {
      if (present.has(key) || seen.has(key)) continue;
      const label = JSON.stringify(key);
      const info = firstParagraph(
        property.description ?? deref(root, property).description,
      );
      seen.set(key, {
        label,
        type: "property",
        ...(withColon ? { apply: `${label}: ` } : {}),
        ...(info === undefined ? {} : { info }),
      });
    }
  }
  return [...seen.values()].sort((a, b) => a.label.localeCompare(b.label));
}

function valueOptions(
  root: Schema,
  path: readonly Segment[],
): SpecCompletion[] {
  const values = new Map<string, SpecCompletion>();
  const add = (value: unknown, description: string | undefined) => {
    const label = JSON.stringify(value);
    if (!values.has(label)) {
      const info = firstParagraph(description);
      values.set(label, {
        label,
        type: "enum",
        ...(info === undefined ? {} : { info }),
      });
    }
  };
  for (const schema of schemasAt(root, path)) {
    for (const value of schema.enum ?? []) add(value, schema.description);
    if (schema.const !== undefined) add(schema.const, schema.description);
    if (types(schema).includes("boolean")) {
      add(true, schema.description);
      add(false, schema.description);
    }
  }
  return [...values.values()];
}

/**
 * Whether the key being typed already has its colon right after it, so a key completion
 * must not add another. A colon further on belongs to the next key, which the parser can
 * join to an unfinished one.
 */
function hasColon(property: SyntaxNode | null, text: string): boolean {
  const name = property?.getChild("PropertyName");
  const colon = property?.getChild(":");
  if (name == null || colon == null) return false;
  return text.slice(name.to, colon.from).trim() === "";
}

const TOKENS = new Set([
  "PropertyName",
  "String",
  "Number",
  "True",
  "False",
  "Null",
  "⚠",
]);

/**
 * The completions at `pos` in `text`, from the generated spec schema (B05-T03): the keys an
 * object may still take, with their descriptions, or the values a key's schema allows
 * (enums, constants and booleans). `null` where nothing is known.
 */
export function completeAt(
  root: Schema,
  text: string,
  pos: number,
): SpecCompletions | null {
  const { tree } = parseSpec(text);
  const node = tree.resolveInner(pos, -1);
  const typing = TOKENS.has(node.name) && node.from < pos;
  const from = typing ? node.from : pos;
  const to = typing ? Math.max(node.to, pos) : pos;
  const done = (options: SpecCompletion[]): SpecCompletions | null =>
    options.length === 0 ? null : { from, to, options };

  if (node.name === "PropertyName") {
    const property = node.parent;
    const object = property?.parent;
    return object?.name === "Object"
      ? done(keyOptions(root, object, text, !hasColon(property, text)))
      : null;
  }
  if (node.name === "Object") {
    return done(keyOptions(root, node, text, true));
  }
  if (TOKENS.has(node.name) && node.parent?.name === "Object") {
    return done(keyOptions(root, node.parent, text, true));
  }
  // A value: after a key's colon, or in an array.
  const property =
    node.name === "Property"
      ? node
      : node.parent?.name === "Property"
        ? node.parent
        : null;
  if (property !== null) {
    const colon = property.getChild(":");
    if (colon === null || colon.to > pos) {
      const object = property.parent;
      return object?.name === "Object"
        ? done(keyOptions(root, object, text, colon === null))
        : null;
    }
    const valueNode = node.name === "Property" ? null : node;
    const path =
      valueNode === null
        ? [...pathTo(property, text), keyName(property, text)]
        : pathTo(valueNode, text);
    return path.includes("") ? null : done(valueOptions(root, path));
  }
  if (node.name === "Array" || node.parent?.name === "Array") {
    const array = node.name === "Array" ? node : node.parent;
    if (array === null) return null;
    const index = countBefore(array, pos);
    return done(valueOptions(root, [...pathTo(array, text), index]));
  }
  return null;
}

function keyName(property: SyntaxNode, text: string): string {
  const name = property.getChild("PropertyName");
  if (name === null) return "";
  try {
    const key: unknown = JSON.parse(text.slice(name.from, name.to));
    return typeof key === "string" ? key : "";
  } catch {
    return "";
  }
}

/** How many items of `array` end before `pos`: the index a value typed there gets. */
function countBefore(array: SyntaxNode, pos: number): number {
  let count = 0;
  for (
    let child = array.firstChild;
    child !== null;
    child = child.nextSibling
  ) {
    if (child.name === "," && child.to <= pos) count += 1;
  }
  return count;
}
