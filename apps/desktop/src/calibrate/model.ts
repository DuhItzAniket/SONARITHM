// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  AnchorText,
  AxisFitInfo,
  RasterAxisKind,
  RasterCalibration,
} from "@sonarithm/ipc";
import type { Localizer } from "@sonarithm/i18n";

/** An axis of the calibration dialog (C02-T10). */
export type Axis = "x" | "y";

/** One anchor as the dialog keeps it: the pixel (an index from 0) and the value entered. */
export interface DialogAnchor {
  readonly id: number;
  readonly axis: Axis;
  readonly at: readonly [number, number];
  /** A number for linear and log axes; the text of a date or label, which Rust reads. */
  readonly value: AnchorText;
}

/** What the dialog has entered: each axis's scale and every anchor in entry order. */
export interface DialogState {
  readonly kinds: Readonly<Record<Axis, RasterAxisKind>>;
  readonly anchors: readonly DialogAnchor[];
}

/** The axes' scales a calibration starts with. */
export const START: DialogState = {
  kinds: { x: "linear", y: "linear" },
  anchors: [],
};

/** Whether an axis's values are numbers (linear, log) or text that Rust reads (date, label). */
export function takesNumbers(kind: RasterAxisKind): boolean {
  return kind === "linear" || kind === "log";
}

/** The anchors of one axis, in entry order. */
export function anchorsOf(
  state: DialogState,
  axis: Axis,
): readonly DialogAnchor[] {
  return state.anchors.filter((anchor) => anchor.axis === axis);
}

/** An anchor's short name: its axis and its place among that axis's anchors, "x1", "y2". */
export function labelOf(state: DialogState, anchor: DialogAnchor): string {
  const index = anchorsOf(state, anchor.axis).findIndex(
    (a) => a.id === anchor.id,
  );
  return `${anchor.axis}${String(index + 1)}`;
}

/**
 * The calibration the app core reads (`RasterCalibration`, RFC-0004 §5): each anchor at its
 * pixel's centre, `i + 0.5`, with y down. An anchor whose value no longer fits its axis's
 * scale (a number left on an axis switched to dates) is sent as it is, and the app core
 * reports it.
 */
export function toCalibration(state: DialogState): RasterCalibration {
  const axis = (which: Axis) => ({
    kind: state.kinds[which],
    anchors: anchorsOf(state, which).map((anchor) => ({
      at: [anchor.at[0] + 0.5, anchor.at[1] + 0.5] as [number, number],
      value: anchor.value,
    })),
  });
  return { plotArea: null, x: axis("x"), y: axis("y") };
}

/** A value read back from the app core, for speech and display. */
export function valueText(
  t: Localizer,
  value: AnchorText | null,
): string | null {
  if (value === null) return null;
  return typeof value === "number"
    ? t.format("common-number", {
        value: { number: value, maximumFractionDigits: 4 },
      })
    : value;
}

/**
 * Each axis's state in words, x then y: how many anchors it has and how well they fit, or
 * the message of the check that refused it. The dialog shows these lines and announces
 * them only when they change.
 */
export function checkLines(
  t: Localizer,
  state: DialogState,
  fits: { readonly x: AxisFitInfo | null; readonly y: AxisFitInfo | null },
  messages: readonly string[],
): string[] {
  const lines = (["x", "y"] as const).map((axis) => {
    const name = t.format(
      axis === "x" ? "ui-calibrate-axis-x" : "ui-calibrate-axis-y",
    );
    const fit = fits[axis];
    const count = anchorsOf(state, axis).length;
    if (fit !== null) {
      return t.format("ui-calibrate-axis-fits", {
        axis: name,
        count: { number: fit.anchors },
        value: t.format("unit-percent", {
          style: "symbol",
          value: { number: fit.residual * 100, maximumFractionDigits: 2 },
        }),
      });
    }
    return count < 2
      ? t.format("ui-calibrate-axis-pending", {
          axis: name,
          count: { number: count },
        })
      : null;
  });
  return [
    ...lines.filter((line): line is string => line !== null),
    ...messages,
  ];
}

/**
 * Runs `send` with the latest value it was given, at most once per frame and never while a
 * send is in flight: values given meanwhile replace each other, and only the newest is sent
 * when the previous send ends. A crosshair held on an arrow key so asks the app core about
 * where it is now, not about every pixel it passed (ADR-0017 "Raster calibration").
 */
export class Latest<T> {
  private pending: { readonly value: T } | null = null;
  private busy = false;
  private scheduled = false;
  private readonly send: (value: T) => Promise<void>;
  private readonly schedule: (run: () => void) => void;

  constructor(
    send: (value: T) => Promise<void>,
    schedule: (run: () => void) => void,
  ) {
    this.send = send;
    this.schedule = schedule;
  }

  /** Asks for `value` to be sent; replaces any value not yet sent. */
  push(value: T): void {
    this.pending = { value };
    this.kick();
  }

  private kick(): void {
    if (this.busy || this.scheduled || this.pending === null) return;
    this.scheduled = true;
    this.schedule(() => {
      this.scheduled = false;
      const next = this.pending;
      if (next === null || this.busy) return;
      this.pending = null;
      this.busy = true;
      void this.send(next.value)
        .catch(() => undefined)
        .finally(() => {
          this.busy = false;
          this.kick();
        });
    });
  }
}
