// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

export interface Summary {
  n: number;
  mean: number;
  p50: number;
  p95: number;
  p99: number;
  max: number;
}

/** Nearest-rank percentiles, rounded to 0.001. */
export function summarize(values: number[]): Summary {
  const s = [...values].sort((a, b) => a - b);
  const q = (p: number) => s[Math.min(s.length, Math.max(1, Math.ceil((p / 100) * s.length))) - 1] ?? NaN;
  const r = (x: number) => Math.round(x * 1000) / 1000;
  const mean = s.reduce((a, b) => a + b, 0) / Math.max(1, s.length);
  return { n: s.length, mean: r(mean), p50: r(q(50)), p95: r(q(95)), p99: r(q(99)), max: r(q(100)) };
}

/** Runs `f` `warm` times untimed, then `n` times, and returns the durations in ms. */
export async function timed(n: number, warm: number, f: () => Promise<unknown>): Promise<number[]> {
  for (let i = 0; i < warm; i++) await f();
  const out: number[] = [];
  for (let i = 0; i < n; i++) {
    const t0 = performance.now();
    await f();
    out.push(performance.now() - t0);
  }
  return out;
}
