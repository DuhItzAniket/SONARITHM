# Sound design guidelines

> **Status:** Baseline v1 (2026-10-18). **Owner:** Sound design lead.
> Related: [design rules](../research/design-rules.md) (SDR-xx, A11Y-xx), [research notes 02](../research/notes/02-theory-perception.md) (perception evidence), [research report](../research/research-report.md), [engine catalogue](../product/engines.md), [interaction and accessibility architecture](../architecture/08-interaction-and-accessibility.md), [keyboard and navigation](keyboard-and-navigation.md) (earcon catalogue, announcement sequencing), [accessibility conformance plan](accessibility-conformance-plan.md), [findings log](../research-paper/findings-log.md).

## 1. Scope and evidence status

These guidelines say how SONARITHM should sound by default and how sounds are added or changed. They implement the SDR rules in the [design rules catalogue](../research/design-rules.md) and do not restate their evidence; the rule ID is the link.

Three kinds of statement appear here:

- **Rule-backed:** carries an SDR or A11Y ID and that rule's strength (MUST, SHOULD, DEFAULT, INFER).
- **INFER rules** are hypotheses. They cannot become defaults until a SONARITHM experiment supports them ([design rules](../research/design-rules.md), rule 2). Where this document uses one, it says so.
- **Proposals:** concrete values (levels, durations, presets) that no rule fixes. They are starting points for implementation and must be confirmed by a finding (`F-###`) before they are treated as settled.

## 2. Modes

Analysis, Presentation and Monitoring are explicit modes recorded in the spec (SDR-36), following Neuhoff's advice to design for either scientific or artistic goals rather than mixing them ([research report](../research/research-report.md), "Monitoring works…"). Technique names (PMSon, Audification, MBS, Earcon, Auditory icon) label what each layer does (SDR-06).

| Aspect | Analysis (default) | Presentation | Monitoring |
|---|---|---|---|
| Purpose | Read values, trends and shape accurately | Communicate to an audience; pleasant and informative | Peripheral awareness of live data |
| Pitch | Continuous, unquantized (SDR-10, MUST) | Quantization to a scale allowed; the UI states the resolution trade-off (SDR-10) | Continuous; drives soundscape parameters |
| Default mark | Discrete notes at 2–4 points/s (SDR-19) | Free choice; lint still warns | Continuous bed plus event sounds |
| Context cues | Tick track for discrete marks (SDR-16), dynamic reference tone (SDR-17), spoken legend (SDR-18) | Optional; spoken legend still available | Not used; status is spoken on request |
| Timbre | Harmonic mallet default (SDR-29) | AXONOTE instruments (Stage C) | Ecologically coherent soundscape (SDR-35) |
| Space | Only for segregation or positional variables; no redundant spatial encodings (SDR-27, SDR-30) | Ambience and spatial placement allowed | Stream segregation |
| Reverb | None (dry) | Allowed on non-data layers | Defined by the soundscape |
| Loudness | Constant while pitch carries data (SDR-23) | Same rule for data layers; dynamics only on non-data layers | Salience changes on events; urgency only for true alarms (SDR-35) |
| Earcons | Full set ([keymap §11](keyboard-and-navigation.md)) | During interactive use; excluded from exports unless requested | `E-STREAM`; `E-ALARM` only when configured |
| Lint | All rules active | All rules active; trade-off warnings stay visible | Rate limits and comfort budget emphasised (SDR-39) |
| Announcements | Navigation and responses | Same | Rate-limited, pausable, hideable (A11Y-16) |

Changing mode is an undoable spec edit. It is announced with the mode-change earcon and the trade-off text ("Presentation mode. Pitch is quantized to a scale; values are approximate."), and the status bar always shows the current mode.

## 3. Instruments and the point cue

### 3.1 Data instruments

| Preset | Description | Use | Basis |
|---|---|---|---|
| **Mallet** (default) | Harmonic partials at integer multiples of the fundamental, amplitudes falling with partial number (proposal: 1, 0.5, 0.25, 0.125), percussive decay within the note, band-limited | General chart reading and overview | SDR-29 |
| **Sine** | Pure sine with the same envelope | Users who prefer the plainest tone; psychoacoustic experiments | SDR-29 |
| **Discrete square** (BLV chart reading) | Band-limited square wave (PolyBLEP, per A05), discrete notes separated by silence, gentle low-pass for comfort (section 11) | Screen-reader users reading charts | SDR-29; Sharif, Wang & Muongchan 2022: screen-reader users "preferred distinct non-continuous responses generated using oscillators with square waveforms" and wanted to personalise the sound (10 participants) |
| **Glide** | Continuous pitch glide | Dense data above about 10 events/s | SDR-20 (INFER: experimental flag until tested) |
| **Grain** | Granular texture, 80–300 ms grains, 100–1000 active | Distributions and uncertainty (Stage D) | SDR-32 |

Categorical variables map to timbre and interval or ratio variables to pitch, duration or rate (SDR-25). For precise values, a redundant duration or tap-count encoding is available (A11Y-13); blind and low-vision participants decoded values most accurately from taps and sound length (Wang, Jung & Kim 2022).

### 3.2 Point cue

The point cue is the one-shot tone played when the cursor moves ([keymap §9.2](keyboard-and-navigation.md)).

| Property | Specification |
|---|---|
| Voice | The same voice and mapping as playback, so a step and the overview sound alike ([architecture §6.2](../architecture/01-overview.md)) |
| Duration | Proposal: 120 ms including ramps, short enough for the announcement to follow promptly |
| Latency | Key press to cue onset ≤ 50 ms p95 at 48 kHz with a 256-frame buffer ([architecture §7](../architecture/01-overview.md)) |
| Estimates | No timbre change by default; marking low-confidence values in timbre is a C02 experiment |
| Off switch | Sonification toggle (S) silences cues and playback together ([keymap §5.5](keyboard-and-navigation.md)) |

## 4. Pitch

| Setting | Default | Rule |
|---|---|---|
| Primary channel | Pitch carries single-series values and trends | SDR-07 (DEFAULT) |
| Range | 400–1400 Hz (about MIDI 67–89); expandable to 65–1480 Hz | SDR-08 (DEFAULT; one study, n = 48, Moderate) |
| Scale | Log-frequency (semitones or cents); mel, ERB-rate and Bark as options | SDR-09 (DEFAULT) |
| Resolution | Continuous in Analysis; quantization only in Presentation, with the trade-off shown | SDR-10 (MUST) |
| Scaling | Optional psychophysical scaling f = f_ref·(d/d_ref)^(1/m), labelled as guidance | SDR-11 (SHOULD) |
| Polarity | From the concept table: positive for generic quantities, negative for mass, flagged for size and money | SDR-12, SDR-13, SDR-14 |
| Polarity flip | One action (P in the explorer, a toggle in the inspector), persisted per profile | SDR-15 (MUST) |

Hearing differences (A11Y-14):

- Essential information is never carried only by pitch extremes. The range is configurable, and the spoken legend states it (SDR-18).
- **Low range preset (proposal):** about 150–700 Hz for listeners with high-frequency hearing loss, to be validated in B07 before it ships as a preset. It does not replace the text and braille paths; a MAIDR participant with high-frequency hearing loss relied on braille and text ([notes 03 §1](../research/notes/03-accessibility-hci.md)).

## 5. Timing and ramps

| Topic | Specification | Rule |
|---|---|---|
| Overview rate | 2–4 points/s, default 3; lint warning above about 8 discrete points/s | SDR-19 (DEFAULT) |
| Dense data | Above about 10 events/s (inter-onset interval under about 100 ms), switch to glide, grain or audification | SDR-20 (INFER; experimental) |
| Volatile series | Slow down or smooth; trend reversals hurt more than density | SDR-21 (SHOULD) |
| Note ramps | Every note has onset and offset ramps, about 40 ms reference, **never zero** | SDR-22 (MUST) |
| Short notes | Proposal: ramp = min(40 ms, 25 % of note duration), floor 2 ms; lint info when below 40 ms | Proposal under SDR-22 |
| Parameter changes | Continuous parameter and level changes are smoothed (proposal: 10 ms) to avoid zipper noise | Proposal |
| Transport | Start fades in (proposal: 10 ms); pause, stop and "stop all audio" fade out (proposal: 20 ms), inside the interaction budget | Proposal; A11Y-15 |

## 6. Loudness, levels and exports

### 6.1 Policy

- Loudness never carries precise values, and stays constant while pitch carries data (SDR-23, MUST). The lint flags a loudness mapping on a precise quantitative field.
- ISO 226:2023 equal-loudness compensation is on by default so that different pitches sound equally salient (SDR-24). SONARITHM does not claim it improves accuracy; Fitzpatrick & Neff 2023 found no significant effect of basic amplitude compensation.
- The sonification level is independent of system volume (A11Y-15).

### 6.2 Levels (gain staging)

Levels are relative to the data bus. All values are proposals.

| Bus | Default | Rules |
|---|---|---|
| Data (sonification) | 0 dB (reference) | Constant loudness (SDR-23); ISO 226 compensation (SDR-24) |
| Context (ticks, reference tone) | −10 dB | Distinct timbre from data (section 7) |
| Earcons | −6 dB | Never louder than data, except `E-ALARM` |
| Monitoring bed | −12 dB | Salience changes by density and brightness, not level jumps (section 10) |
| Self-voicing speech | Set by the user | When speech and sonification overlap, sonification sits about 20 dB below speech (A11Y-20) |
| Master | User-set; first-launch level in section 12 | True-peak limiter (section 12) |

Screen-reader speech is outside SONARITHM's mix. The main protection is sequencing: cue first, speech after, no overlap ([keymap §9.2](keyboard-and-navigation.md); A11Y-20). The user documentation recommends NVDA ducking and sound-split settings, and the conformance matrix tests each configuration (A11Y-20, A11Y-32). NVDA's sound split can put SONARITHM on one channel, which is one reason no information is carried by pan alone (A11Y-14).

### 6.3 Export loudness (proposed)

| Item | Proposal |
|---|---|
| Target | Integrated loudness −16 LUFS for stereo exports |
| Peak | True peak ≤ −1 dBTP |
| Measurement | ITU-R BS.1770 / EBU R128, via the `ebur128` crate already in the stack |
| Method | One static gain; no compression. The gain is written to the render manifest (SDR-01) |
| Limit case | If the target would exceed the true-peak limit, export at the highest level that meets it and report the shortfall ("Exported at −17.8 LUFS; true-peak limit reached") |
| Research exports | "Preserve level" option exports without normalisation |
| Mono | Target to be decided with a finding |
| Speech | OS speech cannot be mixed into offline exports, so exports carry a speech and cue sidecar instead ([research report](../research/research-report.md); `sonarithm-export`) |

These targets are a proposal and need a finding before the export defaults are fixed.

## 7. Context sounds

### 7.1 Tick track (SDR-16)

| Property | Specification |
|---|---|
| Sound | About 10 ms broadband click (proposal: noise burst with 1 ms ramps, low-passed near 8 kHz for comfort) |
| Placement | Major x-axis ticks |
| Applies to | Discrete-note marks only; off for continuous glides (Smith & Walker 2005 found ticks did not help with continuous tones) |
| Level | −10 dB relative to data (proposal) |
| Space | Centred; never panned |
| Equivalent | The visual axis ticks; tick positions are listed in the spoken legend |
| B03 build | The render engine's `click` instrument: seeded white noise through an 8 kHz low-pass. Its ramps are the engine's 5 ms minimum (SDR-22), so a 10 ms click rises and falls over 5 ms each rather than the 1 ms of the proposal above. Without `context.ticks.every`, ticks fall on a "nice" step (1, 2 or 5 × 10ⁿ) for about five intervals across the x range (provisional, OD-19) |

### 7.2 Dynamic reference tone (SDR-17)

| Property | Specification |
|---|---|
| Behaviour | Sounds the series maximum while values rise and the minimum while they fall. A static first-value reference is never used (SDR-17; Smith & Walker 2005 reported RMS error 18.09 → 12.14 with the dynamic reference, and no significant help from a static one) |
| Timbre | Distinct from the active data instrument (proposal: a sustained soft triangle when data is mallet or square, a filtered pulse when data is sine) |
| Level | −10 dB relative to data (proposal) |
| Direction changes | Trend direction from a smoothed slope with hysteresis (proposal), so noise does not make the reference flap; switches crossfade over at least 40 ms |
| Lint | The reference converging with the data tone near an extreme is the information, so the critical-band spacing check (SDR-28) treats the reference as an intentional exception |
| Equivalent | The plot can draw the current reference level as a faint line; the legend states that the reference is on |
| B03 build | One note per trend run, at the pitch the data gives the series maximum (rising) or minimum (falling), so the reference follows the track's own mapping. The trend is a 3-point moving average with hysteresis of 5 % of the value range (provisional). Consecutive notes overlap by 40 ms, and the engine's 40 ms ramps make the overlap a crossfade. An explicit `static-value` reference is built with warning `SNR-ENG-104` (SDR-17) |

### 7.3 Spoken legend (SDR-18)

Available before playback (L in the explorer) and at the start of an audio narrative: series name; axis ranges and units; mapping (channel, range, scale); polarity; mode; which context cues are on. Example: "Temperature. Pitch from 400 to 1400 hertz for 3.1 to 28.7 degrees Celsius; higher sounds higher. Analysis mode. Ticks on months. Reference tone on."

B03 builds it from the navigation tree: the title, the x range, each series' value range with its unit, the pitch range with the polarity heard in the Score, the mode and the cues. With `speak: before-play` (the default), the Score carries it as a speech track at time 0 in the source locale, `en-US`, for exports; the app speaks it in the listener's locale.

## 8. Earcon design rules

The catalogue (IDs, meanings, acoustic descriptions, text equivalents) is in [keyboard and navigation §11](keyboard-and-navigation.md). Every earcon, current or proposed, must satisfy these rules.

| ID | Rule | Basis |
|---|---|---|
| ED-01 | **Distinct from data.** Earcons use timbre families no data voice uses (noise-based or inharmonic), so they cannot be heard as data. | A11Y-08 |
| ED-02 | **Short.** At most about 150 ms (proposal), except `E-ALARM`. | Proposal |
| ED-03 | **Learnable.** A small fixed set with consistent families: rising means on or enter, falling means off or exit. Earcons are abstract and may need learning, so the tutorial teaches them. | Walker & Nees 2011 on earcon learning ([notes 02 §1](../research/notes/02-theory-perception.md)); SDR-37; A11Y-30 |
| ED-04 | **Calm.** Non-alarming, except true alarms, which the user configures. | SDR-35; Edworthy et al. 1991 |
| ED-05 | **Text equivalent.** Every earcon has one, written to the log and spoken according to the announcement policy. | A11Y-08, A11Y-24 |
| ED-06 | **Robust to listening conditions.** No information in pan; survives mono, NVDA sound split and high-frequency hearing loss (proposal: main energy below about 4 kHz). | A11Y-14 |
| ED-07 | **Below speech, never over it.** Earcons are sequenced with speech like cues are. | A11Y-20 |
| ED-08 | **Deterministic and versioned.** Synthesised from code with fixed parameters, or a CC0 sample with a provenance record (section 14). | SDR-01, SDR-02 |
| ED-09 | **Optional.** Each earcon can be set to earcon only, earcon plus speech, or speech only, and the set can be muted. | SDR-38 |
| ED-10 | **No masking of data.** During playback, the scheduler offsets earcons from data-note onsets (proposal). | Proposal |

## 9. Concurrent streams

| Topic | Specification | Rule |
|---|---|---|
| Separation | Timbre, pan and staggered onset separate concurrent series; at most 3–4 simultaneous streams by default, with a lint warning above | SDR-26 (INFER: the cap is inferred from concurrent-earcon results, McGookin & Brewster 2004; tested by experiment H-04 in D01) |
| Stage B default | Multi-series playback is sequential (one series after another, each announced); simultaneous playback is opt-in and linted (proposal, given the weak evidence for the cap) | Proposal |
| Spatial redundancy | Analysis mode does not stack pan, elevation and pitch on one variable | SDR-27 (SHOULD) |
| Spacing | Simultaneous tones stay more than about 1 ERB apart (lint) | SDR-28 (INFER) |
| Space | Stereo or binaural for kinematic or positional variables or segregation; warn that generic HRTFs cause front–back confusion | SDR-30; D03 |
| Robustness | Pan never carries information alone; test in mono and with NVDA sound split. AUDAXIS checks every compiled Score: only pan, only timbre, or data pitches outside 65–1480 Hz along a data track is a warning (B03-T11), and a mono-downmix test checks that pitch information survives | A11Y-14 |

## 10. Monitoring soundscape

Monitoring mode uses a continuous ambient soundscape with event-driven salience changes, and reserves urgency cues for true alarms (SDR-35, SHOULD). The evidence is promising but thin: a within-subjects study with 18 participants found a continuous forest soundscape improved process monitoring without significantly hurting the main task (Hildebrandt et al. 2016), SoNSTAR raised situational awareness with lower NASA-TLX workload (Debashi & Vickers 2018), and a 2023 study found less ecologically coherent soundscapes were detected less accurately ([notes 03 §5](../research/notes/03-accessibility-hci.md); [notes 02 §4](../research/notes/02-theory-perception.md)). No independent replication of the 2016 study was found.

Design rules:

1. **Coherence.** Soundscape layers come from one coherent family (proposal), because coherence affected detection.
2. **Salience, not volume.** State changes alter density, brightness or rhythm; they do not jump in level (hearing safety, section 12).
3. **Events.** Stream events play `E-STREAM` and produce rate-limited announcements ([keymap §9.1](keyboard-and-navigation.md); A11Y-16).
4. **Alarms.** `E-ALARM` exists only for conditions the user configured; nothing else sounds urgent.
5. **Control.** Every stream can be paused, stopped or hidden, with its announcements (A11Y-16).
6. **Equivalents.** Events appear as visual markers and in the announcement log (A11Y-24).
7. **Validation.** The default soundscape is a B06 deliverable, evaluated with users and recorded as a finding before it becomes a default.

## 11. Comfort and fatigue

The comfort budget is an inference, not a validated standard (SDR-39, INFER). No validated psychoacoustic thresholds for long sonification sessions and no longitudinal fatigue study were found ([notes 02 §3](../research/notes/02-theory-perception.md)).

- **Lint.** ECHOMETRIC estimates sustained sharpness and roughness (Fastl & Zwicker models) and high-frequency energy over a session. Thresholds start as provisional, information-level diagnostics and are tuned by findings; they cannot become defaults without an experiment.
- **Defaults that follow the evidence.** Discrete tones for chart reading (Sharif, Wang & Muongchan 2022), ambient sound for long monitoring (Hildebrandt et al. 2016), a low-pass on the square preset, and no sustained high-frequency-heavy timbres.
- **Session reminders (proposal).** After 45 minutes of cumulative audio output in a session, a polite status announcement and log entry: "Audio has been on for 45 minutes. Consider a short break." The interval is configurable; reminders are off in Presentation mode, never modal, and never interrupt playback.

## 12. Hearing safety

| ID | Rule |
|---|---|
| HS-01 | No unramped onsets anywhere (SDR-22); all gain changes are smoothed. |
| HS-02 | A true-peak limiter with look-ahead sits on the master, ceiling −1 dBTP (proposal). It is transparent in normal use. In Analysis mode, limiter engagement raises a diagnostic, because limiting breaks loudness constancy (SDR-23). |
| HS-03 | First launch starts the master at a conservative level (proposal: −20 dB), and the tutorial includes a "set a comfortable level" step with a reference tone. Later launches restore the last level. |
| HS-04 | No autoplay: opening a project or dataset never starts sound, which is stricter than WCAG 1.4.2's 3-second limit (A11Y-15). Instrument previews in the inspector last at most 1 s (proposal). |
| HS-05 | When the output device changes (hot-plug), playback pauses with an announcement and resumes only on user action (proposal). |
| HS-06 | Level controls move in steps of at most 2 dB (proposal). |
| HS-07 | "Stop all audio" is always available: Ctrl+Period (Cmd+Period) and the toolbar Stop button (A11Y-15). |
| HS-08 | `E-ALARM` is bounded by the limiter like everything else; its urgency comes from pattern (rate, repetition, pitch) rather than level (proposal). |
| HS-09 | Studies use calibrated, stated playback levels ([research report](../research/research-report.md), study methods). |

## 13. Personalization and profile presets

Users can personalise timbre, discrete or continuous marks, polarity, range and rate (SDR-38, MUST), and also earcons, context cues and verbosity. Settings persist per user profile (A11Y-10), stored by `sonarithm-app`. Presets are starting points; every value remains editable. All presets below are proposals pending B07 co-design.

| Setting | BLV chart reading | Analyst | Classroom | Presentation | Monitoring |
|---|---|---|---|---|---|
| Mode | Analysis | Analysis | Analysis | Presentation | Monitoring |
| Instrument | Discrete square (SDR-29) | Mallet | Mallet | Chosen instrument (AXONOTE, Stage C) | Soundscape |
| Rate | 3 points/s | 4 points/s | 2 points/s | Set by the narrative | — |
| Tick track (SDR-16) | On | Off | On | Off | — |
| Reference tone (SDR-17) | On | Off | On | Off | — |
| Spoken legend before first playback | On | Off | On | On | On |
| Speech | Through the screen reader | Off (log and readout only) | Screen reader if running, else self-voicing offered | Off | Through the screen reader, rate-limited |
| Verbosity | Terse | Terse | Verbose | — | Terse |
| Earcons | All | Boundary, invalid, landmark | All | Off during playback | `E-STREAM`; `E-ALARM` if configured |
| Braille | Off until enabled (B) | Off | Off | Off | Off |
| Visual highlight | On (for low vision and sighted collaborators) | On | On | On | On |
| Comfort reminders | On | Off | On | Off | On |

## 14. Sound asset licensing

- **Allowed sources:** CC0 assets, and sounds the project synthesises in code or records itself. Own recordings are dedicated CC0 (proposal), so exported audio carries no attribution duty.
- **Not allowed:** CC BY, CC BY-SA, NC licences, "royalty-free" libraries and any asset without a clear licence.
- **Provenance:** each asset has a record next to it (source or recipe, author, date, licence, tool, file hash) and REUSE/SPDX metadata.
- **Preference:** synthesis from code with fixed parameters and seeded noise, because it is reproducible (SDR-01, SDR-02) and needs no provenance chase.
- **Scope:** earcons, soundscape layers, instrument samples (including SF2 instruments for AXONOTE in Stage C) and tutorial audio. Assets live under `assets/` ([repository layout](../architecture/02-repository-and-crate-layout.md)).

## 15. Proposing a new sound

A new earcon, instrument, soundscape layer or changed default is proposed as follows:

1. **Issue.** State the purpose, the user need and the rule IDs affected.
2. **Specification.** Give the acoustic description and the synthesis recipe, or the CC0 source with provenance.
3. **Rule check.** Show the earcon rules (section 8) or instrument constraints are met, and attach the lint output (at least SDR-22, SDR-28, SDR-39).
4. **Robustness check.** Listen in mono, with NVDA sound split, and with a high-frequency roll-off (A11Y-14).
5. **Distinctness test.** Run a short identification test against the existing earcons and data voices with at least five listeners, including blind or low-vision co-designers where possible, and report the confusion matrix.
6. **Text equivalent.** Provide the text for the log and announcements (A11Y-24).
7. **Finding.** Record the results as a finding (`F-###`) in the [findings log](../research-paper/findings-log.md).
8. **Change control.** A change to a default states the old and new value and links the finding ([design rules](../research/design-rules.md), rule 1).
9. **Review.** The sound design lead and the accessibility lead approve; the catalogue in [keyboard and navigation §11](keyboard-and-navigation.md) and this document are updated in the same PR.

## Change log

| Date | Change | By |
|---|---|---|
| 2026-10-18 | Baseline created: modes, instruments, pitch, timing, loudness and exports, context sounds, earcon rules, streams, monitoring, comfort, hearing safety, presets, licensing, proposal process. | Kickoff |
