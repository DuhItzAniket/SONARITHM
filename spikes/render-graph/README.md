# Spike A02-T02/T03: render graph behind a draft `RenderGraph` contract

> **Status:** Throwaway spike for the A02 prototype gate. Deleted or archived after the gate. Not production code.
> **Tasks:** A02-T02 (owned graph), A02-T03 (Firewheel 0.14 behind the same contract), A02-T04 (determinism) in [implementation-plan §2](../../docs/planning/implementation-plan.md). **Decision under test:** [ADR-0004](../../docs/decisions/0004-audio-backend-and-render-graph.md). **Design:** [06 §2–§6](../../docs/architecture/06-audio-engine.md).

## What is here

| File | Content |
|---|---|
| `src/graph.rs` | The draft contract: `RenderGraph` (control half, non-real-time) and `GraphProcessor` (audio thread) |
| `src/score.rs` | The benchmark Score: 4 tracks, 10,000 notes over 60 s at 48 kHz (the DS-006 shape), from SplitMix64 seed 1729 |
| `src/voice.rs` | DSP shared by every graph: voice (sine, triangle, PolyBLEP square; ramps per SDR-22; constant-power pan), oldest-first stealing, master peak limiter |
| `src/own.rs` | The owned graph: a voice pool per track, four fixed buses, master limiter, a scheduler that splits each block at event onsets |
| `src/fw.rs` | Firewheel 0.14 behind the same contract: a synth node per track, a gain node per bus, a master node, events scheduled through `FirewheelContext` |
| `src/main.rs` | The bench: `offline`, `realtime`, `null` (compares two WAV files), `selftest-alloc` |

### The draft contract

```rust
pub trait GraphProcessor: Send {
    fn process(&mut self, out: &mut [f32]); // audio thread: no allocation, locks or panics
}

pub trait RenderGraph: Send {
    fn prepare(&mut self, score: &Score, cfg: &RenderConfig) -> Result<Box<dyn GraphProcessor>, String>;
    fn update(&mut self);           // control-thread tick: feed events, apply commands
    fn seek(&mut self, frame: u64);
    fn counters(&self) -> &Arc<Counters>; // atomics the processor publishes
    // …
}
```

The contract is split in two because a graph engine with its own control context cannot run that context on the audio thread. Firewheel's `FirewheelContext::update` is not real-time safe; its `FirewheelProcessor` is. The split matches the threading model of 06 §2: control thread, then a lock-free hand-off, then the RT graph. The owned graph needs almost no control half: it receives its whole schedule at `prepare`, and a seek is one atomic store.

## Run it

```bash
cargo run --release -- offline --graph own|firewheel --block 256 --seconds 60 [--math std] [--wav out.wav]
cargo run --release -- realtime --graph own|firewheel --block 256 --seconds 120 [--audible]
cargo run --release -- null a.wav b.wav
cargo run -- selftest-alloc
cargo test --release
cargo +nightly miri test
```

`offline` reports the real-time factor (audio seconds per second of `process` time), CPU time per block, loudness and the SHA-256 of the samples and of the Score. Debug builds wrap every `process` call in `assert_no_alloc`, so an allocation on the render path aborts. `selftest-alloc` shows that the checker is live. `realtime` plays the Score through the default output and records CPU time per callback. The output is muted unless `--audible` is given; the graph still renders every block.

## Results: owned graph (Windows, 2026-10-19)

Machine: the owner's laptop (i7-14650HX, 16 GB, Windows 11 build 26300, on AC power, Balanced plan). Release build, Rust 1.99.0.

### Offline (`offline --graph own`, 60 s benchmark Score, 3 runs per block size)

| Block (frames) | Real-time factor (median of 3) | CPU per block p50 | p99 | max (worst run) |
|---|---|---|---|---|
| 128 | 349× | 7.3 µs | 14.2 µs | 237.8 µs |
| 256 | 361× | 14.4 µs | 24.0 µs | 304.0 µs |
| 512 | 352× | 29.7 µs | 48.3 µs | 259.5 µs |

The real-time factor counts `process` time only (single thread, 4 tracks, 10,000 events), the PB-03 shape. That is about 7× the A05 budget of ≥ 50×, so the headroom rule of 12 §8 is met without a CPU-limited rerun. Voices: 64 preallocated (polyphony per track plus 4); 28 at most sounding; 0 stolen; 10,000 dispatched. The render is −18.50 LUFS integrated with a 0.473 sample peak. SHA-256 of the samples: `962fbce201ce76350256bfda35ad6ca48f00d5283586d36f5df15f01cd8088ce`. SHA-256 of the Score bytes: `f21de7dba42b7a2fdf980cf6018866793b1d37079b69beea9e7f17e62f15e5ca`, compared across operating systems in A02-T04.

### Real time (`realtime --graph own`, 120 s, the Score looped by seeking to 0)

| CPU | Callbacks | Period | CPU per callback p50 | p99 | max | Load at p99 | Deadline misses | Late callbacks |
|---|---|---|---|---|---|---|---|---|
| All cores | 12,001 | 480 frames (requested 256) | 31.5 µs | 85.7 µs | 246.5 µs | 0.86% | 0 | 0 |
| 4 E-cores (`/affinity F0000`) | 11,991 | 480 frames | 55.0 µs | 151.5 µs | 345.6 µs | 1.52% | 0 | 7 |

A deadline miss is a callback whose processing took longer than its period. A late callback starts more than 1.5 periods after the previous one. cpal 0.18 does not report WASAPI output underruns, so these two counters stand in for xruns. The 7 late callbacks on the E-cores come from scheduling: processing never came close to the period. The 4 E-core run is a conservative stand-in for the 4-core reference laptop of 12 §8. A 10-minute soak at 128 frames (PB-02) belongs to A05, and on this driver it would still run at 480-frame periods (A02-T01).

### Checks

- `cargo test --release`: 3 tests pass. The output is bit-identical at block sizes 64, 256 and 1000, which shows sample-accurate dispatch. A seek converges to the full render once earlier notes end. A seek to 0 replays all 10,000 events.
- A debug build runs the whole 10 s offline render inside `assert_no_alloc` without aborting; `selftest-alloc` aborts as intended (exit 127). The A05 rule "no allocation on the RT thread" therefore holds for `process`.
- Miri: see the comparison below.

## A02-T03: Firewheel 0.14 behind the same contract

`src/fw.rs` builds the benchmark as a Firewheel graph. Each track is a custom node holding the same voice pool. Bus gains and the master limiter are custom nodes (same code). Notes are `CustomBytes` events scheduled at exact sample times through `FirewheelContext::schedule_event_for`, 200 ms ahead of the playhead. The bench drives `FirewheelProcessor::process` itself, from the offline loop or a cpal callback, so no Firewheel backend is involved. Firewheel features: `std`, `scheduled_events`; default features (cpal backend, sampler, Symphonium) are off.

### Comparison (Windows, 2026-10-19, same machine and builds as above)

| Criterion | Owned graph | Firewheel 0.14.0 |
|---|---|---|
| Output | Reference | **Bit-identical** to the owned graph: null residual −∞ over 5,760,000 samples (`null`), also a unit test |
| Offline real-time factor, 128 / 256 / 512 frames (median of 3) | 349× / 361× / 352× | 331× / 337× / 332× |
| CPU per 256-frame block, p50 / p99 | 14.4 / 24.0 µs | 15.5 / 25.4 µs |
| Control-thread work over the 60 s render | 0.3 ms | 1.8 ms (scheduling events and `update`) |
| Real time, 120 s, all cores: p99 per callback, deadline misses, late callbacks | 85.7 µs, 0, 0 | 85.4 µs, 0, 0 |
| Real time, 120 s, 4 E-cores | 151.5 µs, 0, 7 | 124.0 µs, 0, 12 |
| Allocation in `process` (debug, `assert_no_alloc`) | none | none, after setting `BufferOutOfSpaceMode::Panic`; the default mode, `AllocateOnAudioThread`, allocates on the audio thread when an event buffer fills |
| Code we write for this benchmark (non-blank, non-comment lines) | 190 (graph and scheduler; tests excluded) | 240 (node boilerplate, event encoding, feeding, seek re-anchoring) |
| Third-party code added | none beyond `libm` | 43 crates, including `bevy_platform`, `ringbuf`, `thunderdome`, `triple_buffer` and two `toml_edit` versions. Firewheel itself is 28,100 lines with 81 `unsafe` occurrences |
| Licences (`cargo deny check licenses`, repository policy) | pass | pass (MIT OR Apache-2.0 throughout) |
| Advisories (`cargo deny check advisories`) | ok | ok |
| Miri (see below) | Block-size test passes; other tests still running | The Firewheel test passes with Miri's own allocator; a Stacked Borrows report appears only with the `assert_no_alloc` allocator, inside std's Windows `System::dealloc` |
| Seek | One atomic store; the processor binary-searches the schedule | Cancel scheduled events, send an all-notes-off event, re-anchor the Score on Firewheel's clock (which only moves forward), then refill |

### Ergonomics notes

- **The threading model matches ours.** `FirewheelContext` (control) and `FirewheelProcessor` (audio thread) are the split of 06 §2. The context is `!Send`: it must live on the thread that created it. This is why the draft contract's control half has no `Send` bound.
- **Firewheel brings graph compilation, mixing and sample-accurate event scheduling.** Events land at their exact sample: Firewheel splits the block into sub-chunks at event times, which the bit-identical output shows. It brings no synthesis voices ("built-in synth instruments" are a stated non-goal), so the voice pool, envelopes and stealing are ours either way.
- **The real-time defaults need care.** `buffer_out_of_space_mode` defaults to allocating on the audio thread, and `scheduled_event_capacity` (default 512) caps how far ahead the control thread can schedule.
- **A note costs one event.** `CustomBytes([u8; 36])` carries a note without heap allocation; larger payloads need `Custom(OwnedGc<Box<dyn Any>>)`.
- **Pre-1.0 churn.** 0.14 shipped in September 2026; upstreaming into Bevy may reshape the API (notes/04 §1).

### Miri

`cargo +nightly miri test` (nightly 1.101.0, 2026-10-17, target `x86_64-pc-windows-msvc`). Under `cfg(miri)` the tests use 0.1 s of the benchmark Score, because a 0.1 s render already takes minutes under Miri; the Firewheel test compares Firewheel against the owned graph on that Score. Results so far:

| Run | Allocator | Result |
|---|---|---|
| Default | `assert_no_alloc::AllocDisabler` wrapping `System` | `output_is_independent_of_block_size` passes. `firewheel_matches_owned_graph` stops with a Stacked Borrows error in std's `sys::alloc::windows::dealloc` (it reads the header stored before an over-aligned allocation). It happens while `FirewheelContext` drops a `ringbuf` buffer, and it is reached through the custom global allocator |
| `RUSTFLAGS="--cfg spike_miri_system_alloc"` | Miri's built-in allocator (no custom global allocator) | `firewheel_matches_owned_graph` **passes**; the owned-graph tests are still running |

So the one report is not attributable to Firewheel's code or to ours: it needs a custom global allocator that forwards to `System` on Windows. A rerun under Tree Borrows classifies it; until then it is recorded as `TODO(verify)` in the gate report. Firewheel's own `unsafe` (81 occurrences) is exercised only as far as this benchmark reaches it (graph compile, scheduling, processing, drop); the audit's "nontrivial UB" (notes/04 §1) did not show up in this path.
