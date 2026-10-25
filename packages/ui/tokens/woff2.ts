// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * A minimal WOFF2 reader for the font tests (B02-T02): it lists the OpenType features in
 * GSUB and reads the name table, so a test can check that each bundled weight supports
 * tabular and slashed-zero figures and still carries IBM's own names (Reserved Font Name).
 *
 * WOFF 2.0 (W3C Recommendation): a header, a table directory, then one Brotli stream holding
 * every table in directory order. Only glyf, loca and hmtx can be transformed; GSUB and name
 * are always stored as they are in the font.
 */

import { brotliDecompressSync } from "node:zlib";

/** The table tags a WOFF2 directory can name by index (WOFF 2.0 §5.1). */
const KNOWN_TAGS = [
  "cmap", "head", "hhea", "hmtx", "maxp", "name", "OS/2", "post", "cvt ", "fpgm",
  "glyf", "loca", "prep", "CFF ", "VORG", "EBDT", "EBLC", "gasp", "hdmx", "kern",
  "LTSH", "PCLT", "VDMX", "vhea", "vmtx", "BASE", "GDEF", "GPOS", "GSUB", "EBSC",
  "JSTF", "MATH", "CBDT", "CBLC", "COLR", "CPAL", "SVG ", "sbix", "acnt", "avar",
  "bdat", "bloc", "bsln", "cvar", "fdsc", "feat", "fmtx", "fvar", "gvar", "hsty",
  "just", "lcar", "mort", "morx", "opbd", "prop", "trak", "Zapf", "Silf", "Glat",
  "Gloc", "Feat", "Sill",
] as const; // prettier-ignore

/** A font's tables as stored in the WOFF2 stream, and which of them are transformed. */
export interface Tables {
  readonly data: ReadonlyMap<string, Buffer>;
  readonly transformed: ReadonlySet<string>;
}

function readBase128(bytes: Buffer, offset: number): [number, number] {
  let value = 0;
  for (let i = 0; i < 5; i += 1) {
    const byte = bytes.readUInt8(offset + i);
    if (i === 0 && byte === 0x80) {
      throw new Error("WOFF2: UIntBase128 with a leading zero");
    }
    value = value * 128 + (byte & 0x7f);
    if ((byte & 0x80) === 0) {
      return [value, offset + i + 1];
    }
  }
  throw new Error("WOFF2: UIntBase128 longer than five bytes");
}

/** Every table of a WOFF2 font; transformed tables (glyf, loca, hmtx) stay transformed. */
export function readTables(file: Buffer): Tables {
  if (file.toString("latin1", 0, 4) !== "wOF2") {
    throw new Error("not a WOFF2 file");
  }
  if (file.toString("latin1", 4, 8) === "ttcf") {
    throw new Error("WOFF2 collections are not supported");
  }
  const count = file.readUInt16BE(12);
  const compressedLength = file.readUInt32BE(20);
  let offset = 48;
  const directory: { tag: string; length: number; transformed: boolean }[] = [];
  for (let i = 0; i < count; i += 1) {
    const flags = file.readUInt8(offset);
    offset += 1;
    const index = flags & 0x3f;
    let tag: string;
    if (index === 63) {
      tag = file.toString("latin1", offset, offset + 4);
      offset += 4;
    } else {
      const known = KNOWN_TAGS[index];
      if (known === undefined) {
        throw new Error(`WOFF2: unknown table index ${String(index)}`);
      }
      tag = known;
    }
    const version = flags >> 6;
    const [origLength, next] = readBase128(file, offset);
    offset = next;
    // glyf and loca are transformed unless the version is 3; other tables when it is not 0.
    const transformed =
      tag === "glyf" || tag === "loca" ? version !== 3 : version !== 0;
    let length = origLength;
    if (transformed) {
      [length, offset] = readBase128(file, offset);
    }
    directory.push({ tag, length, transformed });
  }
  const stream = brotliDecompressSync(
    file.subarray(offset, offset + compressedLength),
  );
  const data = new Map<string, Buffer>();
  const transformedTags = new Set<string>();
  let position = 0;
  for (const { tag, length, transformed } of directory) {
    data.set(tag, stream.subarray(position, position + length));
    if (transformed) {
      transformedTags.add(tag);
    }
    position += length;
  }
  if (position !== stream.length) {
    throw new Error("WOFF2: the table lengths do not add up to the stream");
  }
  return { data, transformed: transformedTags };
}

function table(tables: Tables, tag: string): Buffer {
  const data = tables.data.get(tag);
  if (data === undefined) {
    throw new Error(`no ${tag} table`);
  }
  return data;
}

/** The feature tags in the font's GSUB feature list, sorted and without duplicates. */
export function gsubFeatures(tables: Tables): string[] {
  const gsub = table(tables, "GSUB");
  const list = gsub.readUInt16BE(6);
  const count = gsub.readUInt16BE(list);
  const tags = new Set<string>();
  for (let i = 0; i < count; i += 1) {
    const record = list + 2 + i * 6;
    tags.add(gsub.toString("latin1", record, record + 4));
  }
  return [...tags].sort();
}

/** The Windows (platform 3, encoding 1, English US) string with name ID `id`. */
export function fontName(tables: Tables, id: number): string | undefined {
  const name = table(tables, "name");
  const count = name.readUInt16BE(2);
  const strings = name.readUInt16BE(4);
  for (let i = 0; i < count; i += 1) {
    const record = 6 + i * 12;
    const platform = name.readUInt16BE(record);
    const encoding = name.readUInt16BE(record + 2);
    const language = name.readUInt16BE(record + 4);
    if (
      platform === 3 &&
      encoding === 1 &&
      language === 0x409 &&
      name.readUInt16BE(record + 6) === id
    ) {
      const length = name.readUInt16BE(record + 8);
      const start = strings + name.readUInt16BE(record + 10);
      // UTF-16BE: swap to little-endian for Node's decoder.
      const utf16 = Buffer.from(name.subarray(start, start + length));
      return utf16.swap16().toString("utf16le");
    }
  }
  return undefined;
}

/** The weight class from the OS/2 table (400 regular, 500 medium, 600 semibold). */
export function weightClass(tables: Tables): number {
  return table(tables, "OS/2").readUInt16BE(4);
}

/** The glyph ID for `char` from the Unicode cmap (format 12 or 4), 0 when unmapped. */
export function glyphId(tables: Tables, char: string): number {
  const code = char.codePointAt(0) ?? 0;
  const cmap = table(tables, "cmap");
  const count = cmap.readUInt16BE(2);
  const subtables = new Map<string, number>();
  for (let i = 0; i < count; i += 1) {
    const record = 4 + i * 8;
    const key = `${String(cmap.readUInt16BE(record))}.${String(cmap.readUInt16BE(record + 2))}`;
    subtables.set(key, cmap.readUInt32BE(record + 4));
  }
  const full = subtables.get("3.10");
  if (full !== undefined && cmap.readUInt16BE(full) === 12) {
    const groups = cmap.readUInt32BE(full + 12);
    for (let i = 0; i < groups; i += 1) {
      const group = full + 16 + i * 12;
      const start = cmap.readUInt32BE(group);
      if (code >= start && code <= cmap.readUInt32BE(group + 4)) {
        return cmap.readUInt32BE(group + 8) + code - start;
      }
    }
    return 0;
  }
  const bmp = subtables.get("3.1");
  if (bmp === undefined || cmap.readUInt16BE(bmp) !== 4) {
    throw new Error("no Unicode cmap subtable");
  }
  const segments = cmap.readUInt16BE(bmp + 6) / 2;
  const ends = bmp + 14;
  const starts = ends + segments * 2 + 2;
  const deltas = starts + segments * 2;
  const rangeOffsets = deltas + segments * 2;
  for (let i = 0; i < segments; i += 1) {
    if (code > cmap.readUInt16BE(ends + i * 2)) {
      continue;
    }
    const start = cmap.readUInt16BE(starts + i * 2);
    if (code < start) {
      return 0;
    }
    const delta = cmap.readInt16BE(deltas + i * 2);
    const rangeOffset = cmap.readUInt16BE(rangeOffsets + i * 2);
    if (rangeOffset === 0) {
      return (code + delta) & 0xffff;
    }
    const glyph = cmap.readUInt16BE(
      rangeOffsets + i * 2 + rangeOffset + (code - start) * 2,
    );
    return glyph === 0 ? 0 : (glyph + delta) & 0xffff;
  }
  return 0;
}

/** The advance width of glyph `id` in font units, from hmtx (plain or WOFF2-transformed). */
export function advanceWidth(tables: Tables, id: number): number {
  const metrics = table(tables, "hhea").readUInt16BE(34);
  const hmtx = table(tables, "hmtx");
  const index = Math.min(id, metrics - 1);
  // A transformed hmtx starts with a flags byte, then the advance widths alone.
  return tables.transformed.has("hmtx")
    ? hmtx.readUInt16BE(1 + index * 2)
    : hmtx.readUInt16BE(index * 4);
}
