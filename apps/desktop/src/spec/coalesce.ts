// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/** A coalescing trigger: see {@link coalesce}. */
export interface Coalescer {
  /** Asks for a run: now if none ran lately, else once at the end of the quiet time. */
  readonly trigger: () => void;
  /** Drops a pending run (when the view goes away). */
  readonly cancel: () => void;
}

/**
 * Runs `run` at once on the first trigger, then at most once more per `quietMs`, at its end,
 * however many triggers came in between (B05-T03: "The spec changed outside the editor" is
 * said for each further change, but a burst of changes is said once more, not many times).
 */
export function coalesce(quietMs: number, run: () => void): Coalescer {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let waiting = false;
  const settle = () => {
    timer = null;
    if (waiting) {
      waiting = false;
      run();
      timer = setTimeout(settle, quietMs);
    }
  };
  return {
    trigger: () => {
      if (timer === null) {
        run();
        timer = setTimeout(settle, quietMs);
      } else {
        waiting = true;
      }
    },
    cancel: () => {
      if (timer !== null) clearTimeout(timer);
      timer = null;
      waiting = false;
    },
  };
}
