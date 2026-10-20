// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The commands. Each one calls the facade and formats what it returns; none holds
//! business logic (02 §3 rule 6).

use std::fmt::Write as _;
use std::io::Read as _;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};
use sonarithm::score::{Score, ScoreEvent};
use sonarithm::{
    AudioFormat, Compiled, Diagnostics, Mark, Normalization, RenderManifest, RenderOptions,
    Rendered, SchemaKind, Severity, Sonarithm, Spec, WavEncoding,
};

use crate::cli::{
    Checked, Command, Devices, FormatArg, Inspect, MarkArg, Migrate, Output, Play, Quick, Render,
    Schema, SpecCommand, StudyCommand,
};
use crate::output::{Counts, Exit, Out, counts, error, json_value, line, raw, shown};

/// Runs the parsed command line.
pub(crate) fn run(command: &Command, out: Out) -> Exit {
    match command {
        Command::Render(args) => render(args, out),
        Command::Play(args) => play(args, out),
        Command::Devices(args) => devices(args, out),
        Command::Validate(args) => validate(args, out),
        Command::Inspect(args) => inspect(args, out),
        Command::Lint(args) => lint(args, out),
        Command::Quick(args) => quick(args, out),
        Command::Schema(args) => schema(args),
        Command::Spec(SpecCommand::Migrate(args)) => migrate(args, out),
        Command::Study(StudyCommand::Check(args)) => crate::study::check(args, out),
        Command::Study(StudyCommand::Run(args)) => crate::study::run(args, out),
    }
}

/// A spec file's name (never its directory, P8), text and directory.
struct Source {
    name: String,
    text: String,
    base: PathBuf,
}

/// The last path component, for messages and output.
fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.to_string_lossy().into_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn read_source(path: &Path) -> Result<Source, String> {
    if path == Path::new("-") {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|err| format!("standard input cannot be read: {err}"))?;
        return Ok(Source {
            name: "stdin.json".to_owned(),
            text,
            base: PathBuf::from("."),
        });
    }
    let name = file_name(path);
    let text =
        std::fs::read_to_string(path).map_err(|err| format!("`{name}` cannot be read: {err}"))?;
    // A bare file name has the empty path as its parent: the current directory.
    let base = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    Ok(Source { name, text, base })
}

impl Source {
    fn spec(&self) -> Result<Spec, Diagnostics> {
        Spec::from_json_str(self.text.clone())
            .map(|spec| spec.with_name(&self.name).with_base_dir(&self.base))
            .map_err(|err| err.diagnostics())
    }
}

/// A command that could not start: the spec or data cannot be read.
fn unreadable(out: Out, name: &str, message: &str) -> Exit {
    if out.json() {
        json_value(&json!({ "spec": name, "ok": false, "error": message, "diagnostics": [] }));
    } else {
        error(message);
    }
    Exit::Errors
}

#[derive(Serialize)]
struct Checked_<'a> {
    spec: &'a str,
    ok: bool,
    counts: Counts,
    diagnostics: &'a Diagnostics,
}

fn validate(args: &Checked, out: Out) -> Exit {
    let source = match read_source(&args.spec) {
        Ok(source) => source,
        Err(message) => return unreadable(out, &file_name(&args.spec), &message),
    };
    let diagnostics = match source.spec() {
        Ok(spec) => spec.validate(),
        Err(diagnostics) => diagnostics,
    };
    let exit = Exit::of(&diagnostics, args.deny_warnings);
    if out.json() {
        json_value(&Checked_ {
            spec: &source.name,
            ok: !diagnostics.has_errors(),
            counts: Counts::of(&diagnostics),
            diagnostics: &diagnostics,
        });
        return exit;
    }
    let verdict = if diagnostics.has_errors() {
        "is not valid"
    } else {
        "is valid"
    };
    line(format!(
        "{} {verdict}: {}.",
        source.name,
        counts(&diagnostics)
    ));
    out.diagnostics(&source.name, &source.text, &diagnostics, args.verbose);
    hint_verbose(out, &diagnostics, args.verbose);
    exit
}

/// Points at `--verbose` when Info diagnostics were left out.
fn hint_verbose(out: Out, diagnostics: &Diagnostics, verbose: bool) {
    if !verbose && diagnostics.iter().any(|d| d.severity == Severity::Info) {
        out.note("--verbose lists the applied defaults and automatic choices.");
    }
}

/// Why a command produced no result.
enum Failed {
    /// Spec, data or engine diagnostics.
    Diagnostics(Diagnostics),
    /// An error that carries no diagnostic, such as a file that cannot be read.
    Message(String),
}

impl From<sonarithm::Error> for Failed {
    fn from(err: sonarithm::Error) -> Self {
        let diagnostics = err.diagnostics();
        if diagnostics.is_empty() {
            Self::Message(err.to_string())
        } else {
            Self::Diagnostics(diagnostics)
        }
    }
}

/// Compiles a spec.
fn compile(source: &Source) -> Result<Compiled, Failed> {
    let spec = source.spec().map_err(Failed::Diagnostics)?;
    Ok(Sonarithm::new().compile(&spec)?)
}

#[derive(Serialize)]
struct Inspected<'a> {
    spec: &'a str,
    ok: bool,
    summary: Option<sonarithm::Summary>,
    description: Option<sonarithm::Description>,
    counts: Counts,
    diagnostics: &'a Diagnostics,
    #[serde(skip_serializing_if = "Option::is_none")]
    score: Option<&'a Score>,
}

fn inspect(args: &Inspect, out: Out) -> Exit {
    let checked = &args.checked;
    let source = match read_source(&checked.spec) {
        Ok(source) => source,
        Err(message) => return unreadable(out, &file_name(&checked.spec), &message),
    };
    let compiled = match compile(&source) {
        Ok(compiled) => Ok(compiled),
        Err(Failed::Diagnostics(diagnostics)) => Err(diagnostics),
        Err(Failed::Message(message)) => return unreadable(out, &source.name, &message),
    };
    let diagnostics = match &compiled {
        Ok(compiled) => compiled.diagnostics().clone(),
        Err(diagnostics) => diagnostics.clone(),
    };
    let exit = Exit::of(&diagnostics, checked.deny_warnings);
    if out.json() {
        let compiled = compiled.as_ref().ok();
        json_value(&Inspected {
            spec: &source.name,
            ok: compiled.is_some(),
            summary: compiled.map(Compiled::summary),
            description: compiled.map(Compiled::describe),
            counts: Counts::of(&diagnostics),
            diagnostics: &diagnostics,
            score: compiled.filter(|_| args.tree).map(Compiled::score),
        });
        return exit;
    }
    match &compiled {
        Ok(compiled) => {
            print_summary(compiled);
            if args.describe {
                line("Description:");
                raw(&compiled.describe().to_string());
            }
            if args.tree {
                raw(&outline(compiled.score(), out.drawn()));
            }
        }
        Err(_) => line(format!(
            "{} did not compile: {}.",
            source.name,
            counts(&diagnostics)
        )),
    }
    out.diagnostics(&source.name, &source.text, &diagnostics, checked.verbose);
    hint_verbose(out, &diagnostics, checked.verbose);
    exit
}

/// The summary as sentences, one fact per line.
fn print_summary(compiled: &Compiled) {
    let summary = compiled.summary();
    line(format!(
        "{}: spec version {}, {} mode, seed {}.",
        summary.spec,
        summary.spec_version,
        mode_name(summary.mode),
        summary.seed
    ));
    line(format!("Duration: {:.3} seconds.", summary.duration_s));
    let engines: Vec<String> = summary
        .engines
        .iter()
        .map(|(id, version)| format!("{id} {version}"))
        .collect();
    line(format!("Engines: {}.", engines.join(", ")));
    for track in &summary.tracks {
        let mut text = format!(
            "Track {}: {} track, {} events",
            track.id,
            role_name(track.role),
            track.events
        );
        let mut kinds = Vec::new();
        for (count, one, many) in [
            (track.notes, "note", "notes"),
            (track.glides, "glide", "glides"),
            (track.earcons, "earcon", "earcons"),
            (track.speech, "speech event", "speech events"),
        ] {
            if count > 0 {
                kinds.push(format!("{count} {}", if count == 1 { one } else { many }));
            }
        }
        if !kinds.is_empty() {
            let _ = write!(text, " ({})", kinds.join(", "));
        }
        if let (Some(start), Some(end)) = (track.start_s, track.end_s) {
            let _ = write!(text, ", {start:.3} to {end:.3} seconds");
        }
        if let Some([low, high]) = track.pitch_range_hz {
            let _ = write!(text, ", pitch {low:.2} to {high:.2} Hz");
        }
        if track.estimated > 0 {
            let _ = write!(text, ", {} from estimated data", track.estimated);
        }
        let _ = write!(text, ", instrument {}.", track.instrument);
        line(text);
    }
    if let Some(hash) = &summary.score_hash {
        line(format!("Score hash: {hash}."));
    }
    line(format!("Diagnostics: {}.", counts(compiled.diagnostics())));
}

fn mode_name(mode: sonarithm::Mode) -> &'static str {
    match mode {
        sonarithm::Mode::Analysis => "analysis",
        sonarithm::Mode::Presentation => "presentation",
        sonarithm::Mode::Monitoring => "monitoring",
    }
}

fn role_name(role: sonarithm::TrackRole) -> &'static str {
    match role {
        sonarithm::TrackRole::Data => "data",
        sonarithm::TrackRole::Context => "context",
        sonarithm::TrackRole::Earcon => "earcon",
        sonarithm::TrackRole::Speech => "speech",
        sonarithm::TrackRole::Monitor => "monitoring",
    }
}

/// Every event as an outline: indented in plain text, drawn with box characters on a
/// terminal.
fn outline(score: &Score, drawn: bool) -> String {
    let mut text = String::from("Score\n");
    let count = score.tracks.len();
    for (index, track) in score.tracks.iter().enumerate() {
        let last_track = index + 1 == count;
        let (branch, stem) = match (drawn, last_track) {
            (false, _) => ("  ", "    "),
            (true, false) => ("├─ ", "│  "),
            (true, true) => ("└─ ", "   "),
        };
        let _ = writeln!(
            text,
            "{branch}Track {} ({} events)",
            track.id,
            track.events.len()
        );
        let events = track.events.len();
        for (i, event) in track.events.iter().enumerate() {
            let leaf = match (drawn, i + 1 == events) {
                (false, _) => "",
                (true, false) => "├─ ",
                (true, true) => "└─ ",
            };
            let _ = writeln!(text, "{stem}{leaf}{}", event_line(event));
        }
    }
    text
}

fn hz(cents: sonarithm::units::Cents) -> f64 {
    cents.to_hz().0
}

fn event_line(event: &ScoreEvent) -> String {
    match event {
        ScoreEvent::Note(note) => {
            let mut line = format!(
                "note at {:.3} s for {:.3} s, {:.2} Hz",
                note.t.0,
                note.dur.0,
                hz(note.pitch)
            );
            if note.taps > 1 {
                let _ = write!(line, ", {} taps", note.taps);
            }
            if let Some(row) = note.src {
                let _ = write!(line, ", row {}", row.row);
            }
            if let Some(conf) = note.conf {
                let _ = write!(line, ", confidence {:.2}", f32::from(conf));
            }
            line
        }
        ScoreEvent::Glide(glide) => {
            let (first, last) = (glide.curve.first(), glide.curve.last());
            let mut line = format!(
                "glide from {:.3} to {:.3} s, {} points",
                glide.t0.0,
                glide.t1.0,
                glide.curve.len()
            );
            if let (Some(first), Some(last)) = (first, last) {
                let _ = write!(line, ", {:.2} to {:.2} Hz", hz(first.pitch), hz(last.pitch));
            }
            line
        }
        ScoreEvent::Earcon(earcon) => {
            let kind = serde_json::to_value(earcon.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
            format!("earcon {kind} at {:.3} s", earcon.t.0)
        }
        ScoreEvent::Speech(speech) => format!("speech at {:.3} s: {}", speech.t.0, speech.text),
        other => format!("event at {:.3} s", other.start().0),
    }
}

/// The render options a command line asks for.
fn render_options(audio: &Output) -> RenderOptions {
    let mut options = RenderOptions::new();
    if let Some(format) = audio.format {
        options = options.with_format(match format {
            FormatArg::Wav => AudioFormat::Wav(WavEncoding::Pcm24),
            FormatArg::Flac => AudioFormat::Flac,
        });
    }
    if let Some(hz) = audio.sample_rate {
        options = options.with_sample_rate(hz);
    }
    if audio.preserve_level {
        options = options.with_normalization(Normalization::Preserve);
    }
    if audio.no_cues {
        options = options.without_cues();
    }
    options
}

/// What `render` and `quick -o` print in JSON.
#[derive(Serialize)]
struct RenderedJson<'a> {
    audio: &'a str,
    manifest_file: String,
    cues: Option<String>,
    manifest: &'a RenderManifest,
}

impl<'a> RenderedJson<'a> {
    fn of(rendered: &'a Rendered) -> Self {
        Self {
            audio: &rendered.manifest.output.file,
            manifest_file: file_name(&rendered.manifest_path),
            cues: rendered.cues.as_deref().map(file_name),
            manifest: &rendered.manifest,
        }
    }
}

/// Renders `compiled` to `path`, announcing it as a text line first (11 §2: progress as
/// text lines, never a spinner).
fn write_audio_files(
    out: Out,
    compiled: &Compiled,
    path: &Path,
    audio: &Output,
) -> Result<Rendered, String> {
    if !out.json() {
        line(format!("Rendering {} ...", file_name(path)));
    }
    Sonarithm::new()
        .render_to_file(compiled, path, &render_options(audio))
        .map_err(|err| err.to_string())
}

/// "Wrote temps.flac: 3.25 seconds at 48000 Hz, -16.0 LUFS, true peak -9.1 dBTP. Also
/// wrote temps.render.json and temps.cues.vtt."
fn print_rendered(rendered: &Rendered) {
    let manifest = &rendered.manifest;
    let output = &manifest.output;
    let loudness = output.integrated_lufs.map_or_else(
        || "silent".to_owned(),
        |lufs| format!("{lufs:.1} LUFS integrated"),
    );
    line(format!(
        "Wrote {}: {:.2} seconds at {} Hz, {loudness}, true peak {:.1} dBTP.",
        output.file, manifest.render.duration_s, manifest.render.sample_rate, output.true_peak_dbtp
    ));
    let mut also = vec![file_name(&rendered.manifest_path)];
    if let Some(cues) = &rendered.cues {
        also.push(file_name(cues));
    }
    line(format!("Also wrote {}.", also.join(" and ")));
    if output.limited_frames > 0 {
        line(format!(
            "The master limiter reduced the level in {} frames.",
            output.limited_frames
        ));
    }
}

#[derive(Serialize)]
struct RenderJson<'a> {
    spec: &'a str,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    rendered: Option<RenderedJson<'a>>,
    counts: Counts,
    diagnostics: &'a Diagnostics,
}

/// The spec with the command line's data file and seed applied.
fn overridden(source: &Source, data: Option<&Path>, seed: Option<u64>) -> Result<Spec, Failed> {
    let spec = source.spec().map_err(Failed::Diagnostics)?;
    if data.is_none() && seed.is_none() {
        return Ok(spec);
    }
    let mut value: Value = serde_json::from_str(spec.text())
        .map_err(|err| Failed::Message(format!("{} is not JSON: {err}", source.name)))?;
    if let Some(seed) = seed {
        value["seed"] = json!(seed);
    }
    if let Some(data) = data {
        let relative = relative_path(&source.base, data).ok_or_else(|| {
            Failed::Message(format!(
                "`{}` cannot be read, or cannot be named relative to the spec (on another drive)",
                file_name(data)
            ))
        })?;
        value["data"]["source"] = json!({ "kind": "file", "path": relative });
    }
    Ok(Spec::from_value(&value)
        .with_name(&source.name)
        .with_base_dir(&source.base))
}

fn render(args: &Render, out: Out) -> Exit {
    let source = match read_source(&args.spec) {
        Ok(source) => source,
        Err(message) => return unreadable(out, &file_name(&args.spec), &message),
    };
    let compiled = overridden(&source, args.data.as_deref(), args.seed)
        .and_then(|spec| Ok(Sonarithm::new().compile(&spec)?));
    let (compiled, mut diagnostics) = match compiled {
        Ok(compiled) => {
            let diagnostics = compiled.diagnostics().clone();
            (Some(compiled), diagnostics)
        }
        Err(Failed::Diagnostics(diagnostics)) => (None, diagnostics),
        Err(Failed::Message(message)) => return unreadable(out, &source.name, &message),
    };
    let mut exit = Exit::of(&diagnostics, args.deny_warnings);
    let mut rendered = None;
    if let (Some(compiled), Exit::Success) = (&compiled, exit) {
        match write_audio_files(out, compiled, &args.output, &args.audio) {
            Ok(done) => {
                diagnostics.extend(done.diagnostics.clone());
                rendered = Some(done);
            }
            Err(message) => {
                if out.json() {
                    json_value(
                        &json!({ "spec": source.name, "ok": false, "error": message, "diagnostics": diagnostics }),
                    );
                } else {
                    error(&message);
                }
                return Exit::Errors;
            }
        }
        exit = Exit::of(&diagnostics, args.deny_warnings);
    }
    if out.json() {
        json_value(&RenderJson {
            spec: &source.name,
            ok: rendered.is_some(),
            rendered: rendered.as_ref().map(RenderedJson::of),
            counts: Counts::of(&diagnostics),
            diagnostics: &diagnostics,
        });
        return exit;
    }
    match &rendered {
        Some(rendered) => print_rendered(rendered),
        None if compiled.is_some() => line(format!(
            "{} has warnings and --deny-warnings is set; nothing was written.",
            source.name
        )),
        None => line(format!(
            "{} did not compile: {}.",
            source.name,
            counts(&diagnostics)
        )),
    }
    out.diagnostics(&source.name, &source.text, &diagnostics, args.verbose);
    hint_verbose(out, &diagnostics, args.verbose);
    exit
}

#[derive(Serialize)]
struct PlayJson<'a> {
    spec: &'a str,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    played: Option<crate::play::Played>,
    counts: Counts,
    diagnostics: &'a Diagnostics,
}

fn play(args: &Play, out: Out) -> Exit {
    let source = match read_source(&args.spec) {
        Ok(source) => source,
        Err(message) => return unreadable(out, &file_name(&args.spec), &message),
    };
    let compiled = overridden(&source, args.data.as_deref(), args.seed)
        .and_then(|spec| Ok(Sonarithm::new().compile(&spec)?));
    let (compiled, diagnostics) = match compiled {
        Ok(compiled) => {
            let diagnostics = compiled.diagnostics().clone();
            (Some(compiled), diagnostics)
        }
        Err(Failed::Diagnostics(diagnostics)) => (None, diagnostics),
        Err(Failed::Message(message)) => return unreadable(out, &source.name, &message),
    };
    let exit = Exit::of(&diagnostics, args.deny_warnings);
    let mut played = None;
    if let (Some(compiled), Exit::Success) = (&compiled, exit) {
        // Diagnostics come first, so a screen reader hears warnings before the sound.
        if !out.json() {
            out.diagnostics(&source.name, &source.text, &diagnostics, args.verbose);
        }
        let settings = crate::play::Playback {
            name: &source.name,
            device: args.device.as_deref(),
            speed: args.speed,
            json: out.json(),
        };
        match crate::play::play(compiled, args.backend, &settings) {
            Ok(done) => played = Some(done),
            Err(message) => return unreadable(out, &source.name, &message),
        }
    }
    if out.json() {
        json_value(&PlayJson {
            spec: &source.name,
            ok: played.is_some(),
            played,
            counts: Counts::of(&diagnostics),
            diagnostics: &diagnostics,
        });
        return exit;
    }
    match (&compiled, &played) {
        (_, Some(_)) => {}
        (Some(_), None) => line(format!(
            "{} has warnings and --deny-warnings is set; nothing was played.",
            source.name
        )),
        (None, _) => line(format!(
            "{} did not compile: {}.",
            source.name,
            counts(&diagnostics)
        )),
    }
    if played.is_none() {
        out.diagnostics(&source.name, &source.text, &diagnostics, args.verbose);
    }
    hint_verbose(out, &diagnostics, args.verbose);
    exit
}

#[derive(Serialize)]
struct DeviceJson {
    id: String,
    name: String,
    is_default: bool,
    sample_rate: Option<u32>,
    channels: Option<u16>,
}

fn devices(args: &Devices, out: Out) -> Exit {
    let devices = match crate::play::devices(args.backend) {
        Ok(devices) => devices,
        Err(message) => return unreadable(out, "devices", &message),
    };
    if out.json() {
        let list: Vec<DeviceJson> = devices
            .iter()
            .map(|d| DeviceJson {
                id: d.id.clone(),
                name: d.name.clone(),
                is_default: d.is_default,
                sample_rate: d.sample_rate,
                channels: d.channels,
            })
            .collect();
        json_value(&json!({ "ok": true, "devices": list }));
        return Exit::Success;
    }
    line(match devices.len() {
        0 => "No output devices.".to_owned(),
        1 => "1 output device:".to_owned(),
        n => format!("{n} output devices:"),
    });
    for device in &devices {
        let mut facts = Vec::new();
        if device.is_default {
            facts.push("default".to_owned());
        }
        if let Some(rate) = device.sample_rate {
            facts.push(format!("{rate} Hz"));
        }
        if let Some(channels) = device.channels {
            facts.push(format!(
                "{channels} {}",
                if channels == 1 { "channel" } else { "channels" }
            ));
        }
        if facts.is_empty() {
            line(format!("{}.", device.name));
        } else {
            line(format!("{}: {}.", device.name, facts.join(", ")));
        }
    }
    Exit::Success
}

#[derive(Serialize)]
struct Linted<'a> {
    spec: &'a str,
    ok: bool,
    /// The perceptual lint is a stub until ECHOMETRIC (C05).
    stub: bool,
    counts: Counts,
    diagnostics: Diagnostics,
}

fn lint(args: &Checked, out: Out) -> Exit {
    let source = match read_source(&args.spec) {
        Ok(source) => source,
        Err(message) => return unreadable(out, &file_name(&args.spec), &message),
    };
    let diagnostics = match compile(&source) {
        Ok(compiled) => compiled.diagnostics().clone(),
        Err(Failed::Diagnostics(diagnostics)) => diagnostics,
        Err(Failed::Message(message)) => return unreadable(out, &source.name, &message),
    };
    let exit = Exit::of(&diagnostics, args.deny_warnings);
    let findings = shown(&diagnostics, args.verbose);
    if out.json() {
        json_value(&Linted {
            spec: &source.name,
            ok: !diagnostics.has_errors(),
            stub: true,
            counts: Counts::of(&diagnostics),
            diagnostics: findings,
        });
        return exit;
    }
    line(format!("{}: {}.", source.name, counts(&diagnostics)));
    out.diagnostics(&source.name, &source.text, &diagnostics, args.verbose);
    out.note(
        "this lint is a stub: it lists the compiler's warnings and their rules. The perceptual lint (ECHOMETRIC) arrives in phase C05.",
    );
    exit
}

#[derive(Serialize)]
struct Quicked<'a> {
    data: &'a str,
    spec: &'a str,
    spec_written: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rendered: Option<RenderedJson<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    played: Option<crate::play::Played>,
    ok: bool,
    summary: Option<sonarithm::Summary>,
    description: Option<sonarithm::Description>,
    counts: Counts,
    diagnostics: &'a Diagnostics,
}

/// The quick builder with the command line's choices.
fn configure(mut quick: sonarithm::Quick, args: &Quick) -> sonarithm::Quick {
    if let Some(x) = &args.x {
        quick = quick.x(x);
    }
    if let Some(y) = &args.y {
        quick = quick.y(y);
    }
    if let Some(mark) = args.mark {
        quick = quick.mark(match mark {
            MarkArg::Tone => Mark::Tone,
            MarkArg::Glide => Mark::Glide,
            MarkArg::Auto => Mark::Auto,
        });
    }
    if let Some((low, high)) = args.range {
        quick = quick.range_hz(low, high);
    }
    if let Some(concept) = &args.concept {
        quick = quick.concept(concept);
    }
    if let Some(name) = &args.name {
        quick = quick.name(name);
    }
    quick
}

fn quick(args: &Quick, out: Out) -> Exit {
    let data_name = file_name(&args.data);
    let quick = match Sonarithm::new().quick().data_file(&args.data) {
        Ok(quick) => configure(quick, args),
        Err(err) => return unreadable(out, &data_name, &err.to_string()),
    };
    let spec = match quick.spec() {
        Ok(spec) => spec,
        Err(err) => return unreadable(out, &data_name, &err.to_string()),
    };
    let written = match &args.spec_out {
        Some(path) => match write_spec(&spec, &args.data, path) {
            Ok(()) => Some(file_name(path)),
            Err(message) => return unreadable(out, &data_name, &message),
        },
        None => None,
    };
    let compiled = match quick.compile().map_err(Failed::from) {
        Ok(compiled) => Ok(compiled),
        Err(Failed::Diagnostics(diagnostics)) => Err(diagnostics),
        Err(Failed::Message(message)) => return unreadable(out, &data_name, &message),
    };
    let diagnostics = match &compiled {
        Ok(compiled) => compiled.diagnostics().clone(),
        Err(diagnostics) => diagnostics.clone(),
    };
    let mut diagnostics = diagnostics;
    let mut exit = Exit::of(&diagnostics, args.deny_warnings);
    let mut rendered = None;
    if let (Some(path), Ok(compiled), Exit::Success) = (&args.output, &compiled, exit) {
        match write_audio_files(out, compiled, path, &args.audio) {
            Ok(done) => {
                diagnostics.extend(done.diagnostics.clone());
                rendered = Some(done);
            }
            Err(message) => return unreadable(out, &data_name, &message),
        }
        exit = Exit::of(&diagnostics, args.deny_warnings);
    }
    // Without -o, quick plays: after the summary, description and diagnostics, so a screen
    // reader hears them first.
    let playable = compiled
        .as_ref()
        .ok()
        .filter(|_| args.output.is_none() && exit == Exit::Success);
    let settings = crate::play::Playback {
        name: spec.name(),
        device: args.device.as_deref(),
        speed: None,
        json: out.json(),
    };
    if out.json() {
        let played = match playable {
            Some(compiled) => match crate::play::play(compiled, args.backend, &settings) {
                Ok(played) => Some(played),
                Err(message) => return unreadable(out, &data_name, &message),
            },
            None => None,
        };
        let compiled = compiled.as_ref().ok();
        json_value(&Quicked {
            data: &data_name,
            spec: spec.name(),
            spec_written: written.as_deref(),
            rendered: rendered.as_ref().map(RenderedJson::of),
            played,
            ok: compiled.is_some(),
            summary: compiled.map(Compiled::summary),
            description: compiled.map(Compiled::describe),
            counts: Counts::of(&diagnostics),
            diagnostics: &diagnostics,
        });
        return exit;
    }
    if let Some(written) = &written {
        line(format!("Wrote the spec to {written}."));
    }
    match &compiled {
        Ok(compiled) => {
            print_summary(compiled);
            line("Description:");
            raw(&compiled.describe().to_string());
            if let Some(rendered) = &rendered {
                print_rendered(rendered);
            }
        }
        Err(_) => line(format!(
            "{} did not compile: {}.",
            spec.name(),
            counts(&diagnostics)
        )),
    }
    out.diagnostics(spec.name(), spec.text(), &diagnostics, args.verbose);
    hint_verbose(out, &diagnostics, args.verbose);
    if let Some(compiled) = playable
        && let Err(message) = crate::play::play(compiled, args.backend, &settings)
    {
        return unreadable(out, &data_name, &message);
    }
    exit
}

/// Writes the quick spec as pretty JSON to `path`, with its data path relative to the
/// spec's own directory (P8).
fn write_spec(spec: &Spec, data: &Path, path: &Path) -> Result<(), String> {
    let mut value: Value = serde_json::from_str(spec.text()).map_err(|err| err.to_string())?;
    let spec_dir = match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let relative = relative_path(&spec_dir, data).ok_or_else(|| {
        format!(
            "the spec must be saved where it can name `{}` by a relative path (on the same drive)",
            file_name(data)
        )
    })?;
    value["data"]["source"]["path"] = json!(relative);
    let mut text = serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?;
    text.push('\n');
    std::fs::write(path, text)
        .map_err(|err| format!("`{}` cannot be written: {err}", file_name(path)))
}

/// `file` relative to `dir`, with `/` separators, or `None` across drives.
fn relative_path(dir: &Path, file: &Path) -> Option<String> {
    let dir = std::fs::canonicalize(dir).ok()?;
    let file = std::fs::canonicalize(file).ok()?;
    let from: Vec<Component<'_>> = dir.components().collect();
    let to: Vec<Component<'_>> = file.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    if common == 0 {
        return None;
    }
    let mut parts: Vec<String> = vec!["..".to_owned(); from.len() - common];
    parts.extend(
        to[common..]
            .iter()
            .map(|part| part.as_os_str().to_string_lossy().into_owned()),
    );
    Some(parts.join("/"))
}

fn schema(args: &Schema) -> Exit {
    let kind = match args.kind {
        crate::cli::SchemaArg::Spec => SchemaKind::Spec,
    };
    raw(&sonarithm::schema(kind));
    Exit::Success
}

#[derive(Serialize)]
struct Migrated<'a> {
    spec: &'a str,
    ok: bool,
    from: Option<String>,
    to: Option<String>,
    changed: bool,
    written: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    diagnostics: Diagnostics,
}

fn migrate(args: &Migrate, out: Out) -> Exit {
    let source = match read_source(&args.spec) {
        Ok(source) => source,
        Err(message) => return unreadable(out, &file_name(&args.spec), &message),
    };
    let target = if let Some(text) = &args.to {
        if let Ok(version) = text.parse::<sonarithm::SpecVersion>() {
            version
        } else {
            error(&format!(
                "`{text}` is not a spec version; write MAJOR.MINOR, for example 0.1"
            ));
            return Exit::Usage;
        }
    } else {
        sonarithm::SPEC_VERSION
    };
    if args.in_place && args.spec == Path::new("-") {
        error("--in-place needs a file, not standard input");
        return Exit::Usage;
    }
    let migration = source
        .spec()
        .map_err(Failed::Diagnostics)
        .and_then(|spec| spec.migrate_to(target).map_err(Failed::from));
    let migration = match migration {
        Ok(migration) => migration,
        Err(Failed::Message(message)) => return unreadable(out, &source.name, &message),
        Err(Failed::Diagnostics(diagnostics)) => {
            if out.json() {
                json_value(&Migrated {
                    spec: &source.name,
                    ok: false,
                    from: None,
                    to: None,
                    changed: false,
                    written: false,
                    text: None,
                    diagnostics,
                });
            } else {
                line(format!("{} cannot be migrated.", source.name));
                out.diagnostics(&source.name, &source.text, &diagnostics, false);
            }
            return Exit::Errors;
        }
    };
    let mut written = false;
    if args.in_place && migration.changed {
        if let Err(err) = std::fs::write(&args.spec, &migration.text) {
            return unreadable(
                out,
                &source.name,
                &format!("`{}` cannot be written: {err}", source.name),
            );
        }
        written = true;
    }
    if out.json() {
        json_value(&Migrated {
            spec: &source.name,
            ok: true,
            from: Some(migration.from.to_string()),
            to: Some(migration.to.to_string()),
            changed: migration.changed,
            written,
            text: (!args.in_place).then_some(migration.text.as_str()),
            diagnostics: Diagnostics::new(),
        });
        return Exit::Success;
    }
    if args.in_place {
        let message = if written {
            format!(
                "Migrated {} from spec version {} to {}.",
                source.name, migration.from, migration.to
            )
        } else {
            format!(
                "{} is already at spec version {}; nothing to do.",
                source.name, migration.to
            )
        };
        line(message);
    } else {
        if !migration.changed {
            out.note(&format!(
                "{} is already at spec version {}; printing it unchanged.",
                source.name, migration.to
            ));
        }
        raw(&migration.text);
    }
    Exit::Success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_climb_and_descend() {
        let root =
            std::env::temp_dir().join(format!("sonarithm-cli-relative-{}", std::process::id()));
        std::fs::create_dir_all(root.join("data")).unwrap();
        std::fs::create_dir_all(root.join("specs")).unwrap();
        std::fs::write(root.join("data/t.csv"), "x,y\n1,2\n").unwrap();
        assert_eq!(
            relative_path(&root.join("data"), &root.join("data/t.csv")).as_deref(),
            Some("t.csv")
        );
        assert_eq!(
            relative_path(&root.join("specs"), &root.join("data/t.csv")).as_deref(),
            Some("../data/t.csv")
        );
        assert_eq!(
            relative_path(&root, &root.join("data/t.csv")).as_deref(),
            Some("data/t.csv")
        );
    }
}
