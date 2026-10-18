// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM data layer: an Arrow-backed `DataBundle`, file sources, type and concept
//! inference, transforms and M4/LTTB downsampling, with provenance carried on every value.
//!
//! Skeleton created in phase A01; the data layer arrives in A04
//! (docs/architecture/03-data-model.md).

#![forbid(unsafe_code)]
#![deny(missing_docs)]
