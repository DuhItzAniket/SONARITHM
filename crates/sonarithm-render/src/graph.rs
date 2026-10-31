// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The render graph: voice pools per track, four buses, the master stage, a
//! sample-accurate scheduler and the transport (06 §3, §5).
//!
//! [`Graph::process`] is the one render path for real-time playback and offline export
//! (P4). It is real-time safe: no allocation, locks, I/O or panics, and every loop is
//! bounded by the block size, the voice pools, the track count or a fixed cap.
//!
//! **Score time and device time.** The transport maps score frames (the plan's frames) to
//! device frames: one device frame advances the score by `speed` frames. Speed changes only
//! the mapping, never the pitch (06 §5). The score position is computed from an anchor,
//! never accumulated, so it does not drift.
//!
//! **Sample-accurate dispatch.** Each block is split at every event onset inside it, and
//! each event starts at its exact sample. The output is therefore the same for any block
//! size.

use sonarithm_core::{Db, Frames, Hz, Pan, SampleRate, Seconds};
use sonarithm_dsp::env::{note_ramp, ramp_frames};
use sonarithm_dsp::smooth::SMOOTHING_TIME;
use sonarithm_dsp::{LinearSmoother, PanGains, TruePeakLimiter};

use crate::earcon::{EarconSound, splitmix};
use crate::meter::{MeterLevels, MeterWindow};
use crate::plan::{Bus, EventKind, Instrument, PlannedEvent, RenderPlan, Tone, TrackPlan};
use crate::voice::{Start, VoicePool};

#[cfg(test)]
mod tests;

/// Seek and loop crossfade: sounding voices fade out within 10 ms (06 §5).
pub const SEEK_FADE: Seconds = Seconds(0.010);
/// Pause, stop and "stop all audio" fade out over 20 ms (sound-design §5 proposal).
pub const PAUSE_FADE: Seconds = Seconds(0.020);
/// The slowest and fastest playback speeds.
pub const SPEED_RANGE: (f64, f64) = (0.25, 4.0);
/// Events examined per track when looking for notes still sounding at a seek target.
const MAX_RESUME_SCAN: usize = 256;
/// Voices for live cues (06 §3: "Live cue voices", routed to the data bus).
const CUE_VOICES: usize = 4;
/// Voices for live earcons, on the earcon bus (keyboard §11; B03-T06).
const EARCON_VOICES: usize = 4;

/// A transport or mixer command from the control thread (06 §4). Fixed size and `Copy`, so
/// it travels through a wait-free ring.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Command {
    /// Start or resume playback at the current position.
    Play,
    /// Pause at the current position, fading out sounding voices.
    Pause,
    /// Pause and return to the start.
    Stop,
    /// Move the playhead to a score frame, crossfading.
    Seek(Frames),
    /// Change the playback speed, limited to [`SPEED_RANGE`]; pitch is unchanged.
    SetSpeed(f64),
    /// Loop a score range, or stop looping with `None`.
    SetLoop(Option<LoopRange>),
    /// Set a bus or the master level, smoothed.
    SetGain(GainTarget, Db),
    /// Play a one-shot cue now (the point cue of sound-design §3.2).
    Cue(Cue),
    /// Play an earcon now, on the earcon bus: the interactive earcons (invalid action,
    /// mode and level changes, a missing value) and any other of the catalogue.
    Earcon(EarconSound),
    /// Fade out everything, including cues, and pause ("stop all audio", A11Y-15).
    StopAll,
}

/// A loop range in score frames, `start < end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopRange {
    /// The first frame of the loop.
    pub start: Frames,
    /// The frame at which playback wraps to `start`.
    pub end: Frames,
}

/// What a [`Command::SetGain`] changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GainTarget {
    /// One bus.
    Bus(Bus),
    /// The master: the sonification volume, independent of the system volume (A11Y-15).
    Master,
    /// The speech ducker before the master (06 §3): [`DUCK_LEVEL`] while speech plays over
    /// sonification, 0 dB otherwise (A11Y-20).
    Duck,
}

/// How far the ducker lowers sonification while self-voicing speaks over it: about 20 dB
/// below speech (A11Y-20, sound-design §6.2; 08 §4).
pub const DUCK_LEVEL: Db = Db(-20.0);

/// A one-shot cue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cue {
    /// Pitch.
    pub frequency: Hz,
    /// Level relative to the data bus.
    pub gain: Db,
    /// Stereo position.
    pub pan: Pan,
    /// Timbre.
    pub instrument: Instrument,
    /// Length including ramps (sound-design §3.2 proposes 120 ms).
    pub duration: Seconds,
}

/// The default level of each bus relative to the data bus (sound-design §6.2, proposals):
/// data 0 dB, context −10 dB, earcons −6 dB, monitoring −12 dB.
#[must_use]
pub fn default_bus_level(bus: Bus) -> Db {
    Db(match bus {
        Bus::Data => 0.0,
        Bus::Context => -10.0,
        Bus::Earcon => -6.0,
        Bus::Monitor => -12.0,
    })
}

/// Counters and levels published after each block.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct GraphStats {
    /// The score frame of the next frame to play.
    pub position: Frames,
    /// Whether the transport is playing.
    pub playing: bool,
    /// The playback speed.
    pub speed: f64,
    /// Events started since the graph was built.
    pub dispatched: u64,
    /// Voices stolen because a pool was full.
    pub stolen: u64,
    /// Voices sounding at the end of the block.
    pub active_voices: u32,
    /// Peak absolute sample of the last block, left and right.
    pub peak: [f32; 2],
    /// The master's level over the last complete meter window, for the UI's meters
    /// (B01-T04): unlike [`peak`](Self::peak), it misses no block between two reads.
    pub meter: MeterLevels,
    /// Frames rendered since the graph was built.
    pub rendered: u64,
}

/// The start-up ramp of the master: it starts 20 dB down (06 §10; NFR-10).
pub const STARTUP_LEVEL: Db = Db(-20.0);
/// The start-up ramp's length: 500 ms (06 §10).
pub const STARTUP_TIME: Seconds = Seconds(0.5);

/// Graph settings fixed at construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct GraphConfig {
    /// The largest block, in frames, processed at once; larger blocks are split.
    pub max_block: usize,
    /// Whether the master starts at [`STARTUP_LEVEL`] and rises to its level over
    /// [`STARTUP_TIME`], so a stream never opens at full scale (06 §10, NFR-10).
    pub startup_ramp: bool,
}

impl GraphConfig {
    /// Playback through a device: blocks of at most `max_block` frames (at least 1), with
    /// the start-up ramp.
    #[must_use]
    pub fn new(max_block: usize) -> Self {
        Self {
            max_block: max_block.max(1),
            startup_ramp: true,
        }
    }

    /// An offline render: as [`GraphConfig::new`] but without the start-up ramp, so a file
    /// starts exactly as the Score does.
    #[must_use]
    pub fn offline(max_block: usize) -> Self {
        Self {
            startup_ramp: false,
            ..Self::new(max_block)
        }
    }
}

/// A rise in level by a constant factor per frame, from [`STARTUP_LEVEL`] to unity over
/// [`STARTUP_TIME`]: linear in dB. Multiplication only, so it is deterministic.
#[derive(Clone, Copy, Debug)]
struct StartupRamp {
    gain: f32,
    factor: f32,
    remaining: u64,
}

impl StartupRamp {
    fn new(rate: SampleRate) -> Self {
        let frames = STARTUP_TIME.to_frames(rate).map_or(1, |f| f.0.max(1));
        #[expect(clippy::cast_possible_truncation, reason = "gains are f32")]
        let (gain, factor) = (
            STARTUP_LEVEL.to_gain() as f32,
            Db(-STARTUP_LEVEL.0 / frames as f64).to_gain() as f32,
        );
        Self {
            gain,
            factor,
            remaining: frames,
        }
    }

    #[inline]
    fn next_gain(&mut self) -> f32 {
        if self.remaining == 0 {
            return 1.0;
        }
        self.remaining -= 1;
        self.gain = if self.remaining == 0 {
            1.0
        } else {
            (self.gain * self.factor).min(1.0)
        };
        self.gain
    }
}

struct TrackState {
    /// Index of the next event to dispatch.
    next: usize,
    pool: VoicePool,
}

type Stereo = [Vec<f32>; 2];

/// The render graph for one plan.
pub struct Graph {
    plan: RenderPlan,
    rate: SampleRate,
    max_block: usize,
    tracks: Vec<TrackState>,
    cue_track: TrackPlan,
    cues: VoicePool,
    earcon_track: TrackPlan,
    earcons: VoicePool,
    buses: [Stereo; 4],
    master: Stereo,
    /// Per-voice work buffer, `max_block` long.
    scratch: Vec<f32>,
    bus_gains: [LinearSmoother; 4],
    master_gain: LinearSmoother,
    duck_gain: LinearSmoother,
    startup: Option<StartupRamp>,
    limiter: TruePeakLimiter,
    meter: MeterWindow,
    playing: bool,
    /// Score frame at the anchor.
    anchor: f64,
    /// Device frames since the anchor.
    since_anchor: u64,
    speed: f64,
    looping: Option<LoopRange>,
    age: u64,
    stats: GraphStats,
}

impl Graph {
    /// A stopped graph at the start of `plan`. Allocates every voice and buffer: call it
    /// off the audio thread.
    #[must_use]
    pub fn new(plan: RenderPlan, config: GraphConfig) -> Self {
        let rate = plan.sample_rate;
        let max_block = config.max_block;
        let stereo = || [vec![0.0; max_block], vec![0.0; max_block]];
        let tracks = plan
            .tracks
            .iter()
            .map(|track| TrackState {
                next: 0,
                pool: VoicePool::new(track.voices),
            })
            .collect();
        let level = |bus: Bus| {
            LinearSmoother::new(
                to_f32(default_bus_level(bus).to_gain()),
                SMOOTHING_TIME,
                rate,
            )
        };
        Self {
            cue_track: TrackPlan {
                id: sonarithm_core::TrackId::new("cue"),
                bus: Bus::Data,
                gain: 1.0,
                instrument: Instrument::default(),
                polyphony: CUE_VOICES,
                voices: CUE_VOICES,
                longest: 0,
                noise_seed: 0,
                events: Vec::new(),
                breakpoints: Vec::new(),
            buffers: Vec::new(),
            },
            cues: VoicePool::new(CUE_VOICES),
            earcon_track: TrackPlan {
                id: sonarithm_core::TrackId::new("earcon"),
                bus: Bus::Earcon,
                gain: 1.0,
                instrument: Instrument::default(),
                polyphony: EARCON_VOICES,
                voices: EARCON_VOICES,
                longest: 0,
                noise_seed: 0,
                events: Vec::new(),
                breakpoints: Vec::new(),
            buffers: Vec::new(),
            },
            earcons: VoicePool::new(EARCON_VOICES),
            buses: [stereo(), stereo(), stereo(), stereo()],
            master: stereo(),
            scratch: vec![0.0; max_block],
            bus_gains: Bus::ALL.map(level),
            master_gain: LinearSmoother::new(1.0, SMOOTHING_TIME, rate),
            duck_gain: LinearSmoother::new(1.0, SMOOTHING_TIME, rate),
            startup: config.startup_ramp.then(|| StartupRamp::new(rate)),
            limiter: TruePeakLimiter::new(rate),
            meter: MeterWindow::new(rate),
            plan,
            rate,
            max_block,
            tracks,
            playing: false,
            anchor: 0.0,
            since_anchor: 0,
            speed: 1.0,
            looping: None,
            age: 0,
            stats: GraphStats {
                speed: 1.0,
                ..GraphStats::default()
            },
        }
    }

    /// The plan being played.
    #[must_use]
    pub fn plan(&self) -> &RenderPlan {
        &self.plan
    }

    /// The delay the master limiter adds, in frames: an event at score frame `f` is heard
    /// at output frame `f + latency`.
    #[must_use]
    pub fn latency(&self) -> usize {
        self.limiter.latency()
    }

    /// Frames processed with the master limiter engaged, since construction. Sustained
    /// limiting breaks loudness constancy (SDR-23), so the lint warns about it.
    #[must_use]
    pub fn limited_frames(&self) -> u64 {
        self.limiter.engaged_frames()
    }

    /// The largest block processed in one piece, in frames.
    #[must_use]
    pub fn max_block(&self) -> usize {
        self.max_block
    }

    /// Counters and levels after the last block.
    #[must_use]
    pub fn stats(&self) -> GraphStats {
        self.stats
    }

    /// The score position in frames (fractional when the speed is not 1).
    fn position(&self) -> f64 {
        #[expect(clippy::cast_precision_loss, reason = "frame counts below 2^53")]
        let since = self.since_anchor as f64;
        self.anchor + since * self.speed
    }

    fn reanchor(&mut self, score_frame: f64) {
        self.anchor = score_frame;
        self.since_anchor = 0;
    }

    /// Takes over the transport and mixer state of `previous`, the graph this one replaces
    /// in a plan swap: position, speed, loop, play state and levels. If `previous` was
    /// playing, the events sounding at the position restart with a fade-in. RT-safe.
    pub fn adopt(&mut self, previous: &Self) {
        let position = previous.position().min(self.length());
        self.speed = previous.speed;
        self.looping = previous.looping;
        self.bus_gains = previous.bus_gains;
        self.master_gain = previous.master_gain;
        self.duck_gain = previous.duck_gain;
        self.age = previous.age;
        self.reanchor(position);
        self.playing = previous.playing;
        if self.playing {
            self.resume();
        }
    }

    /// Applies a command at the next frame. RT-safe.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Play => {
                if !self.playing {
                    self.playing = true;
                    self.resume();
                }
            }
            Command::Pause => self.halt(),
            Command::Stop => {
                self.halt();
                self.reanchor(0.0);
            }
            Command::StopAll => {
                self.halt();
                let frames = ramp_frames(PAUSE_FADE, self.rate);
                self.cues
                    .iter_mut()
                    .chain(self.earcons.iter_mut())
                    .for_each(|voice| voice.fade_out(frames));
            }
            Command::Seek(frame) => self.seek(frame.0),
            Command::SetSpeed(speed) => {
                if speed.is_finite() {
                    let position = self.position();
                    self.speed = speed.clamp(SPEED_RANGE.0, SPEED_RANGE.1);
                    self.reanchor(position);
                }
            }
            Command::SetLoop(range) => {
                self.looping = range.filter(|r| r.start < r.end);
            }
            Command::SetGain(target, level) => {
                let gain = to_f32(level.to_gain());
                match target {
                    GainTarget::Bus(bus) => {
                        if let Some(smoother) = self.bus_gains.get_mut(bus.index()) {
                            smoother.set_target(gain);
                        }
                    }
                    GainTarget::Master => self.master_gain.set_target(gain),
                    GainTarget::Duck => self.duck_gain.set_target(gain),
                }
            }
            Command::Cue(cue) => self.cue(&cue),
            Command::Earcon(sound) => self.earcon(sound),
        }
    }

    fn halt(&mut self) {
        let frames = ramp_frames(PAUSE_FADE, self.rate);
        self.fade_all(frames);
        self.playing = false;
    }

    fn fade_all(&mut self, frames: u32) {
        for track in &mut self.tracks {
            track
                .pool
                .iter_mut()
                .for_each(|voice| voice.fade_out(frames));
        }
    }

    fn seek(&mut self, frame: u64) {
        let frame = frame.min(self.plan.length.0);
        self.fade_all(ramp_frames(SEEK_FADE, self.rate));
        #[expect(clippy::cast_precision_loss, reason = "frame counts below 2^53")]
        self.reanchor(frame as f64);
        if self.playing {
            self.resume();
        }
    }

    /// Positions every track's cursor at the playhead and restarts the events still
    /// sounding there, part-way through, with a short fade-in.
    fn resume(&mut self) {
        let position = self.position();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the position is a non-negative frame count below 2^53"
        )]
        let target = position.ceil() as u64;
        let speed = self.speed;
        for (state, track) in self.tracks.iter_mut().zip(&self.plan.tracks) {
            state.next = track.events.partition_point(|event| event.start < target);
            let earliest = target.saturating_sub(track.longest);
            let candidates = track.events.get(..state.next).unwrap_or_default();
            for (index, event) in candidates.iter().enumerate().rev().take(MAX_RESUME_SCAN) {
                if event.start < earliest {
                    break;
                }
                if event.end() > target {
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        clippy::cast_precision_loss,
                        reason = "frame counts below 2^53; speed is positive"
                    )]
                    let offset = ((position - event.start as f64) / speed).max(1.0) as u64;
                    let (voice, stolen) = state.pool.allocate();
                    voice.start(
                        event,
                        track,
                        self.rate,
                        Start {
                            offset,
                            speed,
                            age: self.age,
                            noise_seed: track.noise_seed ^ splitmix(index as u64),
                        },
                    );
                    self.age += 1;
                    self.stats.dispatched += 1;
                    self.stats.stolen += u64::from(stolen);
                }
            }
        }
    }

    fn cue(&mut self, cue: &Cue) {
        let ramp = ramp_frames(note_ramp(cue.duration), self.rate);
        let length = cue
            .duration
            .to_frames(self.rate)
            .map_or(0, |f| f.0)
            .max(2 * u64::from(ramp));
        let event = PlannedEvent {
            start: 0,
            length,
            source: 0,
            kind: EventKind::Tone(Tone {
                frequency: cue.frequency,
                gain: to_f32(cue.gain.to_gain()),
                pan: PanGains::constant_power(cue.pan),
                brightness: 0.5,
                instrument: cue.instrument,
                ramp,
            }),
        };
        let (voice, _) = self.cues.allocate();
        voice.start(
            &event,
            &self.cue_track,
            self.rate,
            Start {
                offset: 0,
                speed: 1.0,
                age: self.age,
                noise_seed: 0,
            },
        );
        self.age += 1;
    }

    /// Starts a live earcon on the earcon bus. RT-safe: the voice comes from a fixed pool.
    fn earcon(&mut self, sound: EarconSound) {
        let length = sound
            .duration()
            .to_frames(self.rate)
            .map_or(1, |f| f.0.max(1));
        let event = PlannedEvent {
            start: 0,
            length,
            source: 0,
            kind: EventKind::Earcon {
                kind: sound,
                gain: 1.0,
            },
        };
        let (voice, _) = self.earcons.allocate();
        voice.start(
            &event,
            &self.earcon_track,
            self.rate,
            Start {
                offset: 0,
                speed: 1.0,
                age: self.age,
                noise_seed: self.age,
            },
        );
        self.age += 1;
    }

    /// Renders the next `out.len() / 2` frames into interleaved stereo `out`. Every event
    /// whose start falls in the block starts at its exact sample.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: no allocation, locks, I/O or panics. Blocks longer than the configured
    /// maximum are processed in pieces.
    pub fn process(&mut self, out: &mut [f32]) {
        let (frames, rest) = out.as_chunks_mut::<2>();
        rest.fill(0.0);
        let mut peak = [0.0_f32; 2];
        for chunk in frames.chunks_mut(self.max_block) {
            self.process_block(chunk, &mut peak);
        }
        let active: usize = self.tracks.iter().map(|t| t.pool.active()).sum::<usize>()
            + self.cues.active()
            + self.earcons.active();
        self.stats.peak = peak;
        self.stats.meter = self.meter.push(
            u64::try_from(frames.len()).unwrap_or(u64::MAX),
            peak,
            self.limiter.engaged_frames(),
        );
        self.stats.active_voices = u32::try_from(active).unwrap_or(u32::MAX);
        self.stats.playing = self.playing;
        self.stats.speed = self.speed;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the position is a non-negative frame count below 2^53"
        )]
        let position = self.position().floor() as u64;
        self.stats.position = Frames(position);
    }

    fn process_block(&mut self, out: &mut [[f32; 2]], peak: &mut [f32; 2]) {
        let frames = out.len();
        for [left, right] in &mut self.buses {
            left.iter_mut().take(frames).for_each(|s| *s = 0.0);
            right.iter_mut().take(frames).for_each(|s| *s = 0.0);
        }
        let mut seg = 0;
        while seg < frames {
            if self.playing {
                self.wrap_loop();
                self.dispatch();
            }
            let end = seg
                .saturating_add(self.frames_to_boundary())
                .min(frames)
                .max(seg + 1);
            self.render_voices(seg, end);
            if self.playing {
                self.since_anchor += (end - seg) as u64;
                if self.looping.is_none() && self.position() >= self.length() {
                    self.playing = false;
                }
            }
            seg = end;
        }
        self.mix(out, peak);
        self.stats.rendered += frames as u64;
    }

    #[expect(clippy::cast_precision_loss, reason = "frame counts below 2^53")]
    fn length(&self) -> f64 {
        self.plan.length.0 as f64
    }

    fn wrap_loop(&mut self) {
        if let Some(range) = self.looping {
            #[expect(clippy::cast_precision_loss, reason = "frame counts below 2^53")]
            if self.position() >= range.end.0 as f64 {
                self.seek(range.start.0);
            }
        }
    }

    /// Starts every event due at the current position. Bounded: each event is started once.
    fn dispatch(&mut self) {
        let position = self.position();
        for (state, track) in self.tracks.iter_mut().zip(&self.plan.tracks) {
            while let Some(event) = track.events.get(state.next) {
                #[expect(clippy::cast_precision_loss, reason = "frame counts below 2^53")]
                if event.start as f64 > position {
                    break;
                }
                let (voice, stolen) = state.pool.allocate();
                voice.start(
                    event,
                    track,
                    self.rate,
                    Start {
                        offset: 0,
                        speed: self.speed,
                        age: self.age,
                        noise_seed: track.noise_seed ^ splitmix(state.next as u64),
                    },
                );
                self.age += 1;
                self.stats.dispatched += 1;
                self.stats.stolen += u64::from(stolen);
                state.next += 1;
            }
        }
    }

    /// Device frames until the next event onset or loop end (at least 1), when playing.
    fn frames_to_boundary(&self) -> usize {
        if !self.playing {
            return usize::MAX;
        }
        let position = self.position();
        let until = |frame: f64| -> usize {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a non-negative, finite frame distance, saturating"
            )]
            let frames = ((frame - position) / self.speed).ceil().max(1.0) as usize;
            frames
        };
        let mut nearest = usize::MAX;
        for (state, track) in self.tracks.iter().zip(&self.plan.tracks) {
            if let Some(event) = track.events.get(state.next) {
                #[expect(clippy::cast_precision_loss, reason = "frame counts below 2^53")]
                let start = event.start as f64;
                nearest = nearest.min(until(start));
            }
        }
        if let Some(range) = self.looping {
            #[expect(clippy::cast_precision_loss, reason = "frame counts below 2^53")]
            let end = range.end.0 as f64;
            nearest = nearest.min(until(end));
        }
        nearest
    }

    fn render_voices(&mut self, from: usize, to: usize) {
        for (state, track) in self.tracks.iter_mut().zip(&self.plan.tracks) {
            let Some([left, right]) = self.buses.get_mut(track.bus.index()) else {
                continue;
            };
            let (Some(left), Some(right)) = (left.get_mut(from..to), right.get_mut(from..to))
            else {
                continue;
            };
            for voice in state.pool.iter_mut().filter(|voice| voice.is_active()) {
                voice.render(left, right, track, self.rate, &mut self.scratch);
            }
        }
        // Live cues play on the data bus (06 §3), live earcons on the earcon bus.
        let [[left, right], _, [earcon_left, earcon_right], _] = &mut self.buses;
        if let (Some(left), Some(right)) = (left.get_mut(from..to), right.get_mut(from..to)) {
            for voice in self.cues.iter_mut().filter(|voice| voice.is_active()) {
                voice.render(left, right, &self.cue_track, self.rate, &mut self.scratch);
            }
        }
        if let (Some(left), Some(right)) = (
            earcon_left.get_mut(from..to),
            earcon_right.get_mut(from..to),
        ) {
            for voice in self.earcons.iter_mut().filter(|voice| voice.is_active()) {
                voice.render(
                    left,
                    right,
                    &self.earcon_track,
                    self.rate,
                    &mut self.scratch,
                );
            }
        }
    }

    fn mix(&mut self, out: &mut [[f32; 2]], peak: &mut [f32; 2]) {
        let [master_left, master_right] = &mut self.master;
        for (left, right) in master_left.iter_mut().zip(master_right.iter_mut()) {
            *left = 0.0;
            *right = 0.0;
        }
        for ([left, right], gain) in self.buses.iter().zip(&mut self.bus_gains) {
            let bus = left.iter().zip(right.iter()).take(out.len());
            for ((ml, mr), (l, r)) in master_left.iter_mut().zip(master_right.iter_mut()).zip(bus) {
                let g = gain.next_value();
                *ml += l * g;
                *mr += r * g;
            }
        }
        let frames = out.len();
        let (Some(left), Some(right)) = (
            master_left.get_mut(..frames),
            master_right.get_mut(..frames),
        ) else {
            return;
        };
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut g = self.master_gain.next_value() * self.duck_gain.next_value();
            if let Some(ramp) = &mut self.startup {
                g *= ramp.next_gain();
            }
            *l *= g;
            *r *= g;
        }
        // Hearing safety: a true-peak limiter at −1 dBTP on the master (06 §10).
        self.limiter.process(left, right);
        for (frame, (l, r)) in out.iter_mut().zip(left.iter().zip(right.iter())) {
            *frame = [*l, *r];
            peak[0] = peak[0].max(l.abs());
            peak[1] = peak[1].max(r.abs());
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "gains are f32 (coding-standards §10)"
)]
fn to_f32(x: f64) -> f32 {
    x as f32
}
