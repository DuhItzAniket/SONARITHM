---
status: accepted
date: 2026-10-31
decision-makers: ["coordinator session, under @DuhItzAniket's delegation"]
---

# 0029 · Function cues and audification in the Score IR per RFC-0007

## Context and problem statement

A11Y-28 asks for zero, asymptote, undefined and off-screen cues on function plots, and SDR-31 for audification by resampling. C03-T03 finds the points, but no Score event played them, and the Score IR had no event that plays samples. Changes to the Score IR are under change control (AGENTS.md rule 11), so [RFC-0007](../rfcs/0007-function-cues-and-audification.md) proposed them, with five questions.

## Decision outcome

**RFC-0007 is accepted with three amendments** (coordinator under the owner's delegation, 2026-10-31; 7-day FCP waived; the owner can reopen it), and Q1–Q5 as proposed:

- **Four earcon kinds**, `zero`, `asymptote`, `undefined` and `offscreen`, with render sounds that follow sound design §8. **E-ZERO's partials stay below 4 kHz** (amendment 2: 700, 1932 and 3780 Hz). As built, E-OFFSCREEN's taps are band-passed near 2 kHz, because high-passed noise put most of its energy above 4 kHz (ED-06), and E-ASYMPTOTE is a swell rather than a sweep of band-passed noise (keyboard and navigation §11). The sounds, the 150 ms merge window and the priority order are proposals without study evidence (INFER), experimental until the listening check (HP-21).
- **CURVASONA's `curvasona.cues` context track** plays the critical points of a function source when `analysis.detect` contains `critical-points`: earcons by default, speech with `cue: "speech"`, nothing with `cue: "none"`. No new spec key is added. Off-screen cues need an explicit pitch `scale.domain` (Q5). No automatic sign timbre is added, since there is no evidence for a default.
- **Merged cues keep the most important kind** (amendment 1): asymptote, then undefined, off-screen, zero, landmark.
  - **Refinement (coordinator, 2026-10-31):** an `Earcon`'s `src` is one `RowRef`, so a merged earcon cannot span several rows. Its `src` is the row of the highest-priority feature it carries, and `sonarithm_curvasona::cues::cue_log` names every merged feature. Every feature also stays an insight node, so none disappears from the text log, the insight list or speech.
- **The `Audify` event** (C03-T05) carries its samples inline, capped at 2^22. It is resampled at plan time, and its `k` bounds and SDR-31 report follow amendment 3. A band wholly outside 20 Hz–20 kHz is refused with a code. The limiter and hearing-safety rules apply to its output.
- **The IR version is per Score** (Q1): `IR_VERSION` is 2, the newest a reader understands; a Score records the lowest version that can read it (`Score::required_ir_version`), so Scores without the additions keep version 1 and their hashes.

### Consequences

- Good: a listener hears where a function's features are while it plays, and every cue has a text equivalent.
- Good: no existing snapshot or golden hash changes.
- Neutral: cue times depend on CURVASONA's rows at features and on a glide having one breakpoint per row; a test checks it.
- Bad: four more earcons to learn (ED-03); the tutorial must teach them.

### Confirmation

C03-T04 (issue [#317](https://github.com/DuhItzAniket/SONARITHM/issues/317)): the earcon kinds and sounds, the cue track, the tests named after A11Y-28, A11Y-08, A11Y-14, A11Y-24, SDR-22 and ED-06; the golden-audio case `function-cues` follows in its own PR, once its reference can be rendered. C03-T05 (issue [#318](https://github.com/DuhItzAniket/SONARITHM/issues/318)): the `Audify` event and the audifier voice.
