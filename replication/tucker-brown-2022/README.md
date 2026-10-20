# Tucker Brown et al. (2022): transit detection by signal-to-noise ratio

> **Source:** Tucker Brown, J., Harrison, C. M., Zanella, A. & Trayford, J. (2022). Evaluating the efficacy of sonification for signal detection in univariate, evenly sampled light curves using astronify. *MNRAS* 516(4):5674–5683. [doi:10.1093/mnras/stac2590](https://doi.org/10.1093/mnras/stac2590) (`tuckerbrown2022evaluating`). Read from the accepted version, arXiv:2209.04465v1; findings F-010 and F-112.

Listeners counted transit-like dips in light curves at six signal-to-noise ratios. The study's set had 18 curves: at each SNR, curves with one and two transits, and six curves without a signal. The gallery has the same 18: `snr<SNR>-n<transits>.json` for SNR 3, 5, 7, 10, 30 and 100 and 0, 1 or 2 transits, each with its data in `data/` from GEN-003 (seed 20261019 plus the curve's position, so no two curves share their noise; manifests in [stimuli.json](stimuli.json)). The specs are generated from one template by `cargo xtask replication`.

## Published and reproduced parameters

"Reproduced" values are read from the committed render records in [manifests.json](manifests.json) and the stimulus manifests.

| Parameter | Published (arXiv v1 section) | Reproduced | Match |
|---|---|---|---|
| Curve | 1000 points; box transits 10% of the length wide; "fractional transit depth" 25; Gaussian noise (§2.1) | 1000 points; 100-point box dips 25 flux units deep on a baseline of 100 | Yes; the depth units are inferred (F-112) |
| SNR | SNR = t_d·√N_t/σ; levels 3, 5, 7, 10, 30, 100 (§2.1) | σ = 250/SNR; the SNR the drawn noise reached is in each stimulus's truth (2.95–3.05 at SNR 3, 97.1–99.0 at SNR 100) | Yes |
| Transits | 0, 1 or 2 per curve; 18 curves, 6 without a signal (§2.1) | The same 18 | Yes |
| Positions | Not stated | Seeded, non-overlapping, at least 50 points from the ends and from each other | Not published |
| Sonification | astronify, positive polarity, each curve's range linear onto 440–880 Hz, one tone per point, 0.01 spacing, 0.2 note length (§2.2) | 1000 sine notes of 0.2 s at 0.00–9.99 s, 440.0–880.0 Hz linear in frequency; 10.19 s renders | Yes; timbre not stated |
| Answers | 0, 1, 2, 3, 4 or N/A (§3.1) | GEN-003's truth gives the count | Yes (for the study runner, A08-T03) |

## Deviations

- **Positions and zero-signal noise.** The paper states neither, so positions come from the seed, and a curve without transits has the noise of its SNR level (σ = 250/SNR).
- **Timbre.** The paper names astronify's default mapping but not its waveform; the specs use a sine. The authors' audio is archived ([doi:10.25405/data.ncl.20936749](https://doi.org/10.25405/data.ncl.20936749)) and could settle this (not checked).
- **Level.** About 20 notes overlap at any moment, so each note is at −24 dB; the paper gives no level.
