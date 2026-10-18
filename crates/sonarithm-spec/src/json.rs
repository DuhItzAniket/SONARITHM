// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A plain JSON value for the free-form parts of a spec (inline records, format options,
//! calibration, stream schemas), so no third-party type appears in the public API
//! (coding-standards §6).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Any JSON value. Numbers are `f64`, which is how RFC 8785 treats them for hashing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum JsonValue {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number.
    Number(f64),
    /// A string.
    String(String),
    /// An array.
    Array(Vec<JsonValue>),
    /// An object, with keys in sorted order.
    Object(BTreeMap<String, JsonValue>),
}

/// A JSON object with keys in sorted order.
pub type JsonObject = BTreeMap<String, JsonValue>;
