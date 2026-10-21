// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import {
  BINARY,
  BINARY_VIEW_ERROR,
  PROVENANCE_CLASSES,
} from "./generated/protocol.ts";

/** A column's values, as views into the buffer where the layout allows. */
export type ColumnValues =
  | { readonly dataType: "f64"; readonly values: Float64Array }
  | { readonly dataType: "i64"; readonly values: BigInt64Array }
  | { readonly dataType: "utf8"; readonly values: readonly string[] }
  | { readonly dataType: "bool"; readonly values: Uint8Array }
  /** Provenance class codes; names in `PROVENANCE_CLASSES` (03 §4). */
  | { readonly dataType: "provenance"; readonly values: Uint8Array }
  /** Confidences in [0, 1]; a row without one is null in the validity bitmap. */
  | { readonly dataType: "confidence"; readonly values: Float32Array };

/** One column of a binary view. */
export type ViewColumn = ColumnValues & {
  readonly name: string;
  /** One byte per bit (least significant first), set where a row has a value; `null` when all do. */
  readonly validity: Uint8Array | null;
};

/** A decoded binary view (ADR-0017 §6). */
export interface BinaryView {
  readonly kind: number;
  readonly rows: number;
  readonly generation: number;
  readonly columns: readonly ViewColumn[];
}

/** The buffer is not a binary view this version reads. */
export class BinaryViewError extends Error {
  /** `SNR-APP-050`: the strict reader never guesses (ADR-0017 §6). */
  readonly code: string = BINARY_VIEW_ERROR;

  constructor(reason: string) {
    super(`Not a binary view this version reads: ${reason}.`);
    this.name = "BinaryViewError";
  }
}

const pad = (n: number): number => Math.ceil(n / BINARY.align) * BINARY.align;

/** Whether row `row` of a column has a value. */
export function isValid(column: ViewColumn, row: number): boolean {
  if (column.validity === null) {
    return true;
  }
  const byte = column.validity[Math.floor(row / 8)] ?? 0;
  return (byte & (1 << (row % 8))) !== 0;
}

/** Reads a binary view without copying its numeric columns. */
export function readView(buffer: ArrayBuffer): BinaryView {
  if (buffer.byteLength < BINARY.headerBytes) {
    throw new BinaryViewError("the buffer ends early");
  }
  const view = new DataView(buffer);
  const magic = new TextDecoder().decode(
    new Uint8Array(buffer, BINARY.offset.magic, 4),
  );
  if (magic !== BINARY.magic) {
    throw new BinaryViewError("wrong magic bytes");
  }
  if (view.getUint16(BINARY.offset.version, true) !== BINARY.headerVersion) {
    throw new BinaryViewError("unknown header version");
  }
  const kind = view.getUint16(BINARY.offset.kind, true);
  const count = view.getUint32(BINARY.offset.columns, true);
  const rows = Number(view.getBigUint64(BINARY.offset.rows, true));
  const generation = Number(view.getBigUint64(BINARY.offset.generation, true));
  const directoryEnd = BINARY.headerBytes + count * BINARY.columnEntryBytes;
  if (buffer.byteLength < directoryEnd) {
    throw new BinaryViewError("the buffer ends early");
  }
  const decoder = new TextDecoder("utf-8", { fatal: true });
  let nameAt = directoryEnd;
  const columns: ViewColumn[] = [];
  for (let index = 0; index < count; index += 1) {
    const entry = BINARY.headerBytes + index * BINARY.columnEntryBytes;
    const dataType = view.getUint16(entry + BINARY.offset.columnType, true);
    const flags = view.getUint16(entry + BINARY.offset.columnFlags, true);
    const nameLength = view.getUint32(
      entry + BINARY.offset.columnNameLen,
      true,
    );
    const dataAt = Number(
      view.getBigUint64(entry + BINARY.offset.columnData, true),
    );
    const name = decoder.decode(new Uint8Array(buffer, nameAt, nameLength));
    nameAt += nameLength;
    let validity: Uint8Array | null = null;
    if ((flags & BINARY.flagValidity) !== 0) {
      const length = Math.ceil(rows / 8);
      validity = new Uint8Array(buffer, dataAt - pad(length), length);
    }
    const base = { name, validity };
    switch (dataType) {
      case BINARY.dataType.f64:
        columns.push({
          ...base,
          dataType: "f64",
          values: new Float64Array(buffer, dataAt, rows),
        });
        break;
      case BINARY.dataType.i64:
        columns.push({
          ...base,
          dataType: "i64",
          values: new BigInt64Array(buffer, dataAt, rows),
        });
        break;
      case BINARY.dataType.bool:
        columns.push({
          ...base,
          dataType: "bool",
          values: new Uint8Array(buffer, dataAt, rows),
        });
        break;
      case BINARY.dataType.utf8: {
        const offsets = new Uint32Array(buffer, dataAt, rows + 1);
        const textAt = dataAt + pad(4 * (rows + 1));
        const values: string[] = [];
        for (let row = 0; row < rows; row += 1) {
          const start = offsets[row] ?? 0;
          const end = offsets[row + 1] ?? start;
          values.push(
            decoder.decode(new Uint8Array(buffer, textAt + start, end - start)),
          );
        }
        columns.push({ ...base, dataType: "utf8", values });
        break;
      }
      case BINARY.dataType.provenance: {
        const values = new Uint8Array(buffer, dataAt, rows);
        if (values.some((code) => code >= PROVENANCE_CLASSES.length)) {
          throw new BinaryViewError("unknown provenance class");
        }
        columns.push({ ...base, dataType: "provenance", values });
        break;
      }
      case BINARY.dataType.confidence:
        columns.push({
          ...base,
          dataType: "confidence",
          values: new Float32Array(buffer, dataAt, rows),
        });
        break;
      default:
        throw new BinaryViewError(`unknown data type ${String(dataType)}`);
    }
  }
  return { kind, rows, generation, columns };
}
