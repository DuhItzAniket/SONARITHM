// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A05-T07 acceptance: files decode in independent readers (hound for WAV, claxon for
//! FLAC) with the right sample rate, channel count and bit depth, and losslessly.

#![allow(
    clippy::unwrap_used,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::io::Cursor;

use sonarithm_core::{
    Breakpoint, Cents, Db, EngineId, Glide, InstrumentRef, Mode, Note, SampleRate, Score,
    ScoreEvent, ScoreHeader, ScoreTrack, Seconds, SpecVersion, TrackId, TrackRole,
};
use sonarithm_export::audio::to_pcm24;
use sonarithm_export::{AudioFormat, WavEncoding, write_audio};
use sonarithm_render::{OfflineRender, PlanConfig, RenderPlan};

fn plan(rate: u32) -> RenderPlan {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 5);
    header.duration = Seconds(1.0);
    let mut score = Score::new(header);
    let mut notes = ScoreTrack::new(
        TrackId::new("notes"),
        TrackRole::Data,
        EngineId::from_static("numisona"),
        InstrumentRef("square".to_owned()),
    );
    notes.events = (0..4)
        .map(|i| {
            ScoreEvent::Note(Note::new(
                Seconds(0.2 * f64::from(i)),
                Seconds(0.15),
                Cents(f64::from(i) * 300.0),
            ))
        })
        .collect();
    let mut glide = ScoreTrack::new(
        TrackId::new("glide"),
        TrackRole::Data,
        EngineId::from_static("numisona"),
        InstrumentRef("sine".to_owned()),
    );
    glide.events = vec![ScoreEvent::Glide(Glide::new(
        Seconds(0.0),
        Seconds(1.0),
        vec![
            Breakpoint::new(Seconds(0.0), Cents(-1200.0)),
            Breakpoint::new(Seconds(1.0), Cents(0.0)),
        ],
    ))];
    // −12 dB per track keeps the sum below full scale; the master limiter arrives in T09.
    notes.gain = Db(-12.0);
    glide.gain = Db(-12.0);
    score.tracks = vec![notes, glide];
    RenderPlan::build(&score, &PlanConfig::new(SampleRate::new(rate).unwrap()))
        .unwrap()
        .plan
}

/// The index of the first element that differs, or `None` if the slices are equal. Keeps
/// failure messages short for long sample buffers.
fn first_difference<T: PartialEq>(a: &[T], b: &[T]) -> Option<usize> {
    if a.len() != b.len() {
        return Some(a.len().min(b.len()));
    }
    a.iter().zip(b).position(|(x, y)| x != y)
}

fn render(rate: u32) -> Vec<f32> {
    OfflineRender::new(plan(rate), 256).render_all()
}

fn export(rate: u32, format: AudioFormat, gain: f32) -> (Vec<u8>, sonarithm_export::AudioReport) {
    let mut bytes = Cursor::new(Vec::new());
    let mut offline = OfflineRender::new(plan(rate), 256);
    let report = write_audio(&mut offline, format, gain, &mut bytes).unwrap();
    (bytes.into_inner(), report)
}

#[test]
fn a_24_bit_wav_decodes_losslessly_with_the_right_format() {
    for rate in [44_100, 48_000] {
        let (bytes, report) = export(rate, AudioFormat::Wav(WavEncoding::Pcm24), 1.0);
        let mut reader = hound::WavReader::new(Cursor::new(bytes)).unwrap();
        let spec = reader.spec();
        assert_eq!(
            (
                spec.sample_rate,
                spec.channels,
                spec.bits_per_sample,
                spec.sample_format
            ),
            (rate, 2, 24, hound::SampleFormat::Int)
        );
        let decoded: Vec<i32> = reader.samples::<i32>().map(Result::unwrap).collect();
        let expected: Vec<i32> = render(rate).into_iter().map(to_pcm24).collect();
        assert_eq!(decoded.len(), 2 * rate as usize, "one second of stereo");
        assert_eq!(first_difference(&decoded, &expected), None);
        assert_eq!(report.frames, u64::from(rate));
        assert_eq!(report.clipped, 0);
        assert!(report.peak > 0.1);
    }
}

#[test]
fn a_float_wav_stores_the_render_bit_for_bit() {
    let (bytes, report) = export(48_000, AudioFormat::Wav(WavEncoding::Float32), 1.0);
    let mut reader = hound::WavReader::new(Cursor::new(bytes)).unwrap();
    assert_eq!(reader.spec().sample_format, hound::SampleFormat::Float);
    assert_eq!(report.bits_per_sample, 32);
    let decoded: Vec<u32> = reader
        .samples::<f32>()
        .map(|s| s.unwrap().to_bits())
        .collect();
    let expected: Vec<u32> = render(48_000).iter().map(|s| s.to_bits()).collect();
    assert_eq!(first_difference(&decoded, &expected), None);
}

#[test]
fn a_flac_decodes_losslessly_with_the_right_stream_info() {
    let (bytes, report) = export(48_000, AudioFormat::Flac, 1.0);
    let mut reader = claxon::FlacReader::new(Cursor::new(bytes)).unwrap();
    let info = reader.streaminfo();
    assert_eq!(
        (
            info.sample_rate,
            info.channels,
            info.bits_per_sample,
            info.samples
        ),
        (48_000, 2, 24, Some(48_000))
    );
    let decoded: Vec<i32> = reader.samples().map(Result::unwrap).collect();
    let expected: Vec<i32> = render(48_000).into_iter().map(to_pcm24).collect();
    assert_eq!(first_difference(&decoded, &expected), None);
    assert_eq!(report.frames, 48_000);
}

#[test]
fn gain_scales_the_samples() {
    let (bytes, report) = export(48_000, AudioFormat::Wav(WavEncoding::Float32), 0.5);
    let mut reader = hound::WavReader::new(Cursor::new(bytes)).unwrap();
    let decoded: Vec<f32> = reader.samples::<f32>().map(Result::unwrap).collect();
    let expected: Vec<f32> = render(48_000).iter().map(|s| s * 0.5).collect();
    assert_eq!(first_difference(&decoded, &expected), None);
    assert!(report.peak < 0.5);
}

#[test]
fn quantization_rounds_half_to_even_and_clamps() {
    assert_eq!(to_pcm24(0.0), 0);
    assert_eq!(to_pcm24(1.0), 8_388_607);
    assert_eq!(to_pcm24(-1.0), -8_388_607);
    assert_eq!(to_pcm24(1.5), 8_388_607);
    assert_eq!(to_pcm24(-1.5), -8_388_608);
    assert_eq!(to_pcm24(f32::NAN), 0);
    assert_eq!(to_pcm24(0.5 / 8_388_607.0), 0, "half rounds to even");
    assert_eq!(to_pcm24(1.5 / 8_388_607.0), 2);
}
