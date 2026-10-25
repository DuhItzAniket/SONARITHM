// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The bundled IBM Plex fonts (B02-T02): the files are IBM's, unmodified; every weight has
// tabular figures and a slashed zero; nothing loads from the network; the OFL notice is in
// the licences data; the test page covers every weight (A11Y-25, DL-X10).
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { test } from "node:test";

import {
  FONT_FACES,
  FONT_SOURCES,
  LICENSE_FILE,
  NUMERIC_CLASS,
  renderFontTest,
  renderFontsCss,
  renderLicenses,
} from "../tokens/fonts.ts";
import { FONT_DIR } from "../tokens/generate.ts";
import { STATIC_TOKENS } from "../tokens/recipe.ts";
import {
  advanceWidth,
  fontName,
  glyphId,
  gsubFeatures,
  readTables,
  weightClass,
} from "../tokens/woff2.ts";

function bytes(file: string): Buffer {
  return readFileSync(new URL(file, FONT_DIR));
}

const DIGITS = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

function sha256(data: Buffer): string {
  return createHash("sha256").update(data).digest("hex");
}

test("bundled font files are ibm's unmodified upstream files", () => {
  const onDisk = readdirSync(FONT_DIR).filter((f) => f !== "README.md");
  assert.deepEqual(
    onDisk.sort(),
    [...FONT_FACES.map((face) => face.file), LICENSE_FILE.file].sort(),
    "packages/ui/fonts holds exactly the listed files and the README",
  );
  for (const face of FONT_FACES) {
    assert.equal(sha256(bytes(face.file)), face.sha256, face.file);
  }
  assert.equal(sha256(bytes(LICENSE_FILE.file)), LICENSE_FILE.sha256);
});

test("fonts keep ibm's names and versions (reserved font name)", () => {
  for (const face of FONT_FACES) {
    const tables = readTables(bytes(face.file));
    // A renamed or modified Plex would change these; the OFL reserves the name "Plex".
    const family = fontName(tables, 16) ?? fontName(tables, 1);
    assert.equal(family, face.family, face.file);
    assert.equal(fontName(tables, 5), FONT_SOURCES[face.family].fontVersion);
    assert.match(fontName(tables, 0) ?? "", /IBM Corp/);
    assert.equal(weightClass(tables), face.weight, face.file);
  }
});

test("dl_x10 every weight has tabular figures and a slashed zero", () => {
  const widths = new Set<number>();
  for (const face of FONT_FACES) {
    const tables = readTables(bytes(face.file));
    const digits = DIGITS.map((d) => advanceWidth(tables, glyphId(tables, d)));
    // Plex has no tnum feature: its default figures are already tabular (F-132).
    assert.equal(new Set(digits).size, 1, `${face.file}: ${digits.join(" ")}`);
    widths.add(digits[0] ?? 0);
    const features = gsubFeatures(tables);
    assert.ok(features.includes("zero"), `${face.file}: ${features.join(" ")}`);
    assert.ok(features.includes("lnum"), face.file);
  }
  // The same digit width in every weight, so a readout does not shift when it turns bold.
  assert.equal(widths.size, 1, [...widths].join(" "));
});

test("a11y_25 no font loads from the network", () => {
  const css = renderFontsCss();
  const sources = [...css.matchAll(/src: ([^;]+);/g)].map((m) => m[1] ?? "");
  assert.equal(sources.length, FONT_FACES.length);
  for (const source of sources) {
    // Bundled files only: no URL with a scheme or host, and no local() system fonts.
    assert.match(
      source,
      /^url\("\.\.\/\.\.\/fonts\/[\w-]+\.woff2"\) format\("woff2"\)$/,
    );
  }
  const everyCss = [
    css,
    readFileSync(
      new URL("../src/generated/tokens.css", import.meta.url),
      "utf8",
    ),
    renderFontTest(),
  ].join("\n");
  assert.doesNotMatch(everyCss, /https?:|\/\/fonts\.|@import/);
});

test("the ofl notice is in the licences data", () => {
  const license = bytes(LICENSE_FILE.file).toString("utf8");
  const data = JSON.parse(renderLicenses(license)) as {
    assets: {
      name: string;
      license: string;
      copyright: string;
      files: string[];
    }[];
    licenseTexts: Record<string, string>;
  };
  assert.deepEqual(
    data.assets.map((asset) => asset.name),
    Object.keys(FONT_SOURCES),
  );
  for (const asset of data.assets) {
    assert.equal(asset.license, "OFL-1.1");
    assert.match(asset.copyright, /IBM Corp\. with Reserved Font Name "Plex"/);
    assert.ok(asset.files.length > 0);
  }
  const text = data.licenseTexts["OFL-1.1"] ?? "";
  assert.match(
    text,
    /^Copyright © 2017 IBM Corp\. with Reserved Font Name "Plex"/,
  );
  assert.match(text, /SIL OPEN FONT LICENSE Version 1\.1/);
});

test("dl_x10 the font test page shows tnum and zero for every weight", () => {
  const page = renderFontTest();
  for (const face of FONT_FACES) {
    for (const variant of ["tabular-nums", "slashed-zero", "var(--numeric)"]) {
      assert.ok(
        page.includes(`data-face="${face.file}" data-variant="${variant}"`),
        `${face.file} ${variant}`,
      );
    }
  }
});

test("the numeric class and the type tokens are defined", () => {
  const css = renderFontsCss();
  assert.ok(css.includes(`.${NUMERIC_CLASS} {`));
  assert.ok(css.includes("font-variant-numeric: var(--numeric);"));
  const tokens = new Map(STATIC_TOKENS);
  assert.equal(tokens.get("numeric"), "tabular-nums lining-nums slashed-zero");
  assert.match(tokens.get("font-sans") ?? "", /^"IBM Plex Sans", /);
  assert.match(tokens.get("font-mono") ?? "", /^"IBM Plex Mono", /);
  // design-language §7.1: 11/16, 12/16, 13/20, 15/20, 18/24 px, in rem at 16 px.
  const scale: readonly [number, number, number][] = [
    [11, 11, 16],
    [12, 12, 16],
    [13, 13, 20],
    [15, 15, 20],
    [18, 18, 24],
  ];
  for (const [step, size, line] of scale) {
    assert.equal(
      tokens.get(`type-${String(step)}-size`),
      `${String(size / 16)}rem`,
    );
    assert.equal(
      tokens.get(`type-${String(step)}-line`),
      `${String(line / 16)}rem`,
    );
  }
  assert.deepEqual(
    ["weight-regular", "weight-medium", "weight-semibold"].map((w) =>
      tokens.get(w),
    ),
    ["400", "500", "600"],
  );
});
