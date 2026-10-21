// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The transport: one control thread owns A05's [`Player`] and the authoritative transport
//! state (09 §3, ADR-0017 §1).
//!
//! The thread ticks the player every 10 ms, follows the playhead, stops at the end of the
//! Score, and turns device changes and failures into events. The audio device opens on
//! the first `play` (09 §7) and nothing ever starts sounding without a play command
//! (A11Y-15): a new Score, a device change or a device failure leaves the transport
//! paused.
//!
//! The same thread sequences speech with sonification (08 §4, B03-T07): it runs the
//! [`Sequencer`] every tick, pauses or ducks playback for speech, and hands each
//! announcement to the host's announcer or to the self-voicing speaker.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use sonarithm_audaxis::announce::{Announcement, Delivery};
use sonarithm_audio::{
    AudioBackend, CpalBackend, DeviceInfo, NullBackend, NullConfig, Player, PlayerEvent,
    StreamConfig,
};
use sonarithm_core::{Db, Frames, SampleRate, Score, Seconds};
use sonarithm_render::graph::SPEED_RANGE;
use sonarithm_render::{Command, DUCK_LEVEL, GainTarget, LoopRange, PlanConfig};
use sonarithm_speech::Speaker;

use crate::codes;
use crate::events::EventSink;
use crate::ipc::{ClockAnchor, Event, IpcError, LoopSpan, PlaybackStatus, TransportState};
use crate::speech::{Output, Sequencer, SequencerConfig, SpeakerSource, SpeechRoute};
use crate::streams::{LiveStreams, StreamAction};

/// How often the control thread services the player.
const TICK: Duration = Duration::from_millis(10);

/// The period of clock anchors while playing (06 §9).
const ANCHOR_PERIOD: Duration = Duration::from_secs(2);

/// How long a request waits for the control thread. Opening a device can take a while.
const REPLY_LIMIT: Duration = Duration::from_secs(10);

/// The render graph's block size for playback, in frames.
const BLOCK: usize = 256;

/// Which audio output the app plays to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AudioOutput {
    /// The system's devices, through cpal (ADR-0004).
    System,
    /// A silent, timer-driven device (A05's null backend), for tests and CI. Every test of
    /// the app core uses it (hearing safety).
    Null(NullConfig),
}

impl AudioOutput {
    /// The silent device at 48 kHz in blocks of 256 frames, paced like real time.
    #[must_use]
    pub fn null() -> Self {
        Self::Null(NullConfig::new(48_000, BLOCK))
    }
}

/// Output settings the transport applies.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct OutputConfig {
    pub(crate) device: Option<String>,
    pub(crate) buffer_frames: Option<u32>,
    pub(crate) volume_db: f64,
    pub(crate) speech: SequencerConfig,
}

/// A transport command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Action {
    State,
    Play,
    Pause,
    Stop,
    Seek(f64),
    Speed(f64),
    Loop(Option<LoopSpan>),
    StopAll,
}

type Reply<T> = Sender<Result<T, IpcError>>;

enum Message {
    Load(Option<Arc<Score>>),
    Act(Action, Reply<TransportState>),
    Configure(OutputConfig, Reply<TransportState>),
    Devices(Reply<Vec<DeviceInfo>>),
    Announce(Box<Announcement>),
    Repeat(Reply<bool>),
    StreamOpen(
        Box<(
            String,
            sonarithm_stream::Endpoint,
            sonarithm_stream::Connection,
        )>,
    ),
    StreamAct(String, StreamAction, Reply<Vec<crate::ipc::StreamInfo>>),
    StreamList(Reply<Vec<crate::ipc::StreamInfo>>),
    Unplug,
    Shutdown,
}

/// The handle the app keeps; the thread stops when it is dropped.
pub(crate) struct Transport {
    sender: Sender<Message>,
    thread: Option<JoinHandle<()>>,
}

impl Transport {
    /// Starts the control thread. No device is opened yet.
    pub(crate) fn spawn(
        output: AudioOutput,
        config: OutputConfig,
        sink: Arc<dyn EventSink>,
        speaker: Option<SpeakerSource>,
    ) -> Result<Self, IpcError> {
        let (sender, receiver) = mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("sonarithm-transport".to_owned())
            .spawn(move || match output {
                AudioOutput::System => {
                    Control::new(CpalBackend::new, |_| {}, config, sink, speaker).run(&receiver);
                }
                AudioOutput::Null(null) => Control::new(
                    move || NullBackend::new(null),
                    unplug_null,
                    config,
                    sink,
                    speaker,
                )
                .run(&receiver),
            });
        match spawned {
            Ok(thread) => Ok(Self {
                sender,
                thread: Some(thread),
            }),
            Err(err) => Err(IpcError::new(
                &codes::CONTROL_THREAD,
                format!("The playback control thread could not start: {err}."),
            )),
        }
    }

    pub(crate) fn act(&self, action: Action) -> Result<TransportState, IpcError> {
        self.ask(|reply| Message::Act(action, reply))
    }

    pub(crate) fn configure(&self, config: OutputConfig) -> Result<TransportState, IpcError> {
        self.ask(|reply| Message::Configure(config, reply))
    }

    pub(crate) fn devices(&self) -> Result<Vec<DeviceInfo>, IpcError> {
        self.ask(Message::Devices)
    }

    /// Queues an announcement for the sequencer.
    pub(crate) fn announce(&self, announcement: Announcement) {
        let _ = self.sender.send(Message::Announce(Box::new(announcement)));
    }

    /// Queues the last announcement again; `false` when there is none.
    pub(crate) fn repeat(&self) -> Result<bool, IpcError> {
        self.ask(Message::Repeat)
    }

    /// Hands an approved, open stream to the control thread.
    pub(crate) fn stream_open(
        &self,
        id: String,
        endpoint: sonarithm_stream::Endpoint,
        connection: sonarithm_stream::Connection,
    ) {
        let _ = self
            .sender
            .send(Message::StreamOpen(Box::new((id, endpoint, connection))));
    }

    /// Pauses, resumes, hides or stops a stream; the streams afterwards.
    pub(crate) fn stream_act(
        &self,
        id: String,
        action: StreamAction,
    ) -> Result<Vec<crate::ipc::StreamInfo>, IpcError> {
        self.ask(|reply| Message::StreamAct(id, action, reply))
    }

    /// The open streams.
    pub(crate) fn stream_list(&self) -> Result<Vec<crate::ipc::StreamInfo>, IpcError> {
        self.ask(Message::StreamList)
    }

    /// Loads a new Score (or none). Playback pauses; it never continues into the new
    /// Score on its own (A11Y-15).
    pub(crate) fn load(&self, score: Option<Arc<Score>>) {
        let _ = self.sender.send(Message::Load(score));
    }

    /// Makes the null device disappear, as an unplugged headset does (tests).
    pub(crate) fn unplug(&self) {
        let _ = self.sender.send(Message::Unplug);
    }

    fn ask<T>(&self, message: impl FnOnce(Reply<T>) -> Message) -> Result<T, IpcError> {
        let (reply, answer) = mpsc::channel();
        let stopped = || {
            IpcError::new(
                &codes::CONTROL_THREAD,
                "The playback control thread did not answer.",
            )
        };
        self.sender.send(message(reply)).map_err(|_| stopped())?;
        answer.recv_timeout(REPLY_LIMIT).map_err(|_| stopped())?
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        let _ = self.sender.send(Message::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn unplug_null(backend: &mut NullBackend) {
    if let Some(control) = backend.control() {
        control.unplug();
    }
}

/// The transport as Rust holds it.
#[derive(Clone, Debug, Default)]
struct Model {
    status: PlaybackStatus,
    position_s: f64,
    speed: f64,
    loop_range: Option<LoopSpan>,
    device: Option<String>,
}

struct Control<B: AudioBackend, F: Fn() -> B> {
    make: F,
    unplug: fn(&mut B),
    config: OutputConfig,
    sink: Arc<dyn EventSink>,
    score: Option<Arc<Score>>,
    player: Option<Player<B>>,
    /// The open stream's sample rate.
    rate: SampleRate,
    /// Engine blocks counted when the last seek was sent: the engine's position is stale
    /// until two more blocks have run.
    stale_until: u64,
    model: Model,
    seq: u64,
    last_anchor: Option<Instant>,
    sequencer: Sequencer,
    streams: LiveStreams,
    speaker_source: Option<SpeakerSource>,
    /// The self-voicing speaker, made on first use; `Err` once making it failed.
    speaker: Option<Result<Box<dyn Speaker>, ()>>,
    /// The sequencer's clock.
    epoch: Instant,
}

impl<B: AudioBackend, F: Fn() -> B> Control<B, F> {
    fn new(
        make: F,
        unplug: fn(&mut B),
        config: OutputConfig,
        sink: Arc<dyn EventSink>,
        speaker_source: Option<SpeakerSource>,
    ) -> Self {
        Self {
            sequencer: Sequencer::new(config.speech),
            streams: LiveStreams::new(),
            speaker_source,
            speaker: None,
            epoch: Instant::now(),
            make,
            unplug,
            config,
            sink,
            score: None,
            player: None,
            rate: SampleRate::HZ_48000,
            stale_until: 0,
            model: Model {
                speed: 1.0,
                ..Model::default()
            },
            seq: 0,
            last_anchor: None,
        }
    }

    fn run(mut self, receiver: &Receiver<Message>) {
        loop {
            match receiver.recv_timeout(TICK) {
                Ok(Message::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
                Ok(message) => self.handle(message),
                Err(RecvTimeoutError::Timeout) => {}
            }
            self.service();
            self.speak();
        }
        // Dropping the player closes the audio stream; live streams stop too.
        self.player = None;
        self.streams.stop_all();
    }

    fn duration(&self) -> f64 {
        self.score
            .as_ref()
            .map_or(0.0, |score| score.header.duration.0)
    }

    fn state(&self) -> TransportState {
        TransportState {
            status: self.model.status,
            position_s: self.model.position_s,
            duration_s: self.duration(),
            speed: self.model.speed,
            loop_range: self.model.loop_range,
            loaded: self.score.is_some(),
            device: self.model.device.clone(),
        }
    }

    fn handle(&mut self, message: Message) {
        match message {
            Message::Load(score) => self.load(score),
            Message::Act(action, reply) => {
                let result = self.act(action);
                if action != Action::State && result.is_ok() {
                    self.anchor();
                }
                let _ = reply.send(result);
            }
            Message::Configure(config, reply) => {
                self.configure(config);
                let _ = reply.send(Ok(self.state()));
            }
            Message::Devices(reply) => {
                let devices = (self.make)().devices().map_err(|err| {
                    IpcError::new(
                        &codes::AUDIO_UNAVAILABLE,
                        format!("The output devices could not be listed: {err}."),
                    )
                });
                let _ = reply.send(devices);
            }
            Message::Announce(announcement) => {
                let now = self.now_ms();
                self.sequencer.push(*announcement, now);
            }
            Message::Repeat(reply) => {
                let now = self.now_ms();
                let _ = reply.send(Ok(self.sequencer.repeat(now)));
            }
            Message::StreamOpen(open) => {
                let (id, endpoint, connection) = *open;
                self.streams.open(id, &endpoint, connection);
            }
            Message::StreamAct(id, action, reply) => {
                let result = self.streams.act(&id, action).map(|()| self.streams.list());
                let _ = reply.send(result);
            }
            Message::StreamList(reply) => {
                let _ = reply.send(Ok(self.streams.list()));
            }
            Message::Unplug => {
                if let Some(player) = &mut self.player {
                    (self.unplug)(player.backend_mut());
                }
            }
            Message::Shutdown => {}
        }
    }

    /// Drops the player (closing the device) and pauses a playing transport.
    fn close(&mut self) {
        self.refresh();
        self.player = None;
        if self.model.status == PlaybackStatus::Playing {
            self.model.status = PlaybackStatus::Paused;
        }
    }

    fn load(&mut self, score: Option<Arc<Score>>) {
        self.close();
        self.score = score;
        let duration = self.duration();
        if self.score.is_none() {
            self.model.status = PlaybackStatus::Stopped;
            self.model.position_s = 0.0;
        }
        self.model.position_s = self.model.position_s.min(duration);
        if self
            .model
            .loop_range
            .is_some_and(|range| range.end_s > duration)
        {
            self.model.loop_range = None;
        }
        self.anchor();
    }

    fn configure(&mut self, config: OutputConfig) {
        let reopen = config.device != self.config.device
            || config.buffer_frames != self.config.buffer_frames;
        let volume = config.volume_db;
        let volume_changed = (volume - self.config.volume_db).abs() > f64::EPSILON;
        self.sequencer.set_config(config.speech);
        if config.speech.route != SpeechRoute::SelfVoicing
            && let Some(Ok(speaker)) = &mut self.speaker
        {
            let _ = speaker.stop();
        }
        self.config = config;
        if reopen && self.player.is_some() {
            // A changed device pauses playback (sound-design §12 HS-05).
            self.close();
            self.model.device = None;
            self.anchor();
        } else if volume_changed && let Some(player) = &mut self.player {
            player.send(Command::SetGain(GainTarget::Master, Db(volume)));
        }
    }

    fn act(&mut self, action: Action) -> Result<TransportState, IpcError> {
        match action {
            Action::State => self.refresh(),
            Action::Play => self.play()?,
            Action::Pause => {
                self.refresh();
                self.send(Command::Pause);
                if self.model.status == PlaybackStatus::Playing {
                    self.model.status = PlaybackStatus::Paused;
                }
            }
            Action::Stop => {
                self.send(Command::Stop);
                self.model.status = PlaybackStatus::Stopped;
                self.model.position_s = 0.0;
                self.mark_stale();
            }
            Action::StopAll => {
                self.refresh();
                self.send(Command::StopAll);
                // Stop all audio stops speech too (A11Y-15).
                for output in self.sequencer.silence() {
                    self.carry_out(output);
                }
                if let Some(Ok(speaker)) = &mut self.speaker {
                    let _ = speaker.stop();
                }
                if self.model.status == PlaybackStatus::Playing {
                    self.model.status = PlaybackStatus::Paused;
                }
            }
            Action::Seek(seconds) => self.seek(seconds)?,
            Action::Speed(speed) => {
                if !(SPEED_RANGE.0..=SPEED_RANGE.1).contains(&speed) {
                    return Err(IpcError::new(
                        &codes::TRANSPORT_VALUE,
                        format!("A speed of {speed} is outside 0.25 to 4."),
                    ));
                }
                self.send(Command::SetSpeed(speed));
                self.model.speed = speed;
            }
            Action::Loop(range) => self.set_loop(range)?,
        }
        Ok(self.state())
    }

    fn require_score(&self) -> Result<(), IpcError> {
        if self.score.is_none() {
            return Err(IpcError::new(
                &codes::NOT_COMPILED,
                "There is no compiled Score to play.",
            ));
        }
        Ok(())
    }

    fn play(&mut self) -> Result<(), IpcError> {
        self.require_score()?;
        let restart = self.model.position_s >= self.duration() && self.model.loop_range.is_none();
        if restart {
            self.model.position_s = 0.0;
        }
        if self.player.is_none() {
            self.open()?;
        } else if restart || self.model.status == PlaybackStatus::Stopped {
            self.seek_engine(self.model.position_s);
        }
        self.send(Command::Play);
        self.model.status = PlaybackStatus::Playing;
        Ok(())
    }

    /// Opens the device and replays the transport onto the fresh graph.
    fn open(&mut self) -> Result<(), IpcError> {
        let Some(score) = self.score.clone() else {
            return self.require_score();
        };
        let mut stream = StreamConfig::default();
        if let Some(device) = &self.config.device {
            stream = stream.on_device(device.clone());
        }
        if let Some(frames) = self.config.buffer_frames {
            stream = stream.with_buffer(frames);
        }
        let player = Player::open(
            (self.make)(),
            Score::clone(&score),
            PlanConfig::new(SampleRate::HZ_48000),
            stream,
            BLOCK,
        )
        .map_err(|err| {
            IpcError::new(
                &codes::AUDIO_UNAVAILABLE,
                format!("The audio device could not be opened: {err}."),
            )
        })?;
        let (device, rate) = player
            .stream()
            .map(|stream| (stream.device().to_owned(), stream.sample_rate()))
            .unwrap_or_default();
        self.rate = SampleRate::new(rate).unwrap_or(SampleRate::HZ_48000);
        self.model.device = Some(device);
        self.player = Some(player);
        if self.model.position_s > 0.0 {
            self.seek_engine(self.model.position_s);
        }
        if (self.model.speed - 1.0).abs() > f64::EPSILON {
            self.send(Command::SetSpeed(self.model.speed));
        }
        if let Some(range) = self.model.loop_range {
            let range = self.loop_frames(range);
            self.send(Command::SetLoop(range));
        }
        if self.config.volume_db < 0.0 {
            self.send(Command::SetGain(
                GainTarget::Master,
                Db(self.config.volume_db),
            ));
        }
        Ok(())
    }

    fn seek(&mut self, seconds: f64) -> Result<(), IpcError> {
        self.require_score()?;
        if !seconds.is_finite() || seconds < 0.0 {
            return Err(IpcError::new(
                &codes::TRANSPORT_VALUE,
                format!("{seconds} is not a position in seconds."),
            ));
        }
        let target = seconds.min(self.duration());
        self.seek_engine(target);
        self.model.position_s = target;
        if self.model.status == PlaybackStatus::Stopped && target > 0.0 {
            self.model.status = PlaybackStatus::Paused;
        }
        Ok(())
    }

    fn seek_engine(&mut self, seconds: f64) {
        if let Some(frame) = Seconds(seconds).to_frames(self.rate) {
            self.send(Command::Seek(frame));
            self.mark_stale();
        }
    }

    fn set_loop(&mut self, range: Option<LoopSpan>) -> Result<(), IpcError> {
        if let Some(span) = range {
            self.require_score()?;
            let duration = self.duration();
            let valid = span.start_s.is_finite()
                && span.end_s.is_finite()
                && span.start_s >= 0.0
                && span.start_s < span.end_s
                && span.end_s <= duration;
            if !valid {
                return Err(IpcError::new(
                    &codes::TRANSPORT_VALUE,
                    format!(
                        "A loop from {} to {} seconds does not fit a Score of {duration} seconds.",
                        span.start_s, span.end_s
                    ),
                ));
            }
        }
        let frames = range.and_then(|span| self.loop_frames(span));
        self.send(Command::SetLoop(frames));
        self.model.loop_range = range;
        Ok(())
    }

    fn loop_frames(&self, span: LoopSpan) -> Option<LoopRange> {
        Some(LoopRange {
            start: Seconds(span.start_s).to_frames(self.rate)?,
            end: Seconds(span.end_s).to_frames(self.rate)?,
        })
    }

    fn send(&mut self, command: Command) {
        if let Some(player) = &mut self.player {
            player.send(command);
        }
    }

    fn mark_stale(&mut self) {
        if let Some(player) = &mut self.player {
            self.stale_until = player.state().blocks + 2;
        }
    }

    /// Takes the playhead from the engine while playing.
    fn refresh(&mut self) {
        if self.model.status != PlaybackStatus::Playing {
            return;
        }
        let Some(player) = &mut self.player else {
            return;
        };
        let engine = player.state();
        if engine.blocks >= self.stale_until && engine.blocks > 0 {
            self.model.position_s = position(engine.graph.position, self.rate);
        }
    }

    /// The control tick: services the player, follows the playhead, stops at the end and
    /// sends periodic clock anchors.
    fn service(&mut self) {
        let Some(player) = &mut self.player else {
            return;
        };
        match player.tick() {
            Ok(Some(PlayerEvent::DeviceChanged { device })) => {
                // The player rebuilt the stream paused (HS-05); the model follows.
                if self.model.status == PlaybackStatus::Playing {
                    self.model.status = PlaybackStatus::Paused;
                }
                if let Some(rate) = player
                    .stream()
                    .and_then(|stream| SampleRate::new(stream.sample_rate()))
                {
                    self.rate = rate;
                }
                self.model.device = Some(device.clone());
                self.sink.emit(Event::DeviceChanged { device });
                self.anchor();
                return;
            }
            Ok(_) => {}
            Err(err) => {
                self.player = None;
                self.model.device = None;
                if self.model.status == PlaybackStatus::Playing {
                    self.model.status = PlaybackStatus::Paused;
                }
                self.sink.emit(Event::AudioFailed {
                    error: IpcError::new(
                        &codes::AUDIO_UNAVAILABLE,
                        format!("Audio output stopped: {err}. Playback is paused."),
                    ),
                });
                self.anchor();
                return;
            }
        }
        if self.model.status != PlaybackStatus::Playing {
            return;
        }
        self.refresh();
        let duration = self.duration();
        if self.model.loop_range.is_none() && self.model.position_s >= duration {
            self.send(Command::Stop);
            self.model.status = PlaybackStatus::Stopped;
            self.model.position_s = 0.0;
            self.mark_stale();
            self.sink.emit(Event::PlaybackFinished {
                duration_s: duration,
            });
            self.anchor();
            return;
        }
        if self
            .last_anchor
            .is_none_or(|at| at.elapsed() >= ANCHOR_PERIOD)
        {
            self.anchor();
        }
    }

    fn now_ms(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// One step of the sequencer (08 §4).
    fn speak(&mut self) {
        let now = self.now_ms();
        #[expect(
            clippy::cast_precision_loss,
            reason = "milliseconds since the thread started, far below 2^52"
        )]
        let now_s = Seconds(now as f64 / 1_000.0);
        for announcement in self.streams.tick(now_s) {
            self.sequencer.push(announcement, now);
        }
        let playing = self.model.status == PlaybackStatus::Playing;
        let speaking = match (&self.speaker, self.sequencer.config().route) {
            (Some(Ok(speaker)), SpeechRoute::SelfVoicing) => speaker.is_speaking(),
            _ => None,
        };
        for output in self.sequencer.poll(now, playing, speaking) {
            self.carry_out(output);
        }
    }

    fn carry_out(&mut self, output: Output) {
        match output {
            Output::Pause => {
                if self.act(Action::Pause).is_ok() {
                    self.anchor();
                }
            }
            Output::Duck(on) => {
                let level = if on { DUCK_LEVEL } else { Db(0.0) };
                self.send(Command::SetGain(GainTarget::Duck, level));
            }
            Output::Deliver(delivery) => self.deliver(&delivery),
        }
    }

    /// Speaks a delivery through self-voicing when that is the route and a speaker opens,
    /// and always sends it to the host, which shows it and, unless it was self-voiced,
    /// passes it to the screen reader.
    fn deliver(&mut self, delivery: &Delivery) {
        let mut self_voiced = false;
        if self.sequencer.config().route == SpeechRoute::SelfVoicing {
            if self.speaker.is_none() {
                self.speaker = self
                    .speaker_source
                    .as_ref()
                    .map(|source| source.make().map_err(|_| ()));
            }
            if let Some(Ok(speaker)) = &mut self.speaker {
                self_voiced = speaker
                    .speak(&delivery.announcement.speech, delivery.interrupt)
                    .is_ok();
            }
        }
        self.sink.emit(Event::Announcement(crate::speech::info(
            delivery,
            self_voiced,
        )));
    }

    fn anchor(&mut self) {
        self.seq += 1;
        self.last_anchor = Some(Instant::now());
        let host_time_s = self
            .player
            .as_mut()
            .and_then(|player| player.state().host_time)
            .map(|time| time.as_secs_f64());
        self.sink.emit(Event::Clock(ClockAnchor {
            transport: self.state(),
            host_time_s,
            seq: self.seq,
        }));
    }
}

fn position(frames: Frames, rate: SampleRate) -> f64 {
    frames.to_seconds(rate).0
}
