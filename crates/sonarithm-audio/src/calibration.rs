// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Per-device latency calibration (ADR-0016, 06 §9).
//!
//! A backend's own latency estimate is only a fallback: A02 found cpal's prediction about
//! 80 ms earlier than the sound on Windows (F-033). A calibration stores, per device, the
//! measured offset between the time the engine predicts a frame is heard (the host time it
//! publishes with each block) and the time it actually is. The playhead anchors that the
//! telemetry pump sends to the UI add it. Offsets come from the loopback self-test or a
//! cable measurement (`latency` module, Windows), or are entered by hand.

use std::collections::BTreeMap;
use std::time::Duration;

/// Measured output offsets, per device identifier.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LatencyCalibration {
    offsets: BTreeMap<String, Duration>,
}

impl LatencyCalibration {
    /// An empty table: every device uses its backend's estimate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the offset measured for `device`, replacing any earlier one.
    pub fn set(&mut self, device: &str, offset: Duration) {
        self.offsets.insert(device.to_owned(), offset);
    }

    /// The offset recorded for `device`, if any.
    #[must_use]
    pub fn offset(&self, device: &str) -> Option<Duration> {
        self.offsets.get(device).copied()
    }

    /// When a frame is heard: the engine's predicted time plus the device's calibrated
    /// offset, or the prediction alone for an uncalibrated device.
    #[must_use]
    pub fn heard_at(&self, device: &str, predicted: Duration) -> Duration {
        predicted + self.offset(device).unwrap_or_default()
    }

    /// The devices with an offset, in order.
    pub fn devices(&self) -> impl Iterator<Item = (&str, Duration)> {
        self.offsets
            .iter()
            .map(|(device, offset)| (device.as_str(), *offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibrated_devices_add_their_offset() {
        let mut calibration = LatencyCalibration::new();
        calibration.set("speakers", Duration::from_millis(71));
        let predicted = Duration::from_millis(1_000);
        assert_eq!(
            calibration.heard_at("speakers", predicted),
            Duration::from_millis(1_071)
        );
        assert_eq!(calibration.heard_at("headset", predicted), predicted);
        calibration.set("speakers", Duration::from_millis(68));
        assert_eq!(
            calibration.devices().collect::<Vec<_>>(),
            [("speakers", Duration::from_millis(68))]
        );
    }
}
