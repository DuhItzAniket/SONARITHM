# Replication gallery

> **Status:** v0 (2026-10-19), roadmap A08-T02. **Owner:** Research lead.
> **Related:** [evaluation plan, replication gallery](../docs/research-paper/evaluation-plan.md#replication-gallery) · [datasets and generators](../docs/research-paper/datasets.md) · [findings log](../docs/research-paper/findings-log.md) (F-110 to F-114) · [benchmarks log](../docs/research-paper/benchmarks-log.md) · [P01 draft](../docs/research-paper/papers/P01-icad-2027-toolkit/outline.md) · [ADR-0013](../docs/decisions/0013-determinism-and-provenance.md)

The gallery re-creates published sonification designs from committed specs. Each item renders from its spec and data with a render manifest, and documents every departure from the published design next to the source. It is not the [examples gallery](../examples/README.md), which shows SONARITHM's own designs on five datasets. It demonstrates that the spec language can express these designs and that SONARITHM reproduces them byte for byte. It does **not** show perceptual equivalence to the original studies: no listening study has been run with these stimuli.

## Items

| Group | Published design | Renders | Status |
|---|---|---|---|
| [smith-walker-2005](smith-walker-2005/README.md) | Smith & Walker (2005), Study 1: x-axis clicks × static or dynamic y-axis reference tones | 6 | v0 |
| [nees-walker-2008](nees-walker-2008/README.md) | Nees & Walker (2008): data density × trend reversals | 16 | v0 |
| [tucker-brown-2022](tucker-brown-2022/README.md) | Tucker Brown et al. (2022): transit-like dips at six signal-to-noise ratios | 18 | v0 |
| [erie](erie/README.md) | The Erie gallery subset (Kim, Kim & Hullman 2024), imported by NUMISONA | 9 | v0 |
| [dombois-2001](dombois-2001/README.md) | Dombois (2001): seismograms resampled 2,200×, a near and a far earthquake at one GEOFON station (CC BY 4.0, doi:10.14470/TR560404) | 2 | v0 |
| [curvasona-suite](curvasona-suite/README.md) | Not a published design: CURVASONA's 40 curves with analytically known critical points, each as a glide with its critical-point earcons | 40 | v0 |

## How it is checked

```bash
cargo xtask replication
```

The task regenerates every generated file in memory and fails on any difference from the committed files:

- the stimulus data, from ECHOMETRIC's generators (GEN-001, GEN-002, GEN-003) at seed 20261019, with each stimulus's manifest;
- the generated specs: one per Nees & Walker cell and per Tucker Brown curve from a template each, and the imported Erie specs;
- the reproducible part of every render manifest, in each group's `manifests.json`.

Every spec is compiled with the real compiler and NUMISONA and exported as 24-bit WAV at the rendered level (no loudness normalization, as for research exports) into `target/replication/`, next to its full render manifest. The committed `manifests.json` keeps everything but the platform and the software versions: the Score hash with every engine version written as `*` (the header names the crate versions, so the raw hash changes with every release while no sound does, F-102), the spec hash, the data fingerprint, the audio file's SHA-256, its length, loudness and true peak, and a summary of each track (notes, onsets, durations, pitch range in hertz, instrument). Audio files are not committed (OD-08).

The same check runs as the `xtask` test `the_replication_gallery_renders_reproducibly` in CI on Windows, macOS and Linux, so a render that differs on any platform fails the build. After an intended change, run `cargo xtask replication --update`, review the diff, and say in the pull request why the renders changed.

`cargo run --release -p xtask -- replication --timings` renders the specs one at a time and prints each one's compile and export time and real-time factor, for the benchmarks log (A08-T04).

## Rules for adding an item

1. Name the published source as a `references.bib` key and read the paper itself, not a summary.
2. Write the spec from the paper's description. Never copy the authors' code.
3. Generate data with a registered generator (datasets.md) and a seed, or commit synthetic data with its formula.
4. In the group README, list each published parameter with its page next to SONARITHM's value, and mark what the paper does not state.
5. Run `cargo xtask replication --update` and commit the generated files.

## Licences

Specs, data, stimuli and manifests in this folder are CC BY 4.0 (ADR-0010); the code that generates them is MIT OR Apache-2.0. The Erie fixtures were written for SONARITHM from Erie's documentation, and their data is synthetic.
