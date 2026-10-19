// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The context every capability receives during a compile (05 §3): seeded random streams,
//! the diagnostics sink, cancellation and the budget. The concept table and user-profile
//! overrides join it with NUMISONA (A06) and profiles (B05).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};

use super::CompileError;
use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::digest::Sha256Digest;
use crate::ids::TrackId;
use crate::units::Seconds;

/// Limits that stop a runaway compile with a diagnostic instead of exhausting memory.
///
/// These are engineering safety limits, not research defaults.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Budget {
    /// The most events a Score may hold, across all tracks.
    pub max_events: u64,
    /// The longest Score duration.
    pub max_duration: Seconds,
}

impl Default for Budget {
    /// Ten million events and six hours.
    fn default() -> Self {
        Self {
            max_events: 10_000_000,
            max_duration: Seconds(6.0 * 3600.0),
        }
    }
}

/// A budget limit that was exceeded.
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum BudgetExceeded {
    /// Too many events.
    #[error("the Score would hold more than {limit} events")]
    Events {
        /// The limit.
        limit: u64,
    },
    /// Too long.
    #[error("the Score would last longer than {limit}")]
    Duration {
        /// The limit.
        limit: Seconds,
    },
}

/// Cancels a compile from another thread (the UI cancels stale compiles). Capabilities call
/// [`CompileContext::check_cancelled`] between stages and every few thousand rows.
#[derive(Clone, Debug, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// A token that is not cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests cancellation.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Whether cancellation was requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// A deterministic random stream for one track (SDR-02). Portable: the same seed and track
/// ID give the same numbers on every platform.
#[derive(Clone, Debug)]
pub struct TrackRng(ChaCha8Rng);

impl TrackRng {
    /// The stream for `track` under `seed`.
    ///
    /// Derivation (05 §3, fixed and versioned with the Score IR): `h` is the first 8 bytes
    /// of SHA-256(track ID, UTF-8) read as a little-endian `u64`; the stream key is
    /// SHA-256 of the little-endian bytes of `seed XOR h`; the generator is `ChaCha8` with
    /// that 32-byte key.
    #[must_use]
    pub fn new(seed: u64, track: &TrackId) -> Self {
        let digest = Sha256Digest::of(track.as_str().as_bytes());
        let (head, _) = digest
            .as_bytes()
            .split_first_chunk::<8>()
            .unwrap_or((&[0; 8], &[]));
        let derived = seed ^ u64::from_le_bytes(*head);
        let key = Sha256Digest::of(&derived.to_le_bytes());
        Self(ChaCha8Rng::from_seed(*key.as_bytes()))
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// A uniform number in [0, 1) with 53 random bits.
    pub fn next_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / 9_007_199_254_740_992.0; // 2^-53
        #[expect(
            clippy::cast_precision_loss,
            reason = "a 53-bit integer converts to f64 exactly"
        )]
        let bits = (self.next_u64() >> 11) as f64;
        bits * SCALE
    }
}

/// The per-compile context passed to every capability.
#[derive(Debug)]
pub struct CompileContext {
    seed: u64,
    budget: Budget,
    cancel: CancelToken,
    events: u64,
    diagnostics: Diagnostics,
}

impl CompileContext {
    /// A context for a compile with `seed`.
    #[must_use]
    pub fn new(seed: u64, budget: Budget, cancel: CancelToken) -> Self {
        Self {
            seed,
            budget,
            cancel,
            events: 0,
            diagnostics: Diagnostics::new(),
        }
    }

    /// The spec seed (SDR-02).
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The random stream for `track`; see [`TrackRng::new`].
    #[must_use]
    pub fn rng(&self, track: &TrackId) -> TrackRng {
        TrackRng::new(self.seed, track)
    }

    /// Reports a diagnostic. Errors reported here stop the compile after the current stage.
    pub fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// The diagnostics reported so far, in order.
    #[must_use]
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Takes the diagnostics, leaving none.
    pub fn take_diagnostics(&mut self) -> Diagnostics {
        std::mem::take(&mut self.diagnostics)
    }

    /// The budget.
    #[must_use]
    pub fn budget(&self) -> Budget {
        self.budget
    }

    /// Fails if cancellation was requested.
    ///
    /// # Errors
    ///
    /// [`CompileError::Cancelled`].
    pub fn check_cancelled(&self) -> Result<(), CompileError> {
        if self.cancel.is_cancelled() {
            Err(CompileError::Cancelled)
        } else {
            Ok(())
        }
    }

    /// Counts `events` new Score events against the budget.
    ///
    /// # Errors
    ///
    /// [`CompileError::Budget`] when the total exceeds [`Budget::max_events`].
    pub fn charge_events(&mut self, events: u64) -> Result<(), CompileError> {
        self.events = self.events.saturating_add(events);
        if self.events > self.budget.max_events {
            Err(BudgetExceeded::Events {
                limit: self.budget.max_events,
            }
            .into())
        } else {
            Ok(())
        }
    }

    /// Checks a Score duration against the budget.
    ///
    /// # Errors
    ///
    /// [`CompileError::Budget`] when `duration` exceeds [`Budget::max_duration`] or is not
    /// finite.
    pub fn check_duration(&self, duration: Seconds) -> Result<(), CompileError> {
        if duration.0.is_finite() && duration.0 <= self.budget.max_duration.0 {
            Ok(())
        } else {
            Err(BudgetExceeded::Duration {
                limit: self.budget.max_duration,
            }
            .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SDR-02: the same seed and track give the same stream, on every platform (this test
    /// runs on all CI OSes), and different tracks get different streams.
    #[test]
    fn sdr_02_track_streams_are_seeded_and_portable() {
        let anomaly = TrackId::new("anomaly");
        let mut a = TrackRng::new(1729, &anomaly);
        let mut b = TrackRng::new(1729, &anomaly);
        let first: Vec<u64> = (0..3).map(|_| a.next_u64()).collect();
        assert_eq!(first, (0..3).map(|_| b.next_u64()).collect::<Vec<_>>());
        assert_eq!(
            first,
            [
                2_958_691_992_523_149_899,
                14_457_789_064_745_963_337,
                9_122_797_636_853_457_438
            ]
        );
        let mut other = TrackRng::new(1729, &TrackId::new("co2"));
        assert_ne!(other.next_u64(), first[0]);
        let mut reseeded = TrackRng::new(1730, &anomaly);
        assert_ne!(reseeded.next_u64(), first[0]);
    }

    #[test]
    fn uniform_numbers_lie_in_the_unit_interval() {
        let mut rng = TrackRng::new(0, &TrackId::new("t"));
        assert!(
            (0..1000)
                .map(|_| rng.next_f64())
                .all(|x| (0.0..1.0).contains(&x))
        );
    }

    #[test]
    fn budgets_and_cancellation_fail_safely() {
        let cancel = CancelToken::new();
        let budget = Budget {
            max_events: 10,
            max_duration: Seconds(60.0),
        };
        let mut cx = CompileContext::new(0, budget, cancel.clone());
        assert!(cx.charge_events(10).is_ok());
        assert!(matches!(
            cx.charge_events(1),
            Err(CompileError::Budget(BudgetExceeded::Events { limit: 10 }))
        ));
        assert!(cx.check_duration(Seconds(60.0)).is_ok());
        assert!(cx.check_duration(Seconds(f64::NAN)).is_err());
        assert!(cx.check_cancelled().is_ok());
        cancel.cancel();
        assert!(matches!(cx.check_cancelled(), Err(CompileError::Cancelled)));
    }
}
