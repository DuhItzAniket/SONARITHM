// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Checks of the sections other than tracks.

use std::collections::BTreeSet;

use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};

use super::{path_check, positive};
use crate::codes;
use crate::data::{BinMethod, SmoothMethod, Source, Transform, Transport};
use crate::sections::{DescribeLevel, Pace, ReferenceKind, Sink};
use crate::spec::Spec;

fn root() -> SpecPointer {
    SpecPointer::root()
}

/// `data.source` (051, 052) and `transform` (053).
pub(super) fn data(spec: &Spec, out: &mut Diagnostics) {
    let at = root().key("data").key("source");
    let invalid = |message: &str, out: &mut Diagnostics| {
        out.push(Diagnostic::new(&codes::INVALID_SOURCE, message).at(at.clone()));
    };
    match &spec.data.source {
        Source::File { path, .. } | Source::Chart { path, .. } => {
            if path.trim().is_empty() {
                invalid("The source `path` is empty.", out);
            }
            path_check(path, at.key("path"), out);
        }
        Source::Function {
            domain,
            samples,
            step,
            ..
        } => {
            if !(domain[0].is_finite() && domain[1].is_finite() && domain[0] < domain[1]) {
                invalid(
                    "The function `domain` must be [min, max] with min below max.",
                    out,
                );
            }
            match (samples, step) {
                (Some(_), Some(_)) => invalid("Give `samples` or `step`, not both.", out),
                (Some(n), None) if *n < 2 => invalid("`samples` must be at least 2.", out),
                (None, Some(step)) if !positive(*step) => {
                    invalid("`step` must be a positive number.", out);
                }
                _ => {}
            }
        }
        Source::Stream {
            transport,
            url,
            port,
            path,
            ..
        } => {
            let (key, present) = match transport {
                Transport::Websocket | Transport::Mqtt => ("url", url.is_some()),
                Transport::Serial | Transport::Osc => ("port", port.is_some()),
                Transport::Tail => ("path", path.is_some()),
            };
            if !present {
                invalid(&format!("This stream transport needs `{key}`."), out);
            }
            if let Some(path) = path {
                path_check(path, at.key("path"), out);
            }
        }
        Source::Inline { .. } | Source::Dataset { .. } => {}
    }
    for (index, transform) in spec.transform.iter().enumerate() {
        if let Some(message) = transform_problem(transform) {
            out.push(
                Diagnostic::new(&codes::INVALID_TRANSFORM, message)
                    .at(root().key("transform").index(index)),
            );
        }
    }
}

fn transform_problem(transform: &Transform) -> Option<&'static str> {
    match transform {
        Transform::Aggregate { ops, .. } if ops.is_empty() => {
            Some("`aggregate` needs at least one entry in `ops`.")
        }
        Transform::Smooth {
            method,
            alpha,
            window,
            ..
        } => match method {
            SmoothMethod::Ema
                if !alpha.is_some_and(|a| a > 0.0 && a <= 1.0) || window.is_some() =>
            {
                Some("`ema` smoothing needs `alpha` in (0, 1] and no `window`.")
            }
            SmoothMethod::Sma if !window.is_some_and(|w| w >= 1) || alpha.is_some() => {
                Some("`sma` smoothing needs a `window` of at least 1 row and no `alpha`.")
            }
            _ => None,
        },
        Transform::Bin {
            method,
            count,
            edges,
            ..
        } => match method.unwrap_or(BinMethod::EqualWidth) {
            BinMethod::Edges => {
                let ascending = edges.as_ref().is_some_and(|edges| {
                    edges.len() >= 2 && edges.windows(2).all(|pair| pair[0] < pair[1])
                });
                (!ascending).then_some("`edges` binning needs at least two ascending `edges`.")
            }
            _ => (count.is_some_and(|n| n == 0) || edges.is_some()).then_some(
                "Equal-width and quantile binning take a positive `count`, not `edges`.",
            ),
        },
        Transform::Resample { every, .. } if !positive(every.seconds()) => {
            Some("`every` must be a positive duration.")
        }
        _ => None,
    }
}

/// `substrate.time.pace` (050).
pub(super) fn substrate(spec: &Spec, out: &mut Diagnostics) {
    let pace = spec
        .substrate
        .as_ref()
        .and_then(|substrate| substrate.time.as_ref())
        .and_then(|time| time.pace);
    let valid = match pace {
        None => true,
        Some(Pace::PointsPerSecond(x) | Pace::SecondsPerUnit(x)) => positive(x),
        Some(Pace::Duration(duration)) => positive(duration.seconds()),
    };
    if !valid {
        out.push(
            Diagnostic::new(
                &codes::INVALID_PACE,
                "The pace must be a positive, finite number.",
            )
            .at(root().key("substrate").key("time").key("pace")),
        );
    }
}

/// Track references in `context` (013) and the static reference value (003).
pub(super) fn context(spec: &Spec, ids: &BTreeSet<String>, out: &mut Diagnostics) {
    let Some(context) = &spec.context else {
        return;
    };
    let at = root().key("context");
    let unknown = |track: &str, pointer: SpecPointer, out: &mut Diagnostics| {
        if !ids.contains(track) {
            out.push(
                Diagnostic::new(
                    &codes::UNKNOWN_TRACK,
                    format!("No track has the ID \"{track}\". Name a track from `tracks`."),
                )
                .at(pointer),
            );
        }
    };
    if let Some(reference) = &context.reference {
        let at = at.key("reference");
        if let Some(track) = &reference.track {
            unknown(track, at.key("track"), out);
        }
        if reference.kind == ReferenceKind::StaticValue && reference.value.is_none() {
            out.push(
                Diagnostic::new(
                    &codes::MISSING_KEY,
                    "A `static-value` reference needs a `value`.",
                )
                .at(at),
            );
        }
    }
    if let Some(earcons) = &context.earcons {
        for (index, threshold) in earcons.thresholds.iter().enumerate() {
            let pointer = at
                .key("earcons")
                .key("thresholds")
                .index(index)
                .key("track");
            unknown(&threshold.track, pointer, out);
        }
    }
}

/// `analysis.describe.levels` (041, A11Y-02).
pub(super) fn analysis(spec: &Spec, out: &mut Diagnostics) {
    let levels = spec
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.describe.as_ref())
        .and_then(|describe| describe.levels.as_ref());
    for (index, level) in levels.into_iter().flatten().enumerate() {
        if *level == DescribeLevel::L4 {
            out.push(
                Diagnostic::new(
                    &codes::L4_DESCRIPTION,
                    "Interpretive (L4) descriptions are never generated automatically (A11Y-02). Use levels L1 to L3.",
                )
                .at(root().key("analysis").key("describe").key("levels").index(index)),
            );
        }
    }
}

/// `playback` (061), `output` (060) and sink paths (052).
pub(super) fn playback_and_output(spec: &Spec, out: &mut Diagnostics) {
    if let Some(speed) = spec.playback.as_ref().and_then(|playback| playback.speed)
        && !positive(speed)
    {
        out.push(
            Diagnostic::new(
                &codes::INVALID_PLAYBACK,
                "The playback `speed` must be a positive number.",
            )
            .at(root().key("playback").key("speed")),
        );
    }
    let Some(output) = &spec.output else {
        return;
    };
    let at = root().key("output");
    let mut invalid = |key: &str, message: &str| {
        out.push(Diagnostic::new(&codes::INVALID_OUTPUT, message).at(at.key(key)));
    };
    if output
        .sample_rate
        .is_some_and(|rate| !(8_000..=384_000).contains(&rate))
    {
        invalid(
            "sampleRate",
            "The sample rate must be from 8000 to 384000 Hz.",
        );
    }
    if output
        .channels
        .is_some_and(|channels| !(1..=64).contains(&channels))
    {
        invalid("channels", "The channel count must be from 1 to 64.");
    }
    if output
        .block_size
        .is_some_and(|size| !(size.is_power_of_two() && (16..=8192).contains(&size)))
    {
        invalid(
            "blockSize",
            "The block size must be a power of two from 16 to 8192 frames.",
        );
    }
    if let Some(loudness) = &output.loudness {
        let bad = |x: Option<f64>| x.is_some_and(|x| !(x.is_finite() && x <= 0.0));
        if bad(loudness.target_lufs) || bad(loudness.true_peak_dbtp) {
            invalid(
                "loudness",
                "Loudness targets are at most 0 LUFS and 0 dBTP.",
            );
        }
    }
    for (index, sink) in output.sinks.iter().enumerate() {
        if let Sink::MidiFile { path } = sink {
            path_check(path, at.key("sinks").index(index).key("path"), out);
        }
    }
}
