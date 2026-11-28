# Pinned reference results

`augurs-changepoint-0.10.3.json` holds the change points that **augurs-changepoint 0.10.3** finds on ECHOMETRIC's GEN-012 series. `tests/augurs.rs` compares ECHOMETRIC's PELT change points with them.

- **Detector:** `NormalGammaDetector::default()`, a BOCPD with hazard λ = 250 and the prior NormalGamma(0, 1, 1, 1), through `Detector::detect_changepoints`. Indices are rows, and index 0 is dropped.
- **Series:** GEN-012 version 1 with `ShiftParams::default()` (300 points, 3 shifts of 6σ, spacing 30, noise σ = 1), seeds 0 to 49. Each key is the seed.
- **Written:** 2026-11-26, on HW-01 (Windows 11).

**Why a fixture, not a dependency (ADR-0031's fallback).** augurs-changepoint's dependencies (argmin, through rv) carry unmaintained-crate advisories: bincode, instant and paste. `cargo deny check` refuses them, and the advisory policy is not this crate's to change. augurs-seasons has no such advisory, so it stays a live dev-dependency.

**Regenerate** after GEN-012's version changes, or to check against a newer augurs:

1. Add `augurs-changepoint = "<version>"` to this crate's `[dev-dependencies]`. Do not commit that change.
2. Add a temporary ignored test that writes the file. For each seed in 0..50, build `generate(&Params::Shifts(ShiftParams::default()), seed)`, take its `value` column, run `NormalGammaDetector::default().detect_changepoints(&values)`, drop index 0, and write `{ "<seed>": [rows…] }`.
3. Run it: `cargo nextest run -p sonarithm-echometric-reference --run-ignored only <name>`.
4. Rename the file to the augurs version, update `tests/augurs.rs`'s `include_str!` and this README, and remove the dependency and the temporary test.
