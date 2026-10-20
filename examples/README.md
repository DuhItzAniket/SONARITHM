# Examples gallery

Five small datasets, each with a spec that shows one part of SONARITHM's design. The data is synthetic and generated from seeded formulas (`cargo xtask gallery` writes it, and CI checks that it regenerates byte for byte), so it contains no personal data and sounds the same on every computer. The datasets are registered as DS-001 to DS-005 in [datasets.md](../docs/research-paper/datasets.md).

Render any of them to a file, or play it:

```bash
sonarithm render examples/specs/ds-001-temperature.spec.json -o temperature.flac
```

```bash
sonarithm play examples/specs/ds-003-regions.spec.json
```

`sonarithm inspect --describe` prints what each one encodes, in text. In Python, [gallery.ipynb](gallery.ipynb) compiles, describes and plays all five. CI renders every spec on each pull request; the FLAC files are attached to the `ci` run as the `gallery-renders` artifact.

| Dataset | Spec | What it shows | Listen for |
|---|---|---|---|
| DS-001 Daily temperature (120 days) | [ds-001-temperature](specs/ds-001-temperature.spec.json) | Pitch for a value, with the temperature concept setting positive polarity (SDR-12); the 400–1400 Hz default range (SDR-08) | A slow rise with small jitter, highest near the end |
| DS-002 Weekly price (20 weeks) | [ds-002-prices](specs/ds-002-prices.spec.json) | A concept listeners disagree about (money, SDR-14) with its polarity set explicitly; duration as a redundant channel (A11Y-13) | Ten weeks up, then down; higher prices also last longer |
| DS-003 Rainfall in three regions | [ds-003-regions](specs/ds-003-regions.spec.json) | Concurrent series separated by timbre and position (SDR-26, A11Y-14) | Three voices, left, centre and right, peaking in different months |
| DS-004 Fruit sold | [ds-004-fruit](specs/ds-004-fruit.spec.json) | A bar chart: one tone per bar, a timbre per category (SDR-25), pitch from zero, redundant duration | Five tones; the second (banana) highest and longest |
| DS-005 Dense signal | [ds-005-signal](specs/ds-005-signal.spec.json) | The experimental density switch (SDR-20): 200 points per second become one continuous contour | Low noise with three sharp bursts |

Context cues (axis ticks, the reference tone, the spoken legend) arrive with AUDAXIS in phase B03; until then each compile reports their absence as warning `SNR-COMP-023`.

## Study protocols

[studies/pilot-001](studies/pilot-001/README.md) is the pilot protocol of `sonarithm study run`, ECHOMETRIC's study runner: two conditions after Smith & Walker (2005), point estimation, a practice and a main block. It is a software pilot, not a study: no participant data is collected as research until the project has an ethics review route (OD-04).

## Licence

The datasets (`data/`), specs (`specs/`), the study protocols (`studies/`) and the notebook are licensed CC BY 4.0 (ADR-0010). Credit "SONARITHM contributors". The code that generates the data (`cargo xtask gallery`, `sonarithm_numisona::examples`) is MIT OR Apache-2.0.
