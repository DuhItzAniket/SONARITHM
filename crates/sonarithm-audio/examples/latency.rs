// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Key-to-onset latency and playhead calibration (ADR-0016; testing-strategy §14).
//!
//! ```text
//! cargo run -p sonarithm-audio --example latency --release -- [options]
//!   --mode raw|shared|exclusive   output mode (default raw: RAW shared mode)
//!   --capture loopback|<id>       loopback self-test (default) or a capture device id
//!   --trials N                    cues to send (default 200)
//!   --level DB                    cue level in dBFS: at most -30 with RAW, -50 without
//!                                 (OEM processing can add about 20 dB, F-032)
//!   --list                        list capture devices and their ids, then exit
//! ```
//!
//! The loopback self-test measures only the software path. Sign-off uses a cable from the
//! output to an input (docs/engineering/manual-tests/a05-cable-latency.md). Exclusive mode
//! cannot be measured through the loopback, which only hears the shared mix.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a measurement program reports to the console"
)]

#[cfg(windows)]
fn main() {
    use sonarithm_audio::WasapiMode;
    use sonarithm_audio::latency::{CaptureSource, LatencyRun, MAX_LEVEL, measure};
    use sonarithm_core::Db;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    if args.iter().any(|a| a == "--list") {
        list_inputs();
        return;
    }
    let mut run = LatencyRun::new(match value("--capture").as_deref() {
        None | Some("loopback") => CaptureSource::Loopback,
        Some(id) => CaptureSource::Device(id.to_owned()),
    });
    run.mode = match value("--mode").as_deref() {
        Some("shared") => WasapiMode::Shared,
        Some("exclusive") => WasapiMode::Exclusive,
        _ => WasapiMode::SharedRaw,
    };
    if let Some(trials) = value("--trials").and_then(|t| t.parse().ok()) {
        run.trials = trials;
    }
    if let Some(level) = value("--level").and_then(|l| l.parse::<f64>().ok()) {
        run.level = Db(level.min(MAX_LEVEL.0));
    }
    if run.mode == WasapiMode::Exclusive {
        eprintln!(
            "Exclusive mode takes the device from every other application, including a screen \
             reader speaking on it."
        );
    }
    println!(
        "{} trials, {:?}, capture {:?}, cues at {} dBFS",
        run.trials, run.mode, run.capture, run.level.0
    );
    match measure(&run) {
        Ok(r) => {
            println!(
                "device {}: {:?}\ndetected {}/{}  p50 {:.1} ms  p95 {:.1} ms  p99 {:.1} ms  max {:.1} ms\n\
                 calibration offset (onset - predicted) {:.1} ms  xruns {}",
                r.device,
                r.stream,
                r.detected,
                r.trials,
                r.p50,
                r.p95,
                r.p99,
                r.max,
                r.offset,
                r.xruns
            );
        }
        Err(error) => eprintln!("measurement failed: {error}"),
    }
}

#[cfg(windows)]
fn list_inputs() {
    let _ = wasapi::initialize_mta();
    let Ok(enumerator) = wasapi::DeviceEnumerator::new() else {
        eprintln!("cannot list devices");
        return;
    };
    let Ok(collection) = enumerator.get_device_collection(&wasapi::Direction::Capture) else {
        return;
    };
    for index in 0..collection.get_nbr_devices().unwrap_or(0) {
        if let Ok(device) = collection.get_device_at_index(index) {
            println!(
                "{}  {}",
                device.get_id().unwrap_or_default(),
                device.get_friendlyname().unwrap_or_default()
            );
        }
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("the latency tool needs Windows (WASAPI); macOS and Linux ports are R-22 follow-ups");
}
