# sonarithm

The Rust API of [SONARITHM](https://github.com/DuhItzAniket/SONARITHM), which turns graphs, charts and numerical data into structured, navigable sound. A declarative spec plus data compiles into a Score, the canonical description of the sound; the Score renders offline to WAV or FLAC with a provenance manifest. Every default comes from published auditory-display research and is reported with its design-rule ID, and every result has a deterministic text description for screen-reader users.

```rust
use sonarithm::{RenderOptions, Sonarithm, Spec};

let snr = Sonarithm::new();
let compiled = snr.compile(&Spec::from_file("temps.spec.json")?)?;
println!("{}", compiled.describe());
snr.render_to_file(&compiled, "temps.flac", &RenderOptions::default())?;
# Ok::<(), sonarithm::Error>(())
```

Status: alpha. Before 1.0 a minor release may break the API; every break is in the [changelog](https://github.com/DuhItzAniket/SONARITHM/blob/main/CHANGELOG.md). The command line tool is the `sonarithm-cli` crate; the Python package is `sonarithm` on PyPI.

Licence: MIT OR Apache-2.0.
