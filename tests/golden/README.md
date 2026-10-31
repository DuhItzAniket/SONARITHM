# Golden files

Checked-in expectations that more than one test suite reads.

| Directory | What | Checked by | Regenerate |
|---|---|---|---|
| `parity/` | A spec, its CSV, the Score the facade compiles for it (engine versions written as `*`, since they change with every release) and the SHA-256 of its default WAV render | `crates/sonarithm/tests/parity.rs` (Rust facade) and `bindings/python/tests/test_parity.py` (Python package): both front ends must produce this Score, value for value | `UPDATE_PARITY=1 cargo test -p sonarithm --test parity`, with the reason in the PR |
| `benchmark-10s/`, `compensation-sweep/`, `context-cues/`, `earcons/`, `glide-sweeps/`, `limiter-stress/`, `low-confidence-timbre/`, `monitoring-soundscape/`, `stereo-placement/`, `taps-and-short-notes/` | Golden-audio cases of the render engine: a `case.json` manifest (hashes and metrics, no audio) and, for handcrafted Scores, a `score.json` | `cargo xtask golden` and the `golden` and `golden-compare` CI jobs (A05-T11, below) | `cargo xtask golden --update --reason "<why>"`, in its own PR |

## Golden-audio cases (A05-T11)

Each directory is one case: `case.json` (the manifest) and, for handcrafted Scores, `score.json` (Score IR). The policy is in [testing-strategy §5](../../docs/engineering/testing-strategy.md#5-golden-audio-tests). The harness is `cargo xtask golden` (`xtask/src/golden.rs`).

**Storage (OD-08, decided in A05):** no reference audio is committed. A05 renders are bit-identical across Windows, Linux and macOS (F-039), so each manifest records the reference render's SHA-256 and its metrics: length, integrated loudness, true peak, sample peak, RMS and the spectral centroid of each second. CI renders every case on all three OSes, checks each against its manifest and at block sizes 64 and 1024, and compares the Windows and macOS renders with the Linux ones (PB-06). The renders stay downloadable as workflow artifacts (`golden-<os>`, 14 days) for listening.

| Case | What it covers |
|---|---|
| `benchmark-10s` | The DS-006 benchmark Score, 10 s: mallet and square data tracks, sine ticks, triangle monitoring |
| `compensation-sweep` | ISO 226:2023 compensation (SDR-24) over the expanded SDR-08 range |
| `earcons` | Every compiled earcon kind, with seeded noise (the B03-T06 set) |
| `glide-sweeps` | Glides with moving pitch, level, pan and brightness |
| `limiter-stress` | The master true-peak limiter on a +12 dB chord |
| `low-confidence-timbre` | The experimental low-confidence timbre (C02-T09b, SDR-43, RFC-0006): exact, then estimated values at confidence 1.0 and 0.3, compiled with `experimental.lowConfidenceTimbre` on. `crates/engines/sonarithm-numisona/tests/experimental.rs` checks that `score.json` matches a fresh compile; the Score with the switch off is checked to be byte-identical to one without the block |
| `monitoring-soundscape` | The monitoring soundscape (B06-T04, SDR-35): the bed brightening on three events of rising salience, `E-STREAM` and a configured `E-ALARM`. `crates/engines/sonarithm-sonotrace/src/monitor/tests.rs` checks that `score.json` is the module's output |
| `context-cues` | Tick clicks (SDR-16) and the dynamic reference tone (SDR-17) over a rising and falling mallet series, compiled with NUMISONA and AUDAXIS (B03-T05). `crates/engines/sonarithm-audaxis/tests/context.rs` checks that `score.json` matches a fresh compile |
| `stereo-placement` | x on pan from left to right and y on pitch, compiled with NUMISONA and AUDAXIS (B03-T11, SDR-30). `crates/engines/sonarithm-audaxis/tests/spatial.rs` checks that `score.json` matches a fresh compile |
| `taps-and-short-notes` | Tap counts 1–8 (A11Y-13) and notes from 2 ms to 40 ms (SDR-22) |

### Commands

```bash
cargo xtask golden
```

Checks every case. It fails when a render's length changes, when its loudness or centroid moves beyond the case's tolerances, or when another block size gives different audio. A changed hash within tolerance is reported, not failed.

```bash
cargo xtask golden --update --reason "why the output changed"
```

Rewrites the references, optionally only for named cases, and prints each case's change. Put the reason and the printed deltas in the PR, keep golden updates in their own PR, and listen to every changed case (headphones, low level) before approving (testing-strategy §5.3).
