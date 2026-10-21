// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The earcon catalogue (keyboard-and-navigation §11, A11Y-08): the identifiers that
//! navigation outcomes and compiled cues refer to.

use std::fmt;

/// An earcon of the catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EarconId {
    /// `E-BOUNDARY`: the edge of the structure in the direction moved.
    Boundary,
    /// `E-INVALID`: the command is not available here.
    Invalid,
    /// `E-LANDMARK`: arrived at, or crossed, an insight, landmark or annotation.
    Landmark,
}

impl EarconId {
    /// The catalogue identifier, such as `E-BOUNDARY`.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Boundary => "E-BOUNDARY",
            Self::Invalid => "E-INVALID",
            Self::Landmark => "E-LANDMARK",
        }
    }
}

impl fmt::Display for EarconId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}
