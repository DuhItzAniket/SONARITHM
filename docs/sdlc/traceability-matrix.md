# Requirements Traceability Matrix

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> Links each requirement in [requirements.md](../product/requirements.md) to its design, delivery phase, verification method and current status. Update the row in the same PR that changes any of these.
> **Verification codes:** UT unit test · PT property test · ST snapshot test · GT golden-audio test · BM benchmark · IT integration test · E2E app test · AT assistive-technology manual test · CD co-design/user study · RV review/inspection.

| Req | Design | Phase | Verification | Status |
|---|---|---|---|---|
| FR-001 | [03](../architecture/03-data-model.md), [10 §2](../architecture/10-ingestion-and-extraction.md#2-source-adapters) | A04 | UT, IT (file corpus), fuzz | Planned |
| FR-002 | [03 §3](../architecture/03-data-model.md#3-field-semantics) | A04 | UT (inference corpus ≥ 95%) | Planned |
| FR-003 | [03 §6](../architecture/03-data-model.md#6-transforms) | A04 | UT, PT | Planned |
| FR-004 | [11 §3](../architecture/11-bindings-cli-plugins.md#3-python-package-sonarithm) | A07 | IT (pytest on wheels) | Planned |
| FR-005 | [07 SONOTRACE](../architecture/07-engines.md), [03 §8](../architecture/03-data-model.md#8-streaming-tables-sonotrace) | B06 | IT, soak (PB-11), AT | Planned |
| FR-006 | [10 §3](../architecture/10-ingestion-and-extraction.md#3-the-bring-any-chart-pipeline-plotsona) | C01 | IT (50-chart corpus) | Planned |
| FR-007 | [10 §3](../architecture/10-ingestion-and-extraction.md#3-the-bring-any-chart-pipeline-plotsona) | C02 | IT (raster corpus), AT (correction flow) | Planned |
| FR-008 | [03 §4](../architecture/03-data-model.md#4-provenance-and-confidence) | A04, C01 | PT (propagation), ST (announcements) | Planned |
| FR-009 | [07 CURVASONA](../architecture/07-engines.md) | C03 | UT (40-curve suite) | Planned |
| FR-010 | [10 §3–4](../architecture/10-ingestion-and-extraction.md) | D04 | BM (extraction benchmark), RV (consent flow) | Planned |
| FR-020 | [04](../architecture/04-sonification-spec.md), [RFC-0001](../rfcs/0001-spec-v0.1.md) | A03 | UT (`validate/tests.rs`, a failing and a passing case per rule), fuzz (`spec_parse`), doc-example test (`doc_examples.rs`), schema diff (`docs / schema-diff`) | In review (#66, #96, #97) |
| FR-021 | [04 §2](../architecture/04-sonification-spec.md#2-top-level-structure) | A03 | UT (`sdr_36_sdr_17_mode_and_reference_defaults`, `sdr_10_music_only_in_presentation_mode`, `sdr_10_quantize_runs_only_in_presentation_mode`) | In review (#96, #99, #100) |
| FR-022 | [05 §3](../architecture/05-compiler-and-score-ir.md#3-engine-registry-and-capability-traits), [ADR-0013](../decisions/0013-determinism-and-provenance.md) | A03 | PT (`canonical_form_round_trips_byte_for_byte`), UT on 3 OSes (`canonical_hash_is_identical_on_every_platform`, `sdr_02_track_streams_are_seeded_and_portable`, `spec_hash_is_pinned_and_ignores_key_order`), IT (`the_same_input_gives_byte_identical_scores`) | In review (#8, #10, #99, #100) |
| FR-023 | [04 §7](../architecture/04-sonification-spec.md#7-versioning-and-compatibility) | A03, A07 | UT (`migrate.rs` test chain, idempotence), PT (`migrating_a_current_spec_changes_nothing`), fuzz (`spec_migrate`); `sonarithm spec migrate` in A07 | Framework in review (#99) |
| FR-024 | [04 §9](../architecture/04-sonification-spec.md#9-importers) | A06 | IT (Erie gallery subset) | Planned |
| FR-025 | [07 NUMISONA](../architecture/07-engines.md) | A06 | ST (templates × datasets) | Planned |
| FR-026 | [05 §7](../architecture/05-compiler-and-score-ir.md#7-diagnostics) | A06 | ST (diagnostics) | Planned |
| FR-027 | [04 §9](../architecture/04-sonification-spec.md#9-importers) | C01 | IT | Planned |
| FR-030 | [04 §5.3](../architecture/04-sonification-spec.md#53-marks-and-channels) | A06 | UT, ST | Planned |
| FR-031 | [07 NUMISONA](../architecture/07-engines.md) | A06 | UT (`sdr_08_*`, `sdr_09_*`) | Planned |
| FR-032 | [03 §3.3](../architecture/03-data-model.md#33-concepts-and-the-concept-table) | A06, B03 | UT (`sdr_12..15_*`), PT (flip symmetry) | Planned |
| FR-033 | [04 §5.3](../architecture/04-sonification-spec.md#53-marks-and-channels) | A06 | UT (`sdr_11_*`) | Planned |
| FR-034 | [04 §5.2](../architecture/04-sonification-spec.md#52-tracks) | A06 | UT (A11Y-13) | Planned |
| FR-035 | [07 NUMISONA](../architecture/07-engines.md) | A06 | UT (`sdr_20_*`); EXP for H-02 | Planned |
| FR-040 | [07 AUDAXIS](../architecture/07-engines.md) | B03 | UT, ST; EXP for H-01 | Planned |
| FR-041 | [08](../architecture/08-interaction-and-accessibility.md) | B03 | ST (legend text), AT | Planned |
| FR-042 | [08 §2](../architecture/08-interaction-and-accessibility.md#2-the-navigation-model) | B03 | PT (reachability), AT (task suite) | Planned |
| FR-043 | [08 §3](../architecture/08-interaction-and-accessibility.md#3-what-a-cursor-move-produces) | B03 | UT, AT | Planned |
| FR-044 | [08 §4](../architecture/08-interaction-and-accessibility.md#4-announcement-pipeline) | B03 | ST (templates), AT (NVDA, JAWS, VO, Orca) | Planned |
| FR-045 | [08 §3](../architecture/08-interaction-and-accessibility.md#3-what-a-cursor-move-produces) | B03 | AT (braille display) | Planned |
| FR-046 | [09 §3](../architecture/09-desktop-app.md#3-frontend-architecture) | B04 | E2E, AT | Planned |
| FR-047 | [08 §6](../architecture/08-interaction-and-accessibility.md#6-modalities-profiles-and-settings) | B03 | E2E, AT | Planned |
| FR-048 | [06 §3](../architecture/06-audio-engine.md#3-render-graph) | B03, D03 | GT | Planned |
| FR-049 | [08 §4](../architecture/08-interaction-and-accessibility.md#4-announcement-pipeline) | B03 | AT, CD | Planned |
| FR-050 | [06 §5](../architecture/06-audio-engine.md#5-scheduler-and-transport) | A05, B01 | UT (sample-accurate dispatch), E2E | Planned |
| FR-051 | [08 §4](../architecture/08-interaction-and-accessibility.md#4-announcement-pipeline) | B03 | AT | Planned |
| FR-052 | [07 SONOTRACE](../architecture/07-engines.md) | B06 | IT, CD; EXP for H-09 | Planned |
| FR-060 | [07 CURVASONA](../architecture/07-engines.md) | C03 | UT, CD (teacher pilot) | Planned |
| FR-061 | [07 CURVASONA](../architecture/07-engines.md) | C03 | UT (round trip), GT | Planned |
| FR-070 | [07 AXONOTE](../architecture/07-engines.md) | C04 | PT (never in analysis mode) | Planned |
| FR-071 | [06 §3](../architecture/06-audio-engine.md#3-render-graph) | A05, C04 | UT (spectral), GT | In progress: sine, triangle, PolyBLEP saw and square oscillators (A05-T01, `sonarithm_dsp::osc`) |
| FR-072 | [07 AXONOTE](../architecture/07-engines.md) | C04 | IT (MIDI round trip) | Planned |
| FR-080 | [07 ECHOMETRIC](../architecture/07-engines.md) | C05 | UT (precision/recall) | Planned |
| FR-081 | [10 §4](../architecture/10-ingestion-and-extraction.md#4-ai-assistance-policy) | C05 | ST (description snapshots) | Planned |
| FR-082 | [07 ECHOMETRIC](../architecture/07-engines.md) | C05 | UT | Planned |
| FR-083 | [07 ECHOMETRIC](../architecture/07-engines.md) | A07, C05 | UT (positive and negative per rule) | Planned |
| FR-084 | [07 ECHOMETRIC](../architecture/07-engines.md) | A08 | IT (pilot protocol), RV | Planned |
| FR-090 | [07 TONALENS](../architecture/07-engines.md) | D01 | E2E, AT | Planned |
| FR-091 | [07 TONALENS](../architecture/07-engines.md) | D01 | UT; EXP for H-04 | Planned |
| FR-092 | [07 TONALENS](../architecture/07-engines.md) | D01 | GT | Planned |
| FR-093 | [07 RESONUM](../architecture/07-engines.md) | D02 | GT, CD (pilot) | Planned |
| FR-100 | [06 §7](../architecture/06-audio-engine.md#7-export) | A05 | GT, schema test (manifest) | Planned |
| FR-101 | [06 §7](../architecture/06-audio-engine.md#7-export) | A05, B03 | UT (VTT validity) | Planned |
| FR-102 | [06 §7](../architecture/06-audio-engine.md#7-export) | C04 | GT | Planned |
| FR-110 | [12 §4](../architecture/12-cross-cutting-concerns.md#4-project-file-format) | B05 | IT (round trip, crash injection) | Planned |
| FR-111 | [09 §2.1](../architecture/09-desktop-app.md#21-command-surface-initial) | B05 | IT | Planned |
| FR-112 | [09 §3](../architecture/09-desktop-app.md#3-frontend-architecture) | B05 | E2E, AT | Planned |
| FR-113 | [09 §3](../architecture/09-desktop-app.md#3-frontend-architecture) | B05 | E2E (round trip) | Planned |
| FR-114 | [08 §6](../architecture/08-interaction-and-accessibility.md#6-modalities-profiles-and-settings) | B03 | UT (schema), E2E | Planned |
| FR-115 | [design/sound-design](../design/sound-design.md) | B07 | CD; EXP for H-10 | Planned |
| FR-116 | [design/keyboard-and-navigation](../design/keyboard-and-navigation.md) | B02 | E2E, AT | Planned |
| FR-117 | [design/design-language](../design/design-language.md) | B02 | Visual tests, AT (forced colors) | Planned |
| FR-118 | [06 §10](../architecture/06-audio-engine.md#10-hearing-safety-and-levels) | B01 | E2E | Planned |
| FR-119 | [08 §8](../architecture/08-interaction-and-accessibility.md#8-input-methods) | B04 | E2E, AT | Planned |
| FR-120 | [11 §2](../architecture/11-bindings-cli-plugins.md#2-cli-sonarithm) | A07 | ST (CLI snapshots) | Planned |
| FR-121 | [11 §3](../architecture/11-bindings-cli-plugins.md#3-python-package-sonarithm) | A07 | IT (pytest) | Planned |
| FR-122 | [11 §4](../architecture/11-bindings-cli-plugins.md#4-webassembly-sonarithm-wasm-d05) | D05 | IT (Score parity, H-08) | Planned |
| FR-123 | [11 §5](../architecture/11-bindings-cli-plugins.md#5-plugin-system-d05) | D05 | IT (sandbox tests) | Planned |
| FR-130 | [10 §4](../architecture/10-ingestion-and-extraction.md#4-ai-assistance-policy) | D04 | UT (numeric verification) | Planned |
| FR-131 | [10 §4](../architecture/10-ingestion-and-extraction.md#4-ai-assistance-policy) | D04 | IT | Planned |
| FR-132 | [10 §4](../architecture/10-ingestion-and-extraction.md#4-ai-assistance-policy) | C05, D04 | UT; EXP for H-07 | Planned |
| NFR-01 | [12 §8](../architecture/12-cross-cutting-concerns.md#8-performance-budgets) | A02 → C06 | BM | In progress: A02 spikes measured PB-01 (fails on Windows shared mode), PB-06, PB-08 and PB-09 (pass where measured); [A02 gate report](reviews/A02-prototype-gate.md#performance-budgets) |
| NFR-02 | [design/accessibility-conformance-plan](../design/accessibility-conformance-plan.md) | B02 → | AT, ACR, CD | Planned |
| NFR-03 | [ADR-0013](../decisions/0013-determinism-and-provenance.md) | A05 | GT, PT | Planned |
| NFR-04 | [ADR-0012](../decisions/0012-privacy-and-telemetry.md) | B01 | IT (network-deny) | Planned |
| NFR-05 | [12 §6](../architecture/12-cross-cutting-concerns.md#6-security-and-threat-model) | A01 → C06 | Fuzz, Scorecard, RV | Planned |
| NFR-06 | [09 §6](../architecture/09-desktop-app.md#6-packaging) | A01 → C06 | CI matrix | Planned |
| NFR-07 | [supply chain](../engineering/supply-chain-and-licensing.md) | A01 | cargo-deny, RV | Planned |
| NFR-08 | [testing-strategy](../engineering/testing-strategy.md) | A03 → | CI | Planned |
| NFR-09 | [12 §5](../architecture/12-cross-cutting-concerns.md#5-internationalization-and-localization) | B02 | Pseudo-locale CI | Planned |
| NFR-10 | [06 §10](../architecture/06-audio-engine.md#10-hearing-safety-and-levels) | A05 | UT, GT | Planned |
| NFR-11 | [09 §5](../architecture/09-desktop-app.md#5-persistence) | B05 | IT (fault injection) | Planned |
| NFR-12 | [vision §8](../product/vision.md#8-success-measures) | A07, B07 | CD (observed sessions) | Planned |
| NFR-13 | [research-paper/README](../research-paper/README.md) | A08 → | RV (findings log, preregistrations) | Planned |
| NFR-14 | [GOVERNANCE](../../GOVERNANCE.md) | Gate C | RV | Planned |
