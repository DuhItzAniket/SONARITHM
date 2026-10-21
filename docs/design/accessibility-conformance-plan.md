# Accessibility conformance plan

> **Status:** Baseline v1 (2026-10-18). **Owner:** Accessibility lead.
> Related: [design rules: A11Y-01..34](../research/design-rules.md), [research notes 03 §3, §6, §7](../research/notes/03-accessibility-hci.md) (standards and research methods), [research report](../research/research-report.md) ("Accessibility compliance is a conformance programme"), [roadmap B07 and C06](../planning/roadmap.md), [testing strategy](../engineering/testing-strategy.md), [definition of ready and done](../sdlc/definition-of-ready-and-done.md), [findings log](../research-paper/findings-log.md), [keyboard and navigation](keyboard-and-navigation.md), [design language](design-language.md), [sound design](sound-design.md).

This plan says what SONARITHM claims about accessibility, how each release earns that claim, who decides, and how every requirement is traced to a test. It implements A11Y-31 to A11Y-34 and gives the verification route for the other A11Y requirements. It is not legal advice.

## 1. Conformance target

| Standard | Position | Notes |
|---|---|---|
| **WCAG 2.2 Level AA**, applied to software through **WCAG2ICT** | **Target** for every release from `v0.5.0-beta` | WCAG 2.2 is ISO/IEC 40500:2025. WCAG2ICT (W3C Group Note, 8 October 2024) is informative guidance on applying WCAG to non-web software ([notes 03 §3](../research/notes/03-accessibility-hci.md)). |
| **EN 301 549 v4.1.1** | **Targeted** | Published 2 September 2026 and aligned to WCAG 2.2; not yet cited in the EU Official Journal as of mid-October 2026. |
| **EN 301 549 v3.2.1** | **Documented** | The legal reference until v4.1.1 is cited. Relevant clauses: 4 (functional performance statements), 5, 11 (non-web software, including 11.5 interoperability with assistive technology, 11.6 documented accessibility usage, 11.7 user preferences, 11.8 authoring tools) and 12 (documentation and support). These clause numbers come from prior knowledge in the research and must be re-verified before any ACR quotes them [CHECK]. |
| **Section 508** (Revised 508 Standards) | **Documented** | Still incorporates WCAG 2.0 AA, which WCAG 2.2 AA covers in practice, plus Chapter 3 (functional performance criteria), Chapter 5 (software: AT interoperability, platform preferences) and Chapter 6 (support documentation). |
| European Accessibility Act | Monitored | A free, standalone sonification app is probably outside its product and service categories, unless embedded in an in-scope service. Legal analysis from the research, not legal advice; revisit if SONARITHM is commercialised. |
| WCAG 3.0 | Tracked, never claimed | Working Draft only. |

The audio-first design makes the "use without hearing" requirements as important as "use without vision": EN 301 549 clause 4 and Section 508 302.4/302.5 [CHECK]. Section 7 covers them.

### 1.1 Scope of the claim (proposal)

| Component | In scope | From |
|---|---|---|
| Desktop app on Windows, macOS and Linux, including installers, updater prompts and help | Yes | `v0.5.0-beta` |
| Built-in tutorial and context help (A11Y-30) | Yes | `v0.5.0-beta` |
| Documentation site and README (Section 508 Chapter 6; EN 301 549 clause 12) | Yes | `v1.0.0` |
| CLI (`sonarithm`) | Applicable criteria only (text output, no colour-only meaning, no timing) | `v1.0.0` |
| Exported files (audio with speech/cue sidecar, specs, manifests) | Documented as content, not as UI | `v1.0.0` |
| Python and Rust APIs | Documentation only | `v1.0.0` |

## 2. Accessibility conformance report (ACR) per release

A11Y-31 requires an ACR with each release. ACR v0 ships with `v0.5.0-beta` (B07) and ACR v1 with `v1.0.0` (C06) ([roadmap](../planning/roadmap.md)). Every later minor or major release updates it; patch releases update it when they change UI or fix an accessibility issue.

### 2.1 Process

| Step | Activity | Output | Responsible |
|---|---|---|---|
| 1 | Freeze the release candidate | Build ID, webview and OS versions recorded | Release manager |
| 2 | Automated checks: axe (Storybook and Playwright), keyboard end-to-end, contrast-token check | CI report; zero serious or critical axe violations ([testing strategy](../engineering/testing-strategy.md)) | CI |
| 3 | Manual screen-reader matrix (section 3) | Matrix report with AT versions | Accessibility lead, testers |
| 4 | Chartability audit (section 4) | Audit sheet | Accessibility lead |
| 5 | Braille, low-vision and Deaf/hard-of-hearing checks (sections 5–7) | Check sheets | Testers |
| 6 | Triage against severity definitions (section 9) | Issue labels; release-blocking decision | Accessibility lead |
| 7 | Draft the ACR from the template (2.2); map remarks to A11Y IDs | Draft ACR | Accessibility lead |
| 8 | Sign-off and publication with the release notes; linked from README and docs | Published ACR (proposed location: `docs/accessibility/acr/<version>.md`, plus a release asset) | Accessibility lead, release manager |
| 9 | Retrospective: findings logged, plan updated | Findings (`F-###`) | Accessibility lead |

### 2.2 ACR template outline (VPAT-style)

The structure follows the ITI VPAT INT edition, which covers WCAG, Section 508 and EN 301 549 in one report; the current edition is confirmed at B07.

1. **Product information:** name, version, release date, platforms and webview versions, contact route (the public issue tracker; no personal email addresses).
2. **Report scope:** what is covered and excluded (section 1.1).
3. **Evaluation methods:** automated tools and versions, AT and versions, manual task suite, Chartability audit, co-design inputs, dates.
4. **Applicable standards:** WCAG 2.2 A and AA via WCAG2ICT; EN 301 549 v3.2.1 and v4.1.1; Revised Section 508.
5. **Terms:** Supports; Partially Supports; Does Not Support; Not Applicable; Not Evaluated.
6. **WCAG 2.2 tables:** Level A, Level AA, each criterion with level, conformance term and remarks (remarks cite A11Y IDs and known issues). AAA criteria used as guidance (for example 1.4.7 for speech and sonification overlap, A11Y-20) are noted separately and not claimed.
7. **EN 301 549 tables:** clauses 4, 5, 11 and 12 [CHECK clause numbering].
8. **Section 508 tables:** Chapter 3, Chapter 5, Chapter 6.
9. **Known issues:** open accessibility issues with severity, affected AT and workaround.
10. **Disclaimer:** an ACR describes the evaluated version; it is not a legal opinion.

## 3. Manual screen-reader matrix

Automated scanners cannot judge announcements, reading order, missing context, traps or broken keyboard interaction, so every release has a manual pass ([notes 05 §9](../research/notes/05-desktop-frontend-stack.md)). A11Y-32 fixes the minimum: NVDA, JAWS and Narrator on Windows; VoiceOver on macOS; Orca on X11 and Wayland; NVDA ducking and sound split; at least one refreshable braille display. JAWS and NVDA together held most of the primary desktop screen-reader share in WebAIM Survey #10 (an uncontrolled sample), so Windows coverage is non-negotiable.

### 3.1 Configurations

| ID | OS | Webview | AT | Version policy | Extra conditions | Task set |
|---|---|---|---|---|---|---|
| M-01 | Windows 11 | WebView2 | NVDA | Current stable and previous release | Ducking: off, "duck when outputting speech and sounds", "always duck"; sound split on and off | Full |
| M-02 | Windows 11 | WebView2 | JAWS | Current release | Default verbosity | Full |
| M-03 | Windows 11 | WebView2 | Narrator | Built-in | Scan mode behaviour recorded | Core |
| M-04 | macOS, current and previous major | WKWebView | VoiceOver | Built-in | Quick Nav off; then on, to document behaviour | Full |
| M-05 | Ubuntu LTS, GNOME on Wayland | WebKitGTK | Orca | Distribution version | — | Full |
| M-06 | Ubuntu LTS on X11, or Fedora | WebKitGTK | Orca | Distribution version | — | Core |
| M-07 | Windows 11 | WebView2 | NVDA with a refreshable braille display | Current stable | Contracted and uncontracted braille | Braille |
| M-08 | macOS | WKWebView | VoiceOver with a braille display | Built-in | Rotates with M-07 by release | Braille |
| M-09 | Windows 11 | WebView2 | Magnifier; High Contrast theme | Built-in | 200 % and 400 % | Low vision |
| M-10 | macOS | WKWebView | Zoom; Increase Contrast | Built-in | 200 % and 400 % | Low vision |

Previews and nightlies run M-01, M-04 and M-05 with the core set. Betas and stable releases run every configuration. Exact AT, OS and webview versions are recorded in each report; this plan sets the policy, not version numbers.

### 3.2 Scripted task suite

Each task has a pass criterion. A task passes when the tester, using only the keyboard and the AT, completes it unaided, gets the correct answer, hears or reads correct announcements, and meets no trap. Time, errors and notes are recorded.

| ID | Task | Pass criterion | A11Y |
|---|---|---|---|
| T-01 | Open a CSV and hear the summary | Summary announced with chart type, series count, ranges and units; chart never announced as an unlabeled graphic | 01, 02, 17 |
| T-02 | Play the overview, pause, resume | Playback starts on command only; current value announced on pause | 09, 15 |
| T-03 | **Find the maximum** of a series | Correct value and x position reported | 04, 07, 22 |
| T-04 | **Compare two series** at a given x | Both values reported with series names and units | 04, 11 |
| T-05 | **Read the value at x** | Go-to field reached and correct value reported | 22, 27 |
| T-06 | Where am I, after getting lost | Position, level and neighbours reported; return to the series root by landmark key | 07 |
| T-07 | Toggle modalities and verbosity | Each toggle announced; settings persist after restart | 10, 12 |
| T-08 | **Change a mapping:** flip polarity and change the pitch range in the inspector | Values announced with unit and range; undo restores the previous state | 29; SDR-15 |
| T-09 | Navigate insights and jump to an insight's data | Insight announced; Enter moves the cursor to the data | 08, 23 |
| T-10 | Use the Table view and confirm the linked cursor | Row and column headers announced; plot cursor follows | 03, 11 |
| T-11 | **Export** audio with its cue sidecar | Export completes; result announced with duration, sample rate and loudness | 24, 29 |
| T-12 | Edit the spec, find and fix a diagnostic | Diagnostic reached and announced; fix applied | 29 |
| T-13 | Stop all audio from any region | Silence within the interaction budget, from every region | 15 |
| T-14 | Pause and hide a live stream (from B06) | Stream and announcements pause; rate limits respected | 16 |
| T-15 | Remap a shortcut and disable single-key commands | Remap works; letters inert in the explorer when disabled | 06 |

T-03, T-04 and T-05 are the B03 exit tasks ([roadmap](../planning/roadmap.md)). The core set is T-01, T-02, T-03, T-05, T-06, T-11 and T-13.

## 4. Chartability audit cadence (A11Y-33)

- **Every release:** Chartability's 14 critical tests on the visual mode (sighted and low-vision use).
- **Every major release** (`1.0.0`, `2.0.0`): all 50 heuristics under the seven principles (POUR plus Compromising, Assistive, Flexible).
- Auditors rotate between releases where people are available, because a fresh auditor finds different issues; results go into the ACR remarks and the issue tracker with severities.

Source: Elavsky, Bennett & Moritz 2022 ([notes 03 §1](../research/notes/03-accessibility-hci.md)).

## 5. Braille testing

Run on M-07 and M-08 each release, with at least one display (A11Y-32):

| Check | Pass criterion | A11Y |
|---|---|---|
| Review line | Shows value, unit, x and position for the cursor; persistent, not a flash message | 21 |
| Sparkline mode | Encodes the current segment's shape; cell count follows the user's setting | 21 |
| Routing keys | Pressing a routing key over a sparkline cell moves the cursor to that point | 27 |
| Braille table | Contracted and uncontracted (and computer braille for numbers) render values correctly | 21 |
| Consistency | Braille, speech, log and visual readout agree on value, unit and precision (DL-P12) | 10 |
| No dependence on flash messages | Nothing important appears only as a transient flash message | 17 |

## 6. Low-vision checks

| Check | Method | A11Y / WCAG |
|---|---|---|
| Zoom 200 % and 400 % | In-app zoom and OS scaling; Narrow layout engages; no loss of function or content; F6 order unchanged | 25; 1.4.4 |
| High contrast | High-contrast theme complete; all contrast pairs pass | 25; 1.4.3, 1.4.11 |
| Forced colours | Windows High Contrast: chrome maps to system colours; plots redraw with system colours; signals keep icon or text | 25; 1.4.1 |
| `prefers-contrast` | macOS Increase Contrast selects the high-contrast theme | 25 |
| Magnifier focus tracking | Windows Magnifier and macOS Zoom follow focus and the chart cursor | 25 |
| Focus visible | 2 px ring with offset, at least 3:1; never hidden by sticky regions | 05; 2.4.7, 2.4.11 |
| Target size | All targets at least 24×24 CSS px | 26; 2.5.8 |
| Reduced motion | Transitions off; plot scrolls by page | 25 |
| Colour vision | CVD simulation (protan, deutan, tritan) of series palettes, signals and engine tiles; redundant encodings present | 25; 1.4.1 |

## 7. Deaf and hard-of-hearing equivalents (A11Y-24)

Every sonified insight has a non-audio equivalent, so Deaf, hard-of-hearing and DeafBlind users are not excluded by the audio-first design.

| Audio output | Non-audio equivalent |
|---|---|
| Point cue and playback | Visual highlight and playhead; persistent cursor readout; braille review line |
| Earcons | Announcement-log entry with the earcon's text equivalent; status-bar indicator for mode and modality changes ([keymap §11](keyboard-and-navigation.md)) |
| Spoken announcements | Announcement log (text, timestamped, linked to data); braille |
| Insights heard as landmarks | Visual markers on the plot; Insights branch in the explorer; query answers |
| Monitoring soundscape and stream events | Event markers on the stream plot; log entries; status-bar counters |
| Tutorial audio | Captions and a text transcript |
| Exported audio | Speech and cue sidecar file (text cues with timestamps) and the render manifest |

Checks each release: a tester with audio muted completes T-01, T-03, T-04, T-05, T-09 and T-11 using only visual and text outputs, and a braille-only pass (no audio) completes the core set on M-07.

## 8. Blind and low-vision co-design programme (A11Y-34)

BLV people are paid co-designers and evaluators throughout: about 10–15 participants per evaluation round, remote sessions on their own assistive technology, compensation of about $50–85 per hour, demographics reported only in aggregate, and access planned in advance and adjusted as needed (A11Y-34; Mack et al. 2021, 2022; [notes 03 §6](../research/notes/03-accessibility-hci.md)).

### 8.1 Cadence

| Activity | When | Size | Purpose |
|---|---|---|---|
| Parallel co-design partners | Continuous from B02/B03 to C06 | 2–3 paid partners, one-on-one sessions per phase (proposal, following Chart Reader's parallel co-design) | Shape the explorer, keymap, announcements, sounds before evaluation |
| Evaluation round 1 | B07 (`v0.5.0-beta`) | 10–15 participants | Gate B: a BLV user completes the core tasks unaided in at least 80 % of attempts ([roadmap](../planning/roadmap.md)) |
| Evaluation round 2 | C06 (`v1.0.0`) | 10–15 participants | Validate Stage C engines and fixes |
| Research studies | D06 | Per preregistration | Published studies ([roadmap](../planning/roadmap.md)) |

Ethics approval comes before any study. The owner works alone without an ethics route ([OD-04](../planning/open-decisions.md#closed-decisions)), so evaluation round 1 runs as paid product usability feedback, not as a study: no research data is published from it, and it waits for RV University's policy to be confirmed in writing, a compensation budget and recruited participants ([implementation plan §3.3](../planning/implementation-plan.md#33-owner-and-human-paced-items), HP-09).

### 8.2 Recruiting

- Channels used in the research: blindness organisations such as NFB, and Program-L (MAIDR); blind-community collaborators and public calls (Zong et al. 2022).
- Additional channels (operational, not research-derived): national blindness organisations outside the US, university disability services, and AT user communities.
- Mix: JAWS, NVDA, VoiceOver and Orca users; braille readers; people with hearing differences; congenitally and adventitiously blind people; low-vision magnifier users ([notes 03 §6](../research/notes/03-accessibility-hci.md)).
- Participants may bring their own data, because Chart Reader's partners engaged less with data they did not care about.

### 8.3 Compensation

- About $50–85 per hour (or the local equivalent), including time for scheduling, setup and AT troubleshooting.
- Paid by a method the participant chooses and can use without sighted help; no inaccessible gift-card portals.
- Non-screen-reader comparison participants are paid fairly but separately ([notes 03 §6](../research/notes/03-accessibility-hci.md)).

### 8.4 Consent and session accessibility

- Consent forms, surveys and scheduling tools are tested with screen readers and at 200 % zoom before use.
- Consent is offered as accessible HTML or plain text, with large print or braille-ready formats on request, and with the option of recorded verbal consent.
- Recording is consented separately; participants can withdraw at any time without losing payment for time spent.
- Sessions run remotely on the participant's own configured AT, with breaks; session length follows field practice (about 90 minutes to 3 hours) and the participant's needs.
- Data handling follows the project's privacy principle (offline-first, no telemetry by default; [architecture P8](../architecture/01-overview.md)); demographics are reported only in aggregate.

### 8.5 Outputs

Each session produces findings (`F-###`) in the [findings log](../research-paper/findings-log.md). Findings are triaged into issues with severities (section 9). Keymap, announcement and sound changes go through the change control in [keyboard and navigation](keyboard-and-navigation.md) and [sound design](sound-design.md). Participants are told what changed as a result of their sessions.

## 9. Accessibility issue severity

| Severity | Definition | Examples |
|---|---|---|
| **Critical** | A core task is impossible for a user group, with no reasonable workaround, or the product causes harm | Chart explorer unreachable or silent with NVDA; keyboard trap; "stop all audio" fails; a loud unramped onset or autoplay; a chart announced as an unlabeled "graphic"; the app crashes when a screen reader starts |
| **High** | A core task is possible only with a significant workaround, or output is wrong | Announcements queue behind the cursor instead of superseding; table headers not exposed; primary text below 4.5:1; an earcon with no text equivalent; a screen reader reads a wrong value or unit; an estimate announced without "estimated" |
| **Medium** | A non-core task is blocked, or a core task needs a minor workaround | Verbosity part order ignored; a secondary icon button without a tooltip; focus ring clipped in the Narrow layout; braille sparkline truncated at small cell counts |
| **Low** | Polish; no task impact | An extra pause in an announcement; a redundant phrase; minor misalignment in the high-contrast theme |

Rules:

- A regression of a criterion the last ACR marked "Supports" is at least **High**.
- Severity is set by impact on the affected user group, not by how many users are affected.
- Accessibility issues carry the `a11y` label, a severity label and the A11Y IDs they break.

### 9.1 Release-blocking policy

| Severity | Nightly / preview | Beta (`v0.5.0-beta` onwards) | Stable (`v1.0.0` onwards) |
|---|---|---|---|
| Critical | Fix before the next preview | **Blocks** | **Blocks** |
| High | Allowed, tracked | Blocks unless the accessibility lead grants a written waiver that is disclosed in the ACR | **Blocks** (Gate C: no open critical or high bugs) |
| Medium | Allowed | Allowed; listed in the ACR | Allowed; listed in the ACR with a planned fix |
| Low | Allowed | Allowed | Allowed |

This matches the roadmap: B07 exits with no open critical accessibility issues, and Gate C with no open critical or high bugs ([roadmap](../planning/roadmap.md)). Triage happens within one week of report (proposal); a critical issue reported against a release triggers a patch release.

## 10. Roles

| Role | Responsibilities |
|---|---|
| **Accessibility lead** (may be held by the project lead until staffed) | Owns this plan and the ACR; runs or delegates the matrix and Chartability audits; sets severities; grants or refuses waivers; owns the co-design programme with the research lead; signs off each release for accessibility |
| Design lead | Owns the design language and engine identity; runs design reviews against DL-P and DL-X |
| Sound design lead | Owns sound design; reviews earcons and presets with the accessibility lead |
| Research lead | Ethics approvals, consent materials, compensation, findings log, study design |
| Release manager | Freezes builds, records versions, publishes the ACR with the release |
| BLV co-design partners (paid) | Co-design sessions, evaluation, review of announcements and keymap changes |
| Testers (contributors, volunteers) | Run matrix configurations they own the AT for, report with versions |
| Project lead | Escalation for waivers and scope changes |

## 11. Traceability

Each A11Y requirement has a primary verification route and a phase where it first becomes testable. The [definition of done](../sdlc/definition-of-ready-and-done.md) requires work items to cite the A11Y IDs they implement and the tests that verify them.

| ID | Requirement (short) | Primary verification | First phase |
|---|---|---|---|
| A11Y-01 | Named chart with auto summary | Automated name checks; T-01 | B03/B04 |
| A11Y-02 | Summaries default to L2/L3 | Description unit tests; co-design review | B03, C05 |
| A11Y-03 | Accessible data table | axe; T-10 | B04/B05 |
| A11Y-04 | Hierarchical navigation, four verb types | AUDAXIS tests; T-03, T-04, T-05 | B03 |
| A11Y-05 | Keyboard-operable, no traps, visible focus | Keyboard end-to-end; matrix; section 6 | B02 |
| A11Y-06 | Single-key shortcuts scoped, remappable, disable-able | Keymap tests; T-15 | B03 |
| A11Y-07 | Landmarks and "where am I" | Tests; T-06 | B03 |
| A11Y-08 | Distinct earcons with text equivalents | Earcon catalogue tests; log checks; section 7 | B03 |
| A11Y-09 | Overview then detail; value on pause | T-02 | B03 |
| A11Y-10 | Toggleable modalities, one cursor, persisted | Tests; T-07; section 5 consistency | B03/B04 |
| A11Y-11 | Linked selection across modalities | T-04, T-10 | B04 (cursor), D01 (selection) |
| A11Y-12 | Verbosity and part order | Composer tests; T-07; co-design | B03 |
| A11Y-13 | x→time, y→pitch; redundant duration or taps | NUMISONA unit tests | A06 |
| A11Y-14 | Not only pan, pitch extremes or timbre | Lint tests; mono and sound-split runs (M-01) | A06, B03, C05 |
| A11Y-15 | Independent volume; no autoplay; stop all audio | Tests; T-02, T-13 | B01/B03 |
| A11Y-16 | Streams pausable; rate-limited announcements | B06 tests; T-14 | B06 |
| A11Y-17 | Platform announcements; supersession | Announcer tests; matrix | B03 |
| A11Y-18 | Correct role, name, value, state for custom controls and chart nodes | axe; platform inspectors (Accessibility Insights for Windows, Xcode Accessibility Inspector, Accerciser); matrix | B02 |
| A11Y-19 | Speech through the screen reader; optional self-voicing | Matrix; self-voicing tests | B03 |
| A11Y-20 | Speech and sonification sequenced; ~20 dB rule on overlap | Audio timing tests; M-01 ducking runs | B03 |
| A11Y-21 | Braille sparkline and review line | Section 5 | B03 |
| A11Y-22 | Query commands | Statistics tests; T-03, T-05 | B03 (core), C05 |
| A11Y-23 | Navigable insights and narratives | T-09 | C05 |
| A11Y-24 | Non-audio equivalents | Section 7 | B03 onwards |
| A11Y-25 | Platform preferences and low-vision needs | Section 6 | B02 |
| A11Y-26 | Non-drag alternatives; target size | Automated target checks; manual | B04 |
| A11Y-27 | Discrete-step input beyond the keyboard | Braille routing (section 5); gamepad and switch tests when implemented | B03 (braille), later (gamepad, switch) |
| A11Y-28 | STEM function cues | CURVASONA tests; blind maths teacher pilot | C03 |
| A11Y-29 | Authoring UI accessible | T-08, T-12 | B05 |
| A11Y-30 | Tutorial and context help | Tutorial walkthrough on the matrix | B07 |
| A11Y-31 | Conformance target; ACR per release | This plan, section 2 | B07, C06 |
| A11Y-32 | Screen-reader matrix every release | Section 3 | B07 onwards |
| A11Y-33 | Chartability audits | Section 4 | B07 onwards |
| A11Y-34 | Paid BLV co-design | Section 8 | B02/B03 partners; B07 round 1 |

## Change log

| Date | Change | By |
|---|---|---|
| 2026-10-18 | Baseline created: target, ACR process and template, matrix, task suite, audits, co-design operations, severity and release policy, roles, traceability. | Kickoff |
