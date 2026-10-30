// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Meter frames (B01-T04, ADR-0017 §4 "Meter frames"): the master's level for the UI's
//! meters, sent on the `meters` channel as one 32-byte little-endian frame at most every
//! 1/30 s while playing, and one last frame when playback stops. Meters are a hot path, so
//! a frame is never JSON (09 §2): it is not an [`Event`](super::Event), and it reaches the
//! host through [`EventSink::meters`](crate::EventSink::meters) only.
//!
//! | Offset | Size | Field |
//! |---|---|---|
//! | 0 | 4 | [`METER_MAGIC`] `SNRM` |
//! | 4 | 2 | [`METER_VERSION`] |
//! | 6 | 2 | Flags: bit 0 [`METER_FLAG_LIMITING`], bit 1 [`METER_FLAG_LAST`]; the other bits are zero, and readers ignore them |
//! | 8 | 8 | Sequence number (u64) |
//! | 16 | 8 | Transport position in seconds (f64) |
//! | 24 | 4 | Peak, left (f32, linear; 1.0 is full scale) |
//! | 28 | 4 | Peak, right (f32) |
//!
//! A change of size, field or meaning increments [`METER_VERSION`]; a new flag bit does not.

use crate::codes;
use crate::ipc::IpcError;

/// The first four bytes of every meter frame.
pub const METER_MAGIC: [u8; 4] = *b"SNRM";
/// The meter frame layout version. A layout change increments it.
pub const METER_VERSION: u16 = 1;
/// The size of a meter frame in bytes.
pub const METER_BYTES: usize = 32;
/// Flag bit 0: the master limiter engaged in the frame's window (06 §10).
pub const METER_FLAG_LIMITING: u16 = 1;
/// Flag bit 1: the last frame before the transport stopped, paused or lost its device; its
/// peaks are zero, so a meter falls to silence instead of freezing at its last reading.
pub const METER_FLAG_LAST: u16 = 2;

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
    /// The last frame before playback stopped ([`METER_FLAG_LAST`]).
    pub last: bool,
}

impl MeterFrame {
    /// A reading while playing.
    #[must_use]
    pub fn new(seq: u64, position_s: f64, peak: [f32; 2], limiting: bool) -> Self {
        Self {
            seq,
            position_s,
            peak,
            limiting,
            last: false,
        }
    }

    /// The last frame when playback stops: zero peaks, [`METER_FLAG_LAST`] set.
    #[must_use]
    pub fn last(seq: u64, position_s: f64) -> Self {
        Self {
            last: true,
            ..Self::new(seq, position_s, [0.0; 2], false)
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
        let mut flags = 0;
        if self.limiting {
            flags |= METER_FLAG_LIMITING;
        }
        if self.last {
            flags |= METER_FLAG_LAST;
        }
        put(offset::FLAGS, &flags.to_le_bytes());
        put(offset::SEQ, &self.seq.to_le_bytes());
        put(offset::POSITION, &self.position_s.to_le_bytes());
        put(offset::PEAK_LEFT, &self.peak[0].to_le_bytes());
        put(offset::PEAK_RIGHT, &self.peak[1].to_le_bytes());
        out
    }

    /// Reads a frame. Unknown flag bits are ignored.
    ///
    /// # Errors
    ///
    /// `SNR-APP-050` for another length, magic or version: the reader never guesses
    /// (ADR-0017 §6).
    pub fn decode(bytes: &[u8]) -> Result<Self, IpcError> {
        let refuse = |reason: &str| {
            IpcError::new(
                &codes::BINARY_VIEW,
                format!("Not a meter frame this version reads: {reason}."),
            )
        };
        if bytes.len() != METER_BYTES {
            return Err(refuse(&format!("{} bytes", bytes.len())));
        }
        if bytes.get(..4) != Some(METER_MAGIC.as_slice()) {
            return Err(refuse("wrong magic bytes"));
        }
        let field = |at: usize, len: usize| bytes.get(at..at + len).unwrap_or(&[]);
        let u16_at = |at: usize| u16::from_le_bytes(field(at, 2).try_into().unwrap_or_default());
        let u64_at = |at: usize| u64::from_le_bytes(field(at, 8).try_into().unwrap_or_default());
        let f32_at = |at: usize| f32::from_le_bytes(field(at, 4).try_into().unwrap_or_default());
        if u16_at(offset::VERSION) != METER_VERSION {
            return Err(refuse("unknown version"));
        }
        let flags = u16_at(offset::FLAGS);
        Ok(Self {
            seq: u64_at(offset::SEQ),
            position_s: f64::from_bits(u64_at(offset::POSITION)),
            peak: [f32_at(offset::PEAK_LEFT), f32_at(offset::PEAK_RIGHT)],
            limiting: flags & METER_FLAG_LIMITING != 0,
            last: flags & METER_FLAG_LAST != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b01_t04_a_frame_round_trips() {
        for frame in [
            MeterFrame::new(7, 1.25, [0.5, 0.125], true),
            MeterFrame::last(8, 1.3),
        ] {
            let bytes = frame.encode();
            assert_eq!(&bytes[..4], b"SNRM");
            assert_eq!(MeterFrame::decode(&bytes), Ok(frame));
        }
    }

    #[test]
    fn b01_t04_the_layout_is_little_endian_at_its_offsets() {
        let bytes = MeterFrame::new(0x0102, 0.0, [1.0, 0.0], false).encode();
        assert_eq!(bytes[offset::VERSION], 1);
        assert_eq!(bytes[offset::SEQ], 0x02);
        assert_eq!(bytes[offset::SEQ + 1], 0x01);
        assert_eq!(
            &bytes[offset::PEAK_LEFT..offset::PEAK_LEFT + 4],
            &1.0_f32.to_le_bytes()
        );
        let last = MeterFrame::last(1, 0.0).encode();
        assert_eq!(last[offset::FLAGS], 2);
        assert_eq!(&last[offset::PEAK_LEFT..], &[0; 8]);
    }

    #[test]
    fn b01_t04_a_short_buffer_a_wrong_magic_or_version_is_snr_app_050() {
        let mut bytes = MeterFrame::default().encode();
        let code = |result: Result<MeterFrame, IpcError>| result.err().map(|err| err.code);
        let expected = Some(codes::BINARY_VIEW.code.to_owned());
        assert_eq!(code(MeterFrame::decode(&bytes[..31])), expected);
        bytes[offset::VERSION] = 2;
        assert_eq!(code(MeterFrame::decode(&bytes)), expected);
        bytes[offset::VERSION] = 1;
        bytes[0] = b'X';
        assert_eq!(code(MeterFrame::decode(&bytes)), expected);
    }

    #[test]
    fn b01_t04_unknown_flag_bits_are_ignored() {
        let mut bytes = MeterFrame::new(1, 0.5, [0.25, 0.25], true).encode();
        bytes[offset::FLAGS] |= 0b1000_0100;
        bytes[offset::FLAGS + 1] = 0xff;
        let frame = MeterFrame::decode(&bytes).map_err(|err| err.code);
        assert_eq!(frame, Ok(MeterFrame::new(1, 0.5, [0.25, 0.25], true)));
    }
}
