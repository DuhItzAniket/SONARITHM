# Spike A02-T01: cpal output latency (Windows loopback harness)

> **Status:** Throwaway spike for the [A02 prototype gate](../../docs/sdlc/reviews/README.md). Deleted or archived after the gate. Not production code: no real-time review, no tests.
> **Task:** A02-T01 in [implementation-plan §2](../../docs/planning/implementation-plan.md). **Budget:** PB-01, key press → cue onset ≤ 50 ms p95 at 48 kHz with a 256-frame buffer ([12 §8](../../docs/architecture/12-cross-cutting-concerns.md#8-performance-budgets)). **Method reference:** [testing-strategy §14](../../docs/engineering/testing-strategy.md#14-latency-measurement).

## What it measures

A trigger on a control thread stands in for the key press. The next output callback starts a 1 kHz cue (40 ms, 5 ms raised-cosine ramps per SDR-22) at frame 0 of its buffer. A second stream captures the same endpoint in **WASAPI loopback mode**, and a threshold detector finds the cue onset. On WASAPI every cpal timestamp is a QueryPerformanceCounter (QPC) reading, so the trigger time, the callback entry and the capture timestamp share one clock, with no calibration needed.

| Column | Meaning |
|---|---|
| Total | Trigger → cue onset in the loopback capture (the PB-01 proxy) |
| Wait | Trigger → entry of the output callback that starts the cue (0 to 1 period) |
| Path | Callback entry → onset in the loopback capture |
| Reported | cpal's predicted `playback − callback` for that callback (queued frames plus `GetStreamLatency`) |
| Loopback peak | The loudest captured sample. It shows whether an enhancement APO changed the signal |
| Max gap | Longest interval between two output callbacks |
| Clock check | cpal's `callback` instant minus our QPC reading at callback entry; ≈ 0 confirms the shared clock |

Each mode runs 200 trials with seeded random inter-trial intervals of 400–700 ms, so triggers do not lock to block boundaries. The detector's known offset on the clean cue is subtracted.

### Modes

- `cpal:<frames>` and `cpal:default`: cpal 0.18.2 output with `BufferSize::Fixed(frames)` or `Default`.
- `raw[:<frames>]` and `direct[:<frames>]`: a direct WASAPI shared-mode renderer (`src/direct.rs`, `windows` 0.62). `raw` sets `AUDCLNT_STREAMOPTIONS_RAW`, which bypasses the OEM enhancement APOs; `<frames>` asks `IAudioClient3` for that engine period. cpal 0.18 can do neither (cpal #1173, #1343). These modes explain the cpal numbers and are not a proposal to drop cpal.

## Limits of the method

1. **Software path only.** The loopback tap is the audio engine's output. Time spent after it (endpoint buffer, driver, DAC, amplifier) is not measured, so the true acoustic latency is **at least** the loopback number. Sign-off needs a physical loopback, which this laptop does not have (no line-in, no Stereo Mix); see owner steps.
2. **The input path is excluded.** The trigger enters through the control thread (testing-strategy §14 step 2). Webview input handling and IPC are added from B03.
3. **Enhancement APOs are measured as installed.** On the development laptop, the Realtek endpoints run Nahimic processing: it adds about 20 dB of gain and a reverb-like tail of about 200 ms to the cue. This is a real user configuration, and it is why the trial spacing is 400–700 ms. The `raw` mode shows the same endpoint without it.
4. **Windows only.** macOS has no loopback capture in CoreAudio without a virtual device (BlackHole). Linux needs a PipeWire monitor source. Both are owner or volunteer steps.

## Run it

```bash
cargo run --release -- --trials 200 --modes cpal:128,cpal:256,cpal:512,cpal:default,raw --amplitude -45 --out results/run.json
```

Options: `--device <name part>` picks the output endpoint and `--capture <name part>` measures acoustically through an input device instead of loopback, with `--amplitude` and `--threshold` in dBFS (defaults −30 and −60). `cargo run --release --example devices` lists the endpoints. Use headphones or a low volume: the cue repeats every 0.4–0.7 s for about two minutes per mode.

## Results (Windows, 2026-10-19)

Machine: the owner's laptop (i7-14650HX, 16 GB, Windows 11 build 26300, on AC power, Balanced power plan). Endpoint: Realtek(R) Audio "Speakers", WASAPI shared mode, 48 kHz, 2 channels, f32, enhancements as installed. Cue at −45 dBFS. 200 trials per mode. Raw data: [`results/2026-10-19-windows-speakers.json`](results/2026-10-19-windows-speakers.json); table as printed: [`results/2026-10-19-windows-speakers.md`](results/2026-10-19-windows-speakers.md).

| Mode | Callback period (frames) | Total p50 (ms) | p95 | p99 | max | Wait p50 | Path p50 | cpal reported p50 | Loopback peak |
|---|---|---|---|---|---|---|---|---|---|
| cpal `Fixed(128)` | 480 | 116.8 | 123.7 | 125.9 | 126.1 | 5.4 | 111.7 | 31.8 | −24.4 dBFS |
| cpal `Fixed(256)` | 480 | 116.4 | **123.1** | 123.8 | 124.6 | 5.3 | 112.0 | 31.9 | −24.4 dBFS |
| cpal `Fixed(512)` | 480 | 117.2 | 124.0 | 125.2 | 126.3 | 5.7 | 112.5 | 31.8 | −24.4 dBFS |
| cpal `Default` | 480 | 116.9 | 124.0 | 125.5 | 126.3 | 5.3 | 112.1 | 31.7 | −24.4 dBFS |
| direct WASAPI, RAW | 480 | 79.2 | 83.7 | 84.2 | 84.4 | 4.9 | 74.3 | 12.0¹ | −45.0 dBFS |

¹ Padding plus `GetStreamLatency` at the callback, which counts less than cpal's formula (cpal adds frames the engine has read but the device position has not passed).

Every mode had 200 of 200 cues detected, no stray onsets, no output stream errors, and a clock check of 0.00 ms. The loopback stream reported 5–19 capture discontinuities per run; they do not move packet timestamps.

A 10-trial pilot on the wired "Headphones" endpoint, before they were unplugged, gave the same pattern: cpal `Fixed(256)` p50 113.5 ms and p95 122.7 ms; RAW p50 67.7 ms and p95 72.8 ms; asking `IAudioClient3` for a 128-frame period was clamped to 480 (the driver's minimum and maximum engine period are both 480).

### What the numbers say

1. **The buffer size request does nothing here.** The Realtek driver offers one shared-mode engine period, 480 frames (10 ms), and cpal 0.18 always calls back at that period. `Fixed(128/256/512)` changes neither the callback size (480 frames, 1056 on the first fill) nor the latency.
2. **PB-01 is missed by about 2.5×** on the software path alone (loopback method, see limit 1; not yet confirmed acoustically): p95 123.1 ms at the 256-frame setting against a 50 ms budget, before webview input and IPC are added. The headroom rule of 12 §8 does not apply, because the budget is not met at all.
3. **The enhancement APO costs about 38 ms** (p50 117 → 79 ms with RAW), and changes the signal (+20.6 dB gain, a tail of about 200 ms). Users with OEM audio software get this by default.
4. **Even without the APO, 79 ms p50 remains.** About 5 ms is waiting for the next callback; the remaining 74 ms is between the callback and the loopback tap.
5. **cpal's playback timestamp is about 80 ms early** on this machine, if the loopback timestamps are right: it predicts 31.8 ms, and the cue reaches the loopback tap after 112 ms. The playhead extrapolation of 06 §9 would be off by that much if it trusted cpal's timestamp.

These results trigger risk R-03 ("A02-T01 p95 > 50 ms"). Candidate mitigations for the A02 gate: a WASAPI exclusive-mode or RAW-mode backend (`wasapi` crate), ASIO as an opt-in, a latency calibration step that measures the real offset once per device, and pre-rendered cues.

### Not verified (owner steps)

- **Acoustic or cable confirmation.** The built-in microphone array runs echo cancellation, which removes the cue (captured peak −81 to −86 dBFS), so the speaker-to-microphone check failed. With a USB audio interface and a loopback cable: `cargo run --release -- --device "<interface>" --capture "<interface>" --amplitude -30 --threshold -50`.
- **Enhancements off.** Settings → System → Sound → Speakers → Audio enhancements: Off (and Nahimic disabled), then rerun the command under "Run it". This separates the APO cost from the engine path without RAW mode.
- **macOS (CoreAudio) and Linux (PipeWire).** Need a loopback source (BlackHole on macOS; a PipeWire monitor source on Linux) and a port of `--capture`; owner or volunteer task (risk R-22).
