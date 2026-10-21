# Spec In, Score Out: A Deterministic, Specification-Driven Engine for Reproducible Data Sonification

> **Status:** Draft v0 (2026-10-19), written by the A08 agent session for the paper lead. **Not submitted.** Submission is the owner's act.
> **Venue:** ICAD 2027, Athens. The call for papers is not published yet (checked 2026-10-19). Format, length, deadline and review model are `TODO(verify)` against the call; ICAD 2026 asked for 6–8 pages including references, double-blind review, and accessible camera-ready PDFs ([submission.md](../submission.md)).
> **Related:** [outline](../outline.md) · [contributions](../../../contributions.md) · [evaluation plan](../../../evaluation-plan.md) · [benchmarks log](../../../benchmarks-log.md) · [findings log](../../../findings-log.md) · [references.bib](../../../../research/references.bib)

<!--
Drafting rules (publication plan, research-paper README):
- Cite only keys in docs/research/references.bib, written as [@key]. Every key used here
  was checked against its DOI or publisher record (a08 citation check, see artifact.md).
- Every number comes from the benchmarks log, the findings log or a pinned test, and says
  where. A number that does not exist yet is a placeholder: [[TODO: ...]].
- A claim may be stated only at the evidence level contributions.md records for it. The
  "Evidence ledger" at the end lists each claim, its status today and what must land
  before submission.
- Anonymization: the 2026 cycle was double-blind. Before submission, replace the system
  name and repository links per the 2027 call (TODO(verify)).
-->

## Abstract

*About 180 words; ICAD 2026 asked for 150–200.*

Sonification research has produced well-replicated perceptual results, yet software support lags behind them, and few published mappings were ever evaluated [@dubus2013systematic]. Maintained tools are mostly web libraries or notebooks, and none couples a declarative specification with reproducible offline export and live rendering. We present SONARITHM, an open-source toolkit with a Rust core that compiles a declarative specification, written in the substrate–mark–channel vocabulary [@enge2023unified], into a canonical Score representation. One render graph drives both real-time playback and offline export, and every render carries a manifest with the engine version, specification hash, data fingerprint and seed. Defaults follow a catalogue of evidence-backed design rules whose identifiers appear in tests and diagnostics. Following established strategies for evaluating toolkits [@ledo2018evaluation], we (1) re-create classic auditory-graph designs from committed specifications, with every deviation from the published stimuli documented; (2) measure compile time, offline rendering speed, interactive latency and cross-platform agreement; and (3) release seeded stimulus generators for four published study designs. [[TODO: one sentence with the headline numbers from §6 once A05 and the gallery are on main.]] SONARITHM is available under MIT OR Apache-2.0 with a citable release.

## 1. Introduction

Sonification is "the transformation of data relations into perceived relations in an acoustic signal for the purposes of facilitating communication or interpretation" [@kramer1999sonification]. Hermann adds a method-based definition: a sonification is objective, systematic and reproducible, so that the same data and the same interactions yield structurally identical sound [@hermann2008taxonomy]. Three decades of work have produced perceptual guidance on pitch mapping, polarity, context cues and data density [@walker2011theory; @smith2005effects; @nees2008density; @walker2007consistency], but two gaps keep that guidance from reaching practice.

The first is an *evaluation gap*. Of 495 mapping occurrences in 179 publications, only 30 (6.1%) were assessed at all [@dubus2013systematic], and evaluation practice at the field's venues remains uneven [@seica2020contrasts]. The second is a *software gap*. Sonification tools are often abandoned, and sustainability is a recurring problem in their history [@budziszewski2025tools]. The maintained toolkits are web libraries and Python packages [@kim2024erie; @cantrell2021highcharts; @trayford2025strauss], and only Erie offers a declarative grammar [@kim2024erie]. As of October 2026 we found no maintained, native, cross-platform engine that couples a declarative specification with both live playback and reproducible offline export (F-001; re-verify before submission).

Reproducibility in Hermann's sense is hard to achieve when the sound depends on a browser's audio stack, the order of a hash map, an unseeded random generator or the host's maths library. We therefore treat reproducibility as a property to build and measure, not a convention to follow.

This paper presents SONARITHM, a sonification toolkit built around one rule: *spec in, Score out*. Every sound comes from a declarative specification compiled to a canonical intermediate representation, the Score. We contribute:

- **C1. A native compiler and render engine.** A declarative specification compiles through a staged pipeline to a Score IR, and one render graph plays the Score in real time and renders it offline, with command-line and Python front ends (§4).
- **C2. Reproducibility by construction.** Every render carries a provenance manifest, and we measure where outputs are byte-identical across operating systems and where they agree only within tolerance (§4.4, §6.2).
- **C3. A replication gallery** that re-creates published designs from committed specifications, together with seeded generators for the stimuli of four published studies (§6.1).
- **C4. Evidence-traceable defaults.** Each default cites a design-rule identifier; tests are named after the rule they check, and diagnostics carry the identifier (§5).

The paper describes what exists at release [[TODO: v0.2.0-alpha and its DOI]]. It makes no perceptual claims: no listening study has been run with SONARITHM, and the accessible desktop application is future work (§7).

## 2. Background and related work

**Definitions and vocabulary.** We follow Hermann's criteria [@hermann2008taxonomy] and the unified vocabulary of Enge et al., in which a *temporal substrate* carries *auditory marks* whose *auditory channels* (pitch, loudness, timbre, pan, duration) encode data [@enge2023unified]. The vocabulary is the public interface of SONARITHM's specification language. A state-of-the-art report on audiovisual displays found the field fragmented into teams "without many interconnections" [@enge2024star], which motivates shared infrastructure.

**Grammars and toolkits.** Erie is a declarative grammar for sonification that compiles to the Web Audio API and replicates prior prototypes; interaction and streaming are named as future work [@kim2024erie]. Umwelt shares one data model across visual, textual and sonified representations [@zong2024umwelt]. STRAUSS renders sonifications offline from Python, including multichannel output [@trayford2023strauss; @trayford2025strauss]. Sonecules package reusable "design once, use many" sonification designs [@reinsch2023sonecules], and the Highcharts Sonification Studio offers an accessible, browser-based authoring tool on a proprietary charting library [@cantrell2021highcharts]. SONARITHM imports a subset of Erie specifications (§4.1) and differs from these tools in combining a native deterministic core, offline export with a provenance manifest, and one specification shared by every front end.

**Evaluating toolkits.** Ledo et al. analysed 68 toolkit papers and identified four evaluation strategies: Demonstration, Usage, Technical Performance and Heuristics, and they argue that the strategy should match the claim [@ledo2018evaluation]. This paper uses Demonstration (the replication gallery), Technical Performance (benchmarks) and a heuristic check against Hermann's criteria; it does not run a usage study with participants (§3, §7).

**The designs we re-create.** Smith and Walker showed that a dynamic y-axis reference tone, at the day's maximum while prices rose and its minimum while they fell, cut point-estimation error, while a static reference did not help significantly [@smith2005effects]. Nees and Walker crossed data density with trend reversals and found a small effect of density and large effects of reversals [@nees2008density]. Tucker Brown et al. measured signal detection in light curves across signal-to-noise ratios [@tuckerbrown2022evaluating], and Elmquist et al. measured discrimination of correlation strength in audio, visual and combined displays [@elmquist2024parallel]. Dombois audified seismograms by resampling them 2,200 times faster [@dombois2001using]; that item depends on the audification mark planned for a later release (§7).

## 3. Design goals

We turned Hermann's criteria and the perceptual literature into requirements, each with a rule identifier in the project's design-rules catalogue (cited here as SDR-*n*).

- **G1. A spec fully determines the render** (SDR-01). The specification and the data are the only inputs; the specification is exported next to the audio.
- **G2. Randomness is seeded and declared** (SDR-02), with one portable stream per track.
- **G3. Designs are data-agnostic** (SDR-03): templates bind to any dataset with a compatible schema.
- **G4. One vocabulary** (SDR-04): substrate, mark and channel, as in [@enge2023unified].
- **G5. Defaults come from evidence and say so** (SDR-07 onwards). A default that is an extrapolation is marked `INFER`, treated as a hypothesis, and reported as experimental whenever it applies.
- **G6. Modes are explicit** (SDR-36). Analysis, Presentation and Monitoring are recorded in the specification, following the argument that mixing artistic and analytic goals hindered adoption [@neuhoff2019doomed].
- **G7. One render path.** Real-time playback and offline export share the same graph, so that what a researcher hears interactively is what a reader of the paper can regenerate.

## 4. System

### 4.1 Specification

A specification is a JSON document (version 0.1, frozen in an accepted RFC) with a data section, transforms, a temporal substrate, and tracks. Each track names a mark (`tone` or `glide` in this release) and maps fields to channels through scales (linear, logarithmic, symmetric-log, square-root, power, quantile, band, ordinal and psychophysical) and pitch spaces (log-frequency by default; linear, mel, ERB-rate and Bark). A JSON Schema is generated from the Rust types. Validation reports problems with stable codes and JSON pointers. Canonicalization follows RFC 8785, and the specification hash is SHA-256 over the canonical form.

An importer translates a subset of Erie specifications. On an eight-specification gallery subset it maps 8 of Erie's 15 documented encoding channels and reports 19 constructs that are approximated or have no equivalent (F-081).

[[FIGURE F2: a short specification (the Smith & Walker gallery item) next to its Score timeline. Source: gallery item, A08-T02.]]

### 4.2 Compiler and Score IR

The compiler runs a fixed pipeline: validate and migrate, ingest, transform, analyze, encode, shape. Engines register capabilities into it and never depend on each other; they exchange only core types. The output is a *Score*: tracks of timed events (tones, glides, speech, markers) with frequencies, levels, pan, instrument and the data row each event came from, plus diagnostics. The Score has a canonical serialization and a SHA-256 hash, and it is the artifact that tests snapshot.

Determinism is enforced in the code rather than assumed: no hash-map iteration reaches an output, time is computed by counting rather than accumulating, every transcendental function that affects the Score goes through a pure-Rust maths library instead of the platform's, and seeded `ChaCha8` streams are derived per track from the specification seed. A lint denies the platform's floating-point functions in crates whose output reaches the Score.

[[FIGURE F1: pipeline from spec to Score IR to real-time and offline rendering, with the manifest. Source: architecture overview §5.]]

### 4.3 Rendering

A plan builder turns the Score into frames, frequencies and linear gains, and preallocates voice pools. The render graph mixes voices into data, context, earcon and monitoring buses, schedules events sample-accurately, and implements the transport (play, pause, seek, loop, speed) with crossfades. Its processing function is the single path for both outputs: the device callback for real-time playback, and a fixed-block loop for offline export. On the real-time path the graph does not allocate, lock or perform I/O; a debug build checks for allocation on the audio thread. [[TODO(verify): cite the A05 allocation-check result once A05-T10 is merged.]]

Offline export writes WAV or FLAC with optional loudness normalization. [[TODO(verify): A05-T07/T08 on main; describe normalization (target and true-peak limit) from 06 §7.]]

### 4.4 Provenance manifest

Every export writes a manifest next to the audio file: the software and engine versions, the IR version and Score hash, the specification version and hash, the data fingerprint (SHA-256 over a canonical Arrow IPC encoding of the data), the seed, the sample rate and block size, the audio file's own SHA-256, and loudness figures. It records no timestamp and no directory names, so two renders of the same inputs produce the same manifest, and no user path leaks into a published artifact. [[TODO(verify): field list against `schemas/sonarithm-render-manifest-1.schema.json` once A05-T08 is merged.]]

### 4.5 Front ends

A command-line tool renders, plays, validates and inspects specifications, and a Python package accepts pandas, polars or pyarrow tables and plays renders in notebooks. [[TODO(verify): command names and the Python API from A07 once merged; release DOI.]] Both call the same compiler and render graph, so a specification renders identically from either.

### 4.6 Evaluation harness

The harness re-implements published stimulus designs as seeded generators and runs listening studies. The generators GEN-001 to GEN-004 follow Nees and Walker's reversal-by-density graphs [@nees2008density], Smith and Walker's price series with its click and reference-tone conditions [@smith2005effects], Tucker Brown et al.'s transit light curves at a given signal-to-noise ratio [@tuckerbrown2022evaluating], and Elmquist et al.'s correlation triplets [@elmquist2024parallel]. Each stimulus carries its ground truth and notes wherever it departs from the published design, and has a canonical byte form with a SHA-256 digest. Pinned digests are checked on Windows, macOS and Linux in continuous integration. A study runner sequences trials from a protocol file with a recorded seed, takes keyboard responses, writes one JSON record per event, and pauses and resumes. [[TODO(verify): study runner as merged in A08-T03.]] No participant data has been collected with it (§8).

## 5. Evidence-backed defaults

The design-rules catalogue holds 42 sonification rules and a set of accessibility requirements, each with a strength (`MUST`, `SHOULD`, `DEFAULT` or `INFER`), a confidence and its sources. [[TODO(verify): rule counts at submission.]] Code cites the identifier in documentation and in test names, so a reviewer can trace a default to its evidence:

- **Pitch range.** The default range is 400–1400 Hz in a log-frequency space, because a narrow mid-range band performed as well as a wide one on line-chart tasks [@fitzpatrick2023frequency] (SDR-08, SDR-09).
- **Polarity.** A concept table assigns default polarities with their evidence, flags concepts on which listeners disagree, such as money, and offers a one-key flip [@walker2002magnitude; @walker2007consistency] (SDR-12 to SDR-15). A flip mirrors pitches in the pitch space's own coordinate, which equals a mirror in cents only for log-frequency (F-080).
- **Psychophysical scaling.** A scale of the form f = f_ref·(d/d_ref)^(1/m) reproduces Smith and Walker's dollar mapping, with the exponent labelled as guidance (SDR-11).
- **Density.** Above about ten events per second, an experimental `auto` mark switches from tones to a glide. This threshold is an inference from rhythm perception [@london2012hearing], not a sonification result (SDR-20, `INFER`), and every use is reported as experimental.
- **Volatility.** Series with more than three trend reversals, the most Nees and Walker studied, receive advice to smooth or slow down (SDR-21; F-084, F-110).

An automated conformance test parses the catalogue and fails if a rule owned by the mapping engine lacks a test named after it (A06 phase review). [[TODO: one example diagnostic as a figure or listing.]]

## 6. Evaluation

### 6.1 Demonstration: replication gallery

Each gallery item renders from a committed specification and dataset, with its manifest, and documents every deviation from the published design next to the source (evaluation plan, "Replication gallery").

[[TABLE T2/F3: one row per gallery item. Columns: published design and source; what must match (timing, mapping, cues); SONARITHM parameters; deviations; specification length in lines; Score hash; audio SHA-256. Items planned for v0.2.0-alpha: Smith & Walker context cues (four conditions), Nees & Walker density × reversals (16 cells), Erie gallery subset. Fill from A08-T02.]]

Two items depend on unpublished details, and we say so rather than guess silently. Smith and Walker did not tabulate their price series; our item rebuilds it from the stated open, maximum, minimum and close and from the hourly values plotted in their Figure 2 (F-111). Nees and Walker published no generating function; our density item uses shapes read from their schematic figure and seeded alternatives (F-110). The gallery therefore demonstrates that the specification can express these designs and reproduce them byte for byte from a seed. It does not show perceptual equivalence to the original studies.

Where the record is ambiguous we note it: the published frequency range of 200–1900 Hz for Smith and Walker's mapping is rounded from 193–1928 Hz (F-111), and Elmquist et al. state the added-noise range in two inconsistent ways (F-113).

### 6.2 Technical performance

All measurements follow the method notes of the benchmarks log: median and stated percentiles over repeated runs, with the machine, operating system, audio host and buffer size recorded. The development laptop (HW-01, Intel i7-14650HX, Windows 11) is faster than the intended reference machine; we also report a stand-in restricted to four efficiency cores (HW-01e).

| Metric | Budget | Result | Source |
|---|---|---|---|
| Compile, 100k rows × 4 tracks | ≤ 200 ms | 35.1 ms (HW-01), 54.3 ms (HW-01e), median | F-083 |
| Compile, 1M rows × 4 tracks | ≤ 1 s | 362 ms (HW-01), 545 ms (HW-01e) | F-083 |
| Load a 1M-row CSV | ≤ 1.5 s | 0.51 s (HW-01), 0.91 s (HW-01e) | F-070 |
| Offline render speed, 4-track 10k-event Score | ≥ 50× real time | 91.4× (HW-01); 43–46× (HW-01e), so not declared met | F-053 |
| Export of the replication gallery, end to end (compile, render, WAV, loudness meter), one thread | — | 131.7× real time (HW-01), 70.9× (HW-01e); 443.8 s of audio in 49 renders, 5 runs each | F-115 |
| Key press to cue onset, p95 | ≤ 50 ms at 256 frames | 123.1 ms through cpal in WASAPI shared mode; 83.7 ms with a direct RAW stream (prototype measurements) | F-031, F-032 |
| Score IR across operating systems | identical | identical hashes on Windows, macOS and Linux in CI | A03, A06 tests |
| Offline audio across operating systems | null residual ≤ −90 dBFS | byte-identical on Windows, Linux and macOS (AArch64) for A05's six golden cases and all 49 replication-gallery renders | F-116, F-039; #178 |
| Stimulus generators across operating systems | identical | pinned SHA-256 digests match on all three CI platforms | A08-T01 |

[[FIGURE F4: latency distribution and real-time factor per OS. FIGURE F5: cross-OS null residuals for the golden set. Fill from A08-T04.]]

The latency budget is **not met** on the Windows development machine; the project deferred it to the accessible-navigation phase (B03). Shared-mode WASAPI ignored the requested buffer size and imposed a 480-frame period (F-030), and an OEM enhancement component added about 38 ms (F-032). A direct RAW stream bypasses that component and reaches 83.7 ms p95, still above budget. We report this as measured and discuss it in §7.

### 6.3 Heuristic check against Hermann's criteria

| Criterion [@hermann2008taxonomy] | How SONARITHM meets it | Evidence |
|---|---|---|
| Objectivity: the sound reflects the data, not the designer's intent | Every mapping is explicit in the specification; defaults cite their evidence; experimental defaults are reported | §5; design-rules catalogue |
| Systematicity: a precise definition of how data and interactions change the sound | The specification and the Score are versioned, schema-checked contracts | §4.1–4.2 |
| Reproducibility: same data, same interactions, structurally identical sound | Canonical Score hash; byte-identical offline audio across platforms in our tests; manifest with every render | §4.4, §6.2 |
| Reusability: designs apply to other data with a compatible structure | Templates bind to any compatible dataset; gallery specifications are data-agnostic | §5, §6.1 |

### 6.4 Targeted usage

[[TODO: author case studies (outline §6.3): build three new designs from scratch with the CLI and record the steps and time. No participants; the paper lead runs them. Alternatively drop this section and state the limitation.]]

## 7. Discussion and limitations

**No perceptual validation.** SONARITHM's defaults rest on the literature, and its `INFER` rules are untested hypotheses. The replication gallery reproduces published *stimuli*, not published *results*. Running the replications with listeners, including blind and low-vision listeners, requires ethics review, which this project does not have yet; we release the generators, the gallery and the study runner so that others can run them.

**Latency on Windows.** The measured key-to-cue latency on the development machine exceeds the 50 ms budget, mainly because shared-mode audio on that machine uses a 480-frame period and an enhancement component in the driver stack. [[TODO(verify): the A05 decision on PB-01 and the exclusive-mode result.]] Real-time playback is still deterministic; only its responsiveness is affected.

**Scope.** This release has tone and glide marks. Audification (needed for the Dombois item), model-based sonification and the accessible desktop application with keyboard navigation are planned, not shipped. The Erie importer covers a subset of the grammar (F-081).

**Unpublished details.** Several classic stimuli are underspecified in print (F-110 to F-113). We documented our reconstructions, and we suggest that sonification studies publish their stimulus specifications and seeds, as SONARITHM's manifests make easy.

**Single-machine measurements.** Most numbers come from one laptop. The cross-platform results come from continuous-integration virtual machines without audio devices.

**Landscape claims age.** The survey of maintained tools is dated October 2026 and must be re-checked in the month of submission.

## 8. Conclusion

SONARITHM compiles declarative sonification specifications into a canonical Score and renders it through one graph for live and offline use, with a manifest that makes each render reproducible. Its defaults cite their evidence, its stimulus generators re-create four published study designs from a seed, and its replication gallery re-creates classic designs from committed specifications. The toolkit, the gallery and the harness are released together, so that perceptual studies can be rerun and extended on the same infrastructure.

## Statements

**Artifact availability.** Software, specifications, generated stimuli, render manifests and analysis scripts are archived with release [[TODO: v0.2.0-alpha, Zenodo DOI]] under MIT OR Apache-2.0 (code) and CC BY 4.0 (data, stimuli, figures). See [artifact.md](../artifact.md).

**Ethics.** This work involved no human participants and collected no personal data.

**AI usage.** [[TODO: final statement in the form the ICAD 2027 call requires.]] See [ai-usage.md](../ai-usage.md) for the record.

## References

Keys cited (all in `docs/research/references.bib`): `kramer1999sonification`, `hermann2008taxonomy`, `walker2011theory`, `enge2023unified`, `enge2024star`, `kim2024erie`, `zong2024umwelt`, `trayford2023strauss`, `trayford2025strauss`, `reinsch2023sonecules`, `cantrell2021highcharts`, `budziszewski2025tools`, `dubus2013systematic`, `seica2020contrasts`, `ledo2018evaluation`, `smith2005effects`, `nees2008density`, `tuckerbrown2022evaluating`, `elmquist2024parallel`, `dombois2001using`, `walker2002magnitude`, `walker2007consistency`, `fitzpatrick2023frequency`, `london2012hearing`, `neuhoff2019doomed`.

## Evidence ledger (internal; remove before submission)

| Claim in this draft | contributions.md | Status on 2026-10-19 | Needed before submission |
|---|---|---|---|
| C1, one render graph for real-time and offline | CL-02, CL-12 | in progress: graph merged (A05-T04); export, RT channel and backends in review | A05 export and A07 CLI merged; RTF measured (A08-T04) |
| C2, manifest with every render; cross-OS determinism | CL-02, CL-03 | in progress: Score hashes identical in CI; prototype audio bit-identical (F-039); manifest in review (A05-T08) | Manifest merged; golden set compared across OSes (A05-T11); gallery manifests reproduced |
| C3, replication gallery | CL-04 | aspiration until A08-T02 | Gallery items merged with manifests and deviation notes |
| C3, stimulus generators | CL-05 (harness part only) | in progress: GEN-001 to GEN-004 merged (#163), validated against the published descriptions (F-110 to F-113) | Released in `v0.2.0-alpha` |
| C4, rule-traceable defaults | CL-06 | in progress: NUMISONA rule tests merged (A06); lint with rule IDs is C05 | State "tests and diagnostics cite rule IDs"; do not claim the perceptual lint |
| Latency within budget | CL-12 | **not met** on HW-01 (F-031) | Report as measured; do not claim |
| Landscape gap | CL-01 | literature-only (F-001) | Re-verify the tool survey in the submission month; phrase as "we found no" |
