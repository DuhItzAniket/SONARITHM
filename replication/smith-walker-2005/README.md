# Smith & Walker (2005): auditory context cues

> **Source:** Smith, D. R. & Walker, B. N. (2005). Effects of auditory context cues and training on performance of a point estimation sonification task. *Applied Cognitive Psychology* 19(8):1065–1087. [doi:10.1002/acp.1146](https://doi.org/10.1002/acp.1146) (`smith2005effects`). Read from the author PDF; findings F-003 and F-111.

Study 1 crossed x-axis context (none, clicks) with y-axis context (none, static reference, dynamic reference) on one auditory graph of a stock price. One spec per condition:

| Spec | x axis | y axis |
|---|---|---|
| [none.json](none.json) | none | none |
| [clicks.json](clicks.json) | clicks | none |
| [static.json](static.json) | none | static reference |
| [dynamic.json](dynamic.json) | none | dynamic reference |
| [clicks-static.json](clicks-static.json) | clicks | static reference |
| [clicks-dynamic.json](clicks-dynamic.json) | clicks | dynamic reference |

The data, [data.csv](data.csv), comes from GEN-002 (`series = "reconstructed"`, seed 20261019; its manifest is [stimulus.json](stimulus.json)). The specs sound the context cues as explicit tracks over precomputed columns, because the context providers that will play ticks and reference tones from `context` arrive with AUDAXIS in B03.

## Published and reproduced parameters

"Reproduced" values are read from the committed render records in [manifests.json](manifests.json).

| Parameter | Published (page) | Reproduced | Match |
|---|---|---|---|
| Series | One stock over a 10-hour day, two values per hour; $50 open, $84 maximum, $10 minimum, close equal to the open (p. 1069) | 21 values from 8:00 to 18:00; open $50, maximum $84 at 10:30, minimum $10 at 15:00, close $50 | Anchors exact; values in between reconstructed (below) |
| Data tones | Twenty sine tones of 0.5 s, back to back, 10 s (p. 1069) | 20 sine notes of 0.5 s at 0.0–9.5 s; 10.0 s render | Yes |
| Ramps | 40 ms up and down (p. 1069) | The render's note envelope (about 40 ms raised-cosine ramps, SDR-22) | Yes, by the engine default |
| Mapping | $ = c · f^0.9256, c = 0.0765, so $50 → 1100 Hz; "200–1900 Hz" (p. 1069) | Psychophysical scale, exponent 0.9256 through ($50, 1100.535 Hz); $10 → 193.40 Hz, $84 → 1927.63 Hz | Yes: the formula's exact values; the paper's range is rounded (F-111) |
| Polarity | Positive (p. 1069) | Positive, set explicitly (money is a flagged concept, SDR-14) | Yes |
| Clicks | 10 ms broadband, one per hour (1/s), with every other data point (p. 1069) | 10 notes of 10 ms at 0–9 s, on the whole-hour rows | Timing yes; sound differs (below) |
| Static reference | Triangle-wave "beeping" tone at the opening price for the whole graph (p. 1069) | Triangle notes at 1100.54 Hz ($50), one per data point | Pitch and timbre yes; beep timing not published |
| Dynamic reference | Triangle tone at the day's maximum while the price rises, minimum while it falls; starts at the maximum (p. 1069) | Triangle notes alternating 1927.63 Hz ($84) and 193.40 Hz ($10) by the direction of the step into each data point | Yes, with the turn point SONARITHM's choice (below) |
| Query | Each hour 8 a.m.–6 p.m. in random order, 11 per test (p. 1070) | GEN-002's ground truth lists the 11 hours in seeded order | Yes (for the study runner, A08-T03) |
| Level | About 60 dB SPL over headphones (pp. 1068–1069) | Not set by a render; calibrated at playback | Not applicable |

## Deviations

- **Series values.** The paper does not tabulate the series. GEN-002 rebuilds it from the stated anchors and the hourly values plotted in its Fig. 2 (50, 65, 80, 76, 61, 44, 28, 10, 23, 40, 50; read to about ±$2), with half hours halfway between and the $84 maximum at 10:30, the only place Fig. 2 leaves room for it (F-111).
- **Tone count.** Study 1 reports 20 tones while Study 2 glides through 21 values. The data has 21 rows so that the 6 p.m. close can be queried; the specs filter `hour < 18.0` to sound Study 1's 20 tones, which places the clicks on the hours 8 to 17.
- **Clicks.** The engine has no noise source in this release, so a click is a 10 ms square-wave note at 3 kHz, 6 dB below the data. The paper does not state the click level.
- **Reference beeps.** The paper does not state the beep rate, length or level. Each reference note lasts 0.25 s (half the data interval) at 6 dB below the data. The paper does not say on which tone the dynamic reference turns; SONARITHM uses the direction of the step into each point, so the peak itself sounds the maximum.
- **Context as tracks.** The cues are explicit tracks over the `click`, `ref-static` and `ref-dynamic` columns, not `context` settings, until AUDAXIS provides them.
