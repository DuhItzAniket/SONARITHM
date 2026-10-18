<!--
PR title: a Conventional Commit header, for example `feat(numisona): add mel pitch unit`.
It becomes the squash-merge commit and the changelog entry. Scopes: docs/engineering/git-workflow.md.
Every commit needs a DCO sign-off (`git commit -s`). See CONTRIBUTING.md.
-->

## Summary

<!-- What changes and why, in a few sentences. -->

## Linked issue

Closes #

## Roadmap phase and task

<!--
Phase ID from docs/planning/roadmap.md (for example A06) and, if the work comes from
docs/planning/implementation-plan.md, the task ID (for example "Implements A06-T03").
Write "none" for maintenance.
-->

## Type of change

- [ ] `feat`: new capability
- [ ] `fix`: bug fix
- [ ] `perf`: performance
- [ ] `refactor`: no behaviour change
- [ ] `docs`
- [ ] `test`
- [ ] `build`, `ci` or `chore`
- [ ] Breaking change (`!` in the title and a `BREAKING CHANGE:` footer with migration notes)

## How it sounds

<!--
For changes that affect audio: what does a listener hear before and after?
Attach a short render (WAV or FLAC, about 10 seconds) or give the spec, data and command
to reproduce it, and state the playback level. "No audible change" is a valid answer.
-->

## Screenshots or recordings

<!--
UI changes: screenshots with a text description of each, and for screen-reader behaviour a
transcript of the announcements (for example from NVDA's Speech Viewer) or a recording.
Write "N/A" if there is no UI change.
-->

## AI assistance

<!-- Required by the AI disclosure policy in CONTRIBUTING.md. -->

- [ ] No AI tools were used.
- [ ] AI tools were used: <!-- tool(s) and what for -->. I have reviewed, understood and tested all of the output.

## Definition of Done

<!-- Tick each item, or write "N/A" after it with a short reason. Do not delete items. -->

- [ ] Tests added or updated; each acceptance criterion is covered by a test or a recorded manual check. Local checks (`cargo xtask ci`) pass.
- [ ] Code follows docs/engineering/coding-standards.md: no new warnings, no `#[allow]` without a reason, no new `unsafe` outside the unsafe register.
- [ ] Documentation updated (rustdoc, user docs, CLI or Python reference, architecture docs, traceability matrix) where behaviour, structure or rule coverage changed.
- [ ] The PR title is a Conventional Commit header that reads well as a changelog entry.
- [ ] Architectural or cross-cutting change: the ADR or RFC is linked here: <!-- docs/decisions/NNNN-... or docs/rfcs/NNNN-... -->
- [ ] Golden files or snapshots changed only on purpose (`cargo xtask golden --update`), audible changes listened to, and the reason stated here: <!-- reason -->
- [ ] Generated files (`schemas/`, `packages/ipc/src/generated/`) were regenerated, not edited by hand.
- [ ] UI change: tested with the keyboard only, with a screen reader (name and version: <!-- e.g. NVDA, VoiceOver -->), and with axe (no serious or critical violations); Definition of Done §3.2 holds.
- [ ] Real-time audio rules respected: no allocation, locks, I/O or logging on the audio thread; the RT review checklist (coding standards §11.6) is completed for audio code.
- [ ] New or updated dependencies: licences pass `cargo deny check` (and the npm licence check for frontend packages); significant dependencies have an ADR.
- [ ] Research findings produced by this work are logged in `docs/research-paper/findings-log.md` (F-### IDs listed here).
- [ ] `docs/sdlc/status.md` updated if this changes the progress of a roadmap phase.
- [ ] All commits are signed off (DCO).
