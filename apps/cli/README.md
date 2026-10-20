# sonarithm-cli

The `sonarithm` command line tool of [SONARITHM](https://github.com/DuhItzAniket/SONARITHM): it turns data into structured, navigable sound.

```bash
cargo install sonarithm-cli
sonarithm quick data.csv --x year --y anomaly            # play the default design
sonarithm render temps.spec.json -o temps.flac           # + temps.render.json and temps.cues.vtt
sonarithm --plain inspect --describe temps.spec.json     # what it encodes, as text
```

Every command takes `--json` for machine-readable output and `--plain` for linear text without colour or box drawing, for screen readers. Exit codes: 0 success, 1 errors, 2 usage, 3 warnings denied. On Linux, building needs the ALSA development files (`libasound2-dev`).

Licence: MIT OR Apache-2.0.
