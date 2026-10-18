# Open Decisions

> **Status:** Living document, baseline 2026-10-18. **Owner:** Project lead.
> Decisions that block or shape work and are not yet made. When one is resolved, record the outcome here (and in an ADR if it is architectural), then mark it **Closed**.

| ID | Decision | Options | Recommendation | Owner | Needed by | Status |
|---|---|---|---|---|---|---|
| OD-01 | Code licence | MIT OR Apache-2.0; Apache-2.0; MPL-2.0; GPL-3.0 | MIT OR Apache-2.0 + DCO ([ADR-0010](../decisions/0010-licensing.md)) | **Owner** | A01 (before first public code) | **Closed** |
| OD-02 | Legal and organizational home: individual, university lab, or foundation/fiscal host | Individual; university; fiscal host | Decide before C06. It determines code-signing eligibility, CRA role and grant eligibility. **Known (2026-10-18):** the owner is an individual student at RV University (India). Azure Artifact Signing covers only individuals in the US/Canada and organizations in the EU/UK, so it is not available to the owner as an individual. Options: SignPath Foundation's free open-source signing, or signing through RV University | **Owner** | B07 (for signing planning) | Open |
| OD-03 | AXONOTE icon: the mockup glyph matches the SONARITHM platform icon (`audio-lines`) | Keep `audio-lines`; `audio-waveform`; a music-note glyph | `audio-waveform` (distinct, still "sound") | Owner + design | B02 | Open |
| OD-04 | Institution for ethics review (IRB) of user studies | University affiliation; collaborator's institution; independent review board | **Route identified (2026-10-18): RV University**, the owner's institution. Next: find a faculty mentor and the university's ethics committee process by the end of Nov 2026, so the application is filed in Dec 2026 (RL-T01, RL-T02) | **Owner** + research lead | Dec 2026 | In progress |
| OD-05 | Announcement routing in the desktop app | ARIA live regions only; native notification plugin on Windows/macOS; hybrid | Hybrid if A02-T07 shows live-region unreliability | Accessibility lead | A02 gate / B03 | Open |
| OD-06 | Default keys for lateral and local navigation | Per [keyboard-and-navigation](../design/keyboard-and-navigation.md) v0 | Validate in co-design round 1 | Accessibility lead | B07 | Open |
| OD-07 | Sonic treatment of low-confidence (estimated) data | Timbre change; noise layer; speech only | Speech/text always; timbre only after the H-06 study | Research lead | D06 | Open |
| OD-08 | Golden audio storage | Git LFS; regenerate-on-demand + hashes; external artifact store | Decide in A05-T11 | Core lead | A05 | Open |
| OD-09 | Documentation licence | CC BY 4.0; same as code | CC BY 4.0 for docs and datasets | **Owner** | A01 | **Closed** |
| OD-10 | Team capacity and funding (co-maintainers, BLV co-designer compensation, signing costs) | Volunteers; open-source grants; university funding; sponsorships | Draft a funding plan before Gate A (participant compensation of about $50–85/h is the largest research cost). **Known (2026-10-18):** solo developer for now; ask the faculty mentor (OD-04) about university research funding | **Owner** | Gate A | Open |
| OD-11 | Python API docs tool | pdoc; Sphinx + autodoc | Decide in A07 | Core lead | A07 | Open |
| OD-12 | Second UI locale | Community-driven choice | Pick with the community during B05 | Project lead | B07 | Open |
| OD-13 | Project website and domain (docs site, stable schema URLs) | GitHub Pages; custom domain | GitHub Pages first; a custom domain if available | Owner | C06 | Open |

## Closed decisions

| ID | Decision | Outcome | Date | Record |
|---|---|---|---|---|
| OD-01 | Code licence | **MIT OR Apache-2.0**, contributions under the DCO; copyright notice "SONARITHM contributors" | 2026-10-18 | [ADR-0010](../decisions/0010-licensing.md) |
| OD-09 | Documentation and data licence | **CC BY 4.0** for documentation, example datasets and study stimuli | 2026-10-18 | [ADR-0010](../decisions/0010-licensing.md) |
