// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/** An x range in data units. */
export interface Range {
  readonly min: number;
  readonly max: number;
}

/** The narrowest view, as a share of the whole: zooming stops there. */
const MIN_SHARE = 1e-6;

/** `view` moved and narrowed to lie inside `full`, keeping its width where it fits. */
export function clampRange(view: Range, full: Range): Range {
  const fullSpan = full.max - full.min;
  const span = Math.min(
    Math.max(view.max - view.min, fullSpan * MIN_SHARE),
    fullSpan,
  );
  const min = Math.min(Math.max(view.min, full.min), full.max - span);
  return { min, max: min + span };
}

/** Where zooming centres: the cursor when it is in view, else the view's middle. */
function centreOf(view: Range, at: number | null): number {
  return at !== null && at >= view.min && at <= view.max
    ? at
    : (view.min + view.max) / 2;
}

/**
 * Zoom by `factor` (½ to zoom in, 2 to zoom out) around the cursor or the middle (B04-T07):
 * the keyboard and button alternative to drag zoom (A11Y-26).
 */
export function zoom(
  view: Range,
  full: Range,
  factor: number,
  at: number | null,
): Range {
  const centre = centreOf(view, at);
  const share = (centre - view.min) / (view.max - view.min || 1);
  const span = (view.max - view.min) * factor;
  return clampRange(
    { min: centre - share * span, max: centre + (1 - share) * span },
    full,
  );
}

/** Pan a quarter of the view left (−1) or right (1), the alternative to drag panning. */
export function pan(view: Range, full: Range, direction: -1 | 1): Range {
  const step = ((view.max - view.min) / 4) * direction;
  return clampRange({ min: view.min + step, max: view.max + step }, full);
}

/** Whether `view` shows all of `full`. */
export function isFull(view: Range, full: Range): boolean {
  return view.min <= full.min && view.max >= full.max;
}
