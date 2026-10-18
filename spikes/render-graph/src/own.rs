// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The owned graph (A02-T02): a voice pool per track, fixed buses, a master limiter and a
//! sample-accurate scheduler that splits each block at event onsets (06 §3, §5). The whole
//! schedule is handed to the processor at prepare time, so the control half only forwards
//! seeks through an atomic.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::graph::{Counters, GraphProcessor, RenderConfig, RenderGraph};
use crate::score::{BUS_GAINS, MASTER_GAIN, Note, Score, Wave};
use crate::voice::{Limiter, MathMode, Voice, allocate, polyphony};

const NO_SEEK: u64 = u64::MAX;

#[derive(Clone, Copy)]
struct Event {
    start: u64,
    track: usize,
    note: Note,
}

struct TrackPool {
    wave: Wave,
    bus: usize,
    voices: Vec<Voice>,
    /// Stereo scratch for this track, `max_block` long.
    buf: [Vec<f32>; 2],
}

/// Control half.
pub struct OwnGraph {
    counters: Arc<Counters>,
    seek: Arc<AtomicU64>,
    voices: usize,
}

/// Audio-thread half.
struct OwnProcessor {
    sr: f32,
    math: MathMode,
    pos: u64,
    events: Vec<Event>,
    next: usize,
    tracks: Vec<TrackPool>,
    master: [Vec<f32>; 2],
    limiter: Limiter,
    age: u64,
    counters: Arc<Counters>,
    seek: Arc<AtomicU64>,
}

impl OwnGraph {
    pub fn new() -> Self {
        Self {
            counters: Arc::default(),
            seek: Arc::new(AtomicU64::new(NO_SEEK)),
            voices: 0,
        }
    }
}

impl RenderGraph for OwnGraph {
    fn name(&self) -> &'static str {
        "own"
    }

    fn prepare(
        &mut self,
        score: &Score,
        cfg: &RenderConfig,
    ) -> Result<Box<dyn GraphProcessor>, String> {
        let tracks: Vec<TrackPool> = score
            .tracks
            .iter()
            .map(|t| TrackPool {
                wave: t.wave,
                bus: t.bus,
                voices: vec![Voice::default(); polyphony(&t.notes, 4)],
                buf: [vec![0.0; cfg.max_block], vec![0.0; cfg.max_block]],
            })
            .collect();
        let mut events: Vec<Event> = score
            .tracks
            .iter()
            .enumerate()
            .flat_map(|(track, t)| {
                t.notes.iter().map(move |&note| Event {
                    start: note.start,
                    track,
                    note,
                })
            })
            .collect();
        // Stable sort: ties keep track order, so dispatch order is deterministic.
        events.sort_by_key(|e| e.start);
        self.voices = tracks.iter().map(|t| t.voices.len()).sum();
        self.counters = Arc::default();
        Ok(Box::new(OwnProcessor {
            sr: cfg.sample_rate as f32,
            math: cfg.math,
            pos: 0,
            events,
            next: 0,
            tracks,
            master: [vec![0.0; cfg.max_block], vec![0.0; cfg.max_block]],
            limiter: Limiter::new(cfg.sample_rate as f32),
            age: 0,
            counters: Arc::clone(&self.counters),
            seek: Arc::clone(&self.seek),
        }))
    }

    fn update(&mut self) {}

    fn seek(&mut self, frame: u64) {
        self.seek.store(frame, Ordering::Release);
    }

    fn counters(&self) -> &Arc<Counters> {
        &self.counters
    }

    fn voices(&self) -> usize {
        self.voices
    }
}

impl OwnProcessor {
    fn start_note(&mut self, ev: Event) {
        let pool = &mut self.tracks[ev.track];
        let (i, stolen) = allocate(&pool.voices);
        pool.voices[i].start(&ev.note, pool.wave, self.sr, self.age);
        self.age += 1;
        self.counters.dispatched.fetch_add(1, Ordering::Relaxed);
        self.counters
            .stolen
            .fetch_add(u64::from(stolen), Ordering::Relaxed);
    }
}

impl GraphProcessor for OwnProcessor {
    fn process(&mut self, out: &mut [f32]) {
        let seek = self.seek.swap(NO_SEEK, Ordering::Acquire);
        if seek != NO_SEEK {
            self.pos = seek;
            self.next = self.events.partition_point(|e| e.start < seek);
            // Spike: a hard cut. A05 crossfades voices out within 10 ms (06 §5, SDR-22).
            for v in self.tracks.iter_mut().flat_map(|t| &mut t.voices) {
                v.active = false;
            }
        }
        let frames = (out.len() / 2).min(self.master[0].len());
        for t in &mut self.tracks {
            t.buf[0][..frames].fill(0.0);
            t.buf[1][..frames].fill(0.0);
        }
        let mut seg = 0;
        while seg < frames {
            let now = self.pos + seg as u64;
            while let Some(&ev) = self.events.get(self.next).filter(|e| e.start <= now) {
                self.start_note(ev);
                self.next += 1;
            }
            // The segment ends at the next onset inside this block, or at the block end.
            let end = self
                .events
                .get(self.next)
                .map_or(frames, |e| ((e.start - self.pos) as usize).min(frames));
            let math = self.math;
            for t in &mut self.tracks {
                let [l, r] = &mut t.buf;
                for v in t.voices.iter_mut().filter(|v| v.active) {
                    v.render(&mut l[seg..end], &mut r[seg..end], math);
                }
            }
            let active = self
                .tracks
                .iter()
                .flat_map(|t| &t.voices)
                .filter(|v| v.active)
                .count();
            self.counters
                .max_active
                .fetch_max(active as u64, Ordering::Relaxed);
            seg = end;
        }
        // Buses sum their tracks in track order, then the master sums buses in bus order.
        let [ml, mr] = &mut self.master;
        ml[..frames].fill(0.0);
        mr[..frames].fill(0.0);
        for (bus, gain) in BUS_GAINS.iter().enumerate() {
            for i in 0..frames {
                let (mut l, mut r) = (0.0, 0.0);
                for t in self.tracks.iter().filter(|t| t.bus == bus) {
                    l += t.buf[0][i];
                    r += t.buf[1][i];
                }
                ml[i] += l * gain;
                mr[i] += r * gain;
            }
        }
        for i in 0..frames {
            ml[i] *= MASTER_GAIN;
            mr[i] *= MASTER_GAIN;
        }
        self.limiter.process(&mut ml[..frames], &mut mr[..frames]);
        for (i, frame) in out
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .take(frames)
            .enumerate()
        {
            frame[0] = ml[i];
            frame[1] = mr[i];
        }
        self.pos += frames as u64;
        self.counters.pos.store(self.pos, Ordering::Relaxed);
        self.counters
            .rendered
            .fetch_add(frames as u64, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::short_score;

    fn render(block: usize, seek: Option<u64>) -> Vec<f32> {
        let score = short_score();
        let cfg = RenderConfig {
            sample_rate: 48_000,
            max_block: block,
            math: MathMode::Poly,
        };
        let mut g = OwnGraph::new();
        let mut p = g.prepare(&score, &cfg).expect("prepare");
        let start = seek.unwrap_or(0);
        if let Some(f) = seek {
            g.seek(f);
        }
        let frames = (score.frames - start) as usize;
        let mut out = vec![0.0; frames.div_ceil(block) * block * 2];
        for chunk in out.chunks_mut(block * 2) {
            p.process(chunk);
        }
        out.truncate(frames * 2);
        out
    }

    /// A seek back to the start replays the whole Score (the real-time soak loops this way).
    #[test]
    fn seek_to_start_replays_every_event() {
        let score = short_score();
        let events: usize = score.tracks.iter().map(|t| t.notes.len()).sum();
        let cfg = RenderConfig {
            sample_rate: 48_000,
            max_block: 480,
            math: MathMode::Poly,
        };
        let mut g = OwnGraph::new();
        let mut p = g.prepare(&score, &cfg).expect("prepare");
        let mut buf = vec![0.0; 960];
        for pass in 1..=2u64 {
            for _ in 0..score.frames.div_ceil(480) {
                p.process(&mut buf);
            }
            assert_eq!(g.stats().dispatched, pass * events as u64);
            g.seek(0);
        }
    }

    /// Sample-accurate dispatch: the output does not depend on the block size.
    #[test]
    fn output_is_independent_of_block_size() {
        let a = render(64, None);
        assert_eq!(a, render(256, None));
        assert_eq!(a, render(1000, None));
    }

    /// After a seek, the output matches a full render once notes started before the seek
    /// point have ended (the longest note plus its ramp is under a quarter of the remainder,
    /// 0.25 s). Skipped under Miri, whose shorter Score cannot hold that tail.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn seek_matches_full_render_after_tail() {
        let full = render(256, None);
        let from = short_score().frames / 2;
        let seeked = render(256, Some(from));
        let skip = (from as usize / 4) * 2;
        let diff = full[from as usize * 2 + skip..]
            .iter()
            .zip(&seeked[skip..])
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(diff < 1e-6, "max difference {diff}");
    }
}
