---
status: accepted
date: 2026-10-19
decision-makers: ["@DuhItzAniket"]
consulted: ["coordinator agent (Claude Code), under the owner's delegation of ADR approvals"]
---

# 0016 · A Windows low-latency audio backend and per-device latency calibration

## Context and problem statement

[ADR-0004](0004-audio-backend-and-render-graph.md) puts cpal 0.18 behind our own `AudioBackend` trait as the first backend. The A02 latency spike measured that backend against PB-01 (key press to cue onset ≤ 50 ms p95 at 256 frames, 48 kHz). The result, on the owner's laptop in WASAPI shared mode ([A02 gate report](../sdlc/reviews/A02-prototype-gate.md#spike-1-output-latency-a02-t01)):

* **p50 116.4 ms, p95 123.1 ms** at every buffer setting, because the driver offers only a 480-frame engine period (F-030, F-031).
* The OEM enhancement APO adds about 38 ms and +20.6 dB of gain after SONARITHM's limiter. Opening the stream in RAW mode removes both and gives p95 83.7 ms (F-032).
* cpal's predicted playback time is about 80 ms earlier than the observed onset, so a playhead extrapolated from it drifts (F-033).

cpal 0.18 cannot request RAW mode, exclusive mode or `IAudioClient3` periods (cpal #106, #1173, #1343).

How does SONARITHM meet PB-01 on Windows, and keep the playhead in step with the sound?

## Decision drivers

* PB-01 and interactive exploration: cues must follow key presses quickly (A11Y rules on responsiveness; 08 §3).
* **Coexistence with screen readers.** Exclusive mode takes the device away from every other application, including a screen reader's speech on the same output.
* Hearing safety and loudness calibration: processing added after our limiter defeats 06 §10 and SDR-24.
* Keep cpal as the default cross-platform backend (ADR-0004), with platform specifics behind `AudioBackend`.
* Licences (ADR-0010): the `wasapi` crate is MIT; the Steinberg ASIO SDK has its own terms (`TODO(verify)`).
* Evidence before budgets: the A02 numbers come from loopback capture, a software method. Sign-off needs a cable measurement (testing-strategy §14).

## Considered options

* A second Windows backend on the `wasapi` crate: RAW shared stream by default, exclusive mode as an opt-in "low-latency mode", plus per-device latency calibration for every backend
* Contribute RAW and exclusive support to cpal upstream and wait for it
* ASIO through cpal's `asio` feature only
* Revise PB-01 to the shared-mode floor
* Latency calibration only

## Decision outcome

Chosen option: **a `WasapiBackend` on the `wasapi` crate, behind the existing `AudioBackend` trait**, built in A05.

* **Shared mode with RAW by default** on Windows, falling back to the default processing mode where the driver lacks RAW support.
* **Exclusive mode as an opt-in low-latency setting**, with a clear warning that it can silence the screen reader's speech on the same device.
* **Per-device latency calibration** for every backend: a loopback self-test where the platform offers one, otherwise a stored offset. It is used for the playhead anchors of 06 §9, instead of trusting the backend's reported latency.

Also:

* cpal stays the default backend on macOS and Linux, and on Windows when `WasapiBackend` fails to open.
* Upstream RAW and exclusive support in cpal is welcome, and `WasapiBackend` can be retired if cpal gains it.
* ASIO stays an opt-in build feature, pending a licence check.
* PB-01 is not revised until cable measurements exist.

### Consequences

* Good: RAW removes the OEM APO's latency and its gain after our limiter, and so restores the loudness and hearing-safety assumptions on such systems (F-032).
* Good: exclusive mode can reach the driver's minimum period; the playhead stops drifting once calibrated (F-033).
* Bad: a second backend to maintain, Windows-only code with `unsafe` COM calls (unsafe register entry needed), and more test surface.
* Bad: RAW alone measured 83.7 ms p95, so PB-01 may still need exclusive mode or better hardware. Exclusive mode conflicts with screen-reader speech on a shared device.
* Neutral: macOS and Linux latency are unmeasured (risk R-22); this ADR changes nothing there.

**Acceptance (2026-10-19).** Accepted by the coordinator agent under the owner's standing delegation of ADR and spec approvals ("Approve them yourself", 2026-10-19). The owner reviews it with the morning report. Implementation and confirmation belong to A05 (below).

### Confirmation

A05, with a cable loopback on a USB audio interface and on the owner's laptop:

1. key press to cue onset p95 in the low-latency mode ≤ 50 ms at 256 frames;
2. the RAW shared-mode p95 reported next to it;
3. the calibrated playhead offset reported against the measured onset;
4. an NVDA check that speech and sonification coexist in the default mode.

If no mode meets PB-01 on reference-class hardware, PB-01 is revised through change control, with the measurements attached.

## Pros and cons of the options

### Upstream cpal work only
* Good: one backend for every OS.
* Bad: no timeline (the cpal 0.19 tracking issue lists exclusive mode as open); it blocks PB-01 on outside work.

### ASIO only
* Good: the lowest latency where an ASIO driver exists.
* Bad: most laptops have no native ASIO driver (the owner's Realtek has none); SDK licence terms unchecked; an opt-in build.

### Revise PB-01
* Good: honest about shared mode.
* Bad: no evidence yet on what latency still supports exploration; the measurement is software-only.

### Calibration only
* Good: cheap, and fixes the playhead drift.
* Bad: does not shorten key-to-sound time at all.

## More information

[A02 gate report: Options for PB-01](../sdlc/reviews/A02-prototype-gate.md#options-for-pb-01); findings F-030 to F-033; risk R-03; the A02 latency spike, `spikes/cpal-latency/` ([#9](https://github.com/DuhItzAniket/SONARITHM/pull/9)), which already contains a direct WASAPI RAW renderer.
