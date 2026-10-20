// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The session log, schema `sonarithm-study-log/0`: one JSON object per line (JSONL),
//! following the proposed v0 schema of the evaluation plan ("Logging schema").
//!
//! Every record carries the experiment, session and participant codes, the run number and
//! the event with its monotonic time, so each line stands alone in an analysis. The
//! `session-start` record adds the protocol version and hash, the engine version and the
//! order seed; `trial-start` adds the stimulus provenance (generator, seed, table digest,
//! spec hash, data fingerprint).
//!
//! **Privacy (ADR-0012, P8).** The log holds pseudonymous codes, never names or contact
//! details, and never a path: specs are named relative to the protocol. Audio context
//! (level, device, headphones, assistive technology) is recorded only when the researcher
//! enters it, never detected.

use serde::{Deserialize, Serialize};

/// The log schema this runner writes.
pub const LOG_SCHEMA: &str = "sonarithm-study-log/0";

/// What happened.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// A session (or a resumed run of it) began.
    SessionStart,
    /// A trial began; its stimulus is ready.
    TrialStart,
    /// The stimulus played for the first time in the trial.
    Play,
    /// The stimulus played again.
    Replay,
    /// The session paused.
    Pause,
    /// The session resumed, in the same run or a new one.
    Resume,
    /// A trial stopped before its response, because the session paused or ended; it runs
    /// again from the start.
    TrialAbort,
    /// The listener answered.
    Response,
    /// The trial ended.
    TrialEnd,
    /// A free-text note from the researcher.
    #[default]
    Note,
    /// Every trial is done, or the session was ended early.
    SessionEnd,
}

/// One log record.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogRecord {
    /// The event.
    pub event: Event,
    /// Milliseconds on a monotonic clock since the start of this run.
    pub t_mono_ms: u64,
    /// The run: 1 for the first, one more for each resume from the log file.
    pub run: u32,
    /// The experiment ID.
    pub exp_id: String,
    /// The session ID.
    pub session_id: String,
    /// The pseudonymous participant code.
    pub participant_code: String,

    // Session fields (`session_start`).
    /// The log schema, [`LOG_SCHEMA`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_schema: Option<String>,
    /// The protocol version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    /// SHA-256 of the protocol file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_sha256: Option<String>,
    /// The SONARITHM version that ran the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_version: Option<String>,
    /// The order seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_seed: Option<u64>,
    /// The number of trials in the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trials: Option<usize>,

    // Trial fields.
    /// The trial's position in the session, from 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial: Option<usize>,
    /// The block ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,
    /// The condition ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Whether the trial is practice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub practice: Option<bool>,

    // Stimulus provenance (`trial_start`).
    /// The stimulus generator, such as `GEN-002`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stimulus_id: Option<String>,
    /// The generator's algorithm version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generator_version: Option<u32>,
    /// The stimulus seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// SHA-256 of the stimulus table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stimulus_sha256: Option<String>,
    /// The hash of the spec that sounded the stimulus, after binding its data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_hash: Option<String>,
    /// The data fingerprint of the bound stimulus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_fingerprint: Option<String>,
    /// The condition's spec, relative to the protocol file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_ref: Option<String>,

    // Response fields.
    /// The response as typed, normalized (trimmed, lower case for words).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    /// Whether the response was correct, for count, choice and trend tasks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correct: Option<bool>,
    /// The true value, for point estimation and count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub true_value: Option<f64>,
    /// The true answer as text, for choice and trend tasks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub true_answer: Option<String>,
    /// Response minus true value, for point estimation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<f64>,
    /// Milliseconds from the start of the last play to the response, in this run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rt_ms: Option<u64>,
    /// How many times the stimulus played in the trial.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plays: Option<u32>,

    // Audio context, only as entered by the researcher.
    /// The playback level as set or calibrated, free text such as `60 dB SPL`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playback_level: Option<String>,
    /// The output device kind, such as `USB interface`; never a device's own name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_device: Option<String>,
    /// The headphone model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headphone_model: Option<String>,
    /// Assistive technology as reported, such as `NVDA 2026.3`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistive_tech: Option<String>,
    /// A note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl LogRecord {
    /// The record as one JSON line, without the line break.
    #[must_use]
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parses one JSON line.
    ///
    /// # Errors
    ///
    /// A [`serde_json::Error`] if the line is not a record of this schema.
    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

/// Parses a whole log, skipping blank lines.
///
/// # Errors
///
/// The line number (from 1) and the parse error of the first bad line.
pub fn parse_log(text: &str) -> Result<Vec<LogRecord>, (usize, serde_json::Error)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| LogRecord::from_line(line).map_err(|err| (index + 1, err)))
        .collect()
}
