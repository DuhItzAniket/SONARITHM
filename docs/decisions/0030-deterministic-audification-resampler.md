---
status: accepted
date: 2026-11-01
decision-makers: ["coordinator session, under @DuhItzAniket's delegation"]
---

# 0030 · A deterministic windowed-sinc resampler for audification, not rubato

## Context and problem statement

[RFC-0007](../rfcs/0007-function-cues-and-audification.md) §3 (accepted, [ADR-0029](0029-function-cues-and-audification.md)) says the plan builder resamples an `Audify` event's samples to the output rate "with rubato". Golden audio relies on renders being bit-identical on Windows, Linux and macOS ([F-039](../research-paper/findings-log.md), [ADR-0013](0013-determinism-and-provenance.md)). rubato's resamplers run their filters through FFTs in realfft and rustfft, which pick AVX, SSE or NEON code at run time. Their rounding can therefore differ between x86 and ARM machines ([F-195](../research-paper/findings-log.md#f-195--audification-resamples-with-a-deterministic-windowed-sinc-because-rubatos-fft-path-is-not-bit-identical-across-cpus); not measured, TODO(verify)), and so could the audio.

## Considered options

1. **rubato**, as RFC-0007 wrote, with golden tolerances loosened for audification cases.
2. **rubato in a slower non-FFT mode**, if one exists for every ratio. Its polynomial or sinc interpolators still use SIMD-dependent code paths (TODO(verify)).
3. **SONARITHM's own windowed-sinc resampler on `libm`.**

## Decision outcome

Chosen: **option 3** (coordinator under the owner's delegation, 2026-11-01). Bit-identical renders across CPUs come first under ADR-0013, and this adds no dependency. `sonarithm_render::resample` is a windowed-sinc interpolator:

- Blackman window, 16 taps a side at unity ratio.
- Widened to low-pass at the output's Nyquist frequency when downsampling, up to 4096 taps a side.
- `libm` and IEEE arithmetic only; normalized per output sample and clamped to [−1, 1].

It runs at plan time on a worker thread. The audifier voice reads the result with linear interpolation at the transport speed, so seeks and speed changes need no resampling on the audio thread. RFC-0007's amendments carry a pointer to this ADR; the RFC text is otherwise unchanged.

### Consequences

- Good: audification renders are as reproducible as every other render, and golden hashes stay exact.
- Good: no new crates (rubato would have brought audioadapter, realfft, rustfft and others).
- Neutral: rubato stays licence-checked (C03-T01) in case a deterministic mode of it is wanted later.
- Bad: slower than an FFT resampler: about 2·16/min(1, ratio) multiply-adds per output sample. That is acceptable for at most 2^22 samples on a worker thread, but a benchmark is still TODO.

### Confirmation

C03-T05 (issue [#318](https://github.com/DuhItzAniket/SONARITHM/issues/318)): the resampler's tests on known signals (a tone keeps its frequency within 1.5 Hz; a tone above the output's Nyquist frequency is attenuated below 1e-5), with thresholds set from local measurements on HW-01 (F-195) and other platforms TODO(verify) from CI, and the render test `sdr_31_an_audified_sine_plays_the_expected_tone`.
