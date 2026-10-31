// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * What a dense layer draws (B04-T02; notes/05 §6). The app core does the downsampling: a
 * waveform arrives as one min/max pair per pixel column, a spectrogram as dB-scaled bytes,
 * and a dense line as up to a few million points. The layer only draws.
 */

/** An sRGB colour with channels in [0, 1]. */
export type Rgb = readonly [number, number, number];

/** A value range; `min < max`. */
export interface Range {
  readonly min: number;
  readonly max: number;
}

/** One polyline of a dense line view. */
export interface DenseSeries {
  /** The x values, ascending. */
  readonly x: ArrayLike<number>;
  /** The y values, one per x value. */
  readonly y: ArrayLike<number>;
  readonly color: Rgb;
}

/** What a dense layer draws. */
export type DenseScene =
  | {
      /** Polylines of many points (1M and more), x and y mapped through the ranges. */
      readonly kind: "lines";
      readonly series: readonly DenseSeries[];
      readonly x: Range;
      readonly y: Range;
    }
  | {
      /**
       * A waveform: one vertical span from `min[i]` to `max[i]` per pixel column, as the
       * app core's M4 tiles give it.
       */
      readonly kind: "waveform";
      readonly min: ArrayLike<number>;
      readonly max: ArrayLike<number>;
      readonly y: Range;
      readonly color: Rgb;
    }
  | {
      /**
       * A spectrogram: `columns` × `rows` magnitudes as bytes (0 is the floor, 255 the top
       * of the dB range), column by column from the lowest frequency, mapped through `lut`
       * (256 RGB triples, a perceptual colormap).
       */
      readonly kind: "spectrogram";
      readonly columns: number;
      readonly rows: number;
      readonly magnitudes: Uint8Array;
      readonly lut: Uint8Array;
    };

/** Positions in [0, 1] of `values` in `range`, as 32-bit floats for upload. */
export function normalize(
  values: ArrayLike<number>,
  range: Range,
): Float32Array {
  const span = range.max - range.min;
  const out = new Float32Array(values.length);
  for (let i = 0; i < values.length; i += 1) {
    out[i] = ((values[i] ?? range.min) - range.min) / span;
  }
  return out;
}
