// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The stream soak (B06-T07, PB-11; testing-strategy §9.3). A synthetic source writes
//! `--rate` rows per second to a file; the app core follows the file as a `tail` stream
//! and plays it live through a monitoring spec, compiling the newest 10 s window about
//! once a second and swapping it in while playing. Every 10 s it samples the process's
//! resident memory, the rows written and received, and the transport. At the end it
//! judges:
//!
//! - **bounded memory**: the peak resident size in the second half of the run is within
//!   10 % + 8 MiB of the peak between the warm-up and the half. The stream keeps 100 000
//!   rows, so at 1 kHz it is full after 100 s; the warm-up is the first quarter, at least
//!   120 s. Runs shorter than 300 s report memory without judging it.
//! - **no rows lost**: every row written was received and none was invalid.
//! - **live**: at every sample after the warm-up the timeline is within 5 s of the data
//!   written and the playhead within 5 s of the timeline's end, still playing.
//! - **underruns**: reported; gated only with `--max-xruns` (on reference hardware; the
//!   timer-driven null device counts a late block as one).
//!
//! ```text
//! cargo run -p sonarithm-app --example stream_soak --release -- [options]
//!   --seconds N     soak length (default 3600)
//!   --rate N        rows per second (default 1000)
//!   --output null|system   the silent timer-driven device (default), or the system's
//!                   default device at the lowest volume, -60 dB (the owner's run on
//!                   reference hardware; never in CI)
//!   --report FILE   also write the samples as JSON lines
//!   --max-xruns N   exit with status 1 above N underruns
//!   --strict        exit with status 1 if memory grew, rows were lost or playback fell
//!                   behind (CI)
//! ```
//!
//! Nothing touches the network: the stream follows a local file.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a measurement program reports to the console and stops on setup failures"
)]

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sonarithm_app::ipc::Event;
use sonarithm_app::testing::{FileFeed, TempDir};
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};

/// Seconds between samples.
const EVERY: u64 = 10;

#[derive(Clone, Debug)]
struct Sample {
    at_s: u64,
    resident: Option<u64>,
    written: u64,
    received: u64,
    invalid: u64,
    xruns: u64,
    duration_s: f64,
    position_s: f64,
    playing: bool,
}

impl Sample {
    fn json(&self) -> Value {
        json!({
            "atS": self.at_s,
            "residentBytes": self.resident,
            "rowsWritten": self.written,
            "rowsReceived": self.received,
            "rowsInvalid": self.invalid,
            "xruns": self.xruns,
            "durationS": self.duration_s,
            "positionS": self.position_s,
            "playing": self.playing,
        })
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the soak's steps read best in order: set up, run, drain, judge"
)]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let seconds: u64 = value("--seconds")
        .and_then(|s| s.parse().ok())
        .unwrap_or(3600);
    let rate: f64 = value("--rate")
        .and_then(|s| s.parse().ok())
        .filter(|r: &f64| r.is_finite() && *r > 0.0)
        .unwrap_or(1000.0);
    let system = value("--output").as_deref() == Some("system");
    let max_xruns: Option<u64> = value("--max-xruns").and_then(|s| s.parse().ok());
    let strict = args.iter().any(|a| a == "--strict");
    let mut report = value("--report")
        .map(|path| std::fs::File::create(path).expect("creating the report file"));

    let dir = TempDir::new("stream-soak");
    let file = dir.path().join("feed.ndjson");
    std::fs::write(&file, "").expect("creating the feed file");
    let sink = CollectingSink::new();
    let audio = if system {
        AudioOutput::System
    } else {
        AudioOutput::null()
    };
    let config = AppConfig::new(dir.path().join("config")).with_audio(audio);
    let mut app = App::new(config, Arc::new(sink.clone())).expect("starting the app core");
    let mut send = |request: Value| -> Value {
        app.dispatch_json(&request.to_string())
            .unwrap_or_else(|err| panic!("{request} failed: {err}"))
    };
    if system {
        // Hearing safety: the soak measures timing, not sound.
        let mut settings = send(json!({ "command": "settings_get" }));
        settings.as_object_mut().unwrap().remove("reply");
        settings["audio"]["volumeDb"] = json!(-60.0);
        send(json!({ "command": "settings_set", "args": { "settings": settings } }));
    }
    let mut compiles = window_compile_ms(&file, rate);
    compiles.sort_by(f64::total_cmp);
    let compile_line = format!(
        "window compile (10 s, {} rows): median {:.1} ms, max {:.1} ms over {} compiles",
        (rate * 10.0).round(),
        compiles[compiles.len() / 2],
        compiles[compiles.len() - 1],
        compiles.len()
    );
    println!("{compile_line}");
    start_live(&mut send, &file);
    let feed = FileFeed::start(&file, rate);
    send(json!({ "command": "live_start", "args": { "stream": "s1" } }));
    let ready = Instant::now();
    while send(json!({ "command": "transport_state" }))["loaded"] != true {
        assert!(
            ready.elapsed() < Duration::from_secs(30),
            "no live window compiled in 30 s"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    send(json!({ "command": "transport_play" }));
    println!(
        "stream soak: {seconds} s at {rate} rows/s on the {} output",
        if system { "system" } else { "null" }
    );

    let start = Instant::now();
    let mut samples = Vec::new();
    let mut notices = 0_usize;
    for tick in 1..=seconds.div_ceil(EVERY) {
        let due = Duration::from_secs((tick * EVERY).min(seconds));
        if let Some(wait) = due.checked_sub(start.elapsed()) {
            std::thread::sleep(wait);
        }
        let sample = sample(&mut send, &feed, start);
        notices += count_notices(&sink);
        println!("{}", sample.json());
        if let Some(report) = &mut report {
            let _ = writeln!(report, "{}", sample.json());
        }
        samples.push(sample);
    }
    // Let the stream catch up with the last rows written.
    drop(feed);
    let written = samples.last().map_or(0, |s| s.written);
    let caught_up = Instant::now();
    let mut last = samples.last().cloned();
    while caught_up.elapsed() < Duration::from_secs(3) {
        let streams = send(json!({ "command": "stream_list" }));
        let received = streams["streams"][0]["received"].as_u64().unwrap_or(0);
        if let Some(last) = &mut last {
            last.received = received;
        }
        if received >= written {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    send(json!({ "command": "live_stop" }));

    let verdict = judge(&samples, last.as_ref(), seconds, rate, max_xruns);
    println!("{compile_line}");
    println!("notices: {notices} (SNR-APP-074 and audio failures)");
    for line in &verdict.lines {
        println!("{line}");
    }
    if (strict && !verdict.clean) || verdict.xruns_over {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Follows `file` as stream `s1` and sets the monitoring spec the soak plays.
fn start_live(send: &mut impl FnMut(Value) -> Value, file: &Path) {
    let offer = send(json!({ "command": "stream_offer",
        "args": { "transport": "tail", "path": file } }));
    send(json!({ "command": "stream_connect", "args": { "token": offer["token"] } }));
    let spec = soak_spec(file);
    send(json!({ "command": "spec_set", "args": { "text": spec.to_string() } }));
}

/// The soak's spec: a glide over the newest 10 s, one Score second per second of data.
fn soak_spec(file: &Path) -> Value {
    json!({
        "specVersion": "0.1",
        "name": "Stream soak",
        "mode": "monitoring",
        "data": { "source": { "kind": "stream", "transport": "tail", "path": file,
            "window": { "s": 10 } } },
        "substrate": { "time": { "field": "t", "pace": { "secondsPerUnit": 1 } } },
        "tracks": [{ "id": "v", "mark": "glide", "channels": { "pitch": { "field": "v" } } }]
    })
}

/// Milliseconds per compile of one full 10 s window at `rate` rows per second, 20 times,
/// with the facade as the live worker calls it: whether encoding the whole window every
/// second keeps up (07 SONOTRACE, B06-T03).
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "row counts of a synthetic window"
)]
fn window_compile_ms(file: &Path, rate: f64) -> Vec<f64> {
    use std::fmt::Write as _;
    let rows = (rate * 10.0) as u64;
    let mut csv = String::from(
        "t,v
",
    );
    for n in 0..rows {
        let t = n as f64 / rate;
        let _ = writeln!(csv, "{t},{}", (t * 0.5).sin());
    }
    let options = sonarithm::ReadOptions::default();
    let loaded = sonarithm::read_bytes(
        csv.as_bytes(),
        sonarithm::Format::Csv,
        "window.csv",
        &options,
    )
    .expect("reading the synthetic window");
    let table = loaded.bundle.primary().expect("a table");
    let mut stream = sonarithm::StreamTable::new(
        sonarithm_core::TableId(0),
        "window",
        table.schema().clone(),
        sonarithm::Capacity::rows(rows),
    )
    .expect("a stream table");
    for batch in table.batches() {
        stream.append(batch.clone()).expect("appending");
    }
    let window = stream.snapshot();
    let snr = sonarithm::Sonarithm::builder()
        .with_default_engines()
        .with_navigation()
        .build()
        .expect("the facade");
    let spec = sonarithm::Spec::from_json_str(soak_spec(file).to_string()).expect("the spec");
    (0..20)
        .map(|_| {
            let start = Instant::now();
            snr.compile_stream(&spec, &sonarithm::Datasets::new(), &window)
                .expect("compiling the window");
            start.elapsed().as_secs_f64() * 1_000.0
        })
        .collect()
}

fn sample(send: &mut impl FnMut(Value) -> Value, feed: &FileFeed, start: Instant) -> Sample {
    let state = send(json!({ "command": "transport_state" }));
    let streams = send(json!({ "command": "stream_list" }));
    let stream = &streams["streams"][0];
    Sample {
        at_s: start.elapsed().as_secs(),
        resident: resident_bytes(),
        written: feed.rows(),
        received: stream["received"].as_u64().unwrap_or(0),
        invalid: stream["invalid"].as_u64().unwrap_or(0),
        xruns: state["xruns"].as_u64().unwrap_or(0),
        duration_s: state["durationS"].as_f64().unwrap_or(0.0),
        position_s: state["positionS"].as_f64().unwrap_or(0.0),
        playing: state["status"] == "playing",
    }
}

/// Live notices and audio failures since the last call; the sink is emptied, so its
/// own memory stays flat.
fn count_notices(sink: &CollectingSink) -> usize {
    sink.take()
        .into_iter()
        .filter(|e| matches!(e, Event::LiveNotice { .. } | Event::AudioFailed { .. }))
        .count()
}

struct Verdict {
    clean: bool,
    xruns_over: bool,
    lines: Vec<String>,
}

#[expect(
    clippy::cast_precision_loss,
    reason = "byte counts and row counts far below 2^52"
)]
fn judge(
    samples: &[Sample],
    last: Option<&Sample>,
    seconds: u64,
    rate: f64,
    max_xruns: Option<u64>,
) -> Verdict {
    let mut lines = Vec::new();
    let mut clean = true;
    let warm_up = (seconds / 4).max(120);
    let half = seconds / 2;
    let peak = |from: u64, to: u64| {
        samples
            .iter()
            .filter(|s| s.at_s > from && s.at_s <= to)
            .filter_map(|s| s.resident)
            .max()
    };
    let mib = |bytes: u64| bytes as f64 / 1_048_576.0;
    if let (true, Some(before), Some(after)) =
        (seconds >= 300, peak(warm_up, half), peak(half, seconds))
    {
        let bound = before as f64 * 1.10 + 8.0 * 1_048_576.0;
        let ok = (after as f64) <= bound;
        clean &= ok;
        lines.push(format!(
            "memory: peak {:.1} MiB from {warm_up} s to {half} s, {:.1} MiB after: {}",
            mib(before),
            mib(after),
            if ok { "bounded" } else { "GREW" }
        ));
    } else {
        let top = samples.iter().filter_map(|s| s.resident).max();
        lines.push(format!(
            "memory: peak {} (run of {seconds} s; judged from 300 s)",
            top.map_or_else(|| "unknown".to_owned(), |b| format!("{:.1} MiB", mib(b)))
        ));
    }
    if let Some(last) = last {
        let ok = last.received >= last.written && last.invalid == 0;
        clean &= ok;
        lines.push(format!(
            "rows: {} written, {} received, {} invalid: {}",
            last.written,
            last.received,
            last.invalid,
            if ok { "none lost" } else { "LOST" }
        ));
    }
    let behind = samples
        .iter()
        .filter(|s| s.at_s > warm_up.min(seconds / 2))
        .map(|s| {
            let data = s.written as f64 / rate;
            let timeline = (data - s.duration_s).max(0.0);
            let playhead = (s.duration_s - s.position_s).max(0.0);
            (timeline, playhead, s.playing)
        })
        .fold((0.0_f64, 0.0_f64, true), |(a, b, p), (t, h, playing)| {
            (a.max(t), b.max(h), p && playing)
        });
    let ok = behind.0 <= 5.0 && behind.1 <= 5.0 && behind.2;
    clean &= ok;
    lines.push(format!(
        "live: timeline at most {:.2} s behind the data, playhead at most {:.2} s behind the timeline, {}: {}",
        behind.0,
        behind.1,
        if behind.2 { "playing throughout" } else { "STOPPED" },
        if ok { "kept up" } else { "FELL BEHIND" }
    ));
    let xruns = samples.last().map_or(0, |s| s.xruns);
    let xruns_over = max_xruns.is_some_and(|max| xruns > max);
    lines.push(format!(
        "xruns: {xruns}{}",
        max_xruns.map_or_else(String::new, |max| format!(
            " (at most {max}: {})",
            if xruns_over { "FAIL" } else { "pass" }
        ))
    ));
    lines.push(if clean && !xruns_over {
        "PB-11 soak: pass".to_owned()
    } else {
        "PB-11 soak: FAIL".to_owned()
    });
    Verdict {
        clean,
        xruns_over,
        lines,
    }
}

/// The process's resident memory in bytes, read with the OS's own tools so no unsafe
/// code is needed: `/proc` on Linux, `tasklist` (working set) on Windows, `ps` elsewhere.
fn resident_bytes() -> Option<u64> {
    let pid = std::process::id();
    let kib: u64 = if cfg!(target_os = "linux") {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
        line.split_whitespace().nth(1)?.parse().ok()?
    } else if cfg!(windows) {
        let out = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        // The last column is the memory use, such as "123,456 K" in any digit grouping.
        let field = text.trim().rsplit("\",\"").next()?;
        field
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .ok()?
    } else {
        let out = Command::new("ps")
            .args(["-o", "rss=", "-p", &pid.to_string()])
            .output()
            .ok()?;
        String::from_utf8_lossy(&out.stdout).trim().parse().ok()?
    };
    Some(kib * 1024)
}
