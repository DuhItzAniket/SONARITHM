// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! DSP shared by both graph implementations (voice, limiter), so that the comparison
//! measures the graph and scheduling layer, not different oscillators.

use crate::score::{Note, Wave};

/// SDR-22 (MUST): every note has non-zero onset and offset ramps; 06 §3 sets a 5 ms floor.
pub const MIN_RAMP_S: f32 = 0.005;
/// SDR-22 reference onset of about 40 ms, shortened to 25% of short notes
/// (sound-design.md proposal under SDR-22).
pub const REF_RAMP_S: f32 = 0.040;

/// How the oscillator computes a sine. `Poly` is pure arithmetic; `Std` calls the platform
/// maths library, which the determinism check (A02-T04) compares across operating systems.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathMode {
    Poly,
    Std,
}

#[derive(Clone, Copy, Debug)]
pub struct Voice {
    pub active: bool,
    /// Start order, for deterministic oldest-first stealing.
    pub age: u64,
    wave: Wave,
    phase: f32,
    inc: f32,
    gain_l: f32,
    gain_r: f32,
    pos: u32,
    len: u32,
    ramp: u32,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            active: false,
            age: 0,
            wave: Wave::Sine,
            phase: 0.0,
            inc: 0.0,
            gain_l: 0.0,
            gain_r: 0.0,
            pos: 0,
            len: 0,
            ramp: 1,
        }
    }
}

/// Parabolic sine approximation with one refinement step (error about 0.1%): pure
/// arithmetic, so it is bit-identical on every IEEE 754 platform.
fn poly_sine(phase: f32) -> f32 {
    let x = phase * 2.0 - 1.0;
    let y = 4.0 * x * (1.0 - x.abs());
    -(0.225 * (y * y.abs() - y) + y)
}

fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        t + t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + t + t + 1.0
    } else {
        0.0
    }
}

impl Voice {
    /// Starts a note. Real-time safe: arithmetic and `libm` only.
    pub fn start(&mut self, n: &Note, wave: Wave, sr: f32, age: u64) {
        let ramp = (REF_RAMP_S * sr)
            .min(n.len as f32 * 0.25)
            .max(MIN_RAMP_S * sr) as u32;
        // Constant-power pan (06 §3).
        let theta = (n.pan + 1.0) * core::f32::consts::FRAC_PI_4;
        *self = Self {
            active: true,
            age,
            wave,
            phase: 0.0,
            inc: n.freq / sr,
            gain_l: n.gain * libm::cosf(theta),
            gain_r: n.gain * libm::sinf(theta),
            pos: 0,
            len: n.len.max(2 * ramp),
            ramp: ramp.max(1),
        };
    }

    /// Adds this voice into the stereo bus buffers.
    pub fn render(&mut self, l: &mut [f32], r: &mut [f32], math: MathMode) {
        for (lo, ro) in l.iter_mut().zip(r.iter_mut()) {
            if self.pos >= self.len {
                self.active = false;
                return;
            }
            let p = self.phase;
            let s = match (self.wave, math) {
                (Wave::Sine, MathMode::Poly) => poly_sine(p),
                (Wave::Sine, MathMode::Std) => (core::f32::consts::TAU * p).sin(),
                (Wave::Triangle, _) => 4.0 * (p - 0.5).abs() - 1.0,
                (Wave::Square, _) => {
                    let naive = if p < 0.5 { 1.0 } else { -1.0 };
                    let half = if p < 0.5 { p + 0.5 } else { p - 0.5 };
                    naive + poly_blep(p, self.inc) - poly_blep(half, self.inc)
                }
            };
            let from_end = self.len - self.pos;
            let env = if self.pos < self.ramp {
                self.pos as f32 / self.ramp as f32
            } else if from_end < self.ramp {
                from_end as f32 / self.ramp as f32
            } else {
                1.0
            };
            *lo += s * env * self.gain_l;
            *ro += s * env * self.gain_r;
            self.phase += self.inc;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            self.pos += 1;
        }
    }
}

/// Picks the voice for a new note: a free one, else the oldest (deterministic stealing,
/// coding-standards §11). Returns the index and whether a voice was stolen.
pub fn allocate(voices: &[Voice]) -> (usize, bool) {
    if let Some(i) = voices.iter().position(|v| !v.active) {
        return (i, false);
    }
    let oldest = voices
        .iter()
        .enumerate()
        .min_by_key(|(_, v)| v.age)
        .map_or(0, |(i, _)| i);
    (oldest, true)
}

/// Peak limiter on the master (instant attack, 50 ms release, ceiling −1 dBFS). A stand-in
/// for the look-ahead true-peak limiter of 06 §10.
#[derive(Clone, Copy, Debug)]
pub struct Limiter {
    env: f32,
    release: f32,
}

pub const CEILING: f32 = 0.891;

impl Limiter {
    pub fn new(sr: f32) -> Self {
        Self {
            env: 0.0,
            release: libm::expf(-1.0 / (0.050 * sr)),
        }
    }

    pub fn process(&mut self, l: &mut [f32], r: &mut [f32]) {
        for (a, b) in l.iter_mut().zip(r.iter_mut()) {
            let peak = a.abs().max(b.abs());
            self.env = if peak > self.env {
                peak
            } else {
                self.env * self.release + peak * (1.0 - self.release)
            };
            let g = if self.env > CEILING {
                CEILING / self.env
            } else {
                1.0
            };
            *a *= g;
            *b *= g;
        }
    }
}

/// Maximum number of simultaneously sounding notes in a track, plus headroom (05 §5).
pub fn polyphony(notes: &[Note], headroom: usize) -> usize {
    let mut ends: Vec<u64> = Vec::new();
    let mut max = 0;
    for n in notes {
        ends.retain(|&e| e > n.start);
        ends.push(n.start + u64::from(n.len));
        max = max.max(ends.len());
    }
    max + headroom
}
