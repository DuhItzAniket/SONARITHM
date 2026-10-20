# Nees & Walker (2008): data density × trend reversals

> **Source:** Nees, M. A. & Walker, B. N. (2008). Data density and trend reversals in auditory graphs: Effects on point-estimation and trend-identification tasks. *ACM Transactions on Applied Perception* 5(3), Art. 13. [doi:10.1145/1402236.1402237](https://doi.org/10.1145/1402236.1402237) (`nees2008density`). Read from the author PDF; findings F-012 and F-110.

The study crossed four data densities (1, 2, 4 and 8 points per second) with four numbers of trend reversals (0, 1, 2 and 3). One spec per cell, `d<density>-r<reversals>.json`, each with its data in `data/` from GEN-001 (`shape = "schematic"`, `start = "up"`, seed 20261019; the stimulus manifests are in [stimuli.json](stimuli.json)). The specs are generated from one template by `cargo xtask replication`, so the sixteen differ only in their data file and name.

## Published and reproduced parameters

"Reproduced" values are read from the committed render records in [manifests.json](manifests.json).

| Parameter | Published (section, page) | Reproduced | Match |
|---|---|---|---|
| Densities | 1, 2, 4, 8 points/s (§2.1.3, p. 13:8) | 11, 21, 41 and 81 notes over 0–10 s | Yes; the point counts are inferred (below) |
| Reversals | 0, 1, 2, 3; initially rising or falling, counterbalanced (§2.1.3, pp. 13:8–13:9) | 0–3 reversals, initially rising | Levels yes; only the rising versions (below) |
| Day | 10 hours, 8 a.m. to 6 p.m., one second per hour (§2.1.3–2.1.4) | One second per hour | Yes |
| Range | $6 to $106, every graph reaching both (pp. 13:8, 13:10) | Every series spans exactly $6–$106 | Yes |
| Mapping | $6 → MIDI G2 (98 Hz), $106 → MIDI B6, chromatic exact scale, positive polarity (§2.1.4, pp. 13:9–13:10) | Linear $6–$106 onto MIDI 43–95 in log frequency: 98.00–1975.53 Hz | Yes (the paper prints 1979.5 Hz for B6; equal temperament gives 1975.53 Hz, F-110) |
| Tones | 125 ms, 10 ms onset and offset ramps; 10.125 s per graph (§2.1.4, p. 13:9) | 125 ms notes; 10.125 s renders | Length yes; ramps differ (below) |
| Timbre | MIDI piano, centre mix (§2.1.4) | NUMISONA's default mallet timbre (SDR-29), centred | No (below) |
| Context | No reference tones or clicks; opening price told on each trial (§2.1.5) | None | Yes |
| Queries | Every hour per cell in random order (point estimation); each hourly interval: increased, decreased, stayed the same (trend identification) | GEN-001's ground truth lists the 11 hours and the 10 intervals in seeded order | Yes (for the study runner, A08-T03) |
| Level | About 65 dB SPL (§2.1.2) | Not set by a render; calibrated at playback | Not applicable |

## Deviations

- **Shapes.** The paper publishes no generating function, and its Fig. 1 is labelled schematic. GEN-001's schematic shapes are read from that figure: a ramp for 0 reversals, (1 − cos 2πu)/2 for 1, sin 2πu for 2, sin 3πu for 3, each min–max scaled to $6–$106 (F-110). They are a reading, not the authors' data.
- **Point counts.** The paper gives no count; 10·d + 1 points fit the 10.125 s graphs and the dots in Fig. 1 (F-110).
- **Initial direction.** Only the initially rising versions are in the gallery; GEN-001 makes the falling ones with `start = "down"`.
- **Ramps.** The engine's note ramps are min(40 ms, 25% of the note), never below 5 ms (SDR-22), so these 125 ms notes get 31.25 ms ramps instead of the published 10 ms. Spec v0.1 has no envelope setting.
- **Timbre.** The release has no piano; the default harmonic mallet timbre stands in for the MIDI piano (SDR-29).
