// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A02-T01 spike: cpal 0.18 output latency on Windows, measured by capturing the output
//! endpoint in WASAPI loopback mode on the same QPC clock. Method and limits: `README.md`.
//!
//! Throwaway code: it is not production quality and has had no real-time review.

#[cfg(windows)]
mod direct;

#[cfg(not(windows))]
fn main() {
    eprintln!("This harness uses WASAPI loopback capture and runs on Windows only (README.md).");
}

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    harness::main()
}

#[cfg(windows)]
mod harness {
    use std::error::Error;
    use std::fmt::Write as _;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
    use std::time::Duration;

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{BufferSize, StreamConfig};

    use crate::direct::{self, qpc_ns};

    const TONE_HZ: f32 = 1000.0;
    const TONE_MS: f32 = 40.0;
    /// SDR-22: every cue has non-zero onset and offset ramps.
    const RAMP_MS: f32 = 5.0;
    /// Cue level and onset threshold (defaults −30 and −60 dBFS; `--amplitude`, `--threshold`).
    #[derive(Clone, Copy)]
    struct Levels {
        amp: f32,
        thr: f32,
    }
    /// The detector re-arms after this much silence.
    const QUIET_MS: f32 = 30.0;
    const SEED: u64 = 1729;

    fn tone_sample(k: usize, sr: f32, amp: f32) -> f32 {
        let t = k as f32 / sr;
        let (len, ramp) = (TONE_MS / 1000.0, RAMP_MS / 1000.0);
        let env = if t < ramp {
            0.5 - 0.5 * (std::f32::consts::PI * t / ramp).cos()
        } else if t > len - ramp {
            0.5 - 0.5 * (std::f32::consts::PI * (len - t) / ramp).cos()
        } else {
            1.0
        };
        amp * env * (std::f32::consts::TAU * TONE_HZ * t).sin()
    }

    fn atomics(n: usize) -> Vec<AtomicU64> {
        (0..n).map(|_| AtomicU64::new(0)).collect()
    }

    /// Lock-free state shared by the control thread and both callbacks. Everything is
    /// preallocated before the streams start; callbacks only store into atomics. All times
    /// are QPC nanoseconds (cpal's `StreamInstant` domain on WASAPI).
    struct Shared {
        trigger: AtomicU64,
        cb_entry: Vec<AtomicU64>,
        cb_clock: Vec<AtomicU64>,
        reported: Vec<AtomicU64>,
        onsets: Vec<AtomicU64>,
        onset_count: AtomicU64,
        frames_min: AtomicU64,
        frames_max: AtomicU64,
        gap_max: AtomicU64,
        peak_bits: AtomicU32,
        /// Output stream errors (device invalidated or changed): the run is invalid if > 0.
        errors: AtomicU64,
        /// Loopback capture discontinuities (cpal `Xrun`): packets the capture side dropped.
        discontinuities: AtomicU64,
    }

    /// The output side: plays the cue at frame 0 of the first callback after a trigger.
    struct ToneOut {
        sh: Arc<Shared>,
        seen: u64,
        pos: Option<usize>,
        last_entry: u64,
        tone_len: usize,
        sr: f32,
        amp: f32,
    }

    impl ToneOut {
        fn fill(&mut self, data: &mut [f32], channels: usize, clock_ns: u64, reported_ns: u64) {
            let entry = qpc_ns();
            let sh = &self.sh;
            let frames = (data.len() / channels) as u64;
            sh.frames_min.fetch_min(frames, Ordering::Relaxed);
            sh.frames_max.fetch_max(frames, Ordering::Relaxed);
            if self.last_entry != 0 {
                sh.gap_max
                    .fetch_max(entry - self.last_entry, Ordering::Relaxed);
            }
            self.last_entry = entry;
            let req = sh.trigger.load(Ordering::Acquire);
            if req != self.seen {
                self.seen = req;
                if let Some(i) = (req as usize)
                    .checked_sub(1)
                    .filter(|&i| i < sh.cb_entry.len())
                {
                    sh.cb_entry[i].store(entry, Ordering::Relaxed);
                    sh.cb_clock[i].store(clock_ns, Ordering::Relaxed);
                    sh.reported[i].store(reported_ns, Ordering::Relaxed);
                }
                self.pos = Some(0);
            }
            for frame in data.chunks_mut(channels) {
                let s = match self.pos {
                    Some(k) if k < self.tone_len => {
                        self.pos = Some(k + 1);
                        tone_sample(k, self.sr, self.amp)
                    }
                    _ => {
                        self.pos = None;
                        0.0
                    }
                };
                frame.fill(s);
            }
        }
    }

    #[derive(Clone, Copy)]
    enum Mode {
        Cpal(BufferSize),
        Direct { raw: bool, period: Option<u32> },
    }

    /// Where the cue is played and where it is captured.
    struct Endpoints {
        out: cpal::Device,
        /// `None`: loopback capture of `out`. `Some`: an input device (acoustic path).
        capture: Option<cpal::Device>,
        lv: Levels,
    }

    fn parse_mode(s: &str) -> Result<Mode, Box<dyn Error>> {
        let (kind, arg) = s.split_once(':').unwrap_or((s, ""));
        let period = if arg.is_empty() || arg == "default" {
            None
        } else {
            Some(arg.parse()?)
        };
        Ok(match kind {
            "cpal" => Mode::Cpal(period.map_or(BufferSize::Default, BufferSize::Fixed)),
            "direct" => Mode::Direct { raw: false, period },
            "raw" => Mode::Direct { raw: true, period },
            _ => return Err(format!("unknown mode {s}").into()),
        })
    }

    fn percentile(v: &[f64], p: f64) -> f64 {
        if v.is_empty() {
            return f64::NAN;
        }
        let mut s = v.to_vec();
        s.sort_by(f64::total_cmp);
        let rank = ((p / 100.0) * s.len() as f64).ceil() as usize;
        s[rank.clamp(1, s.len()) - 1]
    }

    struct Run {
        period_frames: u32,
        buffer_frames: String,
        missed: usize,
        stray: usize,
        total: Vec<f64>,
        wait: Vec<f64>,
        path: Vec<f64>,
        reported: Vec<f64>,
        clock_check: Vec<f64>,
    }

    fn measure(
        ep: &Endpoints,
        mode: Mode,
        trials: usize,
        sh: &Arc<Shared>,
    ) -> Result<Run, Box<dyn Error>> {
        let (device, lv) = (&ep.out, ep.lv);
        let default = device.default_output_config()?;
        let sr = default.sample_rate();
        let channels = usize::from(default.channels());
        let sr_f = sr as f32;
        let tone_len = (TONE_MS / 1000.0 * sr_f) as usize;
        let quiet_frames = (QUIET_MS / 1000.0 * sr_f) as usize;
        // Frames from cue start until the clean signal first exceeds the threshold.
        let detect = (0..tone_len)
            .find(|&k| tone_sample(k, sr_f, lv.amp).abs() > lv.thr)
            .unwrap_or(0);
        let detect_ns = detect as u64 * 1_000_000_000 / u64::from(sr);

        let in_sh = Arc::clone(sh);
        let (err_in, err_out) = (Arc::clone(sh), Arc::clone(sh));
        let (mut armed, mut quiet) = (false, 0usize);
        // Loopback: on WASAPI, an input stream on a render endpoint captures what it plays.
        let cap_dev = ep.capture.as_ref().unwrap_or(device);
        let cap_cfg = match &ep.capture {
            Some(d) => d.default_input_config()?,
            None => default,
        };
        let (cap_ch, cap_sr) = (
            usize::from(cap_cfg.channels()),
            u64::from(cap_cfg.sample_rate()),
        );
        let input = cap_dev.build_input_stream::<f32, _, _>(
            StreamConfig {
                channels: cap_cfg.channels(),
                sample_rate: cap_cfg.sample_rate(),
                buffer_size: BufferSize::Default,
            },
            move |data, info| {
                let cap = info.timestamp().capture.as_nanos() as u64;
                for (idx, frame) in data.chunks(cap_ch).enumerate() {
                    let peak = frame.iter().fold(0.0f32, |m, x| m.max(x.abs()));
                    in_sh.peak_bits.fetch_max(peak.to_bits(), Ordering::Relaxed);
                    if peak > lv.thr {
                        if armed {
                            let k = in_sh.onset_count.fetch_add(1, Ordering::AcqRel) as usize;
                            if let Some(slot) = in_sh.onsets.get(k) {
                                slot.store(
                                    cap + idx as u64 * 1_000_000_000 / cap_sr,
                                    Ordering::Release,
                                );
                            }
                            armed = false;
                        }
                        quiet = 0;
                    } else {
                        quiet += 1;
                        armed |= quiet >= quiet_frames;
                    }
                }
            },
            move |err| {
                if err.kind() == cpal::ErrorKind::Xrun {
                    err_in.discontinuities.fetch_add(1, Ordering::Relaxed);
                } else {
                    err_in.errors.fetch_add(1, Ordering::Relaxed);
                    eprintln!("loopback stream: {err}");
                }
            },
            None,
        )?;
        input.play()?;

        let mut tone = ToneOut {
            sh: Arc::clone(sh),
            seen: 0,
            pos: None,
            last_entry: 0,
            tone_len,
            sr: sr_f,
            amp: lv.amp,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (period_frames, buffer_frames, cpal_stream, direct_thread);
        match mode {
            Mode::Cpal(buffer) => {
                let config = StreamConfig {
                    channels: default.channels(),
                    sample_rate: sr,
                    buffer_size: buffer,
                };
                let out = device.build_output_stream::<f32, _, _>(
                    config,
                    move |data, info| {
                        let ts = info.timestamp();
                        let reported = ts.playback.duration_since(ts.callback).as_nanos() as u64;
                        tone.fill(data, channels, ts.callback.as_nanos() as u64, reported);
                    },
                    move |err| {
                        err_out.errors.fetch_add(1, Ordering::Relaxed);
                        eprintln!("output stream: {err}");
                    },
                    None,
                )?;
                out.play()?;
                period_frames = out.buffer_size()?;
                buffer_frames = "n/a".to_owned();
                cpal_stream = Some(out);
                direct_thread = None;
            }
            Mode::Direct { raw, period } => {
                let id = device.id()?.id().to_owned();
                let (handle, info) = direct::run(
                    &id,
                    raw,
                    period,
                    Arc::clone(&stop),
                    move |d, ch, clock, rep| {
                        tone.fill(d, ch, clock, rep);
                    },
                )?;
                period_frames = info.period_frames;
                let (def, fund, min, max) = info.engine_periods;
                buffer_frames = format!(
                    "{} (engine def {def}, fund {fund}, min {min}, max {max})",
                    info.buffer_frames
                );
                cpal_stream = None;
                direct_thread = Some(handle);
            }
        }
        std::thread::sleep(Duration::from_millis(500));

        let mut rng = SEED;
        let mut t0 = Vec::with_capacity(trials);
        for i in 0..trials {
            // xorshift64: randomized inter-trial interval of 400–700 ms (seeded), long enough for
            // the tail that OEM enhancement APOs add (about 200 ms on the dev laptop) to decay.
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            std::thread::sleep(Duration::from_millis(400 + rng % 300));
            t0.push(qpc_ns());
            sh.trigger.store(i as u64 + 1, Ordering::Release);
        }
        std::thread::sleep(Duration::from_millis(500));
        stop.store(true, Ordering::Release);
        drop(cpal_stream);
        if let Some(h) = direct_thread {
            let _ = h.join();
        }
        drop(input);

        let count = (sh.onset_count.load(Ordering::Acquire) as usize).min(sh.onsets.len());
        let onsets: Vec<u64> = sh.onsets[..count]
            .iter()
            .map(|o| o.load(Ordering::Acquire))
            .collect();
        let ms = |a: u64, b: u64| (a as f64 - b as f64) / 1e6;
        let mut run = Run {
            period_frames,
            buffer_frames,
            missed: 0,
            stray: 0,
            total: vec![],
            wait: vec![],
            path: vec![],
            reported: vec![],
            clock_check: vec![],
        };
        let mut used = 0;
        for (i, &t) in t0.iter().enumerate() {
            // Each onset belongs to at most one trial: it must precede the next trigger.
            let next = t0
                .get(i + 1)
                .copied()
                .unwrap_or(u64::MAX)
                .min(t + 1_000_000_000);
            let entry = sh.cb_entry[i].load(Ordering::Relaxed);
            match onsets.iter().find(|&&o| o >= t && o < next) {
                Some(&o) if entry != 0 => {
                    used += 1;
                    let onset = o - detect_ns;
                    run.total.push(ms(onset, t));
                    run.wait.push(ms(entry, t));
                    run.path.push(ms(onset, entry));
                    run.reported
                        .push(sh.reported[i].load(Ordering::Relaxed) as f64 / 1e6);
                    run.clock_check
                        .push(ms(sh.cb_clock[i].load(Ordering::Relaxed), entry));
                }
                _ => run.missed += 1,
            }
        }
        run.stray = count.saturating_sub(used);
        Ok(run)
    }

    pub(crate) fn main() -> Result<(), Box<dyn Error>> {
        let args: Vec<String> = std::env::args().collect();
        let arg = |name: &str| {
            args.iter()
                .position(|a| a == name)
                .and_then(|i| args.get(i + 1))
                .cloned()
        };
        let trials: usize = arg("--trials").map_or(Ok(200), |s| s.parse())?;
        let modes =
            arg("--modes").unwrap_or_else(|| "cpal:128,cpal:256,cpal:512,cpal:default".into());
        let db = |name: &str, d: f32| {
            arg(name)
                .map_or(Ok(d), |s| s.parse::<f32>())
                .map(|x| 10f32.powf(x / 20.0))
        };
        let lv = Levels {
            amp: db("--amplitude", -30.0)?,
            thr: db("--threshold", -60.0)?,
        };
        let host = cpal::default_host();
        let find = |mut devs: Box<dyn Iterator<Item = cpal::Device>>, n: &str| {
            devs.find(|d| d.description().is_ok_and(|x| x.name().contains(n)))
        };
        // Devices are resolved again for every mode, so a run survives a plug event between modes.
        let resolve = || -> Result<Endpoints, Box<dyn Error>> {
            let out = match arg("--device") {
                Some(n) => find(Box::new(host.output_devices()?), &n)
                    .ok_or("no output device matches --device")?,
                None => host
                    .default_output_device()
                    .ok_or("no default output device")?,
            };
            let capture = match arg("--capture") {
                Some(n) => Some(
                    find(Box::new(host.input_devices()?), &n)
                        .ok_or("no input device matches --capture")?,
                ),
                None => None,
            };
            Ok(Endpoints { out, capture, lv })
        };
        let ep = resolve()?;
        let name = ep
            .out
            .description()
            .map(|d| d.name().to_owned())
            .unwrap_or_default();
        let cap_name = ep.capture.as_ref().map_or_else(
            || "loopback".to_owned(),
            |d| {
                d.description()
                    .map(|x| x.name().to_owned())
                    .unwrap_or_default()
            },
        );
        let cfg = ep.out.default_output_config()?;
        println!(
            "host {:?}; device {name}; capture {cap_name}; default config {cfg:?}; {trials} trials per mode\n",
            host.id()
        );
        println!(
            "| Mode | Period | Callback frames | Trials ok | Stray | Errors | Loopback xruns | Total p50 | p95 | p99 | max | Wait p50 | Path p50 | Path p95 | Reported p50 | Loopback peak | Max gap | Clock check p50 |"
        );
        println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
        let mut json = format!(
            "{{\n  \"host\": \"{:?}\", \"device\": {name:?}, \"capture\": {cap_name:?}, \"config\": \"{cfg:?}\", \"trials\": {trials},\n  \"runs\": [\n",
            host.id()
        );
        let modes: Vec<&str> = modes.split(',').map(str::trim).collect();
        for (n, m) in modes.iter().enumerate() {
            let sh = Arc::new(Shared {
                trigger: AtomicU64::new(0),
                cb_entry: atomics(trials),
                cb_clock: atomics(trials),
                reported: atomics(trials),
                onsets: atomics(trials * 4),
                onset_count: AtomicU64::new(0),
                frames_min: AtomicU64::new(u64::MAX),
                frames_max: AtomicU64::new(0),
                gap_max: AtomicU64::new(0),
                peak_bits: AtomicU32::new(0),
                errors: AtomicU64::new(0),
                discontinuities: AtomicU64::new(0),
            });
            let run = match resolve().and_then(|ep| measure(&ep, parse_mode(m)?, trials, &sh)) {
                Ok(run) => run,
                Err(e) => {
                    println!("| {m} | failed: {e} |");
                    continue;
                }
            };
            let p = percentile;
            let peak_db = 20.0 * f32::from_bits(sh.peak_bits.load(Ordering::Relaxed)).log10();
            let (fmin, fmax) = (
                sh.frames_min.load(Ordering::Relaxed),
                sh.frames_max.load(Ordering::Relaxed),
            );
            let gap = sh.gap_max.load(Ordering::Relaxed) as f64 / 1e6;
            let errors = sh.errors.load(Ordering::Relaxed);
            let disc = sh.discontinuities.load(Ordering::Relaxed);
            println!(
                "| {m} | {} | {fmin}–{fmax} | {}/{trials} | {} | {errors} | {disc} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {peak_db:.1} dBFS | {gap:.1} | {:.2} |",
                run.period_frames,
                trials - run.missed,
                run.stray,
                p(&run.total, 50.0),
                p(&run.total, 95.0),
                p(&run.total, 99.0),
                p(&run.total, 100.0),
                p(&run.wait, 50.0),
                p(&run.path, 50.0),
                p(&run.path, 95.0),
                p(&run.reported, 50.0),
                p(&run.clock_check, 50.0),
            );
            let list = |v: &[f64]| {
                v.iter()
                    .map(|x| format!("{x:.3}"))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            let _ = write!(
                json,
                "    {{\"mode\": \"{m}\", \"periodFrames\": {}, \"buffer\": {:?}, \"callbackFrames\": [{fmin}, {fmax}], \"maxGapMs\": {gap:.3}, \"loopbackPeakDbfs\": {peak_db:.2}, \"missed\": {}, \"stray\": {}, \"errors\": {errors}, \"loopbackXruns\": {disc},\n     \"totalMs\": [{}],\n     \"waitMs\": [{}],\n     \"pathMs\": [{}],\n     \"reportedMs\": [{}]}}{}\n",
                run.period_frames,
                run.buffer_frames,
                run.missed,
                run.stray,
                list(&run.total),
                list(&run.wait),
                list(&run.path),
                list(&run.reported),
                if n + 1 < modes.len() { "," } else { "" },
            );
            if std::env::var_os("SPIKE_VERBOSE").is_some() {
                println!("  buffer: {}", run.buffer_frames);
            }
        }
        json.push_str("  ]\n}\n");
        if let Some(path) = arg("--out") {
            std::fs::write(path, json)?;
        }
        Ok(())
    }
}
