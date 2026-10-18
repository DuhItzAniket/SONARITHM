// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Firewheel 0.14 behind the draft `RenderGraph` contract (A02-T03). The voices, bus gains
//! and limiter are the same code as the owned graph; Firewheel supplies the graph compiler,
//! the mixing of edges and the sample-accurate event scheduler (`scheduled_events`).

use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use audioadapter_buffers::direct::InterleavedSlice;
use firewheel::backend::BackendProcessInfo;
use firewheel::channel_config::{ChannelConfig, ChannelCount};
use firewheel::clock::{EventInstant, InstantSamples};
use firewheel::event::{NodeEventType, ProcEvents};
use firewheel::node::{
    AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, EmptyConfig,
    NodeError, NodeID, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus, StreamStatus,
};
use firewheel::processor::{BufferOutOfSpaceMode, FirewheelProcessor};
use firewheel::{ActivateInfo, ClearScheduledEventsType, FirewheelConfig, FirewheelContext};

use crate::graph::{Counters, GraphProcessor, RenderConfig, RenderGraph};
use crate::score::{BUS_GAINS, MASTER_GAIN, Note, Score, Wave};
use crate::voice::{Limiter, MathMode, Voice, allocate, polyphony};

/// Events are handed to Firewheel this far ahead of the rendered position (200 ms).
const LOOKAHEAD: u64 = 9_600;
const TAG_NOTE: u8 = 1;
const TAG_ALL_OFF: u8 = 2;

/// Note payload in a `CustomBytes` event: no heap allocation per event.
fn encode(n: &Note) -> [u8; 36] {
    let mut b = [0u8; 36];
    b[0] = TAG_NOTE;
    b[4..8].copy_from_slice(&n.len.to_le_bytes());
    for (i, x) in [n.freq, n.gain, n.pan].into_iter().enumerate() {
        b[8 + i * 4..12 + i * 4].copy_from_slice(&x.to_le_bytes());
    }
    b
}

fn decode(b: &[u8; 36]) -> Note {
    let f = |i: usize| f32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    Note {
        start: 0,
        len: u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
        freq: f(8),
        gain: f(12),
        pan: f(16),
    }
}

const STEREO_OUT: ChannelConfig = ChannelConfig {
    num_inputs: ChannelCount::ZERO,
    num_outputs: ChannelCount::STEREO,
};
const STEREO_IN_OUT: ChannelConfig = ChannelConfig {
    num_inputs: ChannelCount::STEREO,
    num_outputs: ChannelCount::STEREO,
};

/// One track: a voice pool that starts notes from scheduled events.
struct SynthNode {
    wave: Wave,
    voices: usize,
    math: MathMode,
    counters: Arc<Counters>,
}

struct SynthProc {
    wave: Wave,
    voices: Vec<Voice>,
    sr: f32,
    age: u64,
    math: MathMode,
    counters: Arc<Counters>,
}

impl AudioNode for SynthNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("synth")
            .channel_config(STEREO_OUT))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        Ok(SynthProc {
            wave: self.wave,
            voices: vec![Voice::default(); self.voices],
            sr: cx.stream_info.sample_rate.get() as f32,
            age: 0,
            math: self.math,
            counters: Arc::clone(&self.counters),
        })
    }
}

impl AudioNodeProcessor for SynthProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        for ev in events.drain() {
            let NodeEventType::CustomBytes(b) = ev else {
                continue;
            };
            match b[0] {
                TAG_NOTE => {
                    let (i, stolen) = allocate(&self.voices);
                    self.voices[i].start(&decode(&b), self.wave, self.sr, self.age);
                    self.age += 1;
                    self.counters.dispatched.fetch_add(1, Ordering::Relaxed);
                    self.counters
                        .stolen
                        .fetch_add(u64::from(stolen), Ordering::Relaxed);
                }
                TAG_ALL_OFF => self.voices.iter_mut().for_each(|v| v.active = false),
                _ => {}
            }
        }
    }

    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        let [l, r] = buffers.outputs else {
            return ProcessStatus::ClearAllOutputs;
        };
        l.fill(0.0);
        r.fill(0.0);
        for v in self.voices.iter_mut().filter(|v| v.active) {
            v.render(l, r, self.math);
        }
        ProcessStatus::OutputsModified
    }
}

/// A fixed stereo gain (bus), or the master stage (gain then limiter) when `limiter` is set.
struct GainNode {
    gain: f32,
    limiter: bool,
}

struct GainProc {
    gain: f32,
    limiter: Option<Limiter>,
}

impl AudioNode for GainNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name(if self.limiter { "master" } else { "bus" })
            .channel_config(STEREO_IN_OUT))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let sr = cx.stream_info.sample_rate.get() as f32;
        Ok(GainProc {
            gain: self.gain,
            limiter: self.limiter.then(|| Limiter::new(sr)),
        })
    }
}

impl AudioNodeProcessor for GainProc {
    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        let ([il, ir], [ol, or]) = (buffers.inputs, buffers.outputs) else {
            return ProcessStatus::ClearAllOutputs;
        };
        for (o, i) in ol
            .iter_mut()
            .zip(il.iter())
            .chain(or.iter_mut().zip(ir.iter()))
        {
            *o = *i * self.gain;
        }
        if let Some(lim) = &mut self.limiter {
            lim.process(ol, or);
        }
        ProcessStatus::OutputsModified
    }
}

/// Control half: owns the Firewheel context and feeds it events ahead of the playhead.
pub struct FwGraph {
    cx: Option<FirewheelContext>,
    synths: Vec<NodeID>,
    events: Vec<(u64, usize, Note)>,
    next: usize,
    /// Firewheel clock (samples since activation) at Score frame 0.
    origin: i64,
    counters: Arc<Counters>,
    voices: usize,
}

struct FwProcessor {
    proc: FirewheelProcessor,
    clock: u64,
    sr: f64,
    counters: Arc<Counters>,
}

impl FwGraph {
    pub fn new() -> Self {
        Self {
            cx: None,
            synths: Vec::new(),
            events: Vec::new(),
            next: 0,
            origin: 0,
            counters: Arc::default(),
            voices: 0,
        }
    }
}

impl RenderGraph for FwGraph {
    fn name(&self) -> &'static str {
        "firewheel"
    }

    fn prepare(
        &mut self,
        score: &Score,
        cfg: &RenderConfig,
    ) -> Result<Box<dyn GraphProcessor>, String> {
        let mut cx = FirewheelContext::new(FirewheelConfig {
            // The default (AllocateOnAudioThread) breaks 06 §12 when a buffer fills up.
            buffer_out_of_space_mode: BufferOutOfSpaceMode::Panic,
            scheduled_event_capacity: 4096,
            ..FirewheelConfig::default()
        });
        self.counters = Arc::default();
        let master = cx
            .add_node(
                GainNode {
                    gain: MASTER_GAIN,
                    limiter: true,
                },
                None,
            )
            .map_err(|e| e.to_string())?;
        let out = cx.graph_out_node_id();
        cx.connect(master, out, &[(0, 0), (1, 1)], false)
            .map_err(|e| e.to_string())?;
        let mut buses = Vec::new();
        for gain in BUS_GAINS {
            let bus = cx
                .add_node(
                    GainNode {
                        gain,
                        limiter: false,
                    },
                    None,
                )
                .map_err(|e| e.to_string())?;
            cx.connect(bus, master, &[(0, 0), (1, 1)], false)
                .map_err(|e| e.to_string())?;
            buses.push(bus);
        }
        self.synths.clear();
        self.voices = 0;
        for t in &score.tracks {
            let voices = polyphony(&t.notes, 4);
            self.voices += voices;
            let node = SynthNode {
                wave: t.wave,
                voices,
                math: cfg.math,
                counters: Arc::clone(&self.counters),
            };
            let id = cx.add_node(node, None).map_err(|e| e.to_string())?;
            cx.connect(id, buses[t.bus], &[(0, 0), (1, 1)], false)
                .map_err(|e| e.to_string())?;
            self.synths.push(id);
        }
        self.events = score
            .tracks
            .iter()
            .enumerate()
            .flat_map(|(i, t)| t.notes.iter().map(move |n| (n.start, i, *n)))
            .collect();
        self.events.sort_by_key(|e| e.0);
        self.next = 0;
        self.origin = 0;
        let proc = cx
            .activate(ActivateInfo {
                sample_rate: NonZeroU32::new(cfg.sample_rate).ok_or("sample rate 0")?,
                max_block_frames: NonZeroU32::new(cfg.max_block as u32).ok_or("block 0")?,
                num_stream_in_channels: 0,
                num_stream_out_channels: 2,
                input_to_output_latency_seconds: 0.0,
            })
            .map_err(|e| e.to_string())?;
        cx.update().map_err(|e| e.to_string())?;
        self.cx = Some(cx);
        Ok(Box::new(FwProcessor {
            proc,
            clock: 0,
            sr: f64::from(cfg.sample_rate),
            counters: Arc::clone(&self.counters),
        }))
    }

    fn update(&mut self) {
        let Some(cx) = &mut self.cx else { return };
        let clock = self.counters.rendered.load(Ordering::Relaxed) as i64;
        let horizon = (clock - self.origin).max(0) as u64 + LOOKAHEAD;
        while let Some(&(start, track, note)) = self.events.get(self.next).filter(|e| e.0 < horizon)
        {
            let at = EventInstant::AtClockSamples(InstantSamples(self.origin + start as i64));
            cx.schedule_event_for(
                self.synths[track],
                NodeEventType::CustomBytes(encode(&note)),
                Some(at),
            );
            self.next += 1;
        }
        let _ = cx.update();
    }

    fn seek(&mut self, frame: u64) {
        let Some(cx) = &mut self.cx else { return };
        cx.cancel_all_scheduled_events(ClearScheduledEventsType::All);
        let mut off = [0u8; 36];
        off[0] = TAG_ALL_OFF;
        for &id in &self.synths {
            cx.queue_event_for(id, NodeEventType::CustomBytes(off));
        }
        // Firewheel's clock only moves forward, so the Score is re-anchored on it.
        self.origin = self.counters.rendered.load(Ordering::Relaxed) as i64 - frame as i64;
        self.next = self.events.partition_point(|e| e.0 < frame);
        self.update();
    }

    fn counters(&self) -> &Arc<Counters> {
        &self.counters
    }

    fn voices(&self) -> usize {
        self.voices
    }
}

impl GraphProcessor for FwProcessor {
    fn process(&mut self, out: &mut [f32]) {
        let frames = out.len() / 2;
        let (Ok(input), Ok(mut output)) = (
            InterleavedSlice::new(&[], 0, 0),
            InterleavedSlice::new_mut(out, 2, frames),
        ) else {
            return;
        };
        let info = BackendProcessInfo {
            frames,
            process_timestamp: None,
            duration_since_stream_start: Duration::from_secs_f64(self.clock as f64 / self.sr),
            input_stream_status: StreamStatus::empty(),
            output_stream_status: StreamStatus::empty(),
            dropped_frames: 0,
            process_to_playback_delay: None,
        };
        self.proc.process(&input, &mut output, info);
        self.clock += frames as u64;
        self.counters.rendered.store(self.clock, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::own::OwnGraph;
    use crate::score::short_score;

    fn render(graph: &mut dyn RenderGraph) -> Vec<f32> {
        let score = short_score();
        let cfg = RenderConfig {
            sample_rate: 48_000,
            max_block: 256,
            math: MathMode::Poly,
        };
        let mut p = graph.prepare(&score, &cfg).expect("prepare");
        let mut out = vec![0.0; (score.frames as usize).div_ceil(256) * 512];
        for chunk in out.chunks_mut(512) {
            graph.update();
            p.process(chunk);
        }
        out
    }

    /// Same voices and gains, Firewheel's scheduler and mixing: the audio is bit-identical.
    #[test]
    fn firewheel_matches_owned_graph() {
        assert_eq!(render(&mut FwGraph::new()), render(&mut OwnGraph::new()));
    }
}
