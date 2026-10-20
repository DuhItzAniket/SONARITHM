# A05-T13: cable latency test (ADR-0016 confirmation)

> **Task:** A05-T13 (issue #123). **Decision under test:** [ADR-0016](../../decisions/0016-windows-low-latency-backend.md), its Confirmation section. **Budget:** PB-01, key press to cue onset ≤ 50 ms p95 at 256 frames and 48 kHz ([12 §8](../../architecture/12-cross-cutting-concerns.md#8-performance-budgets)). **Method:** [testing-strategy §14](../testing-strategy.md#14-latency-measurement). **Program:** `crates/sonarithm-audio/examples/latency.rs`.

## What it checks

ADR-0016 is confirmed when a physical cable loopback shows:

1. key press to cue onset p95 in the low-latency mode (exclusive) ≤ 50 ms;
2. the RAW shared-mode p95 next to it;
3. the calibrated playhead offset against the measured onset;
4. that NVDA speech and sonification coexist in the default mode (RAW shared).

The loopback self-test that A05 ran in software measured only the software path (F-052). This test adds the converter and driver latency, and it is the sign-off number.

## You need

- A USB audio interface with a line or headphone output and a line input, or any output and input you can connect with one cable. The laptop's own jack is a combined headset jack, so for the laptop runs, take the laptop's headphone output into the interface's input.
- One cable to match (3.5 mm or 6.35 mm TRS to the input). If the input is a microphone input, add an attenuator or turn its gain to the minimum.
- NVDA (current release) for step 6.
- About 45 minutes.

## Hearing safety

- No one wears headphones during the runs: the output goes into the cable.
- Set the output volume to about 20% and the input gain to its minimum before connecting anything.
- The program plays 50 ms beeps at −30 dBFS in RAW mode. Without RAW it caps them at −50 dBFS, because OEM processing can add about 20 dB after SONARITHM's limiter (F-032).

## Steps

1. Connect the cable from the output under test to the interface's input. Close other audio applications.
2. List the capture devices and copy the identifier of the interface's input:

   ```bash
   cargo run -p sonarithm-audio --example latency --release -- --list
   ```

3. **Interface, RAW shared mode (the default).** Make the interface the default output, then run (replace `<input id>`):

   ```bash
   cargo run -p sonarithm-audio --example latency --release -- --mode raw --capture "<input id>" --trials 200
   ```

   If it reports fewer than 190 detected cues, raise the input gain a little and run it again. Record the whole output.
4. **Interface, exclusive mode (the low-latency setting).** Run the same command with `--mode exclusive`. Exclusive mode silences every other application on the device while it runs. Record the output.
5. **Laptop.** Make the laptop speakers the default output, move the cable to the laptop's headphone jack and the interface input, and repeat steps 3 and 4.
6. **NVDA coexistence, default mode.** Start NVDA. Make the device you use every day the default output. Run `cargo run -p sonarithm-audio --example hotplug --release` (very quiet pulses through cpal in default shared mode; cpal cannot open RAW streams) and, while it plays, let NVDA read a paragraph (NVDA+Down arrow on a text page). Note whether you hear both. Then run step 4's command and let NVDA speak during it: speech is expected to go silent, which is why exclusive mode is opt-in.

## Record

In issue #123, with the program output attached:

| Run | Device | Mode | Buffer or period reported | Detected | p50 ms | p95 ms | p99 ms | Offset ms | Xruns |
|---|---|---|---|---|---|---|---|---|---|
| 3 | Interface | RAW shared | | | | | | | |
| 4 | Interface | Exclusive | | | | | | | |
| 5a | Laptop | RAW shared | | | | | | | |
| 5b | Laptop | Exclusive | | | | | | | |

Also record the OS build, the interface model and driver version, the cable route, and the NVDA result of step 6 (both audible in RAW shared mode: yes or no; speech silent in exclusive mode: yes or no).

The agent session then:

- adds the rows to the [benchmarks log](../../research-paper/benchmarks-log.md);
- writes a finding (`F-###`) for the campaign;
- stores the RAW shared offset of each device as its calibration (`LatencyCalibration`, 06 §9);
- proposes the ADR-0016 confirmation, or a PB-01 revision through change control if no mode meets 50 ms (ADR-0016).

## Notes

- The program's "offset" is the median of the measured onset minus the time the engine predicted for it: the value that playhead anchors must add for that device.
- HW-01's Realtek driver in exclusive mode reported a 144-frame buffer, but signalled two or three events per hardware period and queued about 100 ms by its own clock (F-051). The cable run shows whether exclusive mode lowers latency on that driver at all.
