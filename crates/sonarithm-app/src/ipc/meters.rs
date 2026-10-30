// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Meter frames (B01-T04, ADR-0017 §4): the master's level for the UI's meters, sent on
//! the `meters` channel as one 32-byte little-endian frame at most 30 times a second while
//! playing. Meters are a hot path, so a frame is never JSON (09 §2): it is not an
//! [`Event`](super::Event), and it reaches the host through
//! [`EventSink::meters`](crate::EventSink::meters) only.
//!
//! | Offset | Size | Field |
//! |---|---|---|
//! | 0 | 4 | [`METER_MAGIC`] `SNRM` |
//! | 4 | 2 | [`METER_VERSION`] |
//! | 6 | 2 | Flags: bit 0 [`METER_FLAG_LIMITING`] |
//! | 8 | 8 | Sequence number (u64) |
//! | 16 | 8 | Transport position in seconds (f64) |
//! | 24 | 4 | Peak, left (f32, linear; 1.0 is full scale) |
//! | 28 | 4 | Peak, right (f32) |

/// The first four bytes of every meter frame.
pub const METER_MAGIC: [u8; 4] = *b"SNRM";
/// The meter frame layout version. A layout change increments it.
pub const METER_VERSION: u16 = 1;
/// The size of a meter frame in bytes.
pub const METER_BYTES: usize = 32;
/// Flag bit: the master limiter engaged in the frame's window (06 §10).
pub const METER_FLAG_LIMITING: u16 = 1;

/// Byte offsets of the meter frame fields.
pub mod offset {
    /// [`super::METER_MAGIC`].
    pub const MAGIC: usize = 0;
    /// [`super::METER_VERSION`].
    pub const VERSION: usize = 4;
    /// Flags.
    pub const FLAGS: usize = 6;
    /// Sequence number.
    pub const SEQ: usize = 8;
    /// Transport position in seconds.
    pub const POSITION: usize = 16;
    /// Left peak.
    pub const PEAK_LEFT: usize = 24;
    /// Right peak.
    pub const PEAK_RIGHT: usize = 28;
}

/// One meter reading of the master.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct MeterFrame {
    /// Counts up with every frame of the session.
    pub seq: u64,
    /// The transport position of the reading in seconds.
    pub position_s: f64,
    /// Peak absolute sample, left and right, over the last meter window (linear).
    pub peak: [f32; 2],
    /// Whether the master limiter engaged in that window.
    pub limiting: bool,
}

impl MeterFrame {
    /// A frame.
    #[must_use]
    pub fn new(seq: u64, position_s: f64, peak: [f32; 2], limiting: bool) -> Self {
        Self {
            seq,
            position_s,
            peak,
            limiting,
        }
    }

    /// The frame's bytes.
    #[must_use]
    pub fn encode(&self) -> [u8; METER_BYTES] {
        let mut out = [0_u8; METER_BYTES];
        let mut put = |at: usize, bytes: &[u8]| {
            if let Some(slot) = out.get_mut(at..at + bytes.len()) {
                slot.copy_from_slice(bytes);
            }
        };
        put(offset::MAGIC, &METER_MAGIC);
        put(offset::VERSION, &METER_VERSION.to_le_bytes());
        let flags = if self.limiting {
            METER_FLAG_LIMITING
        } else {
            0
        };
        put(offset::FLAGS, &flags.to_le_bytes());
        put(offset::SEQ, &self.seq.to_le_bytes());
        put(offset::POSITION, &self.position_s.to_le_bytes());
        put(offset::PEAK_LEFT, &self.peak[0].to_le_bytes());
        put(offset::PEAK_RIGHT, &self.peak[1].to_le_bytes());
        out
    }

    /// Reads a frame; `None` for another magic or version, or the wrong length.
    #[must_use]
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != METER_BYTES || bytes.get(..4)? != METER_MAGIC {
            return None;
        }
        let u16_at = |at: usize| Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?));
        let u64_at = |at: usize| Some(u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?));
        let f32_at = |at: usize| Some(f32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
        if u16_at(offset::VERSION)? != METER_VERSION {
            return None;
        }
        Some(Self {
            seq: u64_at(offset::SEQ)?,
            position_s: f64::from_bits(u64_at(offset::POSITION)?),
            peak: [f32_at(offset::PEAK_LEFT)?, f32_at(offset::PEAK_RIGHT)?],
            limiting: u16_at(offset::FLAGS)? & METER_FLAG_LIMITING != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_round_trips() {
        let frame = MeterFrame::new(7, 1.25, [0.5, 0.125], true);
        let bytes = frame.encode();
        assert_eq!(&bytes[..4], b"SNRM");
        assert_eq!(MeterFrame::decode(&bytes), Some(frame));
    }

    #[test]
    fn the_layout_is_little_endian_at_its_offsets() {
        let bytes = MeterFrame::new(0x0102, 0.0, [1.0, 0.0], false).encode();
        assert_eq!(bytes[offset::VERSION], 1);
        assert_eq!(bytes[offset::SEQ], 0x02);
        assert_eq!(bytes[offset::SEQ + 1], 0x01);
        assert_eq!(
            &bytes[offset::PEAK_LEFT..offset::PEAK_LEFT + 4],
            &1.0_f32.to_le_bytes()
        );
    }

    #[test]
    fn other_magics_versions_and_lengths_are_refused() {
        let mut bytes = MeterFrame::default().encode();
        assert!(MeterFrame::decode(&bytes[..31]).is_none());
        bytes[offset::VERSION] = 2;
        assert!(MeterFrame::decode(&bytes).is_none());
        bytes[offset::VERSION] = 1;
        bytes[0] = b'X';
        assert!(MeterFrame::decode(&bytes).is_none());
    }
}
