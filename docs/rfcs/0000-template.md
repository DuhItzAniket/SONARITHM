# RFC NNNN: <title>

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead. This is the RFC template. In a copy, replace this block with: `> **Status:** Draft. **Author(s):** <roles or handles>.` and keep the status line current as the RFC moves through its [lifecycle](README.md#2-lifecycle).
> **Related:** [RFC process](README.md) · [design-rules](../research/design-rules.md) · [hypotheses](../research-paper/hypotheses.md) · [decisions](../decisions/README.md)

<!--
Copy to docs/rfcs/0000-<slug>.md and open a draft PR. Rename to NNNN-<slug>.md when the PR leaves draft.
Write for reviewers who know the architecture docs but not your idea. Delete guidance comments as you fill sections.
-->

| Field | Value |
|---|---|
| RFC | NNNN (assigned when the PR leaves draft) |
| Start date | YYYY-MM-DD |
| RFC PR | <link> |
| Affects | <spec language / Score IR / plugin WIT world / IPC protocol / cross-engine contracts / persisted formats / governance> |
| Phase | <A01…D06, or "unscheduled"> |
| Related ADR(s) | <ADR-NNNN links, filled in after acceptance> |
| Supersedes | <RFC-NNNN or "none"> |

## Summary

<One paragraph: what changes, for whom.>

## Motivation

<The problem, who has it, and the use cases this enables. Evidence: issues, findings (F-###), user or co-design feedback, benchmark data. What happens if we do nothing.>

## Guide-level explanation

<Explain the change as if it were already shipped, to the people who will use it: spec authors, CLI and Python users, desktop users, engine or plugin authors. Include spec examples, CLI invocations, Python snippets or UI behaviour. Introduce new terms here.>

## Reference-level explanation

<The precise design, detailed enough to implement and review:>

- **Types and schemas:** Rust types, JSON Schema changes, Score IR changes, WIT definitions, IPC messages.
- **Behaviour:** interaction with existing features and engines; corner cases; error cases and their diagnostic codes (`SNR-<AREA>-<NNN>`).
- **Compatibility and migration:** `specVersion` bump (minor or major), `ir_version` bump, migration path and `sonarithm spec migrate` behaviour, deprecation timeline.
- **Determinism:** effect on byte-identical Score output and audio tolerances ([ADR-0013](../decisions/0013-determinism-and-provenance.md)).
- **Real-time safety:** anything that reaches the audio thread ([coding-standards §11](../engineering/coding-standards.md#11-real-time-audio-rules)).
- **Performance:** expected effect on the budgets in [01-overview §7](../architecture/01-overview.md#7-quality-attributes-and-budgets), and how it will be measured.
- **Security and privacy:** new inputs to parse, sandboxing, network access (P8), licensing of new dependencies (P9).
- **Testing:** snapshot, golden, property, fuzz and conformance tests that will prove it ([testing-strategy](../engineering/testing-strategy.md)).

## Drawbacks

<Why we might not do this: cost, complexity, maintenance, migration burden, cognitive load for users.>

## Rationale and alternatives

<Why this design is the best of the options. Each alternative considered, and why it lost. The impact of not doing this.>

## Prior art

<How others solved this problem, and what we learn from them, good and bad.>

- **Sonification tools:** for example Erie (declarative sonification grammar), MAIDR (multimodal access to charts: braille, text, sonification), Highcharts sonification, Apple Audio Graphs, Chart Reader, Umwelt.
- **Visualization grammars and accessibility work:** for example Vega-Lite, Chartability.
- **Research literature:** cite papers; link the relevant design rules.
- **Other software:** Rust RFCs or other projects with a comparable mechanism.

## Accessibility impact

<Which A11Y requirements are affected (IDs from the [catalogue](../research/design-rules.md#a11y-accessibility-requirements)). Effect on screen-reader announcements, keyboard interaction, braille output and non-audio equivalents (A11Y-24). Whether essential information still survives mono output and NVDA sound split (A11Y-14). The manual checks this change will need. "No impact" must be argued, not asserted.>

## Research impact

<Which design rules are affected (SDR and A11Y IDs). Whether a default changes: if so, the old value, the new value and the evidence (F-### or EXP-###), per the catalogue's change rules; `INFER` rules cannot become `DEFAULT` or `SHOULD` without a SONARITHM experiment. Hypotheses (H-##) this touches or makes testable. Findings to log. Experiments or studies needed, and whether they need ethics approval.>

## Unresolved questions

<What must be settled during discussion, what can wait for implementation, and what is out of scope. Keep this list current during review.>

## Future possibilities

<Natural extensions this enables, without committing to them.>
