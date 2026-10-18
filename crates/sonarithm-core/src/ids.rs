// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Strongly typed identifiers (coding-standards §6) and row references into data tables
//! (03 §2).

use core::fmt;
use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// A string identifier that is usually a compile-time constant (engine and rule IDs).
macro_rules! static_str_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Cow<'static, str>);

        impl $name {
            /// An identifier from a string constant.
            #[must_use]
            pub const fn from_static(id: &'static str) -> Self {
                Self(Cow::Borrowed(id))
            }

            /// An identifier from an owned string.
            #[must_use]
            pub fn new(id: impl Into<String>) -> Self {
                Self(Cow::Owned(id.into()))
            }

            /// The identifier text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

static_str_id! {
    /// An engine ID: the lowercase engine name, such as `numisona` (02 §5).
    EngineId
}

static_str_id! {
    /// A design-rule ID from the catalogue, such as `SDR-19` or `A11Y-14`
    /// (docs/research/design-rules.md).
    RuleId
}

static_str_id! {
    /// An analyzer ID, such as `extrema` or `critical-points` (05 §3).
    AnalyzerId
}

/// The ID of a track in a spec and in the Score it compiles to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TrackId(String);

impl TrackId {
    /// A track ID. Its syntax is checked by spec validation, not here.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TrackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identifies one table within a data bundle (03 §2).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct TableId(pub u32);

/// A table generation. Tables are immutable; every edit publishes a new generation (03 §2).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Generation(pub u64);

/// One row of one table generation; stable for the life of that generation (03 §2).
///
/// Score events carry it so navigation, announcements and linked selection can point back
/// at the datum that produced a sound (05 §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RowRef {
    /// The table.
    pub table: TableId,
    /// The table generation.
    pub generation: Generation,
    /// The zero-based row index.
    pub row: u64,
}

/// A half-open range of rows `[start, end)` over one table generation (03 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RowRange {
    /// The table.
    pub table: TableId,
    /// The table generation.
    pub generation: Generation,
    /// The first row in the range.
    pub start: u64,
    /// One past the last row in the range.
    pub end: u64,
}

impl RowRange {
    /// The number of rows; zero when `end <= start`.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    /// Whether the range holds no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether `row` is a row of this table generation inside the range.
    #[must_use]
    pub fn contains(&self, row: &RowRef) -> bool {
        row.table == self.table
            && row.generation == self.generation
            && (self.start..self.end).contains(&row.row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_and_owned_ids_compare_equal() {
        assert_eq!(EngineId::from_static("numisona"), EngineId::new("numisona"));
        assert_eq!(RuleId::from_static("SDR-19").to_string(), "SDR-19");
    }

    #[test]
    fn ids_serialize_as_strings() {
        let json = serde_json::to_string(&(TrackId::new("temp"), EngineId::from_static("axonote")));
        assert_eq!(json.unwrap(), r#"["temp","axonote"]"#);
    }

    #[test]
    fn row_ranges_are_half_open_and_generation_specific() {
        let range = RowRange {
            table: TableId(0),
            generation: Generation(2),
            start: 10,
            end: 12,
        };
        let row = |generation, row| RowRef {
            table: TableId(0),
            generation: Generation(generation),
            row,
        };
        assert_eq!(range.len(), 2);
        assert!(range.contains(&row(2, 10)));
        assert!(range.contains(&row(2, 11)));
        assert!(!range.contains(&row(2, 12)));
        assert!(!range.contains(&row(1, 10)));
    }
}
