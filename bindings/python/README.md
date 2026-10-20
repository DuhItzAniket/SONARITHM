# SONARITHM for Python

SONARITHM turns graphs, charts and numerical data into structured, navigable sound. A
declarative spec plus data compiles into a Score, the canonical description of the sound.
Every default comes from published evidence and is reported with its design-rule ID.
Blind and low-vision screen-reader users are a primary audience: every result has a
deterministic text description.

```python
import sonarithm as snr
import pandas as pd

df = pd.DataFrame({"year": [2000, 2001, 2002], "anomaly": [0.39, 0.54, 0.63]})
c = snr.quick(df, x="year", y="anomaly", concept="temperature")
print(c.describe())     # L1/L2 text, the same every time
c.play()                # notebook audio (IPython)
c.render("temps.flac")  # + temps.render.json and temps.cues.vtt
c.diagnostics           # every applied default, with its rule ID
c.spec.to_dict()        # the generated spec, to edit and reuse

spec = snr.Spec.from_dict({
    "specVersion": "0.1",
    "data": {"source": {"kind": "dataset", "id": "main"}},
    "substrate": {"time": {"field": "year"}},
    "tracks": [{"id": "anomaly", "mark": "glide", "channels": {"pitch": {"field": "anomaly"}}}],
})
c2 = snr.compile(spec, data={"main": df})
```

Data can be a pyarrow table, a polars or pandas DataFrame, any object with the Arrow
PyCapsule interface, a dict of columns, a list of records, a 1-D NumPy array, or a path to a
CSV, JSON, Parquet, Arrow or spreadsheet file. pyarrow, pandas and polars are optional.

Status: alpha (`0.1.0a0`). Real-time playback outside notebooks arrives with the desktop
application. Licence: MIT OR Apache-2.0. Documentation and source:
<https://github.com/DuhItzAniket/SONARITHM>.
