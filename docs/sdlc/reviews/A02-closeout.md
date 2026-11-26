# A02 close-out: evidence inventory and owner decisions

> **Status:** Proposal (2026-11-26), written by an AI agent for the coordinator session. The decisions it asks for are the owner's ([OD-33](../../planning/open-decisions.md)). Nothing here changes the gate's decision, an ADR or a budget.
> **Related:** [A02 gate report](A02-prototype-gate.md) · [Narrator walkthrough script](A02-narrator-walkthrough.md) · [roadmap A02](../../planning/roadmap.md#a02--technical-spikes-and-prototype-gate) · [implementation plan §2](../../planning/implementation-plan.md) · [ADR-0002](../../decisions/0002-desktop-shell-tauri.md#confirmation) · [ADR-0003](../../decisions/0003-frontend-react-and-react-aria.md#confirmation) · [accessibility conformance plan §3](../../design/accessibility-conformance-plan.md#3-manual-screen-reader-matrix) · [worklog](../worklog/2026-11-26-a02-closeout.md)

A02 is at about 90%. Every spike is merged and measured on Windows and CI. What is left needs a person, a screen reader or hardware the project doesn't have (R-22): a Mac, and Linux on real GPUs. This page lists, for each open item:

- what evidence the gate asks for, in the words of the documents that set it;
- what evidence exists now, including CI evidence from after the gate report;
- whether that CI evidence is enough.

It ends with a recommendation per item and a short list of decisions for the owner.

## 1. What the gate asks for

These documents set the A02 evidence. Where they differ, the recommendation says which wording it follows.

| Source | Wording that matters for the open items |
|---|---|
| [Roadmap A02](../../planning/roadmap.md#a02--technical-spikes-and-prototype-gate), exit criteria | "ADR-0002 (shell), ADR-0004 (render graph) and ADR-0003 (frontend) are confirmed or amended. If Linux fails (4) or (5), the Electron fallback is triggered per ADR-0002." Deliverable (1): cpal output "on all three OSes with a measured loopback latency" |
| [Implementation plan](../../planning/implementation-plan.md), A02 acceptance | T01: "p50/p95 output latency measured at 128/256/512 frames on each OS". T05: "PB-09 measured on all 3 OSes". T06: "fps and frame-time distributions recorded (PB-08)" in WebView2, WKWebView and WebKitGTK. T07: "A pass/fail matrix with notes per AT; OD-05 recommendation" |
| [ADR-0002, Confirmation](../../decisions/0002-desktop-shell-tauri.md#confirmation) | (1) IPC "on all three OSes (PB-09)"; (2) WebGL2 frame rates in WebKitGTK (Intel, AMD, NVIDIA), WebView2 and WKWebView (PB-08); (3) NVDA, VoiceOver and Orca walkthroughs. Electron if (2) or (3) fails on Linux and can't be mitigated within a week. Timing (OD-15): Windows first; Linux and macOS by B03's exit |
| [ADR-0003, Confirmation](../../decisions/0003-frontend-react-and-react-aria.md#confirmation) | React Aria Tree, Slider and Toolbar with NVDA, VoiceOver and Orca inside the shell; the same timing |
| [OD-15](../../planning/open-decisions.md#closed-decisions) (amended 2026-10-29) and [OD-05](../../planning/open-decisions.md) | The owner's Narrator walkthrough before B02-T10 and any component sign-off; OD-05 is final after Narrator; ADR-0002 and ADR-0003 stay provisional until those results are recorded |
| [Gate report](A02-prototype-gate.md#gono-go-criteria), Gate Definition of Done | "Every A02 task meets its acceptance criterion" and "Budgets measured on reference hardware" (no reference machine exists, R-22) |

The gate report's own verdicts add "real macOS and Linux hardware pending" to T05. The plan and ADR-0002 say "on all three OSes" and don't say real hardware, so requiring real hardware is the gate report's reading, not the plan's wording. Item 2 in section 3 follows the wording, and records the limit.

## 2. Inventory

**Evidence since the gate report.**

- **PB-08 in all three webviews.** The desktop smoke run (#439, F-212) traces PB-08 on CI-WIN, CI-MAC and CI-LNX. The values are in the [benchmarks log](../../research-paper/benchmarks-log.md#desktop-application).
- **PB-09 in the product.** The installed release app's smoke run reports PB-09 on all three CI runners (F-227, new in this PR, read from two `desktop-nightly` runs on 2026-11-25 and 2026-11-26).
- **Cross-OS determinism.** It held for the spike (F-039), and since then for the A05 golden cases (F-056) and the replication-gallery renders (F-116).

**CI verdicts.** *Sufficient* means the CI evidence meets the gate's wording. *Partly sufficient* means it meets some of the wording but not all. *Not a substitute* means CI can't produce this kind of evidence.

| # | Item | Evidence the gate needs | Evidence now | CI verdict |
|---|---|---|---|---|
| 1 | A02-T01 output latency on macOS and Linux (P9) | Loopback p50 and p95 at 128, 256 and 512 frames on each OS (plan T01, roadmap (1)) | Windows only: p95 123.1 ms through cpal, 83.7 ms with RAW (F-030 to F-033). The harness is Windows-only; macOS needs a loopback device and a port of the capture side | **Not a substitute.** No CI job measures output latency, and a hosted runner has no audio device to measure (TODO(verify)) |
| 2 | A02-T05 IPC on macOS and Linux | PB-09 (1 MB binary ≤ 30 ms) measured on all 3 OSes (plan T05; ADR-0002 (1)) | Spike: p95 7 ms on CI-LNX, 8 ms on CI-MAC, 7.8 ms on CI-WIN, 11.5–11.8 ms on HW-01 (F-040). Product: 30 round trips of a 1.2 MB view, p95 7–8 ms on CI-LNX, 10–18 ms on CI-MAC, 10.7–14.7 ms on CI-WIN, all within budget; one 84 ms maximum on CI-MAC (F-227) | **Sufficient** for "measured on all 3 OSes" (twice, with at least 1.6 times headroom at p95). **Not** evidence for "measured on reference hardware", which no OS has, Windows included. The App Nap observation (F-041: the 30 Hz stream arrived at 8 messages per second on CI-MAC) concerns the 30 Hz Channel stream, not PB-09, and needs a real Mac |
| 3 | A02-T06 WebGL2 in WKWebView | fps and frame-time distributions recorded in WKWebView (plan T06); WKWebView frame rates (ADR-0002 (2)) | CI-MAC, product smoke check: playhead 59.0–60.1 fps; script per frame p95 21–26 ms over the 8 ms budget, on a shared runner without a GPU, so an upper bound (F-212) | **Partly sufficient.** Distributions are recorded in WKWebView, so the plan's acceptance wording is met. But they come from a virtual machine without a GPU, and they don't show PB-08 passing, so they can't confirm ADR-0002 (2) for macOS |
| 4 | A02-T06 WebGL2 in WebKitGTK (Intel, AMD, NVIDIA) | Frame rates in WebKitGTK per GPU vendor (ADR-0002 (2)); a Linux failure here is the Electron trigger | CI-LNX under Xvfb: `backend` reports webgl2, but WebGL is drawn in software; script p95 87–110 ms. F-212: "not a WebKitGTK figure" | **Not a substitute.** No GPU, so it can't answer the Electron trigger question |
| 5 | A02-T07 Narrator | Pass/fail matrix with notes for Narrator; the OD-05 final decision (steps 11–15); ADR-0002 and ADR-0003 confirmed for Windows (HP-01, HP-04) | None. CI runs the UI Automation tree dump on Windows (B02-T03, F-045): it shows roles and names, not what Narrator speaks or what it does with UI Automation notifications | **Not a substitute.** Only a screen-reader run shows speech |
| 6 | A02-T07 Orca | Orca walkthrough in WebKitGTK; a Linux failure is the Electron trigger (ADR-0002 (3)) | None | **Not a substitute** |
| 7 | A02-T07 VoiceOver | VoiceOver walkthrough in WKWebView (ADR-0002 (3), ADR-0003) | None | **Not a substitute** |
| 8 | NVDA leftovers (P1) | Step 20 (contrast theme) and listening checks on steps 4, 13 and 17 (focus-mode sound, alert interruption, held key) | 16 pass, 2 partial, 1 fail, 1 skipped (F-046); the Speech Viewer has no timing | **Not a substitute.** Needs someone listening |
| 9 | A02-T04 determinism | Null residual and ΔLUFS against PB-06 | Bit-identical on CI-LNX, CI-WIN, CI-MAC (AArch64) and HW-01 (F-039); the golden cases since then (F-056) | **Sufficient** (already done: this task was always measured on CI) |
| 10 | A02-T08: ADR-0002 and ADR-0003 provisional (OD-15) | Windows: confirmed from items 5 and 8 (HP-04). Linux and macOS: from items 3, 4, 6 and 7, by B03's exit | Windows: NVDA only. Linux and macOS: none beyond items 2 and 3 | Follows items 2 to 8 |

Items already moved out of A02, for completeness:

- **PB-01 on Windows** (the fail), the cable confirmation (P6) and the rerun with enhancements off (P7) are HP-05, under ADR-0016, B03-T12 and #186.
- **The proprietary NVIDIA driver on Linux** (P8) needs a dual boot and stays under R-01.

## 3. Recommendations

Each item gets one of three outcomes: **close with CI evidence**, **defer to B07**, or **still needs the owner's hardware**. B07 already plans the full manual screen-reader matrix on real hardware and assistive technology (B07-T02: M-03 Narrator, M-04 VoiceOver on WKWebView, M-05 and M-06 Orca on WebKitGTK; HP-10) and co-design round 1 (B07-T04). So deferring a macOS or Linux item to B07 doesn't drop it; it moves it to the session that already needs the same machines.

| # | Item | Recommendation | Why |
|---|---|---|---|
| 1 | T01 latency, macOS and Linux | **Defer to B07.** Measure PB-01 with the product's audio backend, not the spike, on the M-04 Mac and the M-05 Linux machine. Before that, an agent ports the loopback capture (a loopback device on macOS, a PipeWire monitor on Linux; TODO(verify) the tools). Close A02-T01 as "Windows measured; macOS and Linux deferred" | The spike harness is Windows-only, and #494 (open) retires the spike workflows. Windows already fails PB-01, and its fix (ADR-0016) is a Windows backend, so a macOS or Linux number changes no A02 decision |
| 2 | T05 IPC, macOS and Linux | **Close with CI evidence.** Record the limit: CI runners are virtual machines, not reference hardware. Move the App Nap check (F-041) to the first session on a real Mac (HP-03 or B07's M-04) | The gate's wording ("measured on all 3 OSes") is met twice, by the spike and by the installed product, with headroom at p95. A real Mac isn't expected to change a pass into a fail, but that is an expectation, not a measurement, so the record says so |
| 3 | T06 WebGL2, WKWebView | **Defer to B07** (M-04), or HP-03 if a Mac turns up sooner. The product's smoke run (`--smoke`) gives the PB-08 trace there without the spike | Without a Mac, nothing better than CI exists. macOS is not part of the Electron trigger, which concerns Linux |
| 4 | T06 WebGL2, WebKitGTK on real GPUs | **Still needs the owner's hardware:** the Ubuntu live-USB session on HW-01 (HP-02, Intel and the discrete GPU), by B03's exit as OD-15 says. Use the `desktop-nightly` Linux build and its smoke run, because the spike's CI artifacts end once #494 retires its workflow | This is ADR-0002's Electron trigger. CI can't answer it, and the later it runs, the more UI work a switch would sit on. The thin-host rule caps the cost of a switch at the host, but doesn't remove it |
| 5 | T07 Narrator | **Still needs the owner's hardware, and HW-01 is enough.** The script is ready ([A02-narrator-walkthrough.md](A02-narrator-walkthrough.md)). It can be owner-run, or agent-run through desktop control once the owner says go | It decides OD-05, gates B02-T10, and confirms ADR-0002 and ADR-0003 for Windows (HP-01, HP-04). Nothing else stands in for it |
| 6 | T07 Orca | **Still needs the owner's hardware**, in the same live-USB session as item 4. If the owner can't run that session before B03's exit, the fallback is to defer to B07 (M-05, M-06) and accept that the Electron trigger is evaluated late | Orca is the other half of the Linux trigger (ADR-0002 (3)) |
| 7 | T07 VoiceOver | **Defer to B07** (M-04), or HP-03 if a Mac turns up sooner | No Mac is available (R-22). macOS is not part of the Electron trigger |
| 8 | NVDA leftovers | **Still needs the owner:** four checks, folded into the Narrator session (Part C of the script) | Listening only. If the Narrator run is agent-driven, the owner still has to listen |
| 9 | T04 determinism | **Closed** (already Done) | — |
| 10 | ADR-0002 and ADR-0003 | **Confirm for Windows** after items 5 and 8 (the coordinator, under the owner's delegation, HP-04). **Keep provisional for macOS** until B07's M-04 session, and **for Linux** until item 4 and item 6, by B03's exit. A02 then closes as "conditional go, closed with recorded deferrals" | Confirming an ADR for an OS it hasn't been tested on would stretch the evidence. Keeping A02 open for hardware that may never come blocks nothing useful |

## 4. Decisions for the owner

These are recorded as [OD-33](../../planning/open-decisions.md). The recommendation comes first in each.

1. **Narrator (items 5 and 8).** Run the [Narrator walkthrough](A02-narrator-walkthrough.md) on HW-01 now. Choose: (a) agent-driven through desktop control, which takes over your screen for the whole run, and you still listen for the timing checks; or (b) you run it yourself and paste the results. It needs your explicit go-ahead either way.
2. **IPC on macOS and Linux (item 2).** Accept the CI evidence (spike F-040 and product F-227) as closing A02-T05, with the virtual-machine limit recorded? Recommended: yes.
3. **macOS (items 1, 3 and 7, plus the App Nap check).** Defer the macOS runs from "by B03" to B07's M-04 session, since no Mac is available? Recommended: yes. This amends OD-15's macOS timing and the macOS part of ADR-0002's and ADR-0003's Confirmation sections.
4. **Linux (items 4 and 6, plus item 1's Linux part).** Keep the Ubuntu live-USB session (WebKitGTK on both GPUs, Orca) by B03's exit, because it decides the Electron trigger? Recommended: yes. The alternative is to defer it to B07 (M-05) and accept the risk of a late shell decision.
5. **Close A02.** Once decision 1's run is recorded and the deferrals in 2 to 4 are written into the gate report, sign the gate report as "conditional go, closed with deferrals", and mark A02 Done. Recommended: yes.
