// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { parser } from "@lezer/json";

/** The result of formatting: the text, or why it was left as it was. */
export type Formatted =
  | { readonly ok: true; readonly text: string }
  | { readonly ok: false; readonly offset: number };

const OPEN = new Set(["{", "["]);
const CLOSE = new Set(["}", "]"]);
const VALUE_TOKENS = new Set([
  "PropertyName",
  "String",
  "Number",
  "True",
  "False",
  "Null",
]);

/**
 * Re-indents JSON by its tokens (B05-T03 Format, Shift+Alt+F): two spaces per level, one
 * member or item per line, empty objects and arrays kept as `{}` and `[]`. Every token is
 * copied as written, so a number keeps all its digits (a seed above 2^53 stays exact) and
 * duplicate keys stay for the validator to report; nothing is parsed into JavaScript values.
 * Text with a syntax error is not changed, and the error's offset is returned.
 */
export function formatJson(text: string): Formatted {
  const tree = parser.parse(text);
  const tokens: { name: string; text: string }[] = [];
  // Where the first syntax error is, if any (an object, so the callback's write is seen).
  const found: { error: number | null } = { error: null };
  tree.iterate({
    enter: (node) => {
      if (found.error !== null) return false;
      if (node.type.isError) {
        found.error = node.from;
        return false;
      }
      const name = node.name;
      if (
        VALUE_TOKENS.has(name) ||
        OPEN.has(name) ||
        CLOSE.has(name) ||
        name === "," ||
        name === ":"
      ) {
        tokens.push({ name, text: text.slice(node.from, node.to) });
        return false;
      }
      return true;
    },
  });
  if (found.error !== null) return { ok: false, offset: found.error };
  if (tokens.length === 0) return { ok: false, offset: 0 };

  let out = "";
  let depth = 0;
  const newline = () => `\n${"  ".repeat(depth)}`;
  tokens.forEach((token, index) => {
    const next = tokens[index + 1];
    if (OPEN.has(token.name)) {
      out += token.text;
      if (next !== undefined && CLOSE.has(next.name)) return;
      depth += 1;
      out += newline();
    } else if (CLOSE.has(token.name)) {
      const previous = tokens[index - 1];
      if (previous === undefined || !OPEN.has(previous.name)) {
        depth -= 1;
        out += newline();
      }
      out += token.text;
    } else if (token.name === ",") {
      out += `,${newline()}`;
    } else if (token.name === ":") {
      out += ": ";
    } else {
      out += token.text;
    }
  });
  return { ok: true, text: `${out}\n` };
}
