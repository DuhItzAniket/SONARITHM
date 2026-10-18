# Glossary

> **Status:** Living document (2026-10-18). Terms follow Enge et al. (2023) where possible, so sonification and visualization share one vocabulary.

| Term | Definition |
|---|---|
| **ACR** | Accessibility Conformance Report: a per-release statement of conformance to WCAG 2.2 / EN 301 549 / Section 508 (VPAT-style) |
| **Analysis / Presentation / Monitoring mode** | Spec-level modes separating analytic accuracy (continuous, unquantized), aesthetic presentation (may be musically quantized) and peripheral monitoring (ambient soundscape). SDR-36 |
| **Annotation** | An analyzer output attached to data (extremum, outlier, change point, critical point, insight) |
| **AT** | Assistive technology: screen readers (NVDA, JAWS, Narrator, VoiceOver, Orca), braille displays, switches |
| **Audification** | Playing a data series directly as an audio waveform, usually time-compressed by a speed-up factor *k*. SDR-31 |
| **Auditory icon** | A recorded, real-world-like sound standing for an event or object |
| **Auditory legend** | A spoken or sonic description of the mapping (ranges, units, polarity) heard before playback. SDR-18 |
| **BLV** | Blind and low-vision |
| **Capability** | What an engine contributes to the compiler: Source, Analyzer, Encoder, Context, Shaper, Voice, Sink or Interactive ([05 §3](architecture/05-compiler-and-score-ir.md#3-engine-registry-and-capability-traits)) |
| **Channel (auditory)** | A perceptual sound attribute that carries data: pitch, loudness, duration, timbre, pan, brightness, rate, density |
| **Concept / concept table** | A label for what a field measures (temperature, mass, money…), and the table that maps concepts to evidence-backed polarity and scaling defaults |
| **Context track** | Non-data sound that aids interpretation: tick clicks, reference tones, earcons, legends (AUDAXIS) |
| **Cue** | A short sound played in response to navigation (for example the tone of the focused point) |
| **DataBundle** | SONARITHM's Arrow-backed container of tables, metadata, provenance and annotations |
| **Determinism** | The same spec, data, seed and engine versions give a byte-identical Score and audio within tolerance. ADR-0013 |
| **Dynamic reference tone** | A context tone in a distinct timbre that sounds the series maximum while values rise and the minimum while they fall (Smith & Walker 2005). SDR-17 |
| **Earcon** | A short abstract, structured sound motif with a learned meaning (boundary, invalid action, landmark) |
| **Engine** | One of the nine named capability providers (NUMISONA … RESONUM) registered in the compiler |
| **ERB / Bark / mel** | Perceptual frequency scales (equivalent rectangular bandwidth, critical bands, pitch perception) |
| **Gate** | A formal stage-end review with go/no-go criteria |
| **L1–L4** | Lundgard & Satyanarayan's levels of chart description: encodings, statistics, trends/patterns, interpretation/context |
| **Lint (perceptual)** | ECHOMETRIC's checks of a compiled Score against the design rules, emitting diagnostics with rule IDs |
| **Manifest (render)** | `*.render.json`: the provenance record written with every export |
| **Mark (auditory)** | An individual sound event or object that represents data: tone, glide, grain, audify, sample, model |
| **MBS** | Model-based sonification: data is turned into a dynamic model that sounds when excited by the user (Hermann). RESONUM |
| **NavTree** | AUDAXIS's hierarchical navigation structure over dataset, series, axes, insights, segments and points |
| **PMSon** | Parameter-mapping sonification: data attributes mapped to sound parameters |
| **Polarity** | The direction of a mapping. Positive means larger values give higher pitch (or more of the channel) |
| **Provenance class** | How a value was obtained: exact, vector, cv_estimate, ml_estimate, cloud_estimate, user_edited, derived, synthetic |
| **RenderPlan** | The preallocated, frame-based form of a Score that the audio thread executes |
| **RowRef** | A stable reference from a sound event back to the data row that produced it |
| **RT / real-time thread** | The audio callback thread, where allocation, locks and I/O are forbidden |
| **Score (Score IR)** | The canonical, serializable timeline of sound events produced by the compiler and consumed by renderers and sinks |
| **SDR-xx / A11Y-xx** | IDs of the sonification design rules and accessibility requirements in [design-rules.md](research/design-rules.md) |
| **Shaper** | A compiler stage that transforms a whole Score: quantize, loudness compensation, spatialize, lint |
| **Sonecule** | A reusable, data-agnostic sonification design template (Reinsch & Hermann 2023) |
| **Spec** | The declarative JSON description of a sonification |
| **Spearcon** | Speech that is time-compressed until it is no longer fully intelligible but stays recognizable, used as a fast auditory label |
| **Substrate** | The temporal canvas onto which auditory marks are placed (Enge et al.) |
| **xrun** | An audio buffer underrun or overrun, heard as a glitch |
