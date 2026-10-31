// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The earcon catalogue (keyboard-and-navigation §11; sound-design §8; A11Y-08, A11Y-24).
//!
//! Every earcon has an identifier, a meaning, a default (on, or off for `E-ALARM` and
//! `E-LEVEL`), a text equivalent from the message catalogue for the log and for speech
//! (ED-05), and, for the earcons a Score can contain, its Score earcon kind. The sounds
//! are synthesised by the render engine (`sonarithm_render::EarconSound`); the app plays
//! the interactive ones live and maps [`Cue`]s to those sounds.
//!
//! `E-GAP` is new in B03: the gap of a missing value at the cursor, when the spec asks for
//! `data.missing: "earcon"` (03 §5: "an optional gap earcon in exploration mode"). It is a
//! proposal awaiting the listening check (HP-11), like every acoustic description here.

use std::fmt;

use sonarithm_core::nav::{NavKind, NavTree};
use sonarithm_core::score::EarconKind;

use crate::nav::cursor::Outcome;

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
    /// `E-THRESHOLD`: crossed a user threshold or a y gridline.
    Threshold,
    /// `E-MODE`: a mode or modality change (rising for on, falling for off).
    Mode,
    /// `E-STREAM`: a stream event (anomaly, threshold, connect, disconnect).
    Stream,
    /// `E-ALARM`: a user-configured true alarm only (SDR-35).
    Alarm,
    /// `E-LEVEL`: moved into a child level or out to the parent.
    Level,
    /// `E-GAP`: the cursor is on a missing value.
    Gap,
    /// `E-ZERO`: a function crosses or touches zero (CURVASONA, RFC-0007).
    Zero,
    /// `E-ASYMPTOTE`: a vertical asymptote of a function.
    Asymptote,
    /// `E-UNDEFINED`: an undefined span of a function starts.
    Undefined,
    /// `E-OFFSCREEN`: a function leaves the y-window.
    Offscreen,
}

impl EarconId {
    /// The catalogue identifier, such as `E-BOUNDARY`.
    #[must_use]
    pub fn id(self) -> &'static str {
        self.info().code
    }

    /// The catalogue entry.
    #[must_use]
    pub fn info(self) -> &'static EarconInfo {
        CATALOGUE
            .iter()
            .find(|info| info.id == self)
            .unwrap_or(&CATALOGUE[0])
    }
}

impl fmt::Display for EarconId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

/// One catalogue entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EarconInfo {
    /// The earcon.
    pub id: EarconId,
    /// The catalogue identifier.
    pub code: &'static str,
    /// What it means (keyboard §11).
    pub meaning: &'static str,
    /// The message of its text equivalent (A11Y-24).
    pub text: &'static str,
    /// The arguments the text takes.
    pub args: &'static [&'static str],
    /// Whether it plays unless the listener turns it off (ED-09).
    pub default_on: bool,
    /// The Score earcon kind, for the earcons a compiled Score can contain.
    pub compiled: Option<EarconKind>,
}

/// The catalogue, in keyboard §11 order: the B03 set, `E-GAP`, then CURVASONA's
/// function cues (RFC-0007, played by its `curvasona.cues` track).
pub const CATALOGUE: [EarconInfo; 13] = [
    EarconInfo {
        id: EarconId::Boundary,
        code: "E-BOUNDARY",
        meaning: "Edge of the structure in the direction moved",
        text: "announce-boundary",
        args: &["edge"],
        default_on: true,
        compiled: Some(EarconKind::Boundary),
    },
    EarconInfo {
        id: EarconId::Invalid,
        code: "E-INVALID",
        meaning: "Command not available here",
        text: "announce-invalid",
        args: &["reason"],
        default_on: true,
        compiled: None,
    },
    EarconInfo {
        id: EarconId::Landmark,
        code: "E-LANDMARK",
        meaning: "Arrived at, or crossed, an insight, landmark or annotation",
        text: "earcon-landmark",
        args: &["name"],
        default_on: true,
        compiled: Some(EarconKind::Landmark),
    },
    EarconInfo {
        id: EarconId::Threshold,
        code: "E-THRESHOLD",
        meaning: "Crossed a user threshold or a y gridline",
        text: "earcon-threshold",
        args: &["value"],
        default_on: true,
        compiled: Some(EarconKind::Threshold),
    },
    EarconInfo {
        id: EarconId::Mode,
        code: "E-MODE",
        meaning: "Mode or modality change",
        text: "earcon-mode",
        args: &["name", "state"],
        default_on: true,
        compiled: None,
    },
    EarconInfo {
        id: EarconId::Stream,
        code: "E-STREAM",
        meaning: "Stream event: anomaly, threshold, connect, disconnect",
        text: "earcon-stream",
        args: &["stream", "event"],
        default_on: true,
        compiled: Some(EarconKind::Stream),
    },
    EarconInfo {
        id: EarconId::Alarm,
        code: "E-ALARM",
        meaning: "User-configured true alarm only",
        text: "earcon-alarm",
        args: &["condition"],
        default_on: false,
        compiled: Some(EarconKind::Alarm),
    },
    EarconInfo {
        id: EarconId::Level,
        code: "E-LEVEL",
        meaning: "Moved into a child level or out to the parent",
        text: "announce-level",
        args: &["kind"],
        default_on: false,
        compiled: None,
    },
    EarconInfo {
        id: EarconId::Gap,
        code: "E-GAP",
        meaning: "The cursor is on a missing value",
        text: "announce-missing",
        args: &[],
        default_on: true,
        compiled: None,
    },
    EarconInfo {
        id: EarconId::Zero,
        code: "E-ZERO",
        meaning: "A function crosses or touches zero",
        text: "earcon-zero",
        args: &[],
        default_on: true,
        compiled: Some(EarconKind::Zero),
    },
    EarconInfo {
        id: EarconId::Asymptote,
        code: "E-ASYMPTOTE",
        meaning: "A vertical asymptote of a function",
        text: "earcon-asymptote",
        args: &[],
        default_on: true,
        compiled: Some(EarconKind::Asymptote),
    },
    EarconInfo {
        id: EarconId::Undefined,
        code: "E-UNDEFINED",
        meaning: "An undefined span of a function starts",
        text: "earcon-undefined",
        args: &[],
        default_on: true,
        compiled: Some(EarconKind::Undefined),
    },
    EarconInfo {
        id: EarconId::Offscreen,
        code: "E-OFFSCREEN",
        meaning: "A function leaves the y-window",
        text: "earcon-offscreen",
        args: &["side"],
        default_on: true,
        compiled: Some(EarconKind::Offscreen),
    },
];

/// An earcon to play now, with its direction for `E-MODE` (on) and `E-LEVEL` (into a
/// child level); other earcons ignore it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cue {
    /// The earcon.
    pub id: EarconId,
    /// Rising: on, or into a child level.
    pub up: bool,
}

/// Which earcons play (ED-09: each can be turned off).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EarconSettings {
    enabled: Vec<EarconId>,
    /// Whether missing values get `E-GAP` (`data.missing: "earcon"`).
    pub gap: bool,
}

impl EarconSettings {
    /// The catalogue defaults, with `E-GAP` when the tree's spec asks for it.
    #[must_use]
    pub fn for_tree(tree: &NavTree) -> Self {
        let gap = matches!(
            tree.node(tree.root()).map(|n| &n.kind),
            Some(NavKind::Dataset(d)) if d.gap_earcons
        );
        Self {
            enabled: CATALOGUE
                .iter()
                .filter(|info| info.default_on)
                .map(|info| info.id)
                .collect(),
            gap,
        }
    }

    /// Whether `id` plays.
    #[must_use]
    pub fn is_on(&self, id: EarconId) -> bool {
        self.enabled.contains(&id)
    }

    /// Turns `id` on or off.
    pub fn set(&mut self, id: EarconId, on: bool) {
        self.enabled.retain(|e| *e != id);
        if on {
            self.enabled.push(id);
        }
    }
}

/// The earcons a verb's outcome plays: boundary or invalid action, a landmark, a level
/// change (off by default) and the gap of a missing value.
#[must_use]
pub fn for_outcome(tree: &NavTree, outcome: &Outcome, settings: &EarconSettings) -> Vec<Cue> {
    let mut cues = Vec::new();
    let mut push = |id: EarconId, up: bool| {
        if settings.is_on(id) {
            cues.push(Cue { id, up });
        }
    };
    match outcome {
        Outcome::Boundary { .. } => push(EarconId::Boundary, false),
        Outcome::Invalid { .. } => push(EarconId::Invalid, false),
        Outcome::Moved(m) => {
            if m.level_changed {
                push(EarconId::Level, tree.depth(m.to) > tree.depth(m.from));
            }
            if m.landmark {
                push(EarconId::Landmark, false);
            }
            let missing = matches!(
                tree.node(m.to).map(|n| &n.kind),
                Some(NavKind::Point(p)) if p.value.is_none()
            );
            if missing && settings.gap {
                push(EarconId::Gap, false);
            }
        }
        _ => {}
    }
    cues
}
