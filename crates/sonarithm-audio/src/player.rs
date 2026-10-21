// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Playback on a device, with recovery from device loss (06 §11).
//!
//! A [`Player`] owns a backend, the Score, the control half of the render graph and the open
//! stream. Call [`Player::tick`] on the control thread every few milliseconds: it flushes
//! queued commands, frees retired graphs, and when the device is lost or the callback has
//! faulted, it rebuilds the stream on the default device. The rebuilt stream keeps the
//! transport position (converted if the new device runs at another sample rate), the speed,
//! the loop and the levels, and it stays **paused** until the user resumes (sound-design §12
//! HS-05), so a changed device never starts sounding unexpectedly.

use std::collections::BTreeMap;

use sonarithm_core::{Db, SampleRate, Score};
use sonarithm_render::graph::{Command, GainTarget, LoopRange};
use sonarithm_render::{
    Controller, EngineState, Graph, GraphConfig, PlanConfig, PlanError, RenderPlan, realtime,
};

use crate::backend::{AudioBackend, AudioError, Stream, StreamConfig};

/// Why playback could not start or recover.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PlayerError {
    /// The device failed.
    #[error(transparent)]
    Audio(#[from] AudioError),
    /// The Score could not be planned.
    #[error(transparent)]
    Plan(#[from] PlanError),
    /// The device reports a sample rate of zero.
    #[error("the audio device reports no sample rate")]
    NoSampleRate,
}

/// Something the user should hear about (the UI announces it, 06 §11).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlayerEvent {
    /// Playback moved to another device and is paused: "Audio device changed to …".
    DeviceChanged {
        /// The new device's name.
        device: String,
    },
}

/// Transport settings replayed onto a rebuilt graph.
#[derive(Clone, Debug, Default)]
struct Replay {
    speed: Option<f64>,
    /// The last loop command, if any was sent (its range may be `None`: looping off).
    looping: Option<LoopCommand>,
    gains: BTreeMap<GainKey, Db>,
}

/// The range of the last [`Command::SetLoop`].
#[derive(Clone, Copy, Debug)]
struct LoopCommand(Option<LoopRange>);

/// An ordered key for gain targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum GainKey {
    Bus(usize),
    Master,
    Duck,
}

/// Plays a Score on a device and recovers from device loss.
pub struct Player<B: AudioBackend> {
    backend: B,
    score: Score,
    plan_config: PlanConfig,
    stream_config: StreamConfig,
    block: usize,
    controller: Controller,
    stream: Option<Stream>,
    replay: Replay,
    last: EngineState,
}

impl<B: AudioBackend> Player<B> {
    /// Plans `score` at the device's rate and opens a stream. The transport starts stopped.
    ///
    /// # Errors
    ///
    /// [`PlayerError`] if the Score cannot be planned or the device cannot be opened.
    pub fn open(
        mut backend: B,
        score: Score,
        plan_config: PlanConfig,
        stream_config: StreamConfig,
        block: usize,
    ) -> Result<Self, PlayerError> {
        let (controller, stream) = start(
            &mut backend,
            &score,
            plan_config,
            &stream_config,
            block,
            None,
        )?;
        Ok(Self {
            backend,
            score,
            plan_config,
            stream_config,
            block,
            controller,
            stream: Some(stream),
            replay: Replay::default(),
            last: EngineState::default(),
        })
    }

    /// Sends a transport or mixer command.
    pub fn send(&mut self, command: Command) {
        match command {
            Command::SetSpeed(speed) => self.replay.speed = Some(speed),
            Command::SetLoop(range) => self.replay.looping = Some(LoopCommand(range)),
            Command::SetGain(target, level) => {
                let key = match target {
                    GainTarget::Bus(bus) => GainKey::Bus(bus.index()),
                    GainTarget::Master => GainKey::Master,
                    GainTarget::Duck => GainKey::Duck,
                };
                self.replay.gains.insert(key, level);
            }
            _ => {}
        }
        self.controller.send(command);
    }

    /// The latest engine state.
    pub fn state(&mut self) -> EngineState {
        self.last = self.controller.state();
        self.last
    }

    /// The open stream.
    #[must_use]
    pub fn stream(&self) -> Option<&Stream> {
        self.stream.as_ref()
    }

    /// The backend.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// The control tick: flushes commands, frees retired graphs, and rebuilds the stream if
    /// the device was lost or the callback faulted.
    ///
    /// # Errors
    ///
    /// [`PlayerError`] if no device can be opened; call `tick` again to retry.
    pub fn tick(&mut self) -> Result<Option<PlayerEvent>, PlayerError> {
        self.controller.flush();
        self.controller.collect();
        let state = self.controller.state();
        if state.blocks > 0 {
            self.last = state;
        }
        let broken = self
            .stream
            .as_ref()
            .is_none_or(|stream| stream.status().device_lost() || stream.status().fault());
        if broken {
            return self.rebuild();
        }
        // The backend followed a new default device by itself: pause there too (HS-05).
        if self
            .stream
            .as_ref()
            .is_some_and(|stream| stream.status().take_device_changed())
        {
            self.controller.send(Command::Pause);
            let device = self
                .backend
                .devices()
                .ok()
                .and_then(|devices| devices.into_iter().find(|d| d.is_default))
                .map_or_else(|| "the default device".to_owned(), |d| d.name);
            return Ok(Some(PlayerEvent::DeviceChanged { device }));
        }
        Ok(None)
    }

    fn rebuild(&mut self) -> Result<Option<PlayerEvent>, PlayerError> {
        let old_rate = self.stream.as_ref().map(Stream::sample_rate);
        // Drop the old stream (and its processor) before opening the device again.
        self.stream = None;
        // A named device that vanished is replaced by the default device (06 §11).
        let config = StreamConfig {
            device: None,
            ..self.stream_config.clone()
        };
        let position = self.last.graph.position;
        let resume_at = old_rate
            .and_then(SampleRate::new)
            .map(|rate| position.to_seconds(rate));
        let (controller, stream) = start(
            &mut self.backend,
            &self.score,
            self.plan_config,
            &config,
            self.block,
            resume_at,
        )?;
        self.controller = controller;
        if let Some(speed) = self.replay.speed {
            self.controller.send(Command::SetSpeed(speed));
        }
        if let Some(LoopCommand(range)) = self.replay.looping {
            self.controller.send(Command::SetLoop(range));
        }
        for (key, level) in &self.replay.gains {
            let target = match key {
                GainKey::Bus(index) => match sonarithm_render::Bus::ALL.get(*index) {
                    Some(bus) => GainTarget::Bus(*bus),
                    None => continue,
                },
                GainKey::Master => GainTarget::Master,
                GainKey::Duck => GainTarget::Duck,
            };
            self.controller.send(Command::SetGain(target, *level));
        }
        let device = stream.device().to_owned();
        self.stream = Some(stream);
        self.stream_config = config;
        Ok(Some(PlayerEvent::DeviceChanged { device }))
    }
}

/// Plans the Score at the device's rate, builds the graph (seeking to `resume_at`, paused)
/// and opens the stream.
fn start<B: AudioBackend>(
    backend: &mut B,
    score: &Score,
    plan_config: PlanConfig,
    stream_config: &StreamConfig,
    block: usize,
    resume_at: Option<sonarithm_core::Seconds>,
) -> Result<(Controller, Stream), PlayerError> {
    let rate = backend.sample_rate(stream_config.device.as_deref())?;
    let rate = SampleRate::new(rate).ok_or(PlayerError::NoSampleRate)?;
    let plan = RenderPlan::build(score, &plan_config.at_rate(rate))?.plan;
    let mut graph = Graph::new(plan, GraphConfig::new(block));
    if let Some(frame) = resume_at.and_then(|t| t.to_frames(rate)) {
        graph.apply(Command::Seek(frame));
    }
    let (controller, processor) = realtime(graph);
    let stream = backend.open(stream_config, processor)?;
    Ok((controller, stream))
}
