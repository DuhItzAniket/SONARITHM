// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { BINARY } from "./generated/protocol.ts";

/** A column to encode, with plain values; `validity` marks the rows that have one. */
export type EncodeColumn = { readonly name: string } & (
  | { readonly dataType: "f64"; readonly values: readonly number[] }
  | { readonly dataType: "i64"; readonly values: readonly bigint[] }
  | { readonly dataType: "utf8"; readonly values: readonly string[] }
  | { readonly dataType: "bool"; readonly values: readonly boolean[] }
  | { readonly dataType: "provenance"; readonly values: readonly number[] }
  | { readonly dataType: "confidence"; readonly values: readonly number[] }
) & { readonly validity?: readonly boolean[] };

/** A binary view to encode (ADR-0017 §6). */
export interface EncodeView {
  /** `BINARY.kind`: 1 M4 tile, 2 LTTB tile, 3 table page. */
  readonly kind: number;
  readonly generation: number;
  readonly columns: readonly EncodeColumn[];
}

/** Bytes that grow, written little-endian. */
class Bytes {
  private buffer = new Uint8Array(256);
  length = 0;

  private reserve(more: number): void {
    if (this.length + more <= this.buffer.length) return;
    let size = this.buffer.length * 2;
    while (size < this.length + more) size *= 2;
    const next = new Uint8Array(size);
    next.set(this.buffer.subarray(0, this.length));
    this.buffer = next;
  }

  push(bytes: Uint8Array): void {
    this.reserve(bytes.length);
    this.buffer.set(bytes, this.length);
    this.length += bytes.length;
  }

  /** Zero bytes up to the next multiple of `BINARY.align`. */
  pad(): void {
    const target = Math.ceil(this.length / BINARY.align) * BINARY.align;
    this.reserve(target - this.length);
    this.buffer.fill(0, this.length, target);
    this.length = target;
  }

  view(): DataView {
    return new DataView(this.buffer.buffer, 0, this.length);
  }

  done(): ArrayBuffer {
    return this.buffer.slice(0, this.length).buffer;
  }
}

function values(column: EncodeColumn): Uint8Array {
  const n = column.values.length;
  switch (column.dataType) {
    case "f64":
      return new Uint8Array(Float64Array.from(column.values).buffer);
    case "i64":
      return new Uint8Array(BigInt64Array.from(column.values).buffer);
    case "bool":
      return Uint8Array.from(column.values, (v) => (v ? 1 : 0));
    case "provenance":
      return Uint8Array.from(column.values);
    case "confidence":
      return new Uint8Array(Float32Array.from(column.values).buffer);
    case "utf8": {
      const encoder = new TextEncoder();
      const texts = column.values.map((text) => encoder.encode(text));
      const offsets = new Uint32Array(n + 1);
      texts.forEach((text, row) => {
        offsets[row + 1] = (offsets[row] ?? 0) + text.length;
      });
      const offsetBytes = 4 * (n + 1);
      const at = Math.ceil(offsetBytes / BINARY.align) * BINARY.align;
      const out = new Uint8Array(at + (offsets[n] ?? 0));
      out.set(new Uint8Array(offsets.buffer), 0);
      let cursor = at;
      for (const text of texts) {
        out.set(text, cursor);
        cursor += text.length;
      }
      return out;
    }
  }
}

/**
 * Encodes a binary view byte for byte as the app core does (`crates/sonarithm-app/src/ipc/
 * binary.rs`), for mock hosts and tests. The UI itself only reads views (`readView`).
 */
export function encodeView(view: EncodeView): ArrayBuffer {
  const rows = view.columns[0]?.values.length ?? 0;
  if (view.columns.some((column) => column.values.length !== rows)) {
    throw new Error("columns differ in length");
  }
  const out = new Bytes();
  out.push(
    new Uint8Array(
      BINARY.headerBytes + BINARY.columnEntryBytes * view.columns.length,
    ),
  );
  const encoder = new TextEncoder();
  for (const column of view.columns) {
    out.push(encoder.encode(column.name));
  }
  out.pad();
  const entries: { flags: number; dataAt: number }[] = [];
  for (const column of view.columns) {
    let flags = 0;
    if (column.validity !== undefined) {
      flags |= BINARY.flagValidity;
      const bits = new Uint8Array(Math.ceil(column.validity.length / 8));
      column.validity.forEach((valid, row) => {
        if (valid)
          bits[Math.floor(row / 8)] =
            (bits[Math.floor(row / 8)] ?? 0) | (1 << (row % 8));
      });
      out.push(bits);
      out.pad();
    }
    const dataAt = out.length;
    out.push(values(column));
    out.pad();
    entries.push({ flags, dataAt });
  }
  const header = out.view();
  encoder.encodeInto(
    BINARY.magic,
    new Uint8Array(header.buffer, BINARY.offset.magic, 4),
  );
  header.setUint16(BINARY.offset.version, BINARY.headerVersion, true);
  header.setUint16(BINARY.offset.kind, view.kind, true);
  header.setUint32(BINARY.offset.columns, view.columns.length, true);
  header.setBigUint64(BINARY.offset.rows, BigInt(rows), true);
  header.setBigUint64(BINARY.offset.generation, BigInt(view.generation), true);
  view.columns.forEach((column, index) => {
    const entry = BINARY.headerBytes + BINARY.columnEntryBytes * index;
    const { flags, dataAt } = entries[index] ?? { flags: 0, dataAt: 0 };
    header.setUint16(
      entry + BINARY.offset.columnType,
      BINARY.dataType[column.dataType],
      true,
    );
    header.setUint16(entry + BINARY.offset.columnFlags, flags, true);
    header.setUint32(
      entry + BINARY.offset.columnNameLen,
      encoder.encode(column.name).length,
      true,
    );
    header.setBigUint64(entry + BINARY.offset.columnData, BigInt(dataAt), true);
  });
  return out.done();
}
