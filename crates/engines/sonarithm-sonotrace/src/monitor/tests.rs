// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Conformance tests of the monitoring soundscape (SDR-35, SDR-39, A11Y-08; B06-T04).

use serde_json::json;
use sonarithm_core::diagnostic::Severity;
use sonarithm_core::engine::{
    Annotation, Budget, CancelToken, CompileContext, ContextProvider, SpecFragment,
};
use sonarithm_core::score::{Mode, Note, ScoreHeader, SpecVersion, TrackRole};
use sonarithm_core::{
    AnalyzerId, Cents, Earcon, EarconKind, EngineId, Generation, InstrumentRef, RowRange, RowRef,
    Score, ScoreEvent, ScoreTrack, Seconds, SpecPointer, TableId, TrackId,
};
use sonarithm_data::{BundleId, DataBundle, SourceDescriptor, SourceKind};
use sonarithm_spec::sections::{Detector, Salience};

use super::*;

const ENGINE: EngineId = EngineId::from_static("sonotrace");

fn bed_curve(soundscape: &Soundscape) -> Vec<Breakpoint> {
    match soundscape.bed.events.as_slice() {
        [ScoreEvent::Glide(glide)] => glide.curve.clone(),
        other => unreachable!("one glide, not {other:?}"),
    }
}

fn earcon_kinds(soundscape: &Soundscape) -> Vec<EarconKind> {
    soundscape
        .earcons
        .events
        .iter()
        .filter_map(|event| match event {
            ScoreEvent::Earcon(Earcon { kind, .. }) => Some(*kind),
            _ => None,
        })
        .collect()
}

/// SDR-35 (sound-design §10.2): events change the bed's brightness, never its level, and
/// a more salient event brightens it more.
#[test]
fn sdr_35_salience_changes_brightness_not_level() {
    let events = [
        MonitorEvent::new(Seconds(2.0), Salience::Low, false),
        MonitorEvent::new(Seconds(6.0), Salience::High, false),
    ];
    let soundscape = soundscape(&ENGINE, Seconds(10.0), &events);
    assert_eq!(soundscape.bed.role, TrackRole::Monitor);
    let curve = bed_curve(&soundscape);
    assert!(
        curve.iter().all(|p| p.gain == Db::ZERO),
        "the level never changes"
    );
    assert!(curve.iter().all(|p| p.pitch == BED_PITCH));
    let at = |t: f64| {
        curve
            .iter()
            .find(|p| (p.t.0 - t).abs() < 1e-9)
            .map_or(f64::NAN, |p| p.brightness)
    };
    assert!((at(0.0) - BED_BRIGHTNESS).abs() < 1e-12, "it rests");
    let low = at(2.0 + RISE.0);
    let high = at(6.0 + RISE.0);
    assert!(low > BED_BRIGHTNESS && high > low, "{low} {high}");
    assert!(
        (at(6.0 + RISE.0 + SETTLE.0) - BED_BRIGHTNESS).abs() < 1e-12,
        "it settles back"
    );
}

fn score_with_rows(mode: Mode) -> Score {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, mode, 0);
    header.duration = Seconds(10.0);
    let mut score = Score::new(header);
    let mut data = ScoreTrack::new(
        TrackId::new("y"),
        TrackRole::Data,
        EngineId::from_static("numisona"),
        InstrumentRef("mallet".to_owned()),
    );
    data.events = (0..10_u64)
        .map(|row| {
            #[expect(clippy::cast_precision_loss, reason = "a small row number")]
            let t = Seconds(row as f64);
            let mut note = Note::new(t, Seconds(0.2), Cents(0.0));
            note.src = Some(RowRef {
                table: TableId(1),
                generation: Generation(0),
                row,
            });
            ScoreEvent::Note(note)
        })
        .collect();
    score.tracks.push(data);
    score
}

fn annotation(detector: &str, row: u64) -> Annotation {
    Annotation::new(
        AnalyzerId::new(detector),
        "event",
        RowRange {
            table: TableId(1),
            generation: Generation(0),
            start: row,
            end: row + 1,
        },
    )
}

/// SDR-35 (§10.4): only an alarm the user configured with high salience plays `E-ALARM`;
/// every other event plays `E-STREAM` (§10.3).
#[test]
fn sdr_35_only_configured_alarms_sound_urgent() {
    let score = score_with_rows(Mode::Monitoring);
    let annotations = [annotation("outliers", 3), annotation("changepoints", 7)];
    let unconfigured = events(&score, &annotations, &[]);
    assert_eq!(unconfigured.len(), 2);
    assert!(
        unconfigured
            .iter()
            .all(|e| !e.alarm && e.salience == Salience::Low)
    );
    assert_eq!(
        unconfigured[0].t,
        Seconds(3.0),
        "at the note of the first row"
    );
    let kinds = earcon_kinds(&soundscape(&ENGINE, Seconds(10.0), &unconfigured));
    assert_eq!(kinds, [EarconKind::Stream, EarconKind::Stream]);

    let configured = events(
        &score,
        &annotations,
        &[
            (Detector::Outliers, Salience::High),
            (Detector::Changepoints, Salience::Medium),
        ],
    );
    let kinds = earcon_kinds(&soundscape(&ENGINE, Seconds(10.0), &configured));
    assert_eq!(kinds, [EarconKind::Alarm, EarconKind::Stream]);
    assert_eq!(configured[1].salience, Salience::Medium);
}

fn provide(mode: Mode, monitor: &serde_json::Value) -> (Vec<ScoreTrack>, CompileContext) {
    let spec = json!({
        "specVersion": "0.1",
        "mode": if mode == Mode::Monitoring { "monitoring" } else { "analysis" },
        "data": { "source": { "kind": "inline", "values": [{ "y": 1 }] } },
        "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "y" } } }],
        "playback": { "monitor": monitor }
    });
    let fragment = SpecFragment::new(SpecPointer::root(), &spec).unwrap();
    let mut cx = CompileContext::new(0, Budget::default(), CancelToken::new());
    let score = score_with_rows(mode);
    let tracks = ContextProvider::<DataBundle>::context_tracks(
        &MonitorContext,
        &fragment,
        &score,
        &DataBundle::new(
            BundleId(0),
            SourceDescriptor::new(SourceKind::Inline, "test", "test"),
            Vec::new(),
        )
        .unwrap(),
        &[annotation("outliers", 4)],
        &mut cx,
    )
    .unwrap();
    (tracks, cx)
}

/// SDR-36 with SDR-35: the soundscape exists only in monitoring mode.
#[test]
fn sdr_35_the_soundscape_plays_only_in_monitoring_mode() {
    let (none, _) = provide(Mode::Analysis, &json!({}));
    assert_eq!(none, Vec::new());
    let (tracks, _) = provide(
        Mode::Monitoring,
        &json!({ "alarms": [{ "detector": "outliers", "salience": "high" }] }),
    );
    let roles: Vec<TrackRole> = tracks.iter().map(|t| t.role).collect();
    assert_eq!(roles, [TrackRole::Monitor, TrackRole::Earcon]);
    assert_eq!(tracks[1].events.len(), 1, "the outlier");
}

/// SDR-39 (INFER): the comfort budget is reported, flagged experimental, never enforced.
#[test]
fn sdr_39_the_comfort_budget_is_reported_as_experimental() {
    let (_, cx) = provide(Mode::Monitoring, &json!({}));
    let report = cx
        .diagnostics()
        .iter()
        .find(|d| d.code.to_string() == "SNR-ENG-241")
        .unwrap();
    assert_eq!(report.severity, Severity::Info);
    assert!(
        report.message.contains("experimental"),
        "{}",
        report.message
    );
    assert!(report.message.contains("SDR-39"));
}

#[test]
fn an_unknown_soundscape_falls_back_to_the_default_bed() {
    let (tracks, cx) = provide(Mode::Monitoring, &json!({ "soundscape": "forest" }));
    assert_eq!(tracks.len(), 2);
    assert!(
        cx.diagnostics()
            .iter()
            .any(|d| d.code.to_string() == "SNR-ENG-240")
    );
}

#[test]
fn the_comfort_share_measures_the_bright_part_of_the_session() {
    let quiet = soundscape(&ENGINE, Seconds(10.0), &[]);
    assert!(quiet.bright_share.abs() < 1e-12);
    let busy: Vec<MonitorEvent> = (0..10)
        .map(|i| MonitorEvent::new(Seconds(f64::from(i)), Salience::High, false))
        .collect();
    let bright = soundscape(&ENGINE, Seconds(10.0), &busy);
    assert!(bright.bright_share > 0.5, "{}", bright.bright_share);
    assert!(bright.bright_share <= 1.0);
}

/// The golden case `monitoring-soundscape` is this module's output, so the reference
/// audio follows the code (tests/golden/README.md). Set `SONOTRACE_WRITE_GOLDEN=1` to
/// rewrite `score.json` after a deliberate change, then update the reference.
#[test]
fn the_golden_monitoring_score_is_the_soundscape() {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Monitoring, 35);
    header.duration = Seconds(10.0);
    let mut score = Score::new(header);
    let events = [
        MonitorEvent::new(Seconds(1.5), Salience::Low, false),
        MonitorEvent::new(Seconds(4.0), Salience::Medium, false),
        MonitorEvent::new(Seconds(7.0), Salience::High, true),
    ];
    let soundscape = soundscape(&ENGINE, Seconds(10.0), &events);
    score.tracks = vec![soundscape.bed, soundscape.earcons];
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tests/golden/monitoring-soundscape/score.json");
    let mut text = serde_json::to_string_pretty(&score).unwrap();
    text.push('\n');
    if std::env::var_os("SONOTRACE_WRITE_GOLDEN").is_some() {
        std::fs::write(&path, &text).unwrap();
    }
    let committed = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(committed, text);
}
