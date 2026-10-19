# Erie gallery subset (import fixtures)

These Erie specs test NUMISONA's Erie importer (A06-T08, 04 §9). Each is modelled on an example of the [Erie editor gallery](https://see-mike-out.github.io/erie-editor) and uses the constructs that example shows. They were written for SONARITHM from Erie's published [documentation](https://see-mike-out.github.io/erie-documentation); no Erie source code or gallery file was copied. The data is synthetic.

| Fixture | Gallery example | Constructs |
|---|---|---|
| `time-series.json` | Time Series | continued tone, absolute time with `length`, pitch range as note names, `scale.title` |
| `histogram.json` | Histogram | `bin` and `aggregate` transforms, relative time with `band`, `scale.zero`, `config`, a file URL |
| `sparsity-simple.json` | Sparsity (simple) | `tapCount`, nominal time, `speechBefore` |
| `sparsity-and-length.json` | Sparsity and Length | `tapSpeed`, `duration` |
| `interval.json` | Interval | `time2`, explicit domain and range, constant `loudness` |
| `model-fit-overlay.json` | Model fit | `overlay` of a discrete and a continued stream over one dataset, constant `pan` |
| `residuals-sequence.json` | Regression Residuals (bad, good) | `sequence` with named `datasets` |
| `frequency.json` | Frequency | `repeat`, `roundToNote`, a reversed range with negative polarity, JavaScript expressions, `fold`, tone filters, 3D pan, `config` |
