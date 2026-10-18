// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The benchmark Score: a simplified stand-in for the Score IR (05 §4) with the DS-006 shape
//! (4 tracks, 10,000 note events), generated deterministically from a seed. Event times are
//! already resolved to frames, as the plan builder would do (05 §5).

/// Oscillator shape of a track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wave {
    Sine,
    Triangle,
    /// PolyBLEP square (06 §3, SDR-29).
    Square,
}

/// One note event, in frames at the Score's sample rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    pub start: u64,
    pub len: u32,
    pub freq: f32,
    pub gain: f32,
    /// −1 (left) … +1 (right).
    pub pan: f32,
}

/// One track: a voice type routed to one bus.
#[derive(Clone, Debug)]
pub struct Track {
    pub name: &'static str,
    /// Index into [`BUS_GAINS`].
    pub bus: usize,
    pub wave: Wave,
    /// Notes sorted by `start`.
    pub notes: Vec<Note>,
}

/// Data, context, earcon and monitor buses (06 §3) with fixed gains.
pub const BUS_GAINS: [f32; 4] = [1.0, 0.5, 0.7, 0.35];
/// Master gain before the limiter.
pub const MASTER_GAIN: f32 = 0.25;

#[derive(Clone, Debug)]
pub struct Score {
    pub sample_rate: u32,
    pub frames: u64,
    pub tracks: Vec<Track>,
}

/// SplitMix64: integer-only, so the Score is identical on every platform.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1) from the top 24 bits (exact in f32).
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u32 << 24) as f32
    }
}

/// Builds the benchmark Score: `seconds` long, 10,000 events over four tracks.
pub fn benchmark(sample_rate: u32, seconds: u32, seed: u64) -> Score {
    let mut rng = Rng(seed);
    let sr = sample_rate as f32;
    let frames = u64::from(sample_rate) * u64::from(seconds);
    // (name, bus, wave, events, min/max length in ms, pitch range in Hz)
    let specs = [
        (
            "data-a",
            0,
            Wave::Sine,
            3000,
            (40.0, 160.0),
            (400.0, 1400.0),
        ),
        (
            "data-b",
            0,
            Wave::Square,
            3000,
            (40.0, 160.0),
            (200.0, 700.0),
        ),
        (
            "context",
            1,
            Wave::Triangle,
            2000,
            (20.0, 30.0),
            (1000.0, 1000.0),
        ),
        (
            "earcon",
            2,
            Wave::Sine,
            2000,
            (60.0, 120.0),
            (600.0, 1200.0),
        ),
    ];
    let tracks = specs
        .into_iter()
        .map(|(name, bus, wave, count, (lmin, lmax), (fmin, fmax))| {
            let mut level = 0.5f32;
            let mut notes: Vec<Note> = (0..count)
                .map(|_| {
                    // Random walk in log-frequency, clamped to the track's range (SDR-08 style).
                    level = (level + (rng.unit() - 0.5) * 0.2).clamp(0.0, 1.0);
                    let len_ms = lmin + (lmax - lmin) * rng.unit();
                    Note {
                        start: rng.next() % (frames - 9_600),
                        len: (len_ms / 1000.0 * sr) as u32,
                        freq: fmin * libm::powf(fmax / fmin, level),
                        gain: 0.1 + 0.2 * rng.unit(),
                        pan: rng.unit() * 2.0 - 1.0,
                    }
                })
                .collect();
            notes.sort_by_key(|n| n.start);
            Track {
                name,
                bus,
                wave,
                notes,
            }
        })
        .collect();
    Score {
        sample_rate,
        frames,
        tracks,
    }
}

/// Canonical little-endian bytes of the Score, for the byte-identity check (PB-06).
pub fn canonical_bytes(score: &Score) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&score.sample_rate.to_le_bytes());
    out.extend_from_slice(&score.frames.to_le_bytes());
    for t in &score.tracks {
        out.extend_from_slice(t.name.as_bytes());
        out.push(t.bus as u8);
        out.push(t.wave as u8);
        for n in &t.notes {
            out.extend_from_slice(&n.start.to_le_bytes());
            out.extend_from_slice(&n.len.to_le_bytes());
            for x in [n.freq, n.gain, n.pan] {
                out.extend_from_slice(&x.to_bits().to_le_bytes());
            }
        }
    }
    out
}

/// Test Score: the first 2 s of a benchmark Score (0.1 s under Miri, which is slow).
#[cfg(test)]
pub fn short_score() -> Score {
    let frames = if cfg!(miri) { 4_800 } else { 96_000 };
    let mut score = benchmark(48_000, 2, 7);
    for t in &mut score.tracks {
        t.notes.retain(|n| n.start < frames);
    }
    score.frames = frames;
    score
}
