// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A02-T02/T03/T04 spike: render graphs behind a draft `RenderGraph` trait, driven offline
//! and from a cpal stream. Usage and results: `README.md`. Throwaway code.

mod determinism;
mod fw;
mod graph;
mod own;
mod score;
mod voice;

use std::error::Error;
#[cfg(feature = "realtime")]
use std::sync::Arc;
#[cfg(feature = "realtime")]
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[cfg(feature = "realtime")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use sha2::{Digest, Sha256};

use crate::graph::{RenderConfig, RenderGraph};
use crate::voice::MathMode;

// Debug builds abort on any allocation inside `assert_no_alloc` (06 §12). Building with
// `--cfg spike_miri_system_alloc` drops this allocator, to tell Miri's findings apart (README).
#[cfg(all(debug_assertions, not(spike_miri_system_alloc)))]
#[global_allocator]
static ALLOC: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;

const SEED: u64 = 1729;
pub(crate) const SAMPLE_RATE: u32 = 48_000;

fn make_graph(name: &str) -> Result<Box<dyn RenderGraph>, Box<dyn Error>> {
    match name {
        "own" => Ok(Box::new(own::OwnGraph::new())),
        "firewheel" => Ok(Box::new(fw::FwGraph::new())),
        _ => Err(format!("unknown graph {name}").into()),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn percentile(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    let rank = ((p / 100.0) * v.len() as f64).ceil() as usize;
    v.get(rank.clamp(1, v.len().max(1)) - 1)
        .copied()
        .unwrap_or(f64::NAN)
}

/// Integrated loudness (EBU R128) and sample peak of interleaved stereo.
fn loudness(samples: &[f32]) -> Result<(f64, f32), Box<dyn Error>> {
    let mut m = ebur128::EbuR128::new(2, SAMPLE_RATE, ebur128::Mode::I)?;
    m.add_frames_f32(samples)?;
    Ok((
        m.loudness_global()?,
        samples.iter().fold(0.0f32, |a, x| a.max(x.abs())),
    ))
}

/// One offline render of the benchmark Score and what was measured on the way.
pub(crate) struct Rendered {
    pub(crate) samples: Vec<f32>,
    pub(crate) lufs: f64,
    pub(crate) audio_hash: String,
    pub(crate) score_hash: String,
    line: String,
}

pub(crate) fn render_offline(
    graph_name: &str,
    block: usize,
    seconds: u32,
    math: MathMode,
) -> Result<Rendered, Box<dyn Error>> {
    let score = score::benchmark(SAMPLE_RATE, seconds, SEED);
    let cfg = RenderConfig {
        sample_rate: SAMPLE_RATE,
        max_block: block,
        math,
    };
    let mut graph = make_graph(graph_name)?;
    let mut proc = graph.prepare(&score, &cfg)?;
    let blocks = (score.frames as usize).div_ceil(block);
    let mut out = Vec::with_capacity(blocks * block * 2);
    let mut buf = vec![0.0f32; block * 2];
    let mut times = Vec::with_capacity(blocks);
    let mut control = Duration::ZERO;
    for _ in 0..blocks {
        // The control tick runs before every block, as the control thread would.
        let c = Instant::now();
        graph.update();
        control += c.elapsed();
        let t = Instant::now();
        assert_no_alloc::assert_no_alloc(|| proc.process(&mut buf));
        times.push(t.elapsed().as_secs_f64() * 1e6);
        out.extend_from_slice(&buf);
    }
    out.truncate(score.frames as usize * 2);
    let busy: f64 = times.iter().sum::<f64>() / 1e6;
    let (lufs, peak) = loudness(&out)?;
    let audio_hash = hex(&Sha256::digest(
        out.iter()
            .flat_map(|s| s.to_le_bytes())
            .collect::<Vec<u8>>(),
    ));
    let score_hash = hex(&Sha256::digest(score::canonical_bytes(&score)));
    let st = graph.stats();
    let line = format!(
        "graph={} block={block} math={math:?} seconds={seconds} rtf={:.0} control_ms={:.1} block_us_p50={:.1} p99={:.1} max={:.1} voices={} max_active={} dispatched={} stolen={} lufs={lufs:.2} peak={peak:.4} audio_sha256={audio_hash} score_sha256={score_hash}",
        graph.name(),
        f64::from(seconds) / busy,
        control.as_secs_f64() * 1e3,
        percentile(&mut times.clone(), 50.0),
        percentile(&mut times.clone(), 99.0),
        percentile(&mut times, 100.0),
        st.voices,
        st.max_active,
        st.dispatched,
        st.stolen,
    );
    Ok(Rendered {
        samples: out,
        lufs,
        audio_hash,
        score_hash,
        line,
    })
}

/// Writes interleaved stereo as 32-bit float WAV.
pub(crate) fn write_wav(path: &std::path::Path, samples: &[f32]) -> Result<(), Box<dyn Error>> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)?;
    for s in samples {
        w.write_sample(*s)?;
    }
    w.finalize()?;
    Ok(())
}

fn offline(args: &Args) -> Result<(), Box<dyn Error>> {
    let r = render_offline(&args.graph, args.block, args.seconds, args.math)?;
    println!("{}", r.line);
    if let Some(path) = &args.wav {
        write_wav(std::path::Path::new(path), &r.samples)?;
    }
    Ok(())
}

#[cfg(feature = "realtime")]
/// Plays the benchmark Score through the default output for `seconds` and records the CPU
/// time of every callback. Output is muted unless `--audible` (the graph still renders).
fn realtime(args: &Args) -> Result<(), Box<dyn Error>> {
    // Always the 60 s benchmark Score; `--seconds` is the soak length, and the Score loops.
    let score = score::benchmark(SAMPLE_RATE, 60, SEED);
    // WASAPI shared mode delivers 480-frame periods and a first fill of up to 1056 frames.
    let cfg = RenderConfig {
        sample_rate: SAMPLE_RATE,
        max_block: 4096,
        math: args.math,
    };
    let mut graph = make_graph(&args.graph)?;
    let mut proc = graph.prepare(&score, &cfg)?;
    let name = graph.name();
    let device = cpal::default_host()
        .default_output_device()
        .ok_or("no output device")?;
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        buffer_size: cpal::BufferSize::Fixed(args.block as u32),
    };
    let cap = (args.seconds as usize + 2) * 1000;
    let busy: Arc<Vec<AtomicU64>> = Arc::new((0..cap).map(|_| AtomicU64::new(0)).collect());
    let count = Arc::new(AtomicU64::new(0));
    let (misses, gaps, errors) = (
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
    );
    let (b, c, m, g, e) = (
        Arc::clone(&busy),
        Arc::clone(&count),
        Arc::clone(&misses),
        Arc::clone(&gaps),
        Arc::clone(&errors),
    );
    let audible = args.audible;
    let mut last: Option<Instant> = None;
    let stream = device.build_output_stream::<f32, _, _>(
        config,
        move |data, _| {
            let start = Instant::now();
            proc.process(data);
            if !audible {
                data.fill(0.0);
            }
            let spent = start.elapsed();
            let period = Duration::from_secs_f64((data.len() / 2) as f64 / f64::from(SAMPLE_RATE));
            if spent > period {
                m.fetch_add(1, Ordering::Relaxed);
            }
            if last.is_some_and(|l| start - l > period + period / 2) {
                g.fetch_add(1, Ordering::Relaxed);
            }
            last = Some(start);
            let i = c.fetch_add(1, Ordering::Relaxed) as usize;
            if let Some(slot) = b.get(i) {
                slot.store(spent.as_nanos() as u64, Ordering::Relaxed);
            }
        },
        move |_| {
            e.fetch_add(1, Ordering::Relaxed);
        },
        None,
    )?;
    stream.play()?;
    let period = stream.buffer_size()?;
    // The control thread: a 5 ms tick, as 06 §2 plans for transport and live events. The
    // Score loops: at its end the control thread seeks back to 0.
    let end = Instant::now() + Duration::from_secs(u64::from(args.seconds));
    let mut loops = 0u32;
    let mut loop_start = 0u64;
    while Instant::now() < end {
        let rendered = graph.counters().rendered.load(Ordering::Relaxed);
        if rendered - loop_start >= score.frames {
            graph.seek(0);
            loop_start = rendered;
            loops += 1;
        }
        graph.update();
        std::thread::sleep(Duration::from_millis(5));
    }
    drop(stream);
    let st = graph.stats();
    let n = (count.load(Ordering::Relaxed) as usize).min(cap);
    let mut us: Vec<f64> = busy[..n]
        .iter()
        .map(|x| x.load(Ordering::Relaxed) as f64 / 1e3)
        .collect();
    let period_us = f64::from(period) / f64::from(SAMPLE_RATE) * 1e6;
    let p99 = percentile(&mut us, 99.0);
    println!(
        "graph={name} requested={} period={period} callbacks={n} cpu_us_p50={:.1} p99={p99:.1} max={:.1} load_p99={:.2}% deadline_misses={} late_callbacks={} stream_errors={} dispatched={} stolen={} loops={loops}",
        args.block,
        percentile(&mut us, 50.0),
        percentile(&mut us, 100.0),
        p99 / period_us * 100.0,
        misses.load(Ordering::Relaxed),
        gaps.load(Ordering::Relaxed),
        errors.load(Ordering::Relaxed),
        st.dispatched,
        st.stolen,
    );
    Ok(())
}

/// Null test of two WAV files: peak difference in dBFS and the first differing frame.
fn null(paths: &[String]) -> Result<(), Box<dyn Error>> {
    let read = |p: &String| -> Result<Vec<f32>, Box<dyn Error>> {
        Ok(hound::WavReader::open(p)?
            .samples::<f32>()
            .collect::<Result<_, _>>()?)
    };
    let (a, b) = (read(&paths[0])?, read(&paths[1])?);
    let peak = a
        .iter()
        .zip(&b)
        .fold(0.0f32, |m, (x, y)| m.max((x - y).abs()));
    let first = a.iter().zip(&b).position(|(x, y)| x != y).map(|i| i / 2);
    println!(
        "samples={}/{} peak_diff={peak:e} residual_dbfs={:.1} first_diff_frame={first:?}",
        a.len(),
        b.len(),
        20.0 * peak.log10()
    );
    Ok(())
}

struct Args {
    command: String,
    graph: String,
    block: usize,
    seconds: u32,
    math: MathMode,
    wav: Option<String>,
    #[cfg_attr(
        not(feature = "realtime"),
        expect(dead_code, reason = "read by `realtime`")
    )]
    audible: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let raw: Vec<String> = std::env::args().collect();
    let arg = |name: &str| {
        raw.iter()
            .position(|a| a == name)
            .and_then(|i| raw.get(i + 1))
            .cloned()
    };
    let args = Args {
        command: raw.get(1).cloned().unwrap_or_default(),
        graph: arg("--graph").unwrap_or_else(|| "own".into()),
        block: arg("--block").map_or(Ok(256), |s| s.parse())?,
        seconds: arg("--seconds").map_or(Ok(60), |s| s.parse())?,
        math: if arg("--math").as_deref() == Some("std") {
            MathMode::Std
        } else {
            MathMode::Poly
        },
        wav: arg("--wav"),
        audible: raw.iter().any(|a| a == "--audible"),
    };
    match args.command.as_str() {
        "offline" => offline(&args),
        #[cfg(feature = "realtime")]
        "realtime" => realtime(&args),
        "render-set" => determinism::render_set(std::path::Path::new(&arg("--out").unwrap_or_else(|| "renders".into())), args.seconds),
        "compare" => determinism::compare(&raw[2..].iter().map(std::path::PathBuf::from).collect::<Vec<_>>()),
        "null" => null(&raw[2..]),
        // Proves the checker is live: a debug build must abort here.
        "selftest-alloc" => {
            let v = assert_no_alloc::assert_no_alloc(|| vec![0u8; 16]);
            Err(format!("allocated {} bytes without aborting (release build?)", v.len()).into())
        }
        _ => Err("usage: spike-render-graph offline|realtime|render-set|compare|null|selftest-alloc (see README.md)".into()),
    }
}
