# Rust Core Stack for SONARITHM: Technology Evaluation (as of 2026-10-18)

*Method: Versions, release dates, licenses, MSRV (`rust-version`), total and 90-day ("recent") download counts were pulled from the crates.io API (`https://crates.io/api/v1/crates/<name>`) on 2026-10-18. Repo activity (last push, stars, archived flag) came from the GitHub API the same day. Each table row links the crates.io page as its source. Where a crate's newest upload is a backport, both are shown. "Last push" means the last commit to any branch of the linked repo, not a release. The labels Adopt, Trial, Assess and Avoid are the researcher's recommendations (inferences), not facts taken from a source.*

---

## 1. Audio I/O: cpal status, backends, latency; higher layers (rodio, kira, oddio, firewheel, interflow, others). Which suits low-latency interactive playback plus offline rendering?

### Takeaway
cpal 0.18.2 (2026-08-16) is still the de facto low-level I/O layer and is very actively maintained. Since 0.18.0 it has native PipeWire and PulseAudio hosts, latency-aware timestamps, and xrun reporting. It still has no WASAPI or CoreAudio exclusive mode, and v0.19 will bring more breaking changes. No I/O crate renders offline: SONARITHM should own a backend-agnostic render graph, which Firewheel or a fundsp-based graph can supply, and drive it from cpal in real time or from a plain loop for offline export.

### Cited Findings

**Version and adoption table (crates.io and GitHub, 2026-10-18)**

| Crate | Purpose | Latest stable (date) | License | Repo last push | Total / 90-day downloads, stars | Rec. | Source |
|---|---|---|---|---|---|---|---|
| cpal | Low-level cross-platform audio I/O | 0.18.2 (2026-08-16); MSRV 1.85 | Apache-2.0 | 2026-10-13 | 24.8M / 8.54M; 3,975★ | **Adopt** | [crates.io](https://crates.io/crates/cpal), [GitHub](https://github.com/RustAudio/cpal) |
| rodio | Playback library on cpal (pull-based `Source` iterators) | 0.23.0 (2026-10-11); MSRV 1.87 | MIT OR Apache-2.0 | 2026-10-11 | 12.9M / 3.58M; 2,491★; 170 open issues | Assess (fine for CLI preview, not the engine core) | [crates.io](https://crates.io/crates/rodio), [GitHub](https://github.com/RustAudio/rodio) |
| kira | Game audio: tweens, clocks, mixer, spatial | 0.12.5 (2026-09-26) | MIT OR Apache-2.0 | 2026-09-26 | 1.03M / 205k; 1,072★ | Assess | [crates.io](https://crates.io/crates/kira), [GitHub](https://github.com/tesselode/kira) |
| firewheel | Mid-level audio graph engine (BillyDM) | 0.14.0 (2026-09-05) | MIT OR Apache-2.0 | 2026-09-05 | 58.6k / 19.9k; 316★ | **Trial** | [crates.io](https://crates.io/crates/firewheel), [GitHub](https://github.com/BillyDM/Firewheel) |
| oddio | Lightweight game audio/spatial mixing | 0.7.4 (2023-10-15) | MIT OR Apache-2.0 | 2023-10-15 (stale about 3 years) | 60k / 7.2k; 165★ | **Avoid** | [crates.io](https://crates.io/crates/oddio), [GitHub](https://github.com/Ralith/oddio) |
| interflow | "Unified and opinionated audio I/O for Rust" | **Not published** (crates.io returns 404); no GitHub releases | n/a | 2026-08-08 | 64★ | Assess (watch only) | [GitHub](https://github.com/SolarLiner/interflow) |
| cubeb | Rust bindings to Mozilla's cubeb | 0.38.0 (2026-08-18) | ISC | 2026-08-18 | 529k / 217k; 79★ | Assess | [crates.io](https://crates.io/crates/cubeb) |
| web-audio-api | Rust implementation of the W3C Web Audio API (cpal or cubeb backends) | 1.7.0 (2026-08-08); MSRV 1.86 | MIT | 2026-10-12 | 176k / 31k; 388★ | Assess | [crates.io](https://crates.io/crates/web-audio-api), [GitHub](https://github.com/orottier/web-audio-api-rs) |
| tinyaudio | "cross-platform, easy-to-use, low-level, audio output library" (crate description) | 2.0.0 (2025-11-15) | MIT | 2026-09-26 | 130k / 39k; 225★ | Not evaluated in depth | [crates.io](https://crates.io/crates/tinyaudio) |
| wasapi | WASAPI bindings by HEnquist, used by CamillaDSP | 0.26.0 (2026-10-11); MSRV 1.85 | MIT | 2026-10-13 | 423k / 218k; 94★ | Assess (Windows exclusive-mode escape hatch) | [crates.io](https://crates.io/crates/wasapi) |
| asio-sys | ASIO bindings, part of the cpal repo | 0.4.0 (2026-08-16) | Apache-2.0 | 2026-10-13 | 696k / 81k | Via cpal `asio` feature | [crates.io](https://crates.io/crates/asio-sys) |
| jack | JACK bindings | 0.13.5 (2026-02-01) | MIT | 2026-10-05 | 1.17M / 120k; 247★ | Via cpal | [crates.io](https://crates.io/crates/jack) |
| pipewire | PipeWire bindings | 0.10.1 (2026-08-19); MSRV 1.80 | MIT | (GitLab, not checked) | 2.78M / 1.64M | Via cpal | [crates.io](https://crates.io/crates/pipewire) |
| libpulse-binding | PulseAudio bindings | 2.30.1 (2025-04-19) | MIT OR Apache-2.0 | 2025-04-19 | 6.6M / 1.19M | Via cpal | [crates.io](https://crates.io/crates/libpulse-binding) |
| alsa / coreaudio-rs | ALSA / CoreAudio bindings | 0.12.1 (2026-07-31) / 0.14.2 (2026-04-29) | Apache-2.0/MIT; MIT/Apache-2.0 | 2026-10-14 / 2026-06-18 | 25.7M / 18.7M total | Via cpal | [alsa](https://crates.io/crates/alsa), [coreaudio-rs](https://crates.io/crates/coreaudio-rs) |
| awedio | "A low-overhead and adaptable audio playback library" (crate description) | 0.8.0 (2026-06-12) | MIT OR Apache-2.0 | 2026-10-18 | 17k / 1.8k; 98★ | Not evaluated in depth | [crates.io](https://crates.io/crates/awedio) |

**cpal status and backends**
- cpal 0.18.0 was released on 2026-06-06. It added a new **PipeWire host** ("via the native PipeWire API") and a new **PulseAudio host** for Linux and BSD, and changed the default Linux/BSD host priority to PipeWire > PulseAudio > ALSA. — [cpal CHANGELOG](https://github.com/RustAudio/cpal/blob/HEAD/CHANGELOG.md)
- 0.18.0 also brought: a unified `Error`/`ErrorKind` type (including `RealtimeDenied`, `DeviceChanged`, `DeviceBusy`); streams no longer auto-start; default configs now prefer 48 kHz; a `realtime` feature for RT thread scheduling without a D-Bus build dependency (the old `audio_thread_priority` feature was renamed `realtime-dbus`); MSRV raised to 1.85; tvOS support; AAudio requests `PERFORMANCE_MODE_LOW_LATENCY` under `realtime`. — [cpal CHANGELOG](https://github.com/RustAudio/cpal/blob/HEAD/CHANGELOG.md)
- Latency accounting: in 0.18.0, callback timestamps began to include hardware/pipeline latency on WASAPI, CoreAudio (device latency plus safety offset), ASIO (driver-reported latency, re-queried on `kAsioLatenciesChanged`), JACK (hardware deadline), and WebAudio (base plus output latency). ALSA uses `LinkSynchronized` hardware cross-timestamps "for lower jitter". 0.18.2 added iOS hardware latency and JACK port latency. — [cpal CHANGELOG](https://github.com/RustAudio/cpal/blob/HEAD/CHANGELOG.md)
- 0.18.2 (2026-08-16) reports xruns as `ErrorKind::Xrun` on AAudio, CoreAudio, PipeWire and WASAPI capture. It also fixed "stale audio output when a data callback wrote a partial buffer" on CoreAudio, WebAudio and AudioWorklet, and fixed unsound `Send + Sync` on the WebAudio `Stream` under `+atomics`. — [cpal CHANGELOG](https://github.com/RustAudio/cpal/blob/HEAD/CHANGELOG.md)
- Unreleased work on the main branch (expected in 0.19) includes: migration to Rust 2024; `StreamTrait::play` renamed to `start`; a draining `stop()`; `CallbackInfo::xrun()`; a **duplex stream API** (ALSA, JACK, AudioWorklet, WebAudio); input streams on WebAudio and AudioWorklet; Emscripten support; `DeviceTrait`/`StreamTrait: Send + Sync`; removal of `ErrorKind::Xrun` (breaking); and a minimum Windows version of 10 (since 0.17.2). — [cpal CHANGELOG](https://github.com/RustAudio/cpal/blob/HEAD/CHANGELOG.md)
- The v0.19 tracking issue (opened 2026-05-27) still lists these as open: exclusive mode on CoreAudio, PipeWire and WASAPI (#106); WASAPI RAW mode (#1173); a `BufferSize` refactor with range support; and a `SupportedConfig` refactor. Its stated target is "when it's done; feature set may change at any time." — [cpal #1220](https://github.com/RustAudio/cpal/issues/1220), [cpal #106](https://github.com/RustAudio/cpal/issues/106)
- Open latency- and robustness-related issues (all updated in October 2026): "output stream dies with small buffer size" (#487); default ALSA buffer-size strategy undecided (#446); PipeWire+JACK frequent xruns at some block sizes (#709); macOS CoreAudio timestamps double-counting the buffer (#1398); an ASIO TimeBase bug that makes instants "overflow after ~10 minutes" (#1393); and a crashing build when ASIO fails to load although it should be optional (#1083). — [#487](https://github.com/RustAudio/cpal/issues/487), [#446](https://github.com/RustAudio/cpal/issues/446), [#709](https://github.com/RustAudio/cpal/issues/709), [#1398](https://github.com/RustAudio/cpal/issues/1398), [#1393](https://github.com/RustAudio/cpal/issues/1393), [#1083](https://github.com/RustAudio/cpal/issues/1083)
- In the WASAPI `IAudioClient3` discussion (closed 2026-09-07), a commenter points to the separate `wasapi` crate (used by CamillaDSP) as one that "should have good exclusive mode support". — [cpal #1343](https://github.com/RustAudio/cpal/issues/1343)

**Higher layers**
- rodio 0.22 (2026-02-20) and 0.22.2 (2026-03-05) fixed "Incorrectly set system default audio buffer size breaks playback", a problem introduced by the cpal upgrade, and added Opus via `symphonia-adapter-libopus`. The main-branch changelog's unreleased section, presumably what shipped as 0.23.0 (the v0.23.0 tag exists), lists cpal 0.18, `Source::resample` with `ResampleConfig`, a default of 48 kHz, and several breaking changes. — [rodio CHANGELOG](https://github.com/RustAudio/rodio/blob/HEAD/CHANGELOG.md), [crates.io](https://crates.io/crates/rodio)
- A Bevy-side audit says rodio has "many issues which make it unsuitable for game use" (undated; author not shown). — [HackMD Bevy audit](https://hackmd.io/@bevy/rkwFscXObx)
- kira 0.12.2 (2026-07-16) moved to symphonia 0.6 and cpal 0.18.1, re-exports cpal, and adds `cpal-realtime`/`cpal-realtime-dbus` features. 0.12.5 (2026-09-26) moved to rtrb 0.4. 0.12.0 (2026-02-18) changed format feature flags to match Symphonia's one-to-one. — [kira changelog](https://github.com/tesselode/kira/blob/HEAD/changelog.md)
- Firewheel describes itself as "Powerful and flexible mid-level audio engine for games and other applications". Its crates are firewheel-core, -cpal, -rtaudio, -graph, -macros, -nodes and -symphonium. Its README claims "no mutexes", `no_std` compatibility for parts, silence optimization, and fault tolerance for unplugged devices. — [Firewheel GitHub](https://github.com/BillyDM/Firewheel), [crates.io dependencies of 0.14.0](https://crates.io/crates/firewheel)
- Firewheel's design doc gives these details. The audio graph is a DAG that is "compiled" into a schedule and sent to the processor over a "realtime-safe message channel". `update()` flushes events in groups so events meant for the same cycle land in the same cycle. There are three clocks (seconds, sample, musical), and events can be scheduled with `EventDelay::DelayUntilSeconds`. A planned C API includes a "no backend" option "where the user plugs in a firewheel processor object into their own custom audio stream". Built-in nodes include sampler (with sequencing and doppler), simple spatial positioning, convolution, freeverb, SVF filters, noise, peak meter, stream reader/writer and delay compensation. Still open: HRTF spatialization, EQ, compressor, CLAP hosting and C bindings. Non-goals: a DAW engine, multi-threaded graph processing, built-in synth instruments, and VST/LV2/AU hosting. — [Firewheel DESIGN_DOC](https://github.com/BillyDM/Firewheel/blob/HEAD/DESIGN_DOC.md)
- Firewheel "is currently planned to be upstreamed into the Bevy game engine"; the current integration lives in bevy_seedling (0.8.0, 2026-08-09). — [Firewheel GitHub](https://github.com/BillyDM/Firewheel), [bevy_seedling crates.io](https://crates.io/crates/bevy_seedling)
- As of Bevy 0.20, `bevy_audio` 0.20.0 (2026-10-08) still depends on `rodio ^0.22`, so Firewheel has not yet been upstreamed. — [bevy_audio 0.20.0 dependencies](https://crates.io/crates/bevy_audio/0.20.0/dependencies)
- The Bevy audit raises several concerns: "firewheel has nontrivial UB that must be resolved"; firewheel "only has a cpal backend" (a firewheel-rtaudio crate now exists, so the audit may predate it); and depending on bevy creates a publishing "nightmare". It praises the care taken to avoid allocation on the audio thread, the silence optimization, the optional denormal flush-to-zero, and "mostly strong documentation and test coverage". — [HackMD Bevy audit](https://hackmd.io/@bevy/rkwFscXObx)
- A Codeberg copy of Firewheel (codeberg.org/BillyDM/Firewheel) shows its last commit on 2025-09-21 and no tags, while the GitHub repo was pushed 2026-09-05 and has 0.14.0 on crates.io. — [Codeberg](https://codeberg.org/BillyDM/Firewheel), [GitHub](https://github.com/BillyDM/Firewheel)
- Firewheel lists an "[Interflow] backend" and an "[RtAudio] backend" as "Maybe Goals". — [Firewheel DESIGN_DOC](https://github.com/BillyDM/Firewheel/blob/HEAD/DESIGN_DOC.md)

### Inferences
- **For SONARITHM's needs (interactive scrubbing, live streams, and deterministic offline export), adopt cpal behind a thin internal `AudioBackend` trait.** The core render function (`fn process(&mut self, out: &mut [f32], frames, timeline_pos)`) must not depend on cpal. Offline export then calls the same function in a loop at a fixed block size and sample rate, with no device clock. That gives bit-reproducible output on a given platform and build, and it is the same "no backend" pattern Firewheel's design doc describes.
- **Firewheel is the closest fit as a reusable graph and scheduler** because of its sample/seconds/musical clocks, event scheduling, sampler, and RT-safe message channel. It is still pre-1.0 (0.14.0; search results still indexed 0.12.1 on docs.rs, so minor versions move quickly). It also faces audit-reported UB and a possible reshaping during Bevy upstreaming. Trial it behind an internal abstraction, or use it as a reference design while building a slimmer SONARITHM graph on fundsp nodes.
- rodio's pull-iterator `Source` model and kira's game-oriented API are convenient for previews but give less control over sample-accurate, timeline-driven event scheduling and deterministic offline rendering. Neither is recommended as the engine core.
- **Windows low latency:** cpal uses WASAPI shared mode only (exclusive mode is still open in #106). For pro-audio-grade latency, offer the cpal `asio` feature as an opt-in build. Consider a direct `wasapi`-crate exclusive-mode path later.
- **Web:** cpal's WebAudio and AudioWorklet hosts are now first-class (input and duplex unreleased), so a wasm build can reuse the same render core.

### Gaps
- No published, reproducible round-trip latency benchmarks for cpal per backend were found (issue #555 requesting a round-trip latency example is still open: [cpal #555](https://github.com/RustAudio/cpal/issues/555)). Measure in-house.
- Whether cpal #1343 (`IAudioClient3` low-latency shared mode) was closed as implemented or as a documentation change could not be confirmed from the comments.
- Firewheel's offline/manual-processing API surface was not verified in the docs (the design doc mentions the "no backend" processor concept only in the C-bindings plan).
- web-audio-api's `OfflineAudioContext` support was not verified in this pass.

---

## 2. DSP and synthesis: fundsp, dasp, glicol, knyst, firewheel nodes, synthizer, biquad, rubato, rustfft/realfft, synthesis crates; HRTF/binaural; loudness (ebur128)

### Takeaway
fundsp (0.23.0) is the strongest maintained synthesis toolkit: inline graph notation, `Net`/`Sequencer` with preallocation, FIR resampling, convolution, and `no_std`. rubato, rustfft and realfft are the standard building blocks. Glicol, knyst, oddio and synthizer-rs are stale or archived. For HRTF, `sofar` (pure Rust SOFA reader plus renderer, 0.4.0, 2026-10-09) is the best permissive option. Steam Audio via `audionimbus` is powerful but brings a native binary dependency. `ebur128` is stable but changes little.

### Cited Findings

| Crate | Purpose | Latest stable (date) | License | Repo last push | Downloads total / 90-day; stars | Rec. | Source |
|---|---|---|---|---|---|---|---|
| fundsp | Synthesis/DSP library with graph notation | 0.23.0 (2026-01-07) | MIT OR Apache-2.0 | 2026-03-03 | 233k / 53.5k; 1,226★ | **Adopt** | [crates.io](https://crates.io/crates/fundsp), [GitHub](https://github.com/SamiPerttu/fundsp) |
| dasp | Sample/frame/signal primitives, conversion, interpolation | 0.11.0 (**2020-05-29**) | MIT OR Apache-2.0 | 2026-01-12 | 5.35M / 1.09M; 1,195★ | Assess (no release in about 6.4 years) | [crates.io](https://crates.io/crates/dasp), [GitHub](https://github.com/RustAudio/dasp) |
| glicol / glicol_synth | Live-coding language and graph synth engine (wasm) | 0.13.5 (2024-04-23) | MIT (GitHub; crates.io shows "non-standard") | **2025-04-06** (about 18 months idle) | 13.9k / 0.6k; 3,012★ | **Avoid** (reference only) | [crates.io](https://crates.io/crates/glicol), [GitHub](https://github.com/chaosprint/glicol) |
| knyst | Real-time audio graph and synthesis | 0.5.1 (2024-11-25) | MIT OR Apache-2.0 | **2024-11-25** (about 23 months idle) | 13.9k / 129; 44★ | **Avoid** | [crates.io](https://crates.io/crates/knyst), [GitHub](https://github.com/ErikNatanael/knyst) |
| synthizer | Rust bindings to Synthizer (3D audio) | 0.5.6 (2023-03-04) | Unlicense | **Repo archived** | 28k / 192 | **Avoid** | [crates.io](https://crates.io/crates/synthizer), [GitHub](https://github.com/synthizer/synthizer-rs) |
| biquad | Biquad IIR filters (no_std) | 0.6.0 (2026-03-22) | MIT OR Apache-2.0 | 2026-03-22 | 432k / 142k; 78★ | Adopt (or use fundsp filters) | [crates.io](https://crates.io/crates/biquad) |
| rubato | Async/sync sample-rate conversion | 5.0.1 (2026-10-01); MSRV 1.87 | MIT OR Apache-2.0 | 2026-10-01 | 13.4M / 5.38M; 360★ | **Adopt** | [crates.io](https://crates.io/crates/rubato), [GitHub](https://github.com/HEnquist/rubato) |
| rustfft | FFT | 6.4.1 (2025-09-18); MSRV 1.61 | MIT OR Apache-2.0 | 2026-10-17 | 32.5M / 9.9M; 949★ | **Adopt** | [crates.io](https://crates.io/crates/rustfft) |
| realfft | Real-to-complex FFT wrapper over rustfft | 3.5.0 (2025-06-12) | MIT | 2026-03-12 | 18.7M / 5.07M | **Adopt** | [crates.io](https://crates.io/crates/realfft) |
| twang | Synthesis library | 0.9.0 (2022-10-23) | Apache-2.0 OR BSL-1.0 OR MIT | 2025-03-03 | 22k / 0.7k | Avoid (stale) | [crates.io](https://crates.io/crates/twang) |
| rustysynth | SoundFont (SF2) synthesizer | 1.3.7 (2026-10-01) | MIT | 2026-10-01 | 132k / 53k; 234★ | Trial (instrument timbres) | [crates.io](https://crates.io/crates/rustysynth) |
| oxisynth | FluidSynth port | 0.1.0 (2025-05-25) | **LGPL-2.1** | 2025-11-16 | 32k / 10k | Assess (copyleft) | [crates.io](https://crates.io/crates/oxisynth) |
| creek | "Realtime-safe disk streaming to/from audio files" (Meadowlark) | 1.2.3 (2025-09-22) | MIT OR Apache-2.0 | n/a (not GitHub) | 131k / 34k | Assess (long-file playback) | [crates.io](https://crates.io/crates/creek) |
| sofar | Pure Rust SOFA reader plus HRTF renderer | 0.4.0 (2026-10-09) | MIT OR Apache-2.0 | 2026-10-11 | 32k / 13.8k; 19★ | **Trial** | [crates.io](https://crates.io/crates/sofar), [GitHub](https://github.com/andreiltd/sofar) |
| hrtf | HRTF processor using HRIR spheres | 0.9.0 (2026-08-06) | MIT | 2026-09-06 | 687k / 86k; 100★ | Assess | [crates.io](https://crates.io/crates/hrtf), [GitHub](https://github.com/mrDIMAS/hrtf) |
| audionimbus | Safe wrapper for Valve Steam Audio | 0.16.0 (2026-08-08) | MIT OR Apache-2.0 (wrapper) | 2026-08-20 | 16k / 1.9k; 210★ | Assess | [crates.io](https://crates.io/crates/audionimbus), [GitHub](https://github.com/MaxenceMaire/audionimbus) |
| ebur128 | EBU R128 / ITU BS.1770 loudness and true peak | 0.1.10 (2024-10-26); MSRV 1.60 | MIT | 2025-12-13 | 1.05M / 272k; 129★ | **Adopt** | [crates.io](https://crates.io/crates/ebur128), [GitHub](https://github.com/sdroege/ebur128) |

- fundsp: "All `AudioNode` and `AudioUnit` components use 32-bit floating point samples (`f32`)". The `allocate` method "preallocates all needed memory. It should be called last before sending something into a real-time context. This is done automatically in the `Net` and `Sequencer` frontends." It supports `no_std` with `alloc` (file I/O unavailable in `no_std`) and SIMD via `wide`'s `f32x8`. — [fundsp README](https://github.com/SamiPerttu/fundsp)
- fundsp changes: 0.22 added `resample_fir` (FIR sinc via the `resampler` crate) and a 64-bit `prelude64`; 0.23 added Sequencer inputs, FFT `convolve`, and resampling quality settings; the unreleased 0.24 adds Sequencer looping, an `AudioNode` introspection module, and a requirement that Sequencer changes "must now be committed to the backend" (breaking). 0.21 added `noise().seed(1)` builder notation and PolyBLEP oscillators. — [fundsp CHANGES.md](https://github.com/SamiPerttu/fundsp/blob/HEAD/CHANGES.md)
- sofar describes itself as a "pure Rust implementation for reading HRTF filters from SOFA files" with a `render` module implementing a "uniformly partitioned convolution algorithm", "Based on the libmysofa C library". — [sofar README](https://github.com/andreiltd/sofar)
- The hrtf crate uses IRCAM-derived "HRIR spheres" built with `hrir_sphere_builder` (its own format, not SOFA), recorded at 44,100 Hz with "automatic" resampling. Its README warns: "HRTF is **heavy**". — [hrtf README](https://github.com/mrDIMAS/hrtf)
- audionimbus "currently tracks Steam Audio 4.8.1". An `auto-install` feature "automatically downloads and installs Steam Audio". It uses independent versioning and "is subject to breaking changes". The docs list Clang 9.0 or later as a requirement and FMOD/Wwise integration. — [audionimbus README](https://github.com/MaxenceMaire/audionimbus), [docs.rs](https://docs.rs/crate/audionimbus)
- Firewheel's built-in nodes cover only "simple spatial positioning"; "better spatial positioning with HRTF" is unchecked. — [Firewheel DESIGN_DOC](https://github.com/BillyDM/Firewheel/blob/HEAD/DESIGN_DOC.md)

### Inferences
- **Synthesis core:** use fundsp for oscillators (PolyBLEP, FM via graph composition), filters, envelopes, noise with explicit seeds (for deterministic export) and FIR resampling. Wrap fundsp `AudioUnit`s as nodes in SONARITHM's own timeline graph (or as Firewheel custom nodes). fundsp's f32-only processing is adequate for audio. Keep timeline and clock math in f64/i64 sample counts to avoid drift on long renders.
- **Granular, wavetable and modal synthesis:** no dedicated, maintained, widely adopted Rust crates surfaced in this pass. Plan to implement these as custom fundsp or engine nodes; they are small DSP kernels.
- **Spatial/binaural:** start with stereo panning plus ITD/ILD. Use `sofar` for SOFA-based HRTF, since it is permissive, pure Rust and wasm-friendly in principle. Keep Steam Audio (`audionimbus`) as an optional, desktop-only feature because of its native binary download and Clang build requirement.
- Treat `dasp` as a legacy dependency: fine for sample-format conversion, but don't build on it given no release since 2020.
- `ebur128` is mature and low-churn. Use it for export normalization and for loudness metrics in tests.

### Gaps
- No maintained Rust-native granular, wavetable or modal-synthesis crate with meaningful adoption was found. The ecosystem was not exhaustively searched (lib.rs categories could be swept).
- Steam Audio SDK licensing and terms for redistribution were not re-verified in this pass.
- Whether `sofar` builds for `wasm32` was not verified.

---

## 3. Real-time safety: rtrb, ringbuf, crossbeam, triple_buffer, basedrop, assert_no_alloc, no_denormals, audio_thread_priority; lock-free UI↔audio patterns

### Takeaway
The standard Rust pattern is the same one Firewheel and kira use: wait-free SPSC ring buffers (rtrb) carry commands and events to the audio thread, a triple buffer or ring carries meters and state back, deferred-drop smart pointers (basedrop) keep deallocation off the audio thread, and RT thread promotion comes from cpal's `realtime` feature, which is backed by `audio_thread_priority`. All of the key crates are maintained, but `assert_no_alloc` and `basedrop` are tiny, quiet crates.

### Cited Findings

| Crate | Purpose | Latest stable (date) | License | Repo last push | Downloads total / 90-day | Rec. | Source |
|---|---|---|---|---|---|---|---|
| rtrb | Wait-free SPSC ring buffer | 0.4.0 (2026-08-17); 0.3.5 backport (2026-08-18) | MIT OR Apache-2.0 | 2026-09-13 | 13.6M / 4.62M; 349★ | **Adopt** | [crates.io](https://crates.io/crates/rtrb), [GitHub](https://github.com/mgeier/rtrb) |
| ringbuf | Lock-free SPSC ring buffer (also static/no_std) | 0.5.3 (2026-10-08) | MIT OR Apache-2.0 | 2026-10-09 | 21.2M / 6.46M; 619★ | Adopt (alternative) | [crates.io](https://crates.io/crates/ringbuf) |
| crossbeam / crossbeam-channel | MPMC channels, epoch GC | 0.8.5 / 0.5.17 (2026-09-05) | MIT OR Apache-2.0 | 2026-10-01 | 150M / 647M total | Adopt (non-RT threads only) | [crossbeam](https://crates.io/crates/crossbeam), [crossbeam-channel](https://crates.io/crates/crossbeam-channel) |
| triple_buffer | Wait-free triple buffer for latest-value state | 9.0.0 (2026-02-22); MSRV 1.86 | **MPL-2.0** | 2026-07-29 | 1.16M / 465k | **Adopt** | [crates.io](https://crates.io/crates/triple_buffer) |
| basedrop | `Owned`/`Shared` pointers with deferred collection | 0.1.3 (2025-10-29) | MIT/Apache-2.0 | 2025-10-29 | 144k / 13k; 132★ | Trial | [crates.io](https://crates.io/crates/basedrop), [GitHub](https://github.com/micahrj/basedrop) |
| assert_no_alloc | Panics or logs on allocation inside a scope | 1.1.2 (**2021-08-03**) | BSD-1-Clause | 2023-08-09 | 4.76M / 1.22M | Adopt (debug/test builds) | [crates.io](https://crates.io/crates/assert_no_alloc) |
| no_denormals | Flush-to-zero/denormals-are-zero guard | 0.3.0 (2025-09-11) | MIT | 2025-09-11 | 120k / 37k | Adopt | [crates.io](https://crates.io/crates/no_denormals) |
| audio_thread_priority | Mozilla's RT audio thread promotion | 0.38.0 (2026-09-17) | **MPL-2.0** | 2026-09-17 | 766k / 318k | Adopt (via cpal `realtime`) | [crates.io](https://crates.io/crates/audio_thread_priority) |
| arc-swap | Atomic `Arc` swap (RCU-style config) | 1.9.3 (2026-10-18) | MIT OR Apache-2.0 | 2026-10-18 | 365M total | Adopt (non-RT readers; careful on RT) | [crates.io](https://crates.io/crates/arc-swap) |
| atomic_float | `AtomicF32`/`AtomicF64` | 1.1.0 (2024-08-31) | Apache-2.0 OR MIT OR Unlicense | 2024-08-31 | 19M total | Adopt (parameter atomics) | [crates.io](https://crates.io/crates/atomic_float) |
| thread-priority | Generic thread priority control | 3.1.1 (2026-06-20) | MIT | 2026-09-07 | 12.6M total | Assess | [crates.io](https://crates.io/crates/thread-priority) |

- basedrop provides "smart pointers analogous to Box and Arc which mark their contents for deferred collection on another thread", so they can be dropped safely on a real-time thread. `rtrb_basedrop` is a fork of rtrb that uses basedrop `Shared` so that dropping the last reference on the RT thread doesn't free memory there. — [rtrb-basedrop docs](https://docs.rs/rtrb-basedrop), [basedrop](https://github.com/micahrj/basedrop)
- rtrb reads and writes are "lock-free and wait-free", with a fixed-capacity buffer allocated on construction and no later allocation "unless the element type allocates internally". — [rtrb-basedrop docs (describing rtrb)](https://docs.rs/rtrb-basedrop)
- Firewheel's design sends a compiled schedule to the executor over a "realtime-safe message channel" and flushes events in groups once per `update()`. Its nodes directory includes a `triple_buffer.rs` node. — [Firewheel DESIGN_DOC](https://github.com/BillyDM/Firewheel/blob/HEAD/DESIGN_DOC.md), [firewheel-nodes source](https://github.com/BillyDM/Firewheel/tree/HEAD/crates/firewheel-nodes/src)
- kira 0.12.5 depends on rtrb (updated to v0.4.0). — [kira changelog](https://github.com/tesselode/kira/blob/HEAD/changelog.md)
- cpal 0.18 provides a `realtime` feature for RT scheduling without D-Bus; `realtime-dbus` (formerly `audio_thread_priority`) uses rtkit over D-Bus on Linux. 0.18.2 fixed "rtkit burst limit errors from repeated real-time promotion" and RT promotion blocking the callback thread on PipeWire. On JACK, `RealtimeDenied` is reported if the callback isn't at RT priority. — [cpal CHANGELOG](https://github.com/RustAudio/cpal/blob/HEAD/CHANGELOG.md)
- The Bevy audit credits Firewheel with "care to avoid allocation on the audio thread" and "optional denormal flush-to-zero". — [HackMD Bevy audit](https://hackmd.io/@bevy/rkwFscXObx)

### Inferences
- **Recommended pattern for SONARITHM:**
  1. **UI/control → audio:** an `rtrb::Producer<Command>` with `Copy`, fixed-size commands (seek, play/pause, parameter set with a sample-time stamp, swap-timeline). Heavy objects such as a newly compiled timeline or loaded sample buffers travel as `basedrop::Shared<T>` handles (or ID plus pre-registered slot), so the old object is collected on a GC thread.
  2. **Audio → UI:** a `triple_buffer` holding the latest playhead, meters and spectrum (UI needs only the latest value), plus an rtrb ring for discrete events (xruns, marker crossings, earcon triggers).
  3. **Parameters:** `atomic_float` for continuously tweakable scalars, smoothed per block on the audio thread.
  4. **Live data streams:** ingest on tokio tasks, then compile to events, then push through an SPSC ring to the audio thread with look-ahead (the audio thread never parses data).
  5. In debug builds and CI, wrap the audio callback in `assert_no_alloc`. Set FTZ/DAZ with `no_denormals` at callback entry. Never call `tracing` macros on the audio thread; push compact log records into a ring instead.
- crossbeam channels are fine between non-RT threads (ingest, compile, export workers) but should not be used inside the audio callback (they may block or allocate on some paths; a general caution, not verified in crossbeam docs this pass).
- MPL-2.0 (triple_buffer, audio_thread_priority) is file-level copyleft and generally compatible with MIT/Apache distribution. Track it with cargo-deny.

### Gaps
- No authoritative 2025–2026 blog post or talk dedicated to Rust UI↔audio lock-free patterns was found in this pass. The pattern above is synthesized from crate docs (rtrb, basedrop) and engine design docs (Firewheel).
- The claim that crossbeam-channel may allocate or lock on contended paths was not verified against current source.

---

## 4. Audio, MIDI and OSC file and protocol I/O: hound, symphonia, FLAC/Ogg/Opus/MP3 encoders, midir, midly, rosc

### Takeaway
Decoding is solved by symphonia 0.6.1 (MPL-2.0, active; 0.6.0 shipped 2026-05-15). WAV writing by `hound` is stable but has had no release since 2023-09. A pure-Rust FLAC encoder exists (`flacenc` 0.5.1). Ogg Vorbis, Opus and MP3 encoding all require C libraries (`vorbis_rs`, `opus`, `mp3lame-encoder`, which is LGPL-3.0). MIDI I/O is covered by `midir` (active) and `midly` (stable but quiet since 2023). OSC is covered by `rosc` (last release 2025-03).

### Cited Findings

| Crate | Purpose | Latest stable (date) | License | Repo last push | Downloads total / 90-day | Rec. | Source |
|---|---|---|---|---|---|---|---|
| hound | WAV read/write | 3.5.1 (**2023-09-25**) | Apache-2.0 | 2026-08-09 | 21.0M / 6.15M; 637★ | **Adopt** | [crates.io](https://crates.io/crates/hound), [GitHub](https://github.com/ruuda/hound) |
| symphonia | Pure-Rust demux/decode (AAC, ADPCM, AIFF, ALAC, CAF, FLAC, MKV, MP1–3, MP4, OGG, Vorbis, WAV, WebM) | 0.6.1 (2026-08-13); MSRV 1.85 | **MPL-2.0** | 2026-08-13 | 17.6M / 8.99M; 3,437★ | **Adopt** | [crates.io](https://crates.io/crates/symphonia), [GitHub](https://github.com/pdeljanov/Symphonia) |
| claxon | FLAC decoder | 0.4.3 (2020-08-09) | Apache-2.0 | 2025-12-03 | 5.3M | Avoid (use symphonia) | [crates.io](https://crates.io/crates/claxon) |
| flacenc | Pure-Rust FLAC encoder | 0.5.1 (2025-12-18) | Apache-2.0 | 2026-06-29 | 745k / 200k; 41★ | **Trial** | [crates.io](https://crates.io/crates/flacenc), [GitHub](https://github.com/yotarok/flacenc-rs) |
| lewton | Pure-Rust Vorbis decoder | 0.10.2 (2021-01-20) | MIT OR Apache-2.0 | 2023-03-02 | 10M | Avoid (use symphonia) | [crates.io](https://crates.io/crates/lewton) |
| ogg | Ogg container read/write | 0.9.2 (2025-01-12) | BSD-3-Clause | 2025-01-12 | 11.6M | Assess | [crates.io](https://crates.io/crates/ogg) |
| vorbis_rs | Vorbis encode/decode (bundles libvorbis via C) | 0.5.6 (2026-07-30); MSRV 1.87 | BSD-3-Clause | 2026-10-17 | 245k / 74k | Assess | [crates.io](https://crates.io/crates/vorbis_rs) |
| opus | libopus bindings (encode/decode) | 0.4.0 (2026-08-23) | MIT/Apache-2.0 | 2026-08-23 | 3.04M / 1.74M | Assess | [crates.io](https://crates.io/crates/opus) |
| audiopus | libopus bindings | 0.2.0 (**2019-10-11**); 0.3.0-rc.0 (2021) | ISC | 2023-05-09 | 1.77M | **Avoid** | [crates.io](https://crates.io/crates/audiopus) |
| mp3lame-encoder | LAME MP3 encoder bindings | 0.2.5 (2026-08-20) | **LGPL-3.0** | 2026-08-20 | 1.15M / 312k | Assess (copyleft; optional feature only) | [crates.io](https://crates.io/crates/mp3lame-encoder) |
| midir | Cross-platform real-time MIDI I/O | 0.11.1 (2026-10-10) | MIT | 2026-10-10 | 903k / 306k; 831★ | **Adopt** | [crates.io](https://crates.io/crates/midir), [GitHub](https://github.com/Boddlnagg/midir) |
| midly | "Fast MIDI decoder and encoder both for .mid files and real-time MIDI events" | 0.5.3 (**2023-01-01**) | Unlicense | 2024-07-03 | 498k / 172k; 199★ | **Adopt** (stable, quiet) | [crates.io](https://crates.io/crates/midly), [GitHub](https://github.com/kovaxis/midly) |
| wmidi | MIDI message types | 4.0.11 (2026-04-19) | MIT | 2026-04-19 | 155k | Adopt (message types) | [crates.io](https://crates.io/crates/wmidi) |
| rosc | OSC 1.0 encode/decode | 0.11.4 (2025-03-23); MSRV 1.67.1 | MIT/Apache-2.0 | 2025-03-23 | 578k / 98k; 221★ | **Adopt** | [crates.io](https://crates.io/crates/rosc), [GitHub](https://github.com/klingtnet/rosc) |

- Symphonia releases: v0.5.5 (2025-10-11), v0.6.0 (2026-05-15), v0.6.1 (2026-08-13). By default Symphonia enables only royalty-free open codecs; others are behind feature flags. Native Opus status is "-" (no status), and the README lists `symphonia-adapter-libopus` as a third-party Opus adapter. Symphonia is decode and demux only. — [Symphonia GitHub releases](https://github.com/pdeljanov/Symphonia/releases), [Symphonia README](https://github.com/pdeljanov/Symphonia)
- kira 0.12.2 and Firewheel (via Symphonium) both decode through Symphonia. — [kira changelog](https://github.com/tesselode/kira/blob/HEAD/changelog.md), [Firewheel GitHub](https://github.com/BillyDM/Firewheel)
- rodio 0.22.2 added Opus through `symphonia-adapter-libopus`. — [rodio CHANGELOG](https://github.com/RustAudio/rodio/blob/HEAD/CHANGELOG.md)

### Inferences
- **Deterministic export path:** WAV via `hound` (32-bit float or 24-bit PCM), FLAC via `flacenc` (pure Rust, so it also works in wasm and Python wheels without C toolchains), and MIDI via `midly`. Generate SMF directly from the event timeline rather than from audio. MP3, Ogg and Opus should be optional cargo features: they need C libraries, and LAME is LGPL-3.0.
- `hound` and `midly` have had no release in about three years but are mature and narrow in scope. The risk is low; be prepared to vendor or fork.
- Symphonia's MPL-2.0 licence is file-level copyleft. It is compatible with permissive distribution as long as modified Symphonia files stay MPL.

### Gaps
- The output quality and speed of `flacenc` relative to libFLAC were not verified here.
- No pure-Rust Opus or MP3 encoder of production quality was identified.

---

## 5. Speech (tts crate and alternatives) and AccessKit

### Takeaway
`tts` is the only broad cross-platform TTS abstraction. Its backends are WinRT, SAPI via Tolk, NVDA, Speech Dispatcher, AVFoundation, Android and WebAssembly. Its last crates.io release was 0.26.3 on **2024-07-06**, even though the repo is active (last push 2026-08-11), which suggests unreleased work. AccessKit 0.25.1 (2026-09-25) is the active, widely used accessibility layer for native Rust UIs.

### Cited Findings

| Crate | Purpose | Latest stable (date) | License | Repo last push | Downloads total / 90-day | Rec. | Source |
|---|---|---|---|---|---|---|---|
| tts | High-level TTS over OS engines | 0.26.3 (**2024-07-06**) | MIT | 2026-08-11 | 461k / 86k; 143★ | **Trial** (pin a git rev if unreleased fixes are needed) | [crates.io](https://crates.io/crates/tts), [GitHub](https://github.com/ndarilek/tts-rs) |
| speech-dispatcher | Linux speech-dispatcher bindings (used by tts) | 0.16.0 (2022-11-22) | LGPL-2.1 OR MIT OR Apache-2.0 | n/a | 327k / 91k | Via tts | [crates.io](https://crates.io/crates/speech-dispatcher) |
| accesskit | "UI accessibility infrastructure across platforms" | 0.25.1 (2026-09-25); MSRV 1.87 | MIT OR Apache-2.0 | 2026-10-18 | 29.6M / 8.06M; 1,541★ | **Adopt** (if native UI) | [crates.io](https://crates.io/crates/accesskit), [GitHub](https://github.com/AccessKit/accesskit) |

- The tts README lists these backends: Windows ("NVDA via the NVDA Controller Client (requires shipping `nvdaControllerClient.dll`)", "Screen readers/SAPI via Tolk (requires `tolk` Cargo feature)", "WinRT"); Linux via Speech Dispatcher; "macOS/iOS/tvOS/watchOS/visionOS via AVFoundation (macOS 10.14 and above)"; Android (minSdk 26, embedded dex); and WebAssembly. — [tts-rs README](https://github.com/ndarilek/tts-rs)
- The README on the main branch (repo pushed 2026-08-11) describes Android "plug-and-play" support and visionOS, but the newest crates.io release is still 0.26.3 from 2024-07-06. That points to unreleased changes on main. — [tts-rs README](https://github.com/ndarilek/tts-rs), [crates.io tts](https://crates.io/crates/tts)

### Inferences
- For a sonification tool aimed at blind and low-vision users, routing speech through the user's screen reader (Tolk/NVDA on Windows) is often preferable to a separate synthesizer voice. tts supports both routes. Abstract speech behind a SONARITHM `Speaker` trait so the backend can be swapped to the screen reader, OS TTS, or (later) a neural TTS.
- tts outputs through the OS engine, not into SONARITHM's audio graph. Speech therefore can't be mixed into deterministic offline exports through tts. For exports, record cue text in a sidecar (WebVTT/JSON) or use a separate offline TTS engine (gap).
- If the desktop app uses Tauri (webview), accessibility comes from the webview's own a11y tree and AccessKit isn't needed. AccessKit matters for egui, iced, Slint or winit-based native UIs.

### Gaps
- No maintained pure-Rust offline neural TTS crate (for example Piper or eSpeak-NG bindings) was evaluated in this pass. Needs a separate search.
- Whether the unreleased tts changes include breaking API changes was not checked.

---

## 6. Data ingestion: arrow-rs, polars, datafusion, csv, calamine, parquet, chrono/jiff; streaming (tokio-tungstenite, rumqttc, serialport, sqlx)

### Takeaway
Arrow-rs (60.0.0) is the right internal columnar interchange: it is Apache-governed, releases monthly, and offers zero-copy to pyarrow and Polars. Polars' Rust crate is still 0.x (0.55.2), and its versioning policy says Rust API breaking changes "are not deprecated first". Keep Polars an optional feature. csv, calamine, parquet, jiff/chrono, tokio-tungstenite and serialport are all active. Heavy crates such as datafusion and sqlx carry MSRV 1.94.

### Cited Findings

| Crate | Purpose | Latest stable (date) | License | MSRV | Repo last push | Downloads total / 90-day | Rec. | Source |
|---|---|---|---|---|---|---|---|---|
| arrow | Apache Arrow columnar in-memory format | 60.0.0 (2026-09-15); 59.3.1 backport (2026-10-15) | Apache-2.0 | 1.88 | 2026-10-18 | 88.9M / 23.2M | **Adopt** | [crates.io](https://crates.io/crates/arrow) |
| parquet | Parquet read/write | 60.0.0 (2026-09-15) | Apache-2.0 | 1.88 | 2026-10-18 | 85.0M / 20.7M | **Adopt** | [crates.io](https://crates.io/crates/parquet) |
| polars | DataFrame engine | 0.55.2 (2026-08-06) | MIT | n/a | 2026-10-18 | 15.8M / 2.82M; 40,111★ | **Trial** (optional feature) | [crates.io](https://crates.io/crates/polars) |
| datafusion | SQL/DataFrame query engine on Arrow | 55.2.0 (2026-10-07) | Apache-2.0 | **1.94** | 2026-10-18 | 29.6M / 6.5M | Assess | [crates.io](https://crates.io/crates/datafusion) |
| csv | CSV reader/writer | 1.4.0 (2025-10-17) | Unlicense/MIT | 1.73 | 2026-08-04 | 269M | **Adopt** | [crates.io](https://crates.io/crates/csv) |
| calamine | xlsx/xls/xlsb/ods reader | 0.36.1 (2026-07-27) | MIT | 1.88 | 2026-10-17 | 15.5M / 5.48M | **Adopt** | [crates.io](https://crates.io/crates/calamine) |
| jiff | Date/time with tz, by BurntSushi | 0.2.38 (2026-10-06) | Unlicense OR MIT | 1.70 | 2026-10-14 | 231M / 87.6M | **Adopt** (new code) | [crates.io](https://crates.io/crates/jiff) |
| chrono | Date/time | 0.4.45 (2026-06-04) | MIT OR Apache-2.0 | 1.62 | 2026-09-07 | 891M | Adopt (interop; Arrow/Polars use it) | [crates.io](https://crates.io/crates/chrono) |
| tokio | Async runtime | 1.53.3 (2026-10-13) | MIT | 1.71 | 2026-10-16 | 1.08B | **Adopt** | [crates.io](https://crates.io/crates/tokio) |
| tokio-tungstenite | WebSocket client/server | 0.30.0 (2026-07-11) | MIT | 1.85 | 2026-07-11 | 314M | **Adopt** | [crates.io](https://crates.io/crates/tokio-tungstenite) |
| rumqttc | MQTT client | 0.25.1 (2025-11-21) | Apache-2.0 | 1.64 | 2026-05-01 | 9.15M / 2.21M | Trial | [crates.io](https://crates.io/crates/rumqttc) |
| serialport | Serial ports | 4.10.1 (2026-09-08) | **MPL-2.0** | 1.59 | 2026-09-08 | 19.9M | **Adopt** | [crates.io](https://crates.io/crates/serialport) |
| sqlx | Async SQL (Postgres, MySQL, SQLite) | 0.9.0 (2026-05-21) | MIT OR Apache-2.0 | **1.94** | 2026-10-16 (repo now `transact-rs/sqlx`) | 167M / 47M | Trial (optional source) | [crates.io](https://crates.io/crates/sqlx), [GitHub](https://github.com/transact-rs/sqlx) |

- Polars versioning: "Polars adheres to the semantic versioning specification". The public API is tied to the Python API reference. "Breaking changes to the Rust API are not deprecated first, but *will* be listed in the changelog", because "Supporting deprecated functionality would slow down development too much at this point in time." — [Polars versioning policy](https://docs.pola.rs/development/versioning/)
- The Polars repo layout puts the Rust core in `crates/` (30+ crates including `polars-python`, `polars-ffi`, `polars-stream`, `polars-ooc`), the Python package in `py-polars/`, and the plugin helper in `pyo3-polars/`. — [pola-rs/polars](https://github.com/pola-rs/polars)

### Inferences
- **Internal data model:** normalize every source (CSV, XLSX/ODS, Parquet, SQL, JSON, live streams) into Arrow `RecordBatch`es with a known schema. The sonification spec compiler then maps over columns. This gives zero-copy interop with Python (pyarrow, pandas, Polars through the Arrow C Data Interface) and wasm (Arrow JS).
- Make Polars an opt-in feature for advanced transforms: it releases often and Rust API breaks arrive without deprecation, and it adds large compile times (inference, not measured). Datafusion would add SQL-over-files, but it is heavy and raises MSRV to 1.94, so defer it.
- Use **jiff** for new timestamp parsing and time-zone handling. Convert to the Arrow timestamp types at the boundary; Arrow-rs uses chrono internally, so chrono remains a transitive dependency.
- Live streams: tokio-tungstenite, rumqttc and serialport tasks feed a bounded channel, then a windowing/compiler stage, then an SPSC ring to audio (see §3).

### Gaps
- Whether Polars publishes any separate Rust API stability tiers was not found. Only the general versioning page addresses it.
- The reason sqlx's repo moved from `launchbadge` to `transact-rs` was not investigated.

---

## 7. Math/expressions and scripting (evalexpr, exmex, fasteval, meval; rhai, mlua) and analytics (statrs, ndarray, linfa, augurs, changepoint, anomaly detection)

### Takeaway
**evalexpr has been AGPL-3.0-only since 12.0.0 (2024-10-17)**, which disqualifies it for a permissively licensed core. fasteval (last release 2020) and meval (2018) are abandoned. **exmex** (0.21.0, MIT/Apache-2.0, symbolic differentiation) is the permissive expression choice, and **rhai** (1.26.1) is the mature sandboxed scripting choice. statrs, ndarray and augurs (Grafana-maintained) are active. `anomaly_detection` is GPL-3.0.

### Cited Findings

| Crate | Purpose | Latest stable (date) | License | Repo last push | Downloads total / 90-day | Rec. | Source |
|---|---|---|---|---|---|---|---|
| evalexpr | Expression evaluator | 13.1.0 (2025-11-26) | **AGPL-3.0-only** | 2026-10-04 | 11.0M / 2.38M | **Avoid** (licence) | [crates.io](https://crates.io/crates/evalexpr), [CHANGELOG](https://github.com/ISibboI/evalexpr/blob/HEAD/CHANGELOG.md) |
| exmex | Fast math expressions plus symbolic differentiation | 0.21.0 (2026-05-23); MSRV 1.80.1 | MIT OR Apache-2.0 | 2026-05-23 | 144k / 58k; 49★ | **Trial** (low bus factor) | [crates.io](https://crates.io/crates/exmex), [GitHub](https://github.com/bertiqwerty/exmex) |
| fasteval | Fast expression evaluator | 0.2.4 (**2020-01-25**) | MIT | 2023-11-10 | 428k | **Avoid** (abandoned) | [crates.io](https://crates.io/crates/fasteval), [GitHub](https://github.com/likebike/fasteval) |
| meval | Math expression parser | 0.2.0 (**2018-09-30**) | Unlicense/MIT | 2022-03-03 | 1.66M | **Avoid** (abandoned) | [crates.io](https://crates.io/crates/meval), [GitHub](https://github.com/rekka/meval-rs) |
| rhai | Embedded scripting language (pure Rust, sandboxable) | 1.26.1 (2026-09-10); MSRV 1.66 | MIT OR Apache-2.0 | 2026-10-17 | 13.1M / 4.45M; 5,737★ | **Adopt** | [crates.io](https://crates.io/crates/rhai) |
| mlua | Lua/LuaJIT/Luau bindings | 0.12.2 (2026-10-03); MSRV 1.88 | MIT | 2026-10-03 | 7.9M / 2.29M; 2,905★ | Assess | [crates.io](https://crates.io/crates/mlua) |
| statrs | Statistical distributions and functions | 0.19.1 (2026-08-11); MSRV 1.89 | MIT | 2026-09-23 | 42.9M / 11.1M | **Adopt** | [crates.io](https://crates.io/crates/statrs) |
| ndarray | N-d arrays | 0.17.2 (2026-01-10) | MIT OR Apache-2.0 | 2026-07-18 | 134M / 30.9M | **Adopt** | [crates.io](https://crates.io/crates/ndarray) |
| linfa | ML toolkit (clustering, regression…) | 0.8.1 (2025-12-23) | MIT OR Apache-2.0 | 2026-08-22 | 1.98M / 527k; 4,761★ | Assess | [crates.io](https://crates.io/crates/linfa) |
| augurs | "A time-series toolkit for forecasting, outlier detection, clustering and more" (Grafana) | 0.10.2 (2026-02-24) | MIT OR Apache-2.0 | 2026-10-17 | 81k / 21k; 579★ | **Trial** | [crates.io](https://crates.io/crates/augurs), [GitHub](https://github.com/grafana/augurs) |
| changepoint | Bayesian online changepoint detection | 0.15.0 (2025-10-20) | MIT | 2025-10-20 | 193k / 58k | Assess | [crates.io](https://crates.io/crates/changepoint) |
| anomaly_detection | Twitter AnomalyDetection port (ankane) | 0.4.0 (2026-04-07) | **GPL-3.0-or-later** | 2026-04-07 | 17k | **Avoid** (licence) | [crates.io](https://crates.io/crates/anomaly_detection) |

- evalexpr's changelog entry for **12.0.0 (2024-10-17)** states "`evalexpr` is now licensed under the GNU Affero General Public License Version 3". The README says "This crate is primarily distributed under the terms of the AGPL3 license" and offers other licensing on request. — [evalexpr CHANGELOG](https://github.com/ISibboI/evalexpr/blob/HEAD/CHANGELOG.md), [evalexpr README](https://github.com/ISibboI/evalexpr)

### Inferences
- **Spec mapping expressions** (for example `pitch = 200 + 800 * norm(value)`): use exmex for compiled numeric expressions; its derivatives are useful for slope or gradient sonification. Pre-compile expressions at spec-compile time and never evaluate them on the audio thread. exmex has a single maintainer (49★), so keep a thin facade that would let you swap it for a small in-house Pratt parser.
- **User scripting** (custom mappings, data cleaning): rhai is pure Rust, sandboxable (operation limits), compiles to wasm, and is permissively licensed. mlua needs a C Lua build and is harder in wasm (inference). Treat WASM components (§10) as the long-term third-party extension route and rhai as the "power-user formula" route.
- **Analytics for auditory highlighting:** statrs for distributions and z-scores, and augurs for outlier detection and forecasting residuals, which can drive earcons. Check its changepoint and seasonality modules before relying on them (see Gaps). Avoid the GPL `anomaly_detection`; port simple algorithms or use augurs' outlier module instead.

### Gaps
- augurs' current module list (MSTL, ETS, Prophet, DBSCAN outliers, changepoint) and its wasm/JS bindings were not re-verified against its 0.10.2 docs in this pass.
- The performance of exmex against fasteval was not benchmarked here.

---

## 8. ML inference for chart-image digitization: ort 2.x, candle, burn, tract

### Takeaway
`ort` is still at **2.0.0-rc.14 (2026-10-12)**, with no stable 2.0 after rc.11 to rc.14 through 2026, but it is widely used (8.8M downloads in 90 days). It wraps the native ONNX Runtime (rc.14 targets ORT 1.31) with static or `load-dynamic` linking and pure-Rust alternative backends (ort-tract, ort-candle). tract (0.23.9) is the pure-Rust, wasm-friendly ONNX option. candle (0.11) and burn (0.22, MSRV 1.95) are full frameworks. Isolate digitization in a separate optional crate.

### Cited Findings

| Crate | Purpose | Latest (date) | License | MSRV | Repo last push | Downloads total / 90-day; stars | Rec. | Source |
|---|---|---|---|---|---|---|---|---|
| ort | ONNX Runtime bindings (pyke) | **2.0.0-rc.14** (2026-10-12); no stable 2.x | MIT OR Apache-2.0 | 1.92 | 2026-10-17 | 22.5M / 8.83M; 2,572★ | **Trial** (optional crate) | [crates.io](https://crates.io/crates/ort), [releases](https://github.com/pykeio/ort/releases) |
| tract-onnx | Pure-Rust ONNX/NNEF inference (Sonos) | 0.23.9 (2026-10-15); 0.21.19 backport same day | MIT OR Apache-2.0 | 1.91 | 2026-10-18 | 3.52M / 1.39M; 3,093★ | **Trial** (wasm/CLI default) | [crates.io](https://crates.io/crates/tract-onnx) |
| candle-core | Minimalist ML framework (Hugging Face) | 0.11.0 (2026-06-26) | MIT OR Apache-2.0 | n/a | 2026-10-14 | 10.1M / 3.87M; 21,194★ | Assess | [crates.io](https://crates.io/crates/candle-core) |
| burn | Deep learning framework (multi-backend) | 0.22.0 (2026-10-06) | MIT OR Apache-2.0 | **1.95** | 2026-10-16 | 1.80M / 676k; 16,127★ | Assess | [crates.io](https://crates.io/crates/burn) |

- ort GitHub release dates: rc.11 2026-01-07, rc.12 2026-03-05, rc.13 2026-07-28, rc.14 2026-10-12. — [pykeio/ort releases](https://github.com/pykeio/ort/releases)
- rc.14 updates to ONNX Runtime v1.31.0. rc.13 "only ships CUDA 13 binaries", reworked custom operators, and gated execution-provider structs behind their Cargo features. — [newreleases.io rc.14](https://newreleases.io/project/github/pykeio/ort/release/v2.0.0-rc.14), [newreleases.io rc.13](https://newreleases.io/project/github/pykeio/ort/release/v2.0.0-rc.13)
- A March 2026 third-party blog calls rc.12 "stable" and production-ready. This conflicts with the project's own RC labelling. — [dasroot.net](https://dasroot.net/posts/2026/03/onnx-runtime-rust-ml-inference-optimization/); contradicted by [pykeio/ort releases](https://github.com/pykeio/ort/releases)
- ort linking docs: "You should prefer static linking if your execution providers support it". For dynamic linking, "We recommend `load-dynamic` as it gives more control and is often far less troublesome to work with". Alternative backends are listed as ort-candle, ort-tract and ort-web. — [ort linking docs](https://ort.pyke.io/setup/linking)
- ONNX Runtime supports custom "minimal" builds (ORT-format models only, reduced operators) to cut binary size. — [ONNX Runtime custom build docs](https://onnxruntime.ai/docs/build/custom.html)
- Real-world pattern: the `sceptre` OCR crate uses `ort` on desktop/server and `tract` for WASM/Android behind a backend seam. Its tract path pins input shapes because tract can't shape-infer data-dependent dynamic dimensions. — [sceptre source](https://docs.rs/crate/sceptre/0.3.0/source/src/inference/mod.rs)

### Inferences
- Chart digitization (axis detection, line/point extraction) is a secondary feature. Put it in a separate crate (`sonarithm-digitize`) behind a cargo feature, so CLI, Python and wasm builds of the core don't carry the ONNX Runtime binary or the CUDA EP baggage.
- Default to **tract** for portability (pure Rust, wasm, no native library) with fixed input shapes. Offer **ort** with `load-dynamic` on desktop for speed and larger models. Mirror the sceptre backend seam (one `InferenceBackend` trait).
- Don't pin to a single ort RC without watching for breaking changes between RCs; rc.13 changed custom ops and EP gating.

### Gaps
- No published MB figures for ort-linked binary size compared with tract or ort-tract were found. Measure with the actual model.
- ort's exact current MSRV policy and its schedule for a stable 2.0 were not found.

---

## 9. Bindings and targets: PyO3 + maturin, wasm-bindgen/wasm-pack, uniffi, cbindgen; how mature projects structure multi-target workspaces

### Takeaway
PyO3 0.30.1 (2026-10-14, supports Python 3.15 final) with maturin 1.15 is the clear Python path. wasm-bindgen (0.2.129) now lives in its own `wasm-bindgen` GitHub org after the rustwasm org was sunset in 2025, and wasm-pack (0.15.0) moved there too. Mature projects separate a pure-Rust core crate (or many crates) from thin binding crates: polars uses `crates/*` + `py-polars`, tokenizers uses `tokenizers/` + `bindings/{python,node,wasm}`, ruff uses `crates/ruff_wasm` and `crates/ruff_server`, and pydantic-core uses a single crate plus `python/` and `wasm-preview/`.

### Cited Findings

| Crate | Purpose | Latest (date) | License | MSRV | Repo last push | Downloads total / 90-day | Rec. | Source |
|---|---|---|---|---|---|---|---|---|
| pyo3 | Rust↔Python bindings | 0.30.1 (2026-10-14) | MIT OR Apache-2.0 | 1.85 | 2026-10-14 | 278M / 66M; 16,243★ | **Adopt** | [crates.io](https://crates.io/crates/pyo3) |
| maturin | Build/publish PyO3 wheels | 1.15.0 (2026-08-24) | MIT OR Apache-2.0 | 1.89 | 2026-10-18 | 1.15M | **Adopt** | [crates.io](https://crates.io/crates/maturin) |
| numpy (rust-numpy) | NumPy interop for PyO3 | 0.30.0 (2026-10-15) | BSD-2-Clause | 1.85 | 2026-10-15 | 52M | **Adopt** | [crates.io](https://crates.io/crates/numpy) |
| wasm-bindgen | JS↔Rust bindings | 0.2.129 (2026-09-25) | MIT OR Apache-2.0 | 1.81 | 2026-10-17 | 592M | **Adopt** | [crates.io](https://crates.io/crates/wasm-bindgen), [GitHub](https://github.com/wasm-bindgen/wasm-bindgen) |
| wasm-pack | Build/package wasm for npm | 0.15.0 (2026-05-15) | MIT OR Apache-2.0 | n/a | 2026-08-12 (repo `wasm-bindgen/wasm-pack`) | 6.49M | Assess | [crates.io](https://crates.io/crates/wasm-pack) |
| uniffi | Kotlin/Swift/Python bindings generator (Mozilla) | 0.32.2 (2026-09-23) | **MPL-2.0** | n/a | 2026-10-16 | 14.6M | Assess (mobile later) | [crates.io](https://crates.io/crates/uniffi) |
| cbindgen | C/C++ header generation | 0.29.4 (2026-06-09) | **MPL-2.0** (tool; build-time) | 1.74 | 2026-08-22 | 110M | Adopt (if a C ABI is offered) | [crates.io](https://crates.io/crates/cbindgen) |

- PyO3 0.30.0 (2026-10-13): "Support Python 3.15.0 final". The 0.29.x releases include fixes for free-threaded Python, such as atomic loads in `PyList_GET_SIZE` and similar, and `abi3t-py3*` features for free-threaded builds. — [PyO3 CHANGELOG](https://github.com/PyO3/pyo3/blob/HEAD/CHANGELOG.md)
- The Rust project's July 2025 post "Sunsetting the rustwasm GitHub org" announced that wasm-bindgen would move to a new `wasm-bindgen` org with new maintainers, that other repos would be archived or handed over, and that the org would be archived in September 2025. — [Inside Rust blog](https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org)
- crates.io lists wasm-pack's repository as `wasm-bindgen/wasm-pack` (7,288★, last push 2026-08-12). A 2025 forum reply had said it went to an individual maintainer. — [crates.io wasm-pack](https://crates.io/crates/wasm-pack); earlier claim in [users.rust-lang.org](https://users.rust-lang.org/t/future-of-rust-wasm/133089)
- Workspace layouts (GitHub tree, 2026-10-18):
  - **polars**: `crates/` (polars-core, -lazy, -plan, -expr, -io, -stream, -python, -ffi, -dylib, …), `py-polars/`, `pyo3-polars/`, `deny.toml`, `rust-toolchain.toml`. — [pola-rs/polars](https://github.com/pola-rs/polars)
  - **tokenizers**: core `tokenizers/` crate plus `bindings/node`, `bindings/python`, `bindings/wasm`. — [huggingface/tokenizers](https://github.com/huggingface/tokenizers)
  - **ruff**: 55 crates under `crates/`, including `ruff_wasm`, `ty_wasm`, `ruff_server`, `ty_server`, plus top-level `python/`, `playground/`, `fuzz/`, `dist-workspace.toml`, `rust-toolchain.toml`. — [astral-sh/ruff](https://github.com/astral-sh/ruff)
  - **pydantic-core**: a single crate (`src/`) with `python/`, `wasm-preview/`, `pyproject.toml`, `benches/`. — [pydantic/pydantic-core](https://github.com/pydantic/pydantic-core)
  - **arrow-rs**: many top-level `arrow-*` crates (arrow-array, arrow-ipc, arrow-pyarrow, …) plus `parquet`. — [apache/arrow-rs](https://github.com/apache/arrow-rs)

### Inferences
- **Suggested SONARITHM workspace** (modelled on tokenizers and ruff):
  `crates/sonarithm-core` (spec model, compiler, timeline; no I/O, `no_std`-friendly where feasible), `sonarithm-dsp` (nodes and synthesis), `sonarithm-engine` (RT graph and scheduler, backend-agnostic), `sonarithm-audio-cpal` (device backend), `sonarithm-export` (WAV/FLAC/MIDI), `sonarithm-data` (Arrow ingestion and sources, feature-gated), `sonarithm-digitize` (optional ML), `sonarithm-cli`, plus `bindings/python` (PyO3/maturin, abi3 wheels), `bindings/wasm` (wasm-bindgen), and the `app/` desktop shell.
- Keep binding crates thin, converting only types (Arrow via the C Data Interface for Python, `Float32Array` for wasm). Never let `pyo3` or `wasm-bindgen` types leak into the core.
- For wasm, wasm-bindgen is safe to adopt. wasm-pack still works but has had maintainer churn; consider driving `wasm-bindgen-cli` directly or through a bundler plugin.
- Pin a `rust-toolchain.toml` for CI reproducibility, as polars, ruff and arrow-rs do.

### Gaps
- maturin's current support matrix for free-threaded wheels was not verified.
- pyo3's default-on/off status for free-threaded support in 0.30 was not confirmed beyond the changelog references.

---

## 10. Extensibility: wasmtime component model + WIT (WASI 0.2/0.3), extism, dynamic loading pitfalls; best plugin approach for third-party sonification engines

### Takeaway
**WASI 0.3.0 was released 2026-06-11 (0.3.1 on 2026-08-11)**. It moves async into the Component Model with native `stream<T>`/`future<T>`. Wasmtime 46 and later enable it by default, and Wasmtime is now at 49.0.3 (2026-10-14, MSRV 1.96). The component model plus WIT is the best fit for typed, sandboxed, language-agnostic third-party "sonification engines". Extism (1.41.0) is simpler but less typed. Native dylib plugins (libloading, abi_stable) fight Rust's lack of a stable ABI, and abi_stable has been stale since 2023.

### Cited Findings

| Crate | Purpose | Latest (date) | License | MSRV | Repo last push | Downloads total / 90-day | Rec. | Source |
|---|---|---|---|---|---|---|---|---|
| wasmtime | Wasm runtime with component model | 49.0.3 (2026-10-14) | Apache-2.0 WITH LLVM-exception | **1.96** | 2026-10-16 | 40.5M / 11.4M; 18,732★ | **Trial** | [crates.io](https://crates.io/crates/wasmtime) |
| wit-bindgen | Guest bindings from WIT | 0.63.0 (2026-10-14) | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | n/a | 2026-10-15 | 293M | Trial | [crates.io](https://crates.io/crates/wit-bindgen) |
| extism | Cross-language Wasm plugin framework | 1.41.0 (2026-10-09) | BSD-3-Clause | n/a | 2026-10-09 | 774k / 206k; 5,806★ | Assess | [crates.io](https://crates.io/crates/extism), [GitHub](https://github.com/extism/extism) |
| libloading | Dynamic library loading | 0.9.0 (2025-11-05) | ISC | 1.88 | 2026-08-02 | 577M | Avoid for third-party plugins | [crates.io](https://crates.io/crates/libloading) |
| abi_stable | Stable-ABI Rust-to-Rust FFI | 0.11.3 (**2023-10-12**) | MIT/Apache-2.0 | 1.61 | 2023-10-12 | 4.15M | **Avoid** (stale) | [crates.io](https://crates.io/crates/abi_stable) |
| stabby | Stable ABI for Rust | 72.1.16 (2026-07-20) | EPL-2.0 OR Apache-2.0 | n/a | 2026-09-27 | 8.05M | Assess | [crates.io](https://crates.io/crates/stabby) |

- WASI 0.3.0 was released June 11, 2026, and 0.3.1 on August 11, 2026. Async moved from `wasi:io` into the Component Model (`async func`, `stream<T>` replacing `input-stream`/`output-stream`, `future<T>` replacing `pollable`), and `wasi:io` has no 0.3 version. "Wasmtime 46 and later" supports 0.3 with `component-model-async` on by default. Wasmtime 43–45 and 41–42 implemented RC snapshots behind flags. "Migration to WASI 0.3 is not required in order to use a 0.3-capable runtime", and 0.3 runtimes can run 0.2 components. — [wasi.dev WASI P3](https://wasi.dev/releases/wasi-p3)
- Third-party sources disagree on the history: some claim earlier "stable" milestones or preview availability in Wasmtime 37+. The official page's dates are taken as authoritative here. — [codercops blog](https://blog.codercops.com/blog/wasi-0-3-native-async-io-webassembly-2026), [noqta.tn](https://noqta.tn/blog/webassembly-wasm-beyond-browser-production-2026); contradicted by [wasi.dev](https://wasi.dev/releases/wasi-p3)
- Wasmtime release tags v49.0.3 and v48.0.6 were both published 2026-10-14 (patch backports). — [wasmtime releases](https://github.com/bytecodealliance/wasmtime/releases)

### Inferences
- **Plugin approach:** define a WIT world, for example `sonarithm:engine/mapper`, with functions such as `describe() -> engine-info`, `compile(spec-fragment, schema) -> list<event>`, and optionally `render-block(params, frames) -> list<f32>`. Third parties then write engines in Rust, C, Zig, Go, JS or Python (componentize-py) and SONARITHM loads them sandboxed with fuel or epoch limits.
- **Real-time caution:** run plugins at **compile time** (spec → event timeline), not inside the audio callback. Wasmtime calls can allocate, trap or hit epoch interrupts, which is unsuitable for hard RT. If plugin DSP is needed, run it on a worker thread that pre-renders blocks into a ring buffer with look-ahead (this adds latency).
- Wasmtime's MSRV (1.96) and roughly monthly major versions would pull up the whole workspace MSRV. Keep it in a separate `sonarithm-plugins` crate behind a feature flag so the core MSRV can stay around 1.85–1.88.
- Extism is a quicker on-ramp (bytes-in/bytes-out with host functions) but loses WIT's typed interfaces. Choose it only if typed component tooling proves too heavy for contributors.
- Avoid native Rust dylib plugins for third parties (no stable ABI, allocator mismatches, unloading UB, platform signing issues). If native plugins are ever needed, a C ABI through cbindgen is the safer contract.

### Gaps
- Toolchain maturity for authoring WASI 0.3 components in non-Rust languages (componentize-py, jco, TinyGo) as of October 2026 was not verified.
- Wasmtime's own MSRV policy statement was not fetched; only the crate's declared `rust-version` (1.96) is known.

---

## 11. Schema/IPC (serde, schemars, specta/tauri-specta, ts-rs), errors and diagnostics (thiserror, miette), tracing

### Takeaway
serde, schemars 1.x, thiserror 2.x and tracing are safe defaults. The declarative sonification spec should be serde-modelled with schemars generating the JSON Schema. For TypeScript types, ts-rs 12.0.1 is stable. specta and tauri-specta are still in **2.0.0-rc.25** (May 2026), and their only stable line is 1.0.x from 2023. miette (7.6.0, 2025-04-27) suits spec-error diagnostics but releases slowly.

### Cited Findings

| Crate | Purpose | Latest (date) | License | MSRV | Repo last push | Downloads total / 90-day | Rec. | Source |
|---|---|---|---|---|---|---|---|---|
| serde | Serialization | 1.0.229 (2026-07-18) | MIT OR Apache-2.0 | 1.56 | 2026-10-16 | 1.55B | **Adopt** | [crates.io](https://crates.io/crates/serde) |
| schemars | JSON Schema from Rust types | 1.2.3 (2026-10-12) | MIT | 1.74 | 2026-10-12 | 554M / 206M | **Adopt** | [crates.io](https://crates.io/crates/schemars) |
| specta | Export Rust types to TS and other languages | stable 1.0.5 (2023-07-17); **2.0.0-rc.25** (2026-05-07) | MIT | n/a | 2026-10-15 | 3.04M / 1.40M | Trial (RC) | [crates.io](https://crates.io/crates/specta) |
| tauri-specta | Typed Tauri commands/events | stable 1.0.2 (2023); **2.0.0-rc.25** (2026-05-08) | MIT | n/a | 2026-10-15 | 1.55M / 679k | Trial (if Tauri) | [crates.io](https://crates.io/crates/tauri-specta) |
| ts-rs | Generate TS types from Rust | 12.0.1 (2026-01-31) | MIT | 1.78 | 2026-10-02 | 18.9M / 8.11M | **Adopt** | [crates.io](https://crates.io/crates/ts-rs) |
| thiserror | Error derive | 2.0.21 (2026-09-23) | MIT OR Apache-2.0 | 1.77 | 2026-10-16 | 1.65B | **Adopt** | [crates.io](https://crates.io/crates/thiserror) |
| miette | Fancy diagnostics with source spans | 7.6.0 (2025-04-27) | Apache-2.0 | 1.70 | 2026-06-25 | 83M / 21M | **Adopt** (CLI/spec errors) | [crates.io](https://crates.io/crates/miette) |
| tracing | Structured logging/instrumentation | 0.1.44 (2025-12-18) | MIT | 1.65 | 2026-10-09 | 944M | **Adopt** (non-RT threads) | [crates.io](https://crates.io/crates/tracing) |
| tauri | Desktop shell (webview) | 2.12.4 (2026-10-14); 3.0.0-alpha.5 same day | Apache-2.0 OR MIT | 1.90 | 2026-10-15 | 37.3M | (desktop shell is out of scope; see frontend notes) | [crates.io](https://crates.io/crates/tauri) |

### Inferences
- **Spec pipeline:** a Rust `serde` model is the single source of truth. schemars emits `sonarithm-spec.schema.json`, which supports editor validation and Python/JS validation and keeps docs in sync. ts-rs emits TS types for the desktop UI and wasm consumers. Python uses the JSON Schema or pydantic models generated from it.
- Prefer ts-rs over specta for now: specta 2.0 has been in RC for a long time (rc.25). If the desktop app is Tauri 2, tauri-specta's typed commands are attractive, but take on the RC churn knowingly.
- Use miette for human-friendly spec-compile errors with spans (YAML/JSON line and column), thiserror for library error enums, and `tracing` everywhere except the audio thread.

### Gaps
- The roadmap for a specta 2.0 stable release was not found.
- Tauri 3.0's timeline (alpha.5 published 2026-10-14) and its implications are covered in separate desktop frontend notes, not here.

---

## 12. Quality tooling (nextest, insta, proptest, criterion vs divan, cargo-fuzz, deny, audit, vet, semver-checks, machete, Miri, loom), Rust 2024 edition and MSRV norms

### Takeaway
All mainstream tools are active in October 2026. The exceptions are divan (last release 2025-04-10) and loom (last release 2024-04-23, repo touched 2026-02). shuttle (0.9.7, 2026-10-14) is the more actively released concurrency tester. Rust stable is 1.99.1 (2026-10-16). The 2024 edition shipped with Rust 1.85 (Feb 2025), and core audio crates (cpal, pyo3, symphonia) now declare MSRV 1.85.

### Cited Findings

| Tool/crate | Latest (date) | License | MSRV | Repo last push | Downloads total / 90-day | Rec. | Source |
|---|---|---|---|---|---|---|---|
| cargo-nextest | 0.9.148 (2026-10-08) | Apache-2.0 OR MIT | 1.91 | 2026-10-16 | 13.3M | **Adopt** | [crates.io](https://crates.io/crates/cargo-nextest) |
| insta | 1.49.0 (2026-10-03) | Apache-2.0 | 1.66 | 2026-10-10 | 114M | **Adopt** | [crates.io](https://crates.io/crates/insta) |
| proptest | 1.12.0 (2026-10-10) | MIT OR Apache-2.0 | 1.88 | 2026-10-10 | 214M | **Adopt** | [crates.io](https://crates.io/crates/proptest) |
| criterion | 0.8.2 (2026-02-04); repo `criterion-rs/criterion.rs` | Apache-2.0 OR MIT | 1.86 | 2026-10-13 | 310M / 67M | **Adopt** | [crates.io](https://crates.io/crates/criterion) |
| divan | 0.1.21 (2025-04-10) | MIT OR Apache-2.0 | 1.80 | 2026-07-19 | 9.8M / 4.2M | Trial | [crates.io](https://crates.io/crates/divan) |
| cargo-fuzz | 0.13.2 (2026-06-09) | MIT OR Apache-2.0 | n/a | 2026-07-20 | 5.4M | **Adopt** (spec parser, decoders) | [crates.io](https://crates.io/crates/cargo-fuzz) |
| cargo-deny | 0.20.2 (2026-07-09) | MIT OR Apache-2.0 | 1.88 | 2026-10-16 | 6.5M | **Adopt** | [crates.io](https://crates.io/crates/cargo-deny) |
| cargo-audit | 0.22.2 (2026-06-05) | Apache-2.0 OR MIT | 1.88 | 2026-10-15 | 13.4M | Adopt (or rely on cargo-deny advisories) | [crates.io](https://crates.io/crates/cargo-audit) |
| cargo-vet | 0.10.2 (2026-01-13) | Apache-2.0/MIT | 1.82 | 2026-10-06 | 677k | Assess | [crates.io](https://crates.io/crates/cargo-vet) |
| cargo-semver-checks | 0.51.0 (2026-10-03) | Apache-2.0 OR MIT | 1.93 | 2026-10-18 | 691k | **Adopt** (published crates) | [crates.io](https://crates.io/crates/cargo-semver-checks) |
| cargo-machete | 0.9.2 (2026-04-15) | MIT | n/a | 2026-09-07 | 3.1M | **Adopt** | [crates.io](https://crates.io/crates/cargo-machete) |
| loom | 0.7.2 (**2024-04-23**) | MIT | 1.65 | 2026-02-20 | 72.5M | Assess | [crates.io](https://crates.io/crates/loom) |
| shuttle | 0.9.7 (2026-10-14) | Apache-2.0 | n/a | 2026-10-15 | 6.5M / 2.0M | **Trial** | [crates.io](https://crates.io/crates/shuttle) |

- The latest Rust releases are 1.99.0 (2026-10-01) and 1.99.1 (2026-10-16). — [rust-lang/rust releases](https://github.com/rust-lang/rust/releases)
- Rust 1.85.0 (2025-02-20) stabilized the Rust 2024 edition. — [Rust 1.85.0 announcement](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
- Rust 1.84.0 (2025-01-09) stabilized the MSRV-aware dependency resolver, which is the default for the 2024 edition (resolver v3). — [Rust 1.84.0 announcement](https://blog.rust-lang.org/2025/01/09/Rust-1.84.0/)
- Declared MSRVs (crates.io `rust-version`) of key dependencies:
  - 1.85: cpal, pyo3, numpy, symphonia, wasapi, tokio-tungstenite
  - 1.86: triple_buffer, web-audio-api, criterion
  - 1.87: rodio, rubato, accesskit, vorbis_rs
  - 1.88: arrow, parquet, calamine, mlua, proptest, cargo-deny
  - 1.89: statrs, maturin
  - 1.91: tract-onnx
  - 1.92: ort
  - 1.94: datafusion, sqlx
  - 1.95: burn, bevy_seedling
  - 1.96: wasmtime, bevy_kira_audio

  — crates.io pages linked in the tables above (for example [cpal](https://crates.io/crates/cpal), [wasmtime](https://crates.io/crates/wasmtime), [datafusion](https://crates.io/crates/datafusion))
- cpal's unreleased work "Migrated to Rust 2024". — [cpal CHANGELOG](https://github.com/RustAudio/cpal/blob/HEAD/CHANGELOG.md)

### Inferences
- **Edition and MSRV policy for SONARITHM:** use edition 2024 and set the core-crate MSRV to about 1.88. That matches the arrow, accesskit and rodio floor and leaves headroom (about 11 releases behind stable). Optional heavy crates (plugins/wasmtime, datafusion, burn) may declare higher MSRVs in their own crates. CI should test both the MSRV and stable, with `rust-toolchain.toml` pinned for reproducibility.
- **Audio-specific test strategy:**
  - insta snapshots of the compiled event timeline (text/JSON), which are deterministic and reviewable.
  - Golden-audio tests that compare offline renders with a tolerance (RMS diff, LUFS via ebur128), not byte equality, to absorb platform or SIMD float differences.
  - proptest on spec→timeline invariants (monotonic times, value-range mapping).
  - cargo-fuzz on the spec parser and data importers.
  - Miri plus shuttle (or loom) on any custom unsafe ring-buffer or arena code.
  - `assert_no_alloc` in callback tests.
- **Benchmarks:** criterion 0.8 for stable CI benchmarks. divan is ergonomic but hasn't released since 2025-04.
- **Supply chain:** use cargo-deny with an explicit licence allowlist (MIT, Apache-2.0, BSD, ISC, Unlicense, Zlib, MPL-2.0 file-level) that denies AGPL/GPL and flags LGPL. It would catch evalexpr, anomaly_detection, mp3lame-encoder and oxisynth automatically.

### Gaps
- Miri's current status (rustup component) was not separately verified; it ships with nightly.
- No 2026 survey of community MSRV norms was found; the norms above are inferred from declared `rust-version` fields.

---

## 13. Lessons from existing Rust audio projects (Meadowlark/Firewheel, Glicol, nih-plug, Bevy audio, sonification crates)

### Takeaway
The flagship Rust audio projects show recurring patterns. Projects narrow their scope: Firewheel is explicitly "not a DAW engine". Maintainers burn out or move: nih-plug is in maintenance mode with a community fork on Codeberg, Meadowlark moved to Codeberg, and Glicol has been idle since 2025-04. Real-time-safety soundness matters: the Bevy audit found UB in Firewheel. No real sonification library exists in Rust, so SONARITHM would be first in its niche.

### Cited Findings
- **nih-plug:** "NIH-plug the plugin framework is currently in maintenance mode. If you are interested in the framework rather than the plugin, please check out this community fork" (codeberg.org/BillyDM/nih-plug). It is ISC-licensed, but its VST3 bindings are GPLv3, so "VST3 plugins built with NIH-plug need to be able to comply with the terms of the GPLv3 license". It is not published on crates.io (`nih_plug` returns 404). Last commit 2026-09-03 ("Persist latest plugin builds in a release"). — [nih-plug README](https://github.com/robbert-vdh/nih-plug), [crates.io lookup](https://crates.io/crates/nih_plug)
- **Meadowlark:** a "(work in progress) FREE and open-source DAW", AGPL-3.0. "This project has moved to Codeberg"; the GitHub repo was last pushed 2025-09-22. — [Meadowlark GitHub](https://github.com/MeadowlarkDAW/Meadowlark)
- **Firewheel's scope:** "not meant to be a complete DAW … the needs of game audio engine and DAW audio engine are in conflict". The conflict is about state ownership: in a DAW the host may discard parameter events that conflict with transport state, while in apps "parameter update events are only allowed to come from a single source (the user), and are guaranteed to not be discarded". Firewheel's README also cites "more feature creep than what the author originally anticipated" as a reason for upstreaming to Bevy. — [Firewheel DESIGN_DOC](https://github.com/BillyDM/Firewheel/blob/HEAD/DESIGN_DOC.md), [Firewheel README](https://github.com/BillyDM/Firewheel)
- **Bevy audio:** still rodio-based in Bevy 0.20 (bevy_audio 0.20.0 depends on `rodio ^0.22`). The audit of firewheel and bevy_seedling recommends either a `bevy_firewheel` umbrella crate or a lean `bevy_audio` replacement, plus fixing UB, benchmarking, and recruiting maintainers. — [bevy_audio deps](https://crates.io/crates/bevy_audio/0.20.0/dependencies), [HackMD Bevy audit](https://hackmd.io/@bevy/rkwFscXObx)
- **Glicol:** 3,012★, but its last crate release was 2024-04-23 and its last repo push 2025-04-06. **knyst:** last release and push 2024-11-25. **oddio:** 2023-10-15. **synthizer-rs:** archived. — [glicol](https://github.com/chaosprint/glicol), [knyst](https://github.com/ErikNatanael/knyst), [oddio](https://github.com/Ralith/oddio), [synthizer crates.io](https://crates.io/crates/synthizer)
- **Are We Audio Yet:** the RustAudio/areweaudioyet repo was last pushed 2020-08-09, so it is stale as a catalog. — [RustAudio/areweaudioyet](https://github.com/RustAudio/areweaudioyet)
- **Sonification crates:** the closest match is `data2sound`, which encodes raw bytes into WAV (202.86 kHz, educational) rather than doing perceptual sonification. Its last release was 0.2.2 in March 2023. No perceptual data-sonification crate was found. — [docs.rs data2sound](https://docs.rs/data2sound), [lib.rs data2sound](https://www.lib.rs/crates/data2sound)

### Inferences
- **Architectural lessons for SONARITHM:**
  - Separate a "control/compile" world (spec → timeline, non-RT, allocation OK) from the RT render world. Firewheel's single-source-of-truth event model fits SONARITHM, which, like a game, is app-driven rather than transport-driven, but scrubbing introduces DAW-like transport semantics. Decide explicitly that the timeline is authoritative during playback and that UI edits trigger a recompile and atomic swap.
  - Keep the dependency surface narrow and well-maintained. Several once-promising crates (glicol, knyst, oddio, synthizer) went stale within one to two years.
  - Budget for unsafe-code review (Miri, audits) of any lock-free structures; the Bevy audit found UB even in a careful engine.
  - Avoid plugin formats with GPL-entangled bindings (VST3 via vst3-sys) unless the project licence permits it. CLAP is permissive.
- Because no Rust sonification library exists, SONARITHM's spec model and engine API could become ecosystem reference points. Publishing `sonarithm-core` early with semver checks has outsized value.

### Gaps
- Glicol's technical post-mortem or lessons (for example its AudioWorklet and SharedArrayBuffer design) were not fetched in this pass.
- No public Rust audio talks or RustAudio Discord summaries from 2025–2026 were located.

---

## 14. Synthesis: recommended stack and risks & mitigations

### Takeaway
A conservative, well-maintained stack: **cpal** for I/O, a **SONARITHM-owned backend-agnostic render graph** (trialling **Firewheel**) with **fundsp** nodes, **rtrb**, **triple_buffer** and **basedrop** for RT communication, **hound**, **flacenc** and **midly** for deterministic export, **symphonia** for decoding, **Arrow** as the data model, **rhai** and **exmex** for user logic, **PyO3/maturin** and **wasm-bindgen** for bindings, and **wasmtime + WIT** for third-party engines. Licence hygiene (AGPL, GPL, LGPL) and pre-1.0 churn are the main risks.

### Cited Findings
- The recommended stack table below is a synthesis. Every version and date comes from the crates.io and GitHub data cited in sections 1–13 (for example [cpal](https://crates.io/crates/cpal), [firewheel](https://crates.io/crates/firewheel), [fundsp](https://crates.io/crates/fundsp), [rtrb](https://crates.io/crates/rtrb), [wasmtime](https://crates.io/crates/wasmtime)).

### Inferences

**Recommended stack (as of 2026-10-18)**

| Layer | Choice (version) | Ring | Alternates / notes |
|---|---|---|---|
| Audio device I/O | cpal 0.18.2 (`realtime` feature; `asio` opt-in on Windows) | Adopt | wasapi 0.26 for exclusive mode later; web-audio-api 1.7 / cubeb 0.38 assess; interflow watch |
| Engine graph and scheduler | Own `sonarithm-engine` trait; Firewheel 0.14 as trial implementation | Trial | kira 0.12.5 assess; rodio 0.23 for simple CLI preview only; oddio, knyst, glicol avoid |
| Synthesis/DSP | fundsp 0.23 (seeded noise, PolyBLEP, FIR resample, convolve) + custom nodes (granular, wavetable, modal) | Adopt | biquad 0.6; rustysynth 1.3.7 for SF2 timbres |
| Resampling / FFT | rubato 5.0.1; rustfft 6.4.1; realfft 3.5.0 | Adopt | |
| Spatial / binaural | Stereo pan + ITD/ILD; sofar 0.4.0 (SOFA HRTF) | Trial | hrtf 0.9 assess; audionimbus 0.16 (Steam Audio 4.8.1) desktop-only assess |
| Loudness | ebur128 0.1.10 | Adopt | |
| RT comms | rtrb 0.4.0, triple_buffer 9.0.0, basedrop 0.1.3, atomic_float 1.1, no_denormals 0.3, assert_no_alloc 1.1.2 (debug) | Adopt (basedrop: Trial) | ringbuf 0.5.3 alternate; crossbeam-channel for non-RT only |
| Decode | symphonia 0.6.1 | Adopt | symphonia-adapter-libopus for Opus |
| Export | hound 3.5.1 (WAV), flacenc 0.5.1 (FLAC), midly 0.5.3 (SMF) | Adopt / Trial (flacenc) | vorbis_rs, opus 0.4, mp3lame-encoder (LGPL) as optional features |
| Live MIDI / OSC | midir 0.11.1; rosc 0.11.4; wmidi 4.0.11 | Adopt | |
| Speech / a11y | tts 0.26.3 (Trial; screen-reader routing via Tolk/NVDA); AccessKit 0.25.1 if native UI | Trial / Adopt | |
| Data model | arrow 60 + parquet 60; csv 1.4; calamine 0.36.1; jiff 0.2.38 (+ chrono 0.4.45 interop) | Adopt | polars 0.55.2 optional (Trial); datafusion 55.2 assess |
| Streaming sources | tokio 1.53; tokio-tungstenite 0.30; serialport 4.10.1 | Adopt | rumqttc 0.25.1 Trial; sqlx 0.9 Trial |
| Expressions / scripting | exmex 0.21 (expressions); rhai 1.26.1 (scripts) | Trial / Adopt | mlua assess; **evalexpr (AGPL), fasteval, meval avoid** |
| Analytics | statrs 0.19.1; ndarray 0.17.2; augurs 0.10.2 | Adopt / Trial | linfa 0.8.1, changepoint 0.15 assess; anomaly_detection (GPL) avoid |
| Chart digitization ML | tract-onnx 0.23.9 default; ort 2.0.0-rc.14 (`load-dynamic`) desktop opt-in | Trial | candle 0.11, burn 0.22 assess |
| Python | pyo3 0.30.1 + maturin 1.15 + numpy 0.30 (abi3 wheels) | Adopt | Arrow C Data Interface for zero-copy |
| WebAssembly | wasm-bindgen 0.2.129 (cpal WebAudio/AudioWorklet host) | Adopt | wasm-pack 0.15 assess |
| Other FFI | cbindgen 0.29.4 (C ABI); uniffi 0.32.2 (mobile later) | Adopt / Assess | |
| Plugins | wasmtime 49 + WIT (WASI 0.3), compile-time only | Trial | extism 1.41 assess; libloading/abi_stable avoid |
| Spec / schema / IPC | serde 1.0.229, schemars 1.2.3, ts-rs 12.0.1 | Adopt | specta/tauri-specta 2.0.0-rc.25 Trial |
| Errors / diagnostics | thiserror 2.0.21, miette 7.6.0, tracing 0.1.44 | Adopt | |
| Testing / QA | nextest 0.9.148, insta 1.49, proptest 1.12, criterion 0.8.2, cargo-fuzz 0.13.2, Miri, shuttle 0.9.7 | Adopt / Trial | divan 0.1.21, loom 0.7.2 assess |
| Supply chain | cargo-deny 0.20.2 (licence allowlist), cargo-audit 0.22.2, cargo-semver-checks 0.51, cargo-machete 0.9.2 | Adopt | cargo-vet 0.10.2 assess |
| Toolchain | Edition 2024; core MSRV ≈1.88; stable 1.99.1 in CI; `rust-toolchain.toml` | Adopt | Heavy optional crates may require ≥1.94–1.96 |

**Risks & mitigations**

1. **cpal API churn.** Breaking changes landed in 0.17→0.18, and 0.19 is planned with `play`→`start`, merged `CallbackInfo` and duplex changes (cpal CHANGELOG, #1220). *Mitigation:* hide cpal behind an internal `AudioBackend` trait, pin minor versions, and keep the upgrade surface small.
2. **No exclusive mode in cpal.** WASAPI and CoreAudio exclusive mode are still open (#106), so Windows shared-mode latency applies. *Mitigation:* offer an ASIO opt-in build (watch #1083 build-failure behaviour), consider a `wasapi`-crate exclusive backend later, and measure round-trip latency in-house.
3. **Firewheel pre-1.0 churn, reported UB, and Bevy upstreaming** that could reshape its API (HackMD audit; still 0.x, at 0.14.0). *Mitigation:* use it behind the engine trait, contribute fixes upstream, run Miri on the integration, and keep a fallback path (a minimal own graph on fundsp nodes).
4. **Stale but load-bearing crates.** hound (2023-09), midly (2023-01), dasp (2020-05), assert_no_alloc (2021-08), loom (2024-04), and tts (2024-07 release). *Mitigation:* these are small and stable. Pin them, keep the option to vendor or fork, and prefer tests that would catch regressions on upgrades.
5. **Licence contamination.** evalexpr is AGPL-3.0 (since 12.0.0), anomaly_detection is GPL-3.0, mp3lame-encoder is LGPL-3.0, oxisynth is LGPL-2.1, and nih-plug's VST3 bindings are GPLv3. MPL-2.0 applies file-level to symphonia, triple_buffer, uniffi, serialport and audio_thread_priority. *Mitigation:* enforce a cargo-deny licence allowlist in CI and keep copyleft codecs behind non-default features.
6. **Determinism of offline renders** across OS, CPU and SIMD paths (fundsp's `f32x8`, FTZ/DAZ settings). *Mitigation:* make the event timeline (not audio bytes) the canonical snapshot, compare audio with tolerances and LUFS, fix the block size and sample rate for export, seed all noise generators, and avoid wall-clock time in render paths.
7. **ML dependency weight and instability.** ort is still RC, rc.13 is CUDA-13-only, and the native ONNX Runtime binary is large. *Mitigation:* keep digitization in a separate optional crate, use tract by default (wasm-capable), use ort via `load-dynamic` on desktop, and lock the RC version.
8. **MSRV drift from heavy optional dependencies** (wasmtime 1.96, burn 1.95, datafusion and sqlx 1.94). *Mitigation:* use per-crate MSRVs, keep heavy dependencies in leaf crates behind features, and run a CI matrix on the core MSRV.
9. **Polars Rust API breakage without deprecation** (Polars versioning policy). *Mitigation:* make Arrow the internal contract and treat Polars as an optional adapter.
10. **Plugins on the RT path.** Wasm calls can allocate or trap. *Mitigation:* plugins run only at spec-compile time, or pre-render on worker threads into rings with look-ahead, with epoch and fuel limits.
11. **Specta perpetual RC and Tauri 3 alpha** (if the desktop app uses Tauri). *Mitigation:* generate TS with ts-rs plus the schemars JSON Schema, and adopt tauri-specta only behind a thin layer.
12. **Ecosystem fragility and maintainer moves to Codeberg** (Meadowlark, nih-plug fork, Firewheel mirror). *Mitigation:* track canonical repos, cite crates.io repository fields, and budget engineering time for upstream contributions.

### Gaps
- No hands-on latency, CPU or binary-size measurements were performed. All performance statements are qualitative.
- Granular, wavetable and modal synthesis and offline neural TTS remain unresolved crate-selection areas.
