// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Envelopes with mandatory onset and offset ramps (SDR-22, 06 §3).
//!
//! SDR-22 (MUST): every note has onset and offset ramps, about 40 ms for reference, never
//! zero. [`note_ramp`] gives the ramp for a note of a given duration, and [`Adsr`] never
//! changes level in a step: its attack and release are raised-cosine segments of at least
//! [`MIN_RAMP`] that start from the current level, so retriggering or releasing a voice
//! mid-segment cannot click.

use sonarithm_core::{SampleRate, Seconds};

use crate::osc::sine_turns;

/// The shortest ramp anywhere: 5 ms (06 §3, "mandatory ramps ≥ 5 ms"; SDR-22: never zero).
///
/// sound-design §5 proposes a 2 ms floor for short notes; the architecture's 5 ms floor
/// applies until that proposal is accepted.
pub const MIN_RAMP: Seconds = Seconds(0.005);

/// The reference onset and offset ramp: about 40 ms (SDR-22, Smith & Walker 2005).
pub const REFERENCE_RAMP: Seconds = Seconds(0.040);

/// The share of a short note's duration that each ramp may take: 25 % (sound-design §5,
/// proposal under SDR-22).
pub const SHORT_NOTE_RAMP_SHARE: f64 = 0.25;

/// The level change below which a decay is considered to have reached its sustain level.
const SETTLED: f64 = 1e-6;

/// The onset and offset ramp for a note of `duration`: `min(40 ms, 25 % of duration)`,
/// never shorter than [`MIN_RAMP`] (SDR-22). A note shorter than two minimum ramps is
/// played for two minimum ramps.
#[must_use]
pub fn note_ramp(duration: Seconds) -> Seconds {
    let share = if duration.0.is_finite() {
        duration.0 * SHORT_NOTE_RAMP_SHARE
    } else {
        REFERENCE_RAMP.0
    };
    Seconds(share.clamp(MIN_RAMP.0, REFERENCE_RAMP.0))
}

/// A ramp length in frames at `rate`, rounded half to even and at least one frame.
#[must_use]
pub fn ramp_frames(ramp: Seconds, rate: SampleRate) -> u32 {
    let frames = ramp
        .to_frames(rate)
        .map_or(1, |frames| frames.0.clamp(1, u64::from(u32::MAX)));
    u32::try_from(frames).unwrap_or(u32::MAX)
}

/// Parameters of an [`Adsr`] envelope.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdsrParams {
    /// Attack time; raised to [`MIN_RAMP`] if shorter.
    pub attack: Seconds,
    /// Decay time constant towards the sustain level; raised to [`MIN_RAMP`] if shorter.
    pub decay: Seconds,
    /// Sustain level in [0, 1], relative to the attack peak.
    pub sustain: f64,
    /// Release time; raised to [`MIN_RAMP`] if shorter.
    pub release: Seconds,
}

impl AdsrParams {
    /// The envelope of a plain note of `duration`: attack and release of [`note_ramp`],
    /// full sustain (SDR-22).
    #[must_use]
    pub fn note(duration: Seconds) -> Self {
        let ramp = note_ramp(duration);
        Self {
            attack: ramp,
            decay: MIN_RAMP,
            sustain: 1.0,
            release: ramp,
        }
    }
}

/// The envelope stages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Stage {
    /// Silent; the voice can be reused.
    Idle,
    /// Rising to the peak.
    Attack,
    /// Falling towards the sustain level.
    Decay,
    /// Holding the sustain level.
    Sustain,
    /// Falling to silence.
    Release,
}

/// An ADSR envelope with raised-cosine attack and release segments (SDR-22).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Adsr {
    stage: Stage,
    level: f64,
    /// Level at the start of the current attack or release segment.
    from: f64,
    /// Frames into the current attack or release segment.
    pos: u32,
    attack: u32,
    release: u32,
    sustain: f64,
    /// Per-frame decay factor of the distance to the sustain level.
    decay: f64,
}

impl Adsr {
    /// An idle envelope at level 0.
    #[must_use]
    pub fn new(params: &AdsrParams, rate: SampleRate) -> Self {
        let floor = |time: Seconds| {
            Seconds(if time.0 >= MIN_RAMP.0 {
                time.0
            } else {
                MIN_RAMP.0
            })
        };
        let decay_frames = floor(params.decay).0 * f64::from(rate.get());
        Self {
            stage: Stage::Idle,
            level: 0.0,
            from: 0.0,
            pos: 0,
            attack: ramp_frames(floor(params.attack), rate),
            release: ramp_frames(floor(params.release), rate),
            sustain: if params.sustain.is_finite() {
                params.sustain.clamp(0.0, 1.0)
            } else {
                1.0
            },
            decay: libm::exp(-1.0 / decay_frames),
        }
    }

    /// The current stage.
    #[must_use]
    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// The release length in frames.
    #[must_use]
    pub fn release_frames(&self) -> u32 {
        self.release
    }

    /// Whether the envelope is sounding (not [`Stage::Idle`]).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.stage != Stage::Idle
    }

    /// Starts the attack from the current level, so a retrigger never steps.
    ///
    /// # Real-time safety
    ///
    /// RT-safe.
    pub fn trigger(&mut self) {
        self.from = self.level;
        self.pos = 0;
        self.stage = Stage::Attack;
    }

    /// Starts the release from the current level. Does nothing when idle.
    ///
    /// # Real-time safety
    ///
    /// RT-safe.
    pub fn release(&mut self) {
        if self.stage != Stage::Idle {
            self.from = self.level;
            self.pos = 0;
            self.stage = Stage::Release;
        }
    }

    /// The level for the next frame, in [0, 1].
    ///
    /// # Real-time safety
    ///
    /// RT-safe: arithmetic only.
    #[inline]
    pub fn next_level(&mut self) -> f32 {
        match self.stage {
            Stage::Idle => self.level = 0.0,
            Stage::Attack => {
                self.pos += 1;
                self.level = segment(self.from, 1.0, self.pos, self.attack);
                if self.pos >= self.attack {
                    self.stage = if self.sustain < 1.0 {
                        Stage::Decay
                    } else {
                        Stage::Sustain
                    };
                }
            }
            Stage::Decay => {
                self.level = self.sustain + (self.level - self.sustain) * self.decay;
                if self.level - self.sustain < SETTLED {
                    self.level = self.sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => self.level = self.sustain,
            Stage::Release => {
                self.pos += 1;
                self.level = segment(self.from, 0.0, self.pos, self.release);
                if self.pos >= self.release {
                    self.level = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "levels are in [0, 1]; samples are f32 (coding-standards §10)"
        )]
        let level = self.level as f32;
        level
    }

    /// Multiplies `buffer` by the envelope, frame by frame.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: bounded by `buffer.len()`.
    pub fn apply(&mut self, buffer: &mut [f32]) {
        for sample in buffer {
            *sample *= self.next_level();
        }
    }
}

/// Frame `pos` of `len` of a raised-cosine segment from `from` to `to`:
/// `from + (to − from) · sin²(π/2 · pos/len)`.
#[inline]
fn segment(from: f64, to: f64, pos: u32, len: u32) -> f64 {
    let x = f64::from(pos.min(len)) / f64::from(len.max(1));
    let s = sine_turns(x / 4.0);
    from + (to - from) * s * s
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: SampleRate = SampleRate::HZ_48000;

    #[test]
    fn a_decay_settles_on_the_sustain_level() {
        let params = AdsrParams {
            attack: Seconds(0.01),
            decay: Seconds(0.05),
            sustain: 0.5,
            release: Seconds(0.02),
        };
        let mut env = Adsr::new(&params, RATE);
        env.trigger();
        let levels: Vec<f32> = (0..48_000).map(|_| env.next_level()).collect();
        assert_eq!(levels[479], 1.0, "peak at the end of a 10 ms attack");
        assert!(
            levels[480..].windows(2).all(|w| w[1] <= w[0]),
            "decay is monotonic"
        );
        assert_eq!(env.stage(), Stage::Sustain);
        assert_eq!(*levels.last().unwrap(), 0.5);
        env.release();
        let tail: Vec<f32> = (0..960).map(|_| env.next_level()).collect();
        assert_eq!(tail[959], 0.0);
        assert!(!env.is_active());
    }

    #[test]
    fn invalid_parameters_fall_back_to_safe_values() {
        let params = AdsrParams {
            attack: Seconds(f64::NAN),
            decay: Seconds(-1.0),
            sustain: f64::NAN,
            release: Seconds(0.0),
        };
        let env = Adsr::new(&params, RATE);
        assert_eq!(env.release_frames(), 240);
        assert_eq!(env.attack, 240);
        assert_eq!(env.sustain, 1.0);
    }
}
