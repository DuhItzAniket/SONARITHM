---
status: accepted
date: 2026-11-26
decision-makers: ["coordinator session, under @DuhItzAniket's delegation"]
consulted: ["ECHOMETRIC session (C05 phase plan, D2)"]
---

# 0032 · The lint pass: a core `Linter` that reads the compile and writes nothing to the Score

## Context and problem statement

C05-T07 makes perceptual lint real. It covers the rules [RFC-0004](../rfcs/0004-spec-v0.2.md) §4 lists (SDR-14, 17, 19, 21, 23, 26, 27, 28, 39 and A11Y-14), moves NUMISONA's perceptual warnings into it (the [A06 phase review](../sdlc/reviews/A06-phase-review.md) follow-up), and applies the spec's `lint` key: levels, suppressions and `experimental`. [ADR-0014](0014-capability-trait-signatures.md) made lint a `Shaper`, the last stage of 05 §2, and fixed `Shaper` as working "on the `Score` only". Two things don't fit:

- **Inputs.** A shaper gets `&mut Score` and the compile context, nothing else.
  - SDR-21 counts trend reversals in a series's values.
  - SDR-14 needs the field's concept and where its polarity came from.
  - Every rule needs the resolved `lint` levels and suppressions.

  None of this is in the Score.
- **The Score.** The compiler records each shaper stage it runs in `score.header.shapers`. A registered lint shaper would therefore change every Score's header and hash. C05-T07's acceptance says "Score snapshots are unchanged after the move, only diagnostics move". F-197 met the same problem when CURVASONA's context provider was recorded in every Score.

## Decision drivers

- 05 §2: "Lint always runs last and never mutates the Score." RFC-0004 §4: "Effect on the Score. None."
- C05-T07's acceptance: Score snapshots unchanged, only diagnostics move.
- AGENTS rule 4: engines depend on `sonarithm-core` types only, so not on `sonarithm-spec` for the `lint` key's types.
- One owner per rule. Every lint diagnostic carries its rule IDs (design-rules change control).

## Considered options

1. **Keep `Shaper`, widen its signature** with the spec, data and annotations, and leave lint out of `header.shapers`.
2. **A `Lint` stage reading a side channel** the compiler fills in `CompileContext`.
3. **A separate core trait, `Linter`**, run after the shapers on an immutable view of the whole compile.

## Decision outcome

Chosen: **option 3** (coordinator under the owner's delegation, 2026-11-26).

In `sonarithm-core`:

```rust
/// A lint pass (ADR-0032): it reads the finished compile and reports diagnostics. It
/// never changes the Score, and nothing about it is recorded in the Score.
pub trait Linter: Send + Sync {
    fn lint(&self, input: &LintInput<'_>, cx: &mut CompileContext) -> Result<(), CompileError>;
}

#[non_exhaustive]
pub struct LintInput<'a> {
    pub score: &'a Score,                 // after every shaper
    pub config: &'a LintConfig,           // the spec's `lint`, resolved
    pub annotations: &'a [Annotation],    // every analyzer's
    pub tracks: &'a [TrackFacts],         // per track: its mark and channel mappings
    pub fields: &'a [FieldFacts],         // per mapped field: concept, polarity and its
                                          // source, the values (shared, not copied)
}
```

- **`LintConfig`** is a core type the compiler builds from the spec's `lint`. Each rule ID gets an effective level (`off`, `info`, `warning` or `error`): the spec's level if it sets one, else the rule's default level from the linter. It also holds the suppressions (rule, optional track, reason) and the `experimental` flag. Engines never see `sonarithm-spec`'s `Lint`.
- **Registration:** `registrar.linter(Arc<dyn Linter>)`. The compiler runs every linter after the last shaper, in registration order.
  - It **records no linter in the Score**: neither `header.shapers` nor any other field changes.
  - The `Score` is passed by shared reference, so a linter cannot change it.
  - Linters get their diagnostics through `cx` and nothing else.
- **`ShaperStage::Lint`** stays in the IR enum, so existing Scores still read. It is documented as unused, and no shaper registers at that stage.
- **Diagnostics:** area `SNR-LINT`, one code per rule (SNR-LINT-001 onwards, ECHOMETRIC's second code table in `xtask codes`).
  - Each carries its rule IDs, at the rule's effective level.
  - An off or suppressed rule is reported once per rule and scope as `info`, as RFC-0004 §4 says.
  - The INFER rules (SDR-26, SDR-28, SDR-39) are off unless `lint.experimental` is true or the spec names them, and each of their messages says the rule is experimental.
- **The move.**
  - NUMISONA's encoders stop emitting `SNR-ENG-012`, `021`, `049`, `051`, `052`, `053` and `054`, which stay registered as retired and are never reused. ECHOMETRIC's linter reports the same rules from the Score and the facts.
  - NUMISONA keeps computing everything that changes the Score (pace, polarity, timbre), so Scores do not change.
  - AUDAXIS's overlapping warnings (`SNR-ENG-100`, `101`, `102`, `104` and `105`, for A11Y-14 and SDR-17) stay where they are for now. Moving them is a follow-up issue, outside C05.

### Consequences

- Good: lint can see everything its rules need, and the Score and its hash stay exactly as they were.
- Good: one owner and one code per rule. `lint` levels and suppressions apply in one place.
- Good: engines still depend only on `sonarithm-core`.
- Bad: a second extension trait beside `Shaper`, and the facts types to keep in step with what the rules read. New rules may grow `TrackFacts` and `FieldFacts` (additive, `#[non_exhaustive]`).
- Bad: until the AUDAXIS follow-up, A11Y-14 and SDR-17 have diagnostics from two engines.
- Neutral: live mode (05 §6) runs the linter on the sliding window, as it would have run the shaper.

### Confirmation

C05-T07:
- per rule, a passing and a failing test named after the rule;
- tests for suppression and levels;
- an `xtask codes` check that the retired ENG codes stay registered;
- **every insta Score snapshot and golden record unchanged** by the move. Only the diagnostics change, in the CLI and compile-report snapshots, and each change is explained in the PR.

## Pros and cons of the options

### Option 1

- Good: no new trait.
- Bad: every shaper (quantize, loudness, spatial) gains inputs it does not use. Skipping lint in `header.shapers` would be a special case inside a general mechanism.

### Option 2

- Good: small.
- Bad: a stage reading facts from a side channel is an undocumented contract, and lint would still be a stage that is not recorded.

### Option 3

- Good: says what lint is: a read-only pass with the inputs it needs.
- Bad: the new trait and facts types (above).

## More information

- [ADR-0014](0014-capability-trait-signatures.md) (amended here: lint is a `Linter`, not a `Shaper`; the rest stands)
- [RFC-0004](../rfcs/0004-spec-v0.2.md) §4
- 05 §2 (the shaper order, updated with C05-T07)
- 07 ECHOMETRIC
- [F-084](../research-paper/findings-log.md) (SDR-21's threshold, TODO(verify), with hysteresis in ECHOMETRIC)
- [F-197](../research-paper/findings-log.md)
