// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The WebVTT cue sidecar, `<name>.cues.vtt` (06 §7, FR-101): the Score's speech events and
//! markers as timed text. Speech is never rendered into exports (05 §4.1), so the sidecar
//! is how players and Deaf or hard-of-hearing users get it (A11Y-24).

use std::fmt::Write as _;

use sonarithm_core::{Score, ScoreEvent, Seconds, WordsPerMinute, spoken_duration};

/// How long a short cue stays on screen: 3 s (A05 provisional value, 06 §7). A cue whose
/// text takes longer to speak stays until its speech would end.
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
/// start time (ties keep track order, then markers), timed at the default speaking rate.
#[must_use]
pub fn cues(score: &Score) -> Vec<Cue> {
    cues_at(score, WordsPerMinute::DEFAULT)
}

/// [`cues`] for speech at `rate`, with the announcement policy of AUDAXIS (08 §4, A11Y-20):
/// speech is sequenced, never overlapped, so texts that start together share one cue and a
/// cue ends when the next one starts. Otherwise
/// a cue lasts [`CUE_DURATION`] or, when its text takes longer to speak, until its speech
/// would end; never past the end of the Score, and always at least 1 ms.
#[must_use]
pub fn cues_at(score: &Score, rate: WordsPerMinute) -> Vec<Cue> {
    let cue = |t: Seconds, text: &str| Cue {
        start: t,
        end: Seconds(t.0 + CUE_DURATION.0.max(spoken_duration(text, rate).0)),
        text: text.to_owned(),
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
    // Speech that starts together is spoken one after the other: one cue, its texts on
    // separate lines, as long as the longest.
    let mut merged: Vec<Cue> = Vec::with_capacity(all.len());
    for cue in all {
        match merged.last_mut() {
            Some(last) if last.start == cue.start => {
                last.text.push('\n');
                last.text.push_str(&cue.text);
                last.end = Seconds(last.end.0.max(cue.end.0));
            }
            _ => merged.push(cue),
        }
    }
    let duration = score.header.duration.0;
    let starts: Vec<f64> = merged.iter().map(|cue| cue.start.0).collect();
    for (cue, next) in merged
        .iter_mut()
        .zip(starts.iter().skip(1).map(Some).chain([None]))
    {
        let next = next.copied().unwrap_or(f64::INFINITY);
        cue.end = Seconds(cue.end.0.min(next).min(duration).max(cue.start.0 + 0.001));
    }
    merged
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

    /// A11Y-20: cues are sequenced like speech: one ends when the next starts, and a cue
    /// whose speech is longer than the display time lasts until its speech would end.
    #[test]
    fn a11y_20_cues_never_overlap_and_cover_their_speech() {
        let mut score = score();
        let long = "word ".repeat(30);
        if let Some(track) = score.tracks.first_mut() {
            track.events = vec![
                ScoreEvent::Speech(Speech::new(Seconds(1.0), "First point")),
                ScoreEvent::Speech(Speech::new(Seconds(1.5), "Second point")),
                ScoreEvent::Speech(Speech::new(Seconds(20.0), long.clone())),
            ];
        }
        let cues = cues_at(&score, WordsPerMinute(120.0));
        assert_eq!(cues[0].end, Seconds(1.5), "cut at the next cue");
        assert_eq!(cues[1].end, Seconds(4.5), "3 s display");
        assert_eq!(cues[2].start, Seconds(12.25), "the marker");
        assert_eq!(cues[2].end, Seconds(15.25));
        assert_eq!(cues[3].end, Seconds(35.0), "30 words at 2 per second");
        for pair in cues.windows(2) {
            assert!(pair[0].end <= pair[1].start, "{pair:?}");
        }
    }

    #[test]
    fn a11y_20_speech_that_starts_together_shares_one_cue() {
        let mut score = score();
        if let Some(track) = score.tracks.first_mut() {
            track.events = vec![
                ScoreEvent::Speech(Speech::new(Seconds(0.0), "Legend")),
                ScoreEvent::Speech(Speech::new(Seconds(0.0), "8.5")),
                ScoreEvent::Speech(Speech::new(Seconds(1.0), "9.25")),
            ];
        }
        score.markers.clear();
        let cues = cues(&score);
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].text, "Legend\n8.5");
        assert_eq!(cues[0].end, Seconds(1.0));
        assert!(webvtt(&cues).contains("\n00:00:00.000 --> 00:00:01.000\nLegend\n8.5\n"));
    }

    #[test]
    fn timestamps_round_to_milliseconds() {
        assert_eq!(timestamp(Seconds(0.0)), "00:00:00.000");
        assert_eq!(timestamp(Seconds(0.0015)), "00:00:00.002");
        assert_eq!(timestamp(Seconds(3_599.999_9)), "01:00:00.000");
        assert_eq!(timestamp(Seconds(-1.0)), "00:00:00.000");
    }
}
