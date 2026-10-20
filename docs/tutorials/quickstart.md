# Quickstart: your first sonification in five minutes

> **Status:** v0.1.0-alpha (A07-T07). **Type:** tutorial ([Diátaxis](../README.md)).
> **Goal:** from nothing installed to a FLAC file of your own data, in five minutes or less.

This tutorial turns a small table of numbers into sound, saves it as a FLAC file, and shows you how to read, in text, what you are hearing. Every step is one action and shows the text you should see. Nothing here needs a mouse or a screen; the commands work with a screen reader, and `--plain` gives output without colour or box drawing.

Choose one path: **Python** (step 1a) or **the command line** (step 1b). Both make the same sound from the same data, byte for byte.

## 1a. Install the Python package

You need Python 3.10 or later.

```bash
python -m pip install sonarithm
```

Check that it works:

```bash
python -c "import sonarithm; print(sonarithm.__version__)"
```

You should see `0.1.0-alpha`, the version of the Rust core inside the package. pip lists the same release as `0.1.0a0`. Go to step 2.

## 1b. Install the command line tool

Download the archive for your system from the [latest release](https://github.com/DuhItzAniket/SONARITHM/releases): `x86_64-pc-windows-msvc` for Windows, `aarch64-apple-darwin` for Apple silicon Macs, `x86_64-unknown-linux-gnu` for Linux. Unpack it and put `sonarithm` (or `sonarithm.exe`) on your path. If you have Rust installed, `cargo install sonarithm-cli` builds it instead (on Linux, install `libasound2-dev` first).

Check that it works:

```bash
sonarithm --version
```

You should see `sonarithm 0.1.0-alpha`.

> **Before the release is published,** neither command above finds anything. Build from a clone of the repository instead: `pip install ./bindings/python` (needs Rust) or `cargo install --path apps/cli`.

## 2. Make a data file

Create `temps.csv` with these lines: years and a temperature anomaly in kelvin.

```csv
year,anomaly
2015,0.90
2016,1.01
2017,0.92
2018,0.85
2019,0.98
2020,1.01
2021,0.85
2022,0.89
2023,1.17
2024,1.29
```

Any CSV, TSV, JSON, Parquet, Arrow or spreadsheet file with at least two numeric columns works the same way.

## 3. Hear it and save it

**Command line:**

```bash
sonarithm --plain quick temps.csv --x year --y anomaly --concept temperature --spec-out temps.spec.json -o temps.flac
```

**Python:**

```python
import sonarithm as snr

c = snr.quick("temps.csv", x="year", y="anomaly", concept="temperature")
c.render("temps.flac")
open("temps.spec.json", "w", encoding="utf-8").write(c.spec.to_json())
```

You now have four files:

- `temps.flac`: ten tones over 3.25 seconds, normalized to −16 LUFS. Higher anomalies sound higher, because the concept `temperature` maps higher values to higher pitch.
- `temps.render.json`: the render manifest. It records the software and engine versions, the spec and its hash, the data fingerprint, the sample rate and the measured loudness, so the file can be reproduced exactly. It contains no user names or folder paths.
- `temps.cues.vtt`: cues for speech and markers (empty until speech arrives with the desktop app).
- `temps.spec.json`: the spec `quick` wrote for you, to edit and reuse.

To listen without saving, leave out `-o temps.flac`: the command line plays the sound, and in a notebook `c.play()` gives an audio player. Press space to pause, q to stop.

## 4. Read what you heard

**Command line:**

```bash
sonarithm --plain inspect --describe temps.spec.json
```

Or, in Python:

```python
print(c.describe())
```

You should see lines like these:

```text
temps: 1 data track, 3.25 seconds, analysis mode.
A line chart overview: time along x, pitch for y.
Track anomaly: 10 tones from 0.00 to 3.25 seconds. Pitch follows anomaly, 400 to 1400 Hz; time follows year.
In anomaly, the highest pitch, 1400 Hz, is at 3.00 seconds (row 9); the lowest, 400 Hz, at 1.00 seconds (row 3).
```

The description is the same every time for the same data, so you can compare what you heard with what is there.

## 5. See the choices that were made for you

Every default SONARITHM applied is reported with the design rule behind it:

```bash
sonarithm --plain validate --verbose temps.spec.json
```

For example, "a pitch range of 400 to 1400 Hz" cites SDR-08 and "log-frequency pitch" cites SDR-09. Rules are listed in the [design-rules catalogue](../research/design-rules.md). In Python, `c.diagnostics` lists the same diagnostics as dicts.

## Where next

- Change the design: open `temps.spec.json`, change `"pointsPerSecond"` from 3 to 2, or add `"range": { "hz": [300, 900] }` to the pitch channel, and render again with `sonarithm render temps.spec.json -o temps.flac`.
- Listen to the [examples gallery](../../examples/README.md): five datasets that each show one part of the design.
- Machine-readable output: add `--json` to any command.
- Python with your own DataFrame: `snr.quick(df, x="year", y="anomaly")` takes pandas, polars and pyarrow data.

## For maintainers: the five-minute test

The A07 exit criterion is that three people new to SONARITHM each reach step 3's `temps.flac` in five minutes or less, median ([NFR-12](../product/requirements.md)). The owner runs the test (checklist #129):

1. Each person starts at this page, in a fresh terminal on their own computer, without SONARITHM installed. Python or Rust may already be installed; note which path they took.
2. Start the clock when they open the page; stop it when `temps.flac` exists. Do not help unless they are stuck for two minutes, and note any help.
3. Record per person: an ID (P1, P2, P3, no names), operating system, path (Python or command line), assistive technology if any, minutes, and where they hesitated.
4. Record the median and the notes as a finding in the [findings log](../research-paper/findings-log.md) (A07's IDs are F-100 to F-109) and in the traceability matrix row for NFR-12. Fix the steps people stumbled on in this page.
