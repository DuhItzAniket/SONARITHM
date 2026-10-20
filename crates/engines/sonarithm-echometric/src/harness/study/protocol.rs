// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The study protocol file, format `sonarithm-study/0`.
//!
//! A protocol is a TOML file next to the condition specs it names. It fixes everything a
//! session needs: the task and its response, the conditions (one spec each, data-agnostic,
//! SDR-03), and the blocks, each with a stimulus generator and how many trials per
//! condition. The trial order follows from the protocol seed and the session's order seed
//! (evaluation-plan "`sonarithm study run`": deterministic order from a recorded seed).
//!
//! ```toml
//! format = "sonarithm-study/0"
//! exp-id = "PILOT-001"
//! protocol-version = "1"
//! title = "Pilot: point estimation with and without a dynamic reference tone"
//! seed = 20261019
//! instructions = ["You will hear short price series ..."]
//!
//! [task]
//! kind = "point-estimation"
//! prompt = "What was the price at {x} o'clock?"
//! response = { kind = "number", min = 0, max = 120, unit = "dollars" }
//!
//! [[conditions]]
//! id = "none"
//! label = "No context"
//! spec = "none.json"
//!
//! [[blocks]]
//! id = "practice"
//! practice = true
//! feedback = true
//! conditions = ["none"]
//! repeats = 2
//! stimulus = { generator = "GEN-002", queries = 1 }
//! ```

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};

use crate::codes;
use crate::harness::generators::Params;

/// The protocol format this runner reads.
pub const FORMAT: &str = "sonarithm-study/0";

/// A study protocol.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Protocol {
    /// Must be [`FORMAT`].
    pub format: String,
    /// The experiment ID, such as `EXP-001` or `PILOT-001`.
    pub exp_id: String,
    /// The protocol version, recorded in every session.
    pub protocol_version: String,
    /// A one-line title.
    pub title: String,
    /// The seed that, with the session's order seed, fixes the trial order and every
    /// stimulus.
    pub seed: u64,
    /// Paragraphs read before the first trial.
    #[serde(default)]
    pub instructions: Vec<String>,
    /// What the listener does on every trial.
    pub task: Task,
    /// The conditions.
    pub conditions: Vec<Condition>,
    /// The blocks, in order.
    pub blocks: Vec<Block>,
    /// Offer a break after this many trials; `0` never offers one.
    #[serde(default)]
    pub break_every: u32,
}

/// The listener's task.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Task {
    /// The kind of task, which decides how a response is scored.
    pub kind: TaskKind,
    /// The question. `{x}` stands for the point-estimation query's clock hour, `{from}`
    /// and `{to}` for the trend query's interval.
    pub prompt: String,
    /// The response the question takes.
    pub response: ResponseSpec,
}

/// The standard tasks of SDR-40.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskKind {
    /// Estimate the value at the query point; scored by the error (RMS over trials).
    PointEstimation,
    /// Say whether the value increased, decreased or stayed the same between two hours
    /// (Nees & Walker 2008, Exp. 2); scored correct or not.
    TrendIdentification,
    /// Count events, such as transits; scored correct or not.
    Count,
    /// Choose one of the options, such as the strongest of three correlations.
    Choice,
}

/// A response format.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ResponseSpec {
    /// A number between `min` and `max` inclusive.
    Number {
        /// The smallest accepted answer.
        min: f64,
        /// The largest accepted answer.
        max: f64,
        /// The unit read with the prompt, such as `dollars`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit: Option<String>,
    },
    /// One of the options, typed by its number or its text.
    Choice {
        /// The options, read in order and numbered from 1. For trend identification they
        /// are the three answers in the order increased, decreased, stayed the same; for
        /// GEN-004 they name axes B, C and D.
        options: Vec<String>,
    },
}

/// One experimental condition: a spec that sounds each trial's stimulus.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Condition {
    /// The condition ID, recorded with every trial.
    pub id: String,
    /// A label for the researcher; it is not read to the listener.
    pub label: String,
    /// The spec file, relative to the protocol file. Its `data` is replaced by the
    /// trial's stimulus.
    pub spec: String,
}

/// How a block orders its trials.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Order {
    /// A seeded shuffle of all the block's trials.
    #[default]
    Shuffle,
    /// Conditions in the listed order, each repeated `repeats` times.
    Fixed,
}

/// A block of trials.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Block {
    /// The block ID.
    pub id: String,
    /// Practice trials are logged but marked, so analyses can leave them out.
    #[serde(default)]
    pub practice: bool,
    /// Tell the listener the correct answer after each response.
    #[serde(default)]
    pub feedback: bool,
    /// The condition IDs this block runs.
    pub conditions: Vec<String>,
    /// Trials per condition.
    pub repeats: u32,
    /// The order of the trials.
    #[serde(default)]
    pub order: Order,
    /// The stimulus generator and its parameters; every trial gets its own seed.
    pub stimulus: Params,
}

impl Protocol {
    /// Parses and checks a protocol.
    ///
    /// # Errors
    ///
    /// The diagnostics: `SNR-ENG-600` for TOML or shape errors, with the line and column,
    /// or every rule the parsed protocol breaks.
    pub fn from_toml(text: &str) -> Result<Self, Diagnostics> {
        let protocol: Self = toml::from_str(text).map_err(|err| {
            let at = err
                .span()
                .map(|span| {
                    let before = text.get(..span.start).unwrap_or_default();
                    let line = before.matches('\n').count() + 1;
                    let column = before.rsplit('\n').next().map_or(0, str::len) + 1;
                    format!(" (line {line}, column {column})")
                })
                .unwrap_or_default();
            let mut out = Diagnostics::default();
            out.push(Diagnostic::new(
                &codes::PROTOCOL_PARSE,
                format!("{}{at}", err.message().trim_end()),
            ));
            out
        })?;
        let diagnostics = protocol.check();
        if diagnostics.has_errors() {
            return Err(diagnostics);
        }
        Ok(protocol)
    }

    /// Checks the rules a parsed protocol must meet.
    #[must_use]
    pub fn check(&self) -> Diagnostics {
        let mut out = Diagnostics::default();
        let root = SpecPointer::root();
        if self.format != FORMAT {
            out.push(
                Diagnostic::new(
                    &codes::PROTOCOL_FORMAT,
                    format!("This runner reads `{FORMAT}`; set `format = \"{FORMAT}\"`."),
                )
                .at(root.key("format")),
            );
        }
        if !is_code(&self.exp_id) {
            out.push(
                Diagnostic::new(
                    &codes::PROTOCOL_ID,
                    "Use letters, digits and hyphens (1 to 32) for `exp-id`, such as `PILOT-001`.",
                )
                .at(root.key("exp-id")),
            );
        }
        self.check_response(&mut out);
        let ids = self.check_conditions(&mut out);
        self.check_blocks(&ids, &mut out);
        out
    }

    /// Checks the conditions; returns their IDs.
    fn check_conditions(&self, out: &mut Diagnostics) -> BTreeSet<&str> {
        let root = SpecPointer::root();
        let mut ids = BTreeSet::new();
        for (index, condition) in self.conditions.iter().enumerate() {
            let at = root.key("conditions").index(index);
            if !is_code(&condition.id) || !ids.insert(condition.id.as_str()) {
                out.push(
                    Diagnostic::new(
                        &codes::PROTOCOL_ID,
                        "Give each condition a unique `id` of letters, digits and hyphens.",
                    )
                    .at(at.key("id")),
                );
            }
            if !is_relative(&condition.spec) {
                out.push(
                    Diagnostic::new(
                        &codes::PROTOCOL_PATH,
                        "Give `spec` as a path relative to the protocol file, without `..`.",
                    )
                    .at(at.key("spec")),
                );
            }
        }
        if self.conditions.is_empty() {
            out.push(
                Diagnostic::new(
                    &codes::PROTOCOL_EMPTY,
                    "Add at least one `[[conditions]]` table.",
                )
                .at(root.key("conditions")),
            );
        }
        ids
    }

    /// Checks the blocks against the condition IDs, the generators and the trial cap.
    fn check_blocks(&self, ids: &BTreeSet<&str>, out: &mut Diagnostics) {
        let root = SpecPointer::root();
        let mut blocks = BTreeSet::new();
        let mut trials: u64 = 0;
        for (index, block) in self.blocks.iter().enumerate() {
            let at = root.key("blocks").index(index);
            if !is_code(&block.id) || !blocks.insert(block.id.as_str()) {
                out.push(
                    Diagnostic::new(
                        &codes::PROTOCOL_ID,
                        "Give each block a unique `id` of letters, digits and hyphens.",
                    )
                    .at(at.key("id")),
                );
            }
            for (position, condition) in block.conditions.iter().enumerate() {
                if !ids.contains(condition.as_str()) {
                    out.push(
                        Diagnostic::new(
                            &codes::PROTOCOL_REFERENCE,
                            format!(
                                "Block `{}` names condition `{condition}`, which is not defined.",
                                block.id
                            ),
                        )
                        .at(at.key("conditions").index(position)),
                    );
                }
            }
            if block.conditions.is_empty() || block.repeats == 0 {
                out.push(
                    Diagnostic::new(
                        &codes::PROTOCOL_EMPTY,
                        "A block needs at least one condition and `repeats` of at least 1.",
                    )
                    .at(at.clone()),
                );
            }
            if let Err(err) = crate::harness::generators::generate(&block.stimulus, self.seed) {
                out.push(
                    Diagnostic::new(&codes::PROTOCOL_STIMULUS, err.to_string())
                        .at(at.key("stimulus")),
                );
            } else if !task_fits(self.task.kind, &block.stimulus) {
                out.push(
                    Diagnostic::new(
                        &codes::PROTOCOL_STIMULUS,
                        format!(
                            "A {:?} task cannot be scored from {} stimuli; choose a generator whose ground truth answers it.",
                            self.task.kind,
                            block.stimulus.generator().as_str()
                        ),
                    )
                    .at(at.key("stimulus")),
                );
            }
            trials = trials.saturating_add(
                u64::from(block.repeats).saturating_mul(block.conditions.len() as u64),
            );
        }
        if self.blocks.is_empty() {
            out.push(
                Diagnostic::new(
                    &codes::PROTOCOL_EMPTY,
                    "Add at least one `[[blocks]]` table.",
                )
                .at(root.key("blocks")),
            );
        }
        if trials > MAX_TRIALS {
            out.push(
                Diagnostic::new(
                    &codes::PROTOCOL_EMPTY,
                    format!("The protocol has {trials} trials; split it into sessions of at most {MAX_TRIALS}."),
                )
                .at(root.key("blocks")),
            );
        }
    }

    fn check_response(&self, out: &mut Diagnostics) {
        let at = SpecPointer::root().key("task").key("response");
        let ok = match (&self.task.kind, &self.task.response) {
            (TaskKind::PointEstimation, ResponseSpec::Number { min, max, .. }) => {
                min.is_finite() && max.is_finite() && min < max
            }
            (TaskKind::Count, ResponseSpec::Number { min, max, .. }) => {
                *min >= 0.0 && min < max && max.is_finite()
            }
            (
                TaskKind::Choice | TaskKind::TrendIdentification,
                ResponseSpec::Choice { options },
            ) => options.len() == 3,
            _ => false,
        };
        if !ok {
            out.push(
                Diagnostic::new(
                    &codes::PROTOCOL_RESPONSE,
                    "The response does not fit the task: point estimation and counting take a number range with min < max; trend identification takes three choices (increased, decreased, stayed the same); choice takes one option for each of axes B, C and D.",
                )
                .at(at),
            );
        }
    }

    /// The condition with `id`.
    #[must_use]
    pub fn condition(&self, id: &str) -> Option<&Condition> {
        self.conditions.iter().find(|condition| condition.id == id)
    }
}

/// The most trials one protocol may hold. Sessions longer than about 90 minutes should be
/// split (EXP-000 §3); this cap only stops runaway files.
pub const MAX_TRIALS: u64 = 2_000;

fn task_fits(kind: TaskKind, params: &Params) -> bool {
    matches!(
        (kind, params),
        (
            TaskKind::PointEstimation,
            Params::TrendReversals(_) | Params::PriceSeries(_)
        ) | (TaskKind::TrendIdentification, Params::TrendReversals(_))
            | (TaskKind::Count, Params::Transits(_))
            | (TaskKind::Choice, Params::CorrelationTriplets(_))
    )
}

/// A short code: 1 to 32 ASCII letters, digits and hyphens. Used for IDs and participant
/// codes, which must never be names (ADR-0012).
#[must_use]
pub fn is_code(text: &str) -> bool {
    (1..=32).contains(&text.len())
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn is_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains(':')
        && !path.split(['/', '\\']).any(|part| part == "..")
}
