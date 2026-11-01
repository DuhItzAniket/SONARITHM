# C03-T09: blind maths teacher pilot

> **Status:** Draft materials (2026-11-01), not yet used. **Owner:** project owner (runs the sessions). **Route:** pending [OD-24](../../../planning/open-decisions.md) (recommended: (a) paid usability sessions as product feedback). **Gates:** C03-T08 (the function UI) shipped, Gate U, HP-19.
> **Related:** [implementation plan C03-T09](../../../planning/implementation-plan.md) · [accessibility conformance plan §8](../../accessibility-conformance-plan.md#8-blind-and-low-vision-co-design-programme-a11y-34) · [OD-04](../../../planning/open-decisions.md#closed-decisions) · [evaluation plan](../../../research-paper/evaluation-plan.md) · [07 CURVASONA](../../../architecture/07-engines.md)

C03's exit asks for at least three blind maths teachers to try the function features: typing a function, finding its zeros, extrema and asymptotes by listening, reading the critical-point list, and using the audification calculator. Each teacher is paid for their time.

**This is product usability feedback, not a research study** ([OD-04](../../../planning/open-decisions.md#closed-decisions)): the owner works without an ethics route, so nothing from these sessions is published as research data. Findings become product findings (`F-###`) and issues, reported only in aggregate, with no names or other personal data in the repository.

## Files

| File | Use |
|---|---|
| [session-script.md](session-script.md) | What the facilitator says and does, the tasks (P-1 to P-7) with their pass criteria, and the debrief questions |
| [information-and-consent.md](information-and-consent.md) | The information sheet, the consent items (with verbal consent), and the recruitment message |
| [results-template.md](results-template.md) | One record per session and the aggregate summary, with no personal data |
| [owner-steps-129-draft.md](owner-steps-129-draft.md) | What the owner must do, drafted as a comment for checklist issue #129. **Not posted** |

## Rules these materials follow

- Paid participants only, never unpaid volunteers ([A11Y-34](../../../research/design-rules.md), OD-24). The rate follows the plan's band of about $50–85 per hour or the local equivalent; the owner sets the amount (OD-10).
- Consent materials are checked with a screen reader and at 200 % zoom before first use, and offered as plain text or accessible HTML, with large print or braille-ready formats on request and recorded verbal consent as an option (conformance plan §8.4).
- Sessions are remote, on the teacher's own computer and screen reader, with breaks.
- Recording is optional and consented separately. Withdrawing at any time keeps the payment for time spent.
- No participant numbers, results or quotes are written here before the sessions happen. The results file holds only participant codes (P1, P2, …), never names, schools, contact details or anything that identifies a person.

## Before use

The tasks follow C03-T08's design ([the T08 proposal](../../../planning/implementation-plan.md): an expression field with live diagnostics, domain fields, the critical-point list in the Inspector, the audification calculator). When C03-T08 ships, check each step of the script against the real UI and keymap and update the wording. Tasks P-2 to P-4 are the pilot's form of the suite task T-18 ("find a function's zeros and asymptotes"), which C03-T08 adds to the [scripted task suite](../../accessibility-conformance-plan.md#32-scripted-task-suite).
