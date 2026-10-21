# Technical benchmark report, A08 (v0.2.0-alpha candidate)

> **Status:** v0 (2026-10-21), roadmap A08-T04. **Owner:** Research lead (records); architecture lead (measurements).
> **Related:** [benchmarks log](benchmarks-log.md) · [findings log](findings-log.md) · [evaluation plan, technical evaluation](evaluation-plan.md#technical-evaluation) · [replication gallery](../../replication/README.md) · [P01 draft §6.2](papers/P01-icad-2027-toolkit/src/paper.md) · [ADR-0013](../decisions/0013-determinism-and-provenance.md)

This report collects the engine's measured latency, rendering speed and determinism for papers P01 (ICAD 2027 toolkit paper) and P07 (engine paper). Every number links to the benchmarks-log row or finding that holds its evidence. It adds one measurement of its own: the end-to-end export speed of the replication gallery. A result is stated against its budget even when the budget is not met.

## Hardware

Measurements on HW-01 (the development laptop) and HW-01e (HW-01 pinned to four efficiency cores, a stand-in for the slower reference laptop of 12 §8); determinism on the CI runners CI-WIN, CI-LNX and CI-MAC (AArch64). The machines are described in the [benchmarks log](benchmarks-log.md#reference-hardware). No reference-class machine is available yet (risk R-22).

## Results

Budget decisions recorded by the owner on 2026-10-21: PB-01 deferred to B03 ([#186](https://github.com/DuhItzAniket/SONARITHM/issues/186)); PB-03 waived for v0.1.0-alpha, to be reached before Gate B ([#185](https://github.com/DuhItzAniket/SONARITHM/issues/185)); PB-02 accepted, with a rerun at 128 frames ([#187](https://github.com/DuhItzAniket/SONARITHM/issues/187)); PB-06 (cross-OS determinism) accepted.

| Metric | Budget | Result | Verdict | Evidence |
|---|---|---|---|---|
| Key press to cue onset, p95 | ≤ 50 ms at 256 frames, 48 kHz | 104.0 ms through the engine's command path, WASAPI RAW shared mode (loopback self-test, 200 cues); 145.6 ms in non-RAW shared mode; 123.1 ms through cpal in the A02 spike | **Not met** on HW-01; the owner deferred it to B03 ([#186](https://github.com/DuhItzAniket/SONARITHM/issues/186)) | F-031, F-052; benchmarks log "A05 (Windows, WASAPI loopback)". The driver's shared-mode period is 480 frames (F-030) and an OEM enhancement component adds about 38 ms (F-032). A cable measurement is an owner step |
| Offline render speed, 4-track 10k-event Score | ≥ 50× real time | 91.4× (criterion) and 92–96× (`render_speed`) on HW-01; 43–46× on HW-01e | Met on HW-01, **not met** on the four-E-core stand-in; the owner waived it for v0.1.0-alpha, to be reached before Gate B ([#185](https://github.com/DuhItzAniket/SONARITHM/issues/185)) | F-053; benchmarks log "A05 (Windows, offline render)" |
| End-to-end export of the replication gallery (compile, render, 24-bit WAV, loudness meter; one thread) | — (new; PB-03 is the budget for rendering alone) | 131.7× real time on HW-01 (130.5–132.3 over 5 runs), 70.9× on HW-01e (70.3–71.3); 443.8 s of audio in 49 renders | — | F-115; benchmarks log "A08 (replication gallery, end to end)"; raw tables in [benchmarks/a08-replication-timings](benchmarks/a08-replication-timings/) |
| Compile, 100k rows × 4 tracks | ≤ 200 ms | 35.1 ms (HW-01), 54.3 ms (HW-01e), medians | Met | F-083 |
| Compile, 1M rows × 4 tracks | ≤ 1 s | 362 ms (HW-01), 545 ms (HW-01e) | Met | F-083 |
| 1M-row CSV load | ≤ 1.5 s | 0.51 s (HW-01), 0.91 s (HW-01e) | Met | F-070 |
| Score IR identical across OS | identical | NUMISONA's pinned Score hash and the 49 replication records match on Windows, macOS and Linux | Met | A06-T09; F-116 |
| Offline audio across OS | null residual ≤ −90 dBFS; LUFS Δ ≤ 0.1 | Byte-identical: A05's six golden cases (residual at the −240 dBFS floor, ΔLUFS 0 in `golden-compare`), the 49 replication-gallery WAV files (SHA-256 compared on all three OSes) and the A02 spike's renders | Met (stronger than the budget); PB-06 accepted by the owner | F-056 (#178 `golden-compare`); F-116; F-039 |
| Xruns in a 10-minute soak at 128 frames | 0 | 0 underruns and 0 allocating callbacks in 10-minute soaks at the periods HW-01 offers (480-frame WASAPI RAW shared, 144-frame exclusive); HW-01's driver cannot run 128-frame buffers; a 128-frame block renders in at most 14% of its time | Accepted by the owner, with a rerun at 128-frame buffers on capable hardware ([#187](https://github.com/DuhItzAniket/SONARITHM/issues/187)) | F-055 |
| Peak level of the gallery | ≤ −1 dBTP (limiter ceiling) | −6.84 dBTP maximum; the limiter acted on 0 frames in all 49 renders | — | replication `manifests.json` |

### End-to-end export by group

Medians of 5 runs per configuration; each row sums the group's specs. Compile and export times are per spec. The workload differs by group: Tucker Brown's curves have 1000 overlapping 0.2 s notes, about 20 sounding at once.

| Group | Specs | Audio (s) | Real-time factor, HW-01 | Real-time factor, HW-01e | Compile per spec, HW-01 (ms) | Export per spec, HW-01 (ms) |
|---|---|---|---|---|---|---|
| Smith & Walker (2005) | 6 | 60.0 | 204× | 120× | 0.73 | 48.3 |
| Nees & Walker (2008) | 16 | 162.0 | 209× | 124× | 0.54 | 47.9 |
| Tucker Brown et al. (2022) | 18 | 183.4 | 89× | 45× | 0.84 | 113.7 |
| Erie subset | 9 | 38.4 | 162× | 101× | 0.60 | 25.8 |
| All | 49 | 443.8 | 131.7× | 70.9× | — | — |

## Method

- **End-to-end export.** `cargo run --release -p xtask -- replication --timings` (A08-T02) renders every replication-gallery spec one at a time on one thread. For each, it times the compile (spec validation, data load from CSV, encoding, Score checks) and the export (render plan, offline render at 48 kHz in 256-frame blocks, 24-bit stereo WAV, EBU R128 and true-peak metering, the manifest and its SHA-256 of the file). The real-time factor is the audio duration over the sum of both. No loudness normalization (one render pass).
- **Runs and conditions.** Five runs on HW-01 and five on HW-01e (the process's affinity set to the four E-cores `0xF0000` right after it starts), on 2026-10-21: Windows 11 Home 10.0.26300, AC power, Balanced plan, Rust 1.99.0, release profile, commit `0103de8` (the A08-T02 branch on `2af548f`, which includes A05's voice optimizations, #178). No compiler or other heavy process was running; CPU load was 0% before the runs. Medians and ranges over runs. An earlier measurement on the code before #178 gave 99.5× and 55.4×; the only rendering change in between is A05's voice and limiter optimizations (#178, F-053).
- **Cross-OS identity.** The xtask test `the_replication_gallery_renders_reproducibly` re-renders all 49 specs and compares each record, including the WAV file's SHA-256 and the release-independent Score hash, with the committed one. CI runs it on CI-WIN, CI-LNX and CI-MAC for every pull request. Identical hashes mean identical bytes, so the null residual is −∞ dBFS and the loudness difference zero.
- **Reused numbers.** Latency, compile and load figures are those logged by A02, A04, A05 and A06; this report does not re-measure them.

## What the papers may claim

- **P01:** the compile and load budgets are met with headroom; offline exports of the replication gallery run at about 130× real time on the development laptop and 70× on four efficiency cores, one thread; A05's golden cases and the gallery's 49 renders are byte-identical on Windows, macOS (AArch64) and Linux. The latency budget is **not** met on the development machine (deferred to B03, [#186](https://github.com/DuhItzAniket/SONARITHM/issues/186)) and PB-03 is not met on the four-E-core stand-in (waived for v0.1.0-alpha, [#185](https://github.com/DuhItzAniket/SONARITHM/issues/185)); both must be reported as such (CL-12 stays unsupported).
- **P07:** the same, plus the latency analysis of F-030 to F-033 and F-051/F-052. PB-03 and the soak result come from A05's last bundle.
- Neither paper may present HW-01 numbers as reference-hardware results.

## Gaps

| Gap | Needed for | Owner and tracking |
|---|---|---|
| PB-03 on the four-E-core stand-in (43–46×, F-053): SIMD across voices or a reference-class machine | P07; CL-12 | Architecture lead; [#185](https://github.com/DuhItzAniket/SONARITHM/issues/185), before Gate B |
| PB-02 at a 128-frame device buffer (F-055) | P07 | Owner (hardware); [#187](https://github.com/DuhItzAniket/SONARITHM/issues/187) |
| PB-01 key-to-cue latency, including a cable loopback and macOS and Linux hardware | P07; CL-12 | Audio lead and owner; [#186](https://github.com/DuhItzAniket/SONARITHM/issues/186), at B03 (R-22) |
| A reference-class machine | All budgets | Owner (R-22) |
