# 2026-10-25: PB-03 offline render speed on the E-core stand-in (#185)

> **Date(s):** 2026-10-25. **Phase:** B (carry-over from A05, implementation plan §3.4). **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [A05 phase review](../reviews/A05-phase-review.md) · [12 §8](../../architecture/12-cross-cutting-concerns.md#8-performance-budgets) · [F-053](../../research-paper/findings-log.md) · [benchmarks log](../../research-paper/benchmarks-log.md)

## Context and phase

- Issue: [#185](https://github.com/DuhItzAniket/SONARITHM/issues/185). The owner waived PB-03 (≥ 50× real time) for `v0.1.0-alpha` because the four-E-core stand-in HW-01e rendered the DS-006 benchmark Score at only 43–46× (F-053). The waiver needed closing before Gate B.
- Starting point: A05's voices render in runs over a scratch buffer. F-053 put the cost at about 10 ns per voice-sample, with the graph alone at 899×.
- Constraints:
  - output byte-identical (golden-compare across OSes, the golden cases, the replication records);
  - no allocation on the audio thread;
  - `libm` and polynomial oscillators kept (ADR-0013, F-039);
  - `CARGO_BUILD_JOBS=6` while the app-core session builds.

## Goals

- Profile before optimizing, make the render fast enough on HW-01e, prove the output unchanged, and record the result.

## What was done

- **Profiled by timing each primitive.** A sampling profiler on Windows needs administrator rights (ETW), so a temporary example timed each DSP primitive per sample on HW-01 and pinned to four E-cores. Weighted by the Score's voice mix (about 5 mallet, 5 square, 8 triangle and 0.5 sine voices at a time), three costs dominated:
  - the polynomial sine, about 9 ns per call on an E-core;
  - envelope ramps, which call the sine once per sample;
  - the biquad's serial chain.
- `sonarithm-dsp`:
  - `sine_turns_unit`: the same bits as `sine_turns` for turns in [0, 1.5), with a one-comparison reduction;
  - `Oscillator::fill` hoists the waveform out of the loop and computes a chunk of phases ahead, so the waveform vectorizes;
  - `Oscillator::fill_through` generates and filters in one loop;
  - `Adsr::apply` works a stage at a time;
  - the triangle's `floor` became an exact comparison.
- `sonarithm-render`: the mallet fills in chunks with `sine_turns_unit` when its phase and increment allow; filtered voices use `fill_through`; voices without a fade use `Adsr::apply`.
- Tests:
  - `render_identity.rs` pins the FNV-1a hash of the 60 s benchmark render to the A05 value, at 256- and 61-frame blocks;
  - four unit tests compare each fast path with its reference, bit for bit;
  - the existing no-allocation, hearing-safety and render tests pass unchanged.
- Measured and recorded: benchmarks log row, F-134. The A05 review's waiver is closed; status, the plan's carry-over table, traceability (NFR-01), 06 and CHANGELOG are updated.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Exact rewrites only, no change to any sample | Golden cases, golden-compare and the 49 replication records check exact output; a numeric change would force re-recording research artifacts | F-134 |
| No SIMD intrinsics or `unsafe` | Range-specialized reductions, chunked phases and loop fusion reached 60× on HW-01e; vectorization is left to the compiler, which keeps per-lane IEEE semantics on SSE2 and NEON | F-134; 06 §8 |
| The mallet's fast path only while its phase is below 1 and its increment below 1/2 | Then the phase provably stays in [0, 1), the range `sine_turns_unit` needs; a frequency near the sample rate falls back to the general path | `voice.rs` |
| The benchmark render's hash as a test | It guards every future speed change against silent output changes, at about 7 s for both renders in a debug build | `render_identity.rs` |
| No release-profile change (LTO, codegen units) | Not needed for the budget, and it would slow every build on the 16 GB laptop | — |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| Absolute speeds drifted between runs | The other agent session was building and testing on the same machine | Each measurement alternated the saved A05 binaries with the new ones in the same window |
| The two-add rounding trick saved only about 15% | The polynomial's operation count, not `round`, dominated the sine | Changed approach: specialise the reduction to the known range, and let the compiler vectorize across samples |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| `render_speed`, HW-01e | 59–64× (A05: 40–41×) | DS-006, 256-frame blocks, 48 kHz, one thread, `start /affinity F0000`, three alternating rounds | benchmarks log, F-134 |
| criterion, HW-01e | 0.99 s = 60× (A05: 1.51–1.55 s) | as above | benchmarks log, F-134 |
| `render_speed` and criterion, HW-01 | 107× and 689 ms (A05: 72× and 974 ms) | as above, unpinned, busy machine | benchmarks log, F-134 |

## Findings to log

- F-134: exact per-run voice loops render the benchmark Score at 60× real time on four E-cores; PB-03 met with bit-identical output — [findings log](../../research-paper/findings-log.md)

## Next steps

- [ ] Rerun PB-03 on reference-class hardware when one is available (R-22); not required to close #185.
