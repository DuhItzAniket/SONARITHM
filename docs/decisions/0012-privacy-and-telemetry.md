---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0012 · No default telemetry; offline by default

## Context and problem statement

Users sonify research data, health data, financial data and personal data. Some are blind users who rely on the app being trustworthy without visually auditing network indicators. What is SONARITHM's network and telemetry posture?

## Decision drivers

* User trust and data protection (GDPR; the EDPB 2/2023 guidance extends consent requirements to device-side tracking).
* Research ethics: study data must stay under explicit consent.
* Supply-chain simplicity: no analytics SDKs.

## Considered options

* No telemetry; offline by default; per-action consent for any network feature
* Opt-out anonymous telemetry
* Opt-in telemetry following the Go model (collect locally, upload on opt-in)

## Decision outcome

Chosen option: **no telemetry and offline by default**. Every network feature (stream sources, update checks, model downloads, cloud AI) requires explicit consent and is visible in settings. If usage statistics are ever needed, they follow the **Go model** (local collection, explicit opt-in upload, field-level documentation) through a new ADR.

### Consequences

* Good: the strongest privacy stance; simpler compliance; no SDK dependencies.
* Bad: no usage data to guide product decisions. Mitigated by co-design studies, issue templates and opt-in surveys.

### Confirmation

From B01, a **network-deny integration test** runs the desktop app and CLI through standard workflows with networking blocked, and asserts no connection attempts. Release checklists verify that no new network call exists without a consent gate.
