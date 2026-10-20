// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The WebVTT cue sidecar, `<name>.cues.vtt` (06 §7, FR-101): the Score's speech events and
//! markers as timed text. Speech is never rendered into exports (05 §4.1), so the sidecar
//! is how players and Deaf or hard-of-hearing users get it (A11Y-24).

use std::fmt::Write as _;

use sonarithm_core::{Score, ScoreEvent, Seconds};

/// How long a cue stays on screen: 3 s, or until the Score ends. A05 provisional value;
/// AUDAXIS sets cue timing with its announcement policy (B03).
pub const CUE_DURATION: Seconds = Seconds(3.0);

/// One timed text cue.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Cue {
    /// Start.
    pub start: Seconds,
    /// End, after the start.
    pub end: Seconds,
    /// The text, unescaped.
    pub text: String,
}

/// The cues of `score`: every speech event of every track and every marker, sorted by
/// start time (ties keep track order, then markers).
#[must_use]
pub fn cues(score: &Score) -> Vec<Cue> {
    let duration = score.header.duration.0;
    let cue = |t: Seconds, text: &str| {
        let end = (t.0 + CUE_DURATION.0).min(duration).max(t.0 + 0.001);
        Cue {
            start: t,
            end: Seconds(end),
            text: text.to_owned(),
        }
    };
    let speech = score.tracks.iter().flat_map(|track| {
        track.events.iter().filter_map(move |event| match event {
            ScoreEvent::Speech(speech) => Some(cue(speech.t, &speech.text)),
            _ => None,
        })
    });
    let markers = score
        .markers
        .iter()
        .map(|marker| cue(marker.t, &marker.label));
    let mut all: Vec<Cue> = speech.chain(markers).collect();
    all.sort_by(|a, b| a.start.total_cmp(&b.start));
    all
}

/// A WebVTT timestamp, `HH:MM:SS.mmm`, rounded to the nearest millisecond.
#[must_use]
pub fn timestamp(t: Seconds) -> String {
    let millis = (t.0.max(0.0) * 1_000.0).round_ties_even();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a non-negative, rounded millisecond count"
    )]
    let millis = millis as u64;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        millis / 3_600_000,
        millis / 60_000 % 60,
        millis / 1_000 % 60,
        millis % 1_000
    )
}

/// Escapes cue text: `&`, `<` and `>` become character references, so the text can never
/// contain markup or the `-->` separator, and blank lines become single line breaks.
fn escape(text: &str) -> String {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    escaped
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The WebVTT file for `cues`.
#[must_use]
pub fn webvtt(cues: &[Cue]) -> String {
    let mut out = String::from("WEBVTT\n");
    for (index, cue) in cues.iter().enumerate() {
        let text = escape(&cue.text);
        let text = if text.is_empty() {
            "…".to_owned()
        } else {
            text
        };
        let _ = write!(
            out,
            "
{}
{} --> {}
{}
",
            index + 1,
            timestamp(cue.start),
            timestamp(cue.end),
            text
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use sonarithm_core::{
        EngineId, InstrumentRef, Marker, MarkerKind, Mode, ScoreHeader, ScoreTrack, SpecVersion,
        Speech, TrackId, TrackRole,
    };

    use super::*;

    fn score() -> Score {
        let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 0);
        header.duration = Seconds(4_000.0);
        let mut score = Score::new(header);
        let mut legend = ScoreTrack::new(
            TrackId::new("legend"),
            TrackRole::Speech,
            EngineId::from_static("audaxis"),
            InstrumentRef("speech".to_owned()),
        );
        legend.events = vec![
            ScoreEvent::Speech(Speech::new(Seconds(0.0), "Temperature, 1880 to 2020")),
            ScoreEvent::Speech(Speech::new(Seconds(3_725.5), "a < b && c --> d\n\nend")),
        ];
        score.tracks = vec![legend];
        score.markers = vec![Marker::new(
            Seconds(12.25),
            MarkerKind::Insight,
            "Peak: 1.2 °C",
        )];
        score
    }

    /// FR-101: the sidecar is valid WebVTT. The header comes first, every cue has an
    /// identifier and a `start --> end` line with end after start, starts never decrease,
    /// and no cue text contains the separator or markup.
    #[test]
    fn the_sidecar_is_valid_webvtt() {
        let vtt = webvtt(&cues(&score()));
        assert!(vtt.starts_with("WEBVTT\n\n"));
        let blocks: Vec<&str> = vtt.trim_end().split("\n\n").skip(1).collect();
        assert_eq!(blocks.len(), 3);
        let mut previous = String::new();
        for (index, block) in blocks.iter().enumerate() {
            let lines: Vec<&str> = block.lines().collect();
            assert_eq!(lines[0], (index + 1).to_string());
            let (start, end) = lines[1].split_once(" --> ").unwrap();
            assert!(end > start, "{block}");
            assert!(start >= previous.as_str());
            previous = start.to_owned();
            for text in &lines[2..] {
                assert!(!text.contains("-->") && !text.contains('<'), "{text}");
            }
        }
        assert!(
            vtt.contains("\n01:02:05.500 --> 01:02:08.500\na &lt; b &amp;&amp; c --&gt; d\nend\n")
        );
        assert!(vtt.contains("\n00:00:12.250 --> 00:00:15.250\nPeak: 1.2 °C\n"));
    }

    #[test]
    fn cues_end_with_the_score() {
        let mut short = score();
        short.header.duration = Seconds(13.0);
        let cues = cues(&short);
        assert_eq!(
            cues[1].end,
            Seconds(13.0),
            "the marker cue is cut at the end"
        );
    }

    #[test]
    fn timestamps_round_to_milliseconds() {
        assert_eq!(timestamp(Seconds(0.0)), "00:00:00.000");
        assert_eq!(timestamp(Seconds(0.0015)), "00:00:00.002");
        assert_eq!(timestamp(Seconds(3_599.999_9)), "01:00:00.000");
        assert_eq!(timestamp(Seconds(-1.0)), "00:00:00.000");
    }
}
