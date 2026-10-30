// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// DL-X06 (B02-T09): no default Tailwind, shadcn, Radix or Material look. Those libraries are
// not dependencies of any workspace package, directly or through the lockfile; widgets come
// from React Aria Components and styles from CSS Modules and the design tokens.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";

const ROOT = new URL("../../../", import.meta.url);

/** Package names, or scopes ending in "/", that bring a ready-made look. */
const BANNED = [
  "tailwindcss",
  "@tailwindcss/",
  "shadcn",
  "@shadcn/",
  "@radix-ui/",
  "radix-ui",
  "@mui/",
  "@material-ui/",
  "@chakra-ui/",
  "@headlessui/",
  "bootstrap",
  "react-bootstrap",
  "daisyui",
  "styled-components",
  "@emotion/",
  "framer-motion",
] as const;

const banned = (name: string): boolean =>
  BANNED.some((entry) =>
    entry.endsWith("/") ? name.startsWith(entry) : name === entry,
  );

function manifests(): [string, Record<string, unknown>][] {
  const files = ["package.json"];
  for (const dir of ["apps", "packages", "bindings"]) {
    for (const entry of readdirSync(new URL(dir, ROOT), {
      withFileTypes: true,
    })) {
      if (entry.isDirectory()) {
        const file = `${dir}/${entry.name}/package.json`;
        try {
          readFileSync(new URL(file, ROOT));
          files.push(file);
        } catch {
          // Not a JavaScript package (the CLI, the Python bindings).
        }
      }
    }
  }
  return files.map((file) => [
    file,
    JSON.parse(readFileSync(new URL(file, ROOT), "utf8")) as Record<
      string,
      unknown
    >,
  ]);
}

test("dl_x06 no workspace package depends on a ready made look", () => {
  const found: string[] = [];
  for (const [file, manifest] of manifests()) {
    for (const field of [
      "dependencies",
      "devDependencies",
      "peerDependencies",
      "optionalDependencies",
    ]) {
      const deps = (manifest[field] ?? {}) as Record<string, string>;
      for (const name of Object.keys(deps).filter(banned)) {
        found.push(`${file} ${field} ${name}`);
      }
    }
  }
  assert.deepEqual(found, []);
});

test("dl_x06 the lockfile has no ready made look even transitively", () => {
  const lock = readFileSync(new URL("pnpm-lock.yaml", ROOT), "utf8");
  const names = new Set<string>();
  // Package keys in the `packages:` and `snapshots:` sections: `  name@version:` or
  // `  '@scope/name@version':`.
  for (const match of lock.matchAll(/^ {2}'?((?:@[^/@\s]+\/)?[^@\s':]+)@/gm)) {
    if (match[1] !== undefined) {
      names.add(match[1]);
    }
  }
  assert.ok(names.has("react-aria-components"), "the lockfile was parsed");
  assert.deepEqual([...names].filter(banned).sort(), []);
});
