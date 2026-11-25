---
status: accepted
date: 2026-10-21
decision-makers: ["@DuhItzAniket"]
consulted: ["coordinator agent (Claude Code), under the owner's delegation of contract approvals"]
---

# 0018 · Project format v1: `.sonarithm` bundles per RFC-0002

## Context and problem statement

B05 saves desktop work as `.sonarithm` project bundles ([12 §4](../architecture/12-cross-cutting-concerns.md#4-project-file-format)). A persisted format that outlives app versions and arrives from untrusted sources is a contract (AGENTS.md rule 11), so its first version went through [RFC-0002](../rfcs/0002-project-format-v1.md).

## Decision drivers

* Reproducibility: the same content gives the same bytes (ADR-0013).
* Privacy: no absolute user paths, no secrets, no timestamps (ADR-0012, AGENTS.md rule 9).
* Safety against crafted bundles and shared links: zip-slip, size bombs, tampering, links that point elsewhere (12 §6).
* Memory on the 16 GB reference laptop.

## Considered options

* A ZIP bundle with a hashed manifest, as RFC-0002 specifies.
* A folder, SQLite, or Arrow IPC members (RFC-0002 "Rationale and alternatives").

## Decision outcome

Chosen option: **RFC-0002 as accepted on 2026-10-21**, with its answers to the unresolved questions and the two conditions added at acceptance:

* No `created` or `modified` fields in the manifest; 12 §4 updated.
* At most 5 autosaves, at most every 30 s, within a total size budget that B05-T07's soak sets (TODO(verify); 2 GiB until measured).
* A broken link shows the file name only.
* **Linked-data safety:** links are canonicalised; only regular files with an A04 data extension are opened, under A04's limits; a fingerprint mismatch keeps the dataset unavailable and out of every later save.
* **Memory:** stored Parquet members are read in place through a bounded file-slice reader; the total uncompressed limit is 2 GiB, A04's limit for one input; deflated Parquet members are refused.

The 7-day final comment period was waived by the coordinator under the owner's delegation (fast-track, sole maintainer, no other participants).

### Consequences

* Good: projects can be hashed, diffed and cited; opening a shared project cannot read outside regular data files or exceed what opening those files directly would allow.
* Bad: a data-layer entry point for reading Parquet from a seekable source is needed (B05-T04 adds it); linked datasets on another drive must be embedded.

### Confirmation

B05-T04: the round-trip, crash-injection, refusal, linked-data and memory tests listed in RFC-0002 §11, and the `project_open` fuzz target in `nightly.yml`.

## Amendments

| Date | Change | Approved |
|---|---|---|
| 2026-11-01 | Format 1.1 ([RFC-0002 §12](../rfcs/0002-project-format-v1.md#12-amendment-1-2026-11-01-format-11-function-datasets-and-unknown-storages)): `storage: "function"` stores a function dataset's canonical definition (expression, domain, samples or step, variables), not its samples; it is re-sampled on read, with a fingerprint mismatch a warning (`SNR-APP-034`); a dataset with an unknown storage opens as unavailable (`SNR-APP-035`) and is kept on re-save; 1.1 is written only when needed; `SNR-APP-089` is retired | Coordinator, under the owner's delegation (C03-T08) |

Reasons: saving a function's samples would lose the function and its critical points (C03-T08, ADR-0017 "Functions"); a strict reader that refuses an unknown storage as damaged makes every new storage a breaking change, while reading it as unavailable keeps it to a minor version; writing 1.0 when no new feature is used keeps existing projects byte-identical (ADR-0013).

## More information

* [RFC-0002](../rfcs/0002-project-format-v1.md) holds the full design; this ADR records the outcome.
* Builds on [ADR-0013](0013-determinism-and-provenance.md), [ADR-0012](0012-privacy-and-telemetry.md) and [ADR-0017](0017-ipc-protocol-v1.md) (the project commands are additive protocol commands).
