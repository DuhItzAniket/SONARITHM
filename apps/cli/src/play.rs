// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `sonarithm play` and `sonarithm devices`: real-time playback through A05's player, and
//! the output devices (11 §2).
//!
//! Playback announces every change as one text line, so a screen reader reads it as it
//! happens: what is playing and for how long, pauses with their position, device changes,
//! and the end. When standard input is a terminal, keys control the transport: space
//! pauses and resumes, Left and Right step one second (data-point steps arrive with
//! AUDAXIS navigation in B03), Home returns to the start, and q, Escape or Ctrl+C stop.
//! Otherwise the Score plays to its end. The player starts 20 dB down and rises over
//! 500 ms, and the master limiter holds −1 dBTP (06 §10).

use std::io::{IsTerminal as _, Write as _};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use serde::Serialize;
use sonarithm::Compiled;
use sonarithm::units::{Frames, SampleRate, Seconds};
use sonarithm_audio::{
    AudioBackend, CpalBackend, DeviceInfo, NullBackend, NullConfig, Player, PlayerEvent,
    StreamConfig,
};
use sonarithm_render::PlanConfig;
use sonarithm_render::graph::Command;

use crate::cli::BackendArg;

/// How often the control loop runs: flushes commands, checks the device, reads keys.
const TICK: Duration = Duration::from_millis(10);

/// One second, the step of Left and Right.
const STEP: f64 = 1.0;

/// What playback did, for `--json`.
#[derive(Debug, Default, Serialize)]
pub(crate) struct Played {
    pub(crate) device: String,
    pub(crate) duration_s: f64,
    pub(crate) stopped_at_s: f64,
    pub(crate) finished: bool,
    pub(crate) xruns: u64,
    /// Every line playback announced, in order.
    pub(crate) events: Vec<String>,
}

/// Prints announcements: plain lines, or `\r\n`-terminated ones while the terminal is in
/// raw mode; nothing with `--json` (they are collected instead).
struct Announcer {
    raw: bool,
    quiet: bool,
    lines: Vec<String>,
}

impl Announcer {
    fn say(&mut self, text: String) {
        if !self.quiet {
            let mut stdout = std::io::stdout().lock();
            let end = if self.raw { "\r\n" } else { "\n" };
            let _ = write!(stdout, "{text}{end}");
            let _ = stdout.flush();
        }
        self.lines.push(text);
    }
}

/// Raw keyboard mode for the life of the guard.
struct RawMode;

impl RawMode {
    fn enter() -> Option<Self> {
        crossterm::terminal::enable_raw_mode().ok().map(|()| Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

/// A key the player acts on.
#[derive(Clone, Copy)]
enum Key {
    Toggle,
    Back,
    Forward,
    Home,
    Quit,
}

fn key(event: &KeyEvent) -> Option<Key> {
    if event.kind != KeyEventKind::Press {
        return None;
    }
    match event.code {
        KeyCode::Char(' ') => Some(Key::Toggle),
        KeyCode::Left => Some(Key::Back),
        KeyCode::Right => Some(Key::Forward),
        KeyCode::Home => Some(Key::Home),
        KeyCode::Char('q' | 'Q') | KeyCode::Esc => Some(Key::Quit),
        KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => Some(Key::Quit),
        _ => None,
    }
}

/// Settings for one playback.
pub(crate) struct Playback<'a> {
    pub(crate) name: &'a str,
    pub(crate) device: Option<&'a str>,
    pub(crate) speed: Option<f64>,
    pub(crate) json: bool,
}

/// Plays `compiled` on the chosen backend until it ends or the listener stops it.
pub(crate) fn play(
    compiled: &Compiled,
    backend: BackendArg,
    settings: &Playback<'_>,
) -> Result<Played, String> {
    match backend {
        BackendArg::Default => run(CpalBackend::new(), compiled, settings),
        BackendArg::Null => run(
            NullBackend::new(NullConfig::new(48_000, 256)),
            compiled,
            settings,
        ),
    }
}

fn seconds(frames: Frames, rate: u32) -> f64 {
    #[expect(clippy::cast_precision_loss, reason = "frame counts far below 2^52")]
    let seconds = frames.0 as f64 / f64::from(rate);
    seconds
}

fn rate_of(hz: u32) -> SampleRate {
    SampleRate::new(hz).unwrap_or(SampleRate::HZ_48000)
}

/// Opens a player for `compiled` on the device the settings name, or the default.
fn open<B: AudioBackend>(
    backend: B,
    compiled: &Compiled,
    settings: &Playback<'_>,
) -> Result<(Player<B>, String, u32), String> {
    let mut config = StreamConfig::default();
    if let Some(wanted) = settings.device {
        let devices = backend.devices().map_err(|err| err.to_string())?;
        let found = devices
            .iter()
            .find(|d| d.id == wanted || d.name.eq_ignore_ascii_case(wanted))
            .ok_or_else(|| {
                format!("no output device is called `{wanted}`; `sonarithm devices` lists them")
            })?;
        config.device = Some(found.id.clone());
    }
    // The player plans the Score again at the device's rate.
    let plan = PlanConfig::new(SampleRate::HZ_48000);
    let player = Player::open(backend, compiled.score().clone(), plan, config, 256)
        .map_err(|err| err.to_string())?;
    let (device, rate) = player
        .stream()
        .map(|stream| (stream.device().to_owned(), stream.sample_rate()))
        .ok_or_else(|| "the audio stream did not open".to_owned())?;
    Ok((player, device, rate))
}

/// The transport as the loop sees it.
struct Transport {
    playing: bool,
    duration: f64,
    rate: u32,
}

impl Transport {
    /// Acts on a key; `false` when the listener stopped playback.
    fn press<B: AudioBackend>(
        &mut self,
        pressed: Key,
        position: f64,
        player: &mut Player<B>,
        say: &mut Announcer,
    ) -> bool {
        match pressed {
            Key::Toggle if self.playing => {
                player.send(Command::Pause);
                self.playing = false;
                say.say(format!("Paused at {position:.2} seconds."));
            }
            Key::Toggle => {
                player.send(Command::Play);
                self.playing = true;
                say.say(format!("Playing from {position:.2} seconds."));
            }
            Key::Back | Key::Forward | Key::Home => {
                let target = match pressed {
                    Key::Back => (position - STEP).max(0.0),
                    Key::Forward => (position + STEP).min(self.duration),
                    _ => 0.0,
                };
                if let Some(frame) = Seconds(target).to_frames(rate_of(self.rate)) {
                    player.send(Command::Seek(frame));
                }
                say.say(format!("At {target:.2} seconds."));
            }
            Key::Quit => {
                player.send(Command::StopAll);
                let _ = player.tick();
                say.say(format!("Stopped at {position:.2} seconds."));
                return false;
            }
        }
        true
    }
}

/// The next key, waiting at most one tick; with no keyboard, just waits the tick.
fn next_key(keyboard: bool) -> Option<Key> {
    if !keyboard {
        std::thread::sleep(TICK);
        return None;
    }
    if !event::poll(TICK).unwrap_or(false) {
        return None;
    }
    match event::read() {
        Ok(Event::Key(event)) => key(&event),
        _ => None,
    }
}

fn run<B: AudioBackend>(
    backend: B,
    compiled: &Compiled,
    settings: &Playback<'_>,
) -> Result<Played, String> {
    let (mut player, device, rate) = open(backend, compiled, settings)?;
    let duration = compiled.score().header.duration.0;
    let end = Seconds(duration)
        .to_frames(rate_of(rate))
        .unwrap_or(Frames(0));
    let raw = if !settings.json && std::io::stdin().is_terminal() {
        RawMode::enter()
    } else {
        None
    };
    let mut say = Announcer {
        raw: raw.is_some(),
        quiet: settings.json,
        lines: Vec::new(),
    };
    let keys = if raw.is_some() {
        " Space pauses and resumes, Left and Right step one second, Home returns to the start, q stops."
    } else {
        ""
    };
    say.say(format!(
        "Playing {}, {duration:.2} seconds, on {device}.{keys}",
        settings.name
    ));
    if let Some(speed) = settings.speed {
        player.send(Command::SetSpeed(speed));
    }
    player.send(Command::Play);
    let mut transport = Transport {
        playing: true,
        duration,
        rate,
    };
    let mut finished = false;
    loop {
        match player.tick() {
            Ok(Some(PlayerEvent::DeviceChanged { device })) => {
                transport.playing = false;
                say.say(format!(
                    "Audio device changed to {device}; playback is paused. Press space to resume."
                ));
            }
            Ok(_) => {}
            Err(err) => {
                say.say(format!("Playback stopped: {err}."));
                break;
            }
        }
        let position = player.state().graph.position;
        if position >= end {
            player.send(Command::Stop);
            let _ = player.tick();
            finished = true;
            say.say(format!("Finished at {duration:.2} seconds."));
            break;
        }
        if let Some(pressed) = next_key(raw.is_some())
            && !transport.press(pressed, seconds(position, rate), &mut player, &mut say)
        {
            break;
        }
    }
    drop(raw);
    let state = player.state();
    Ok(Played {
        device,
        duration_s: duration,
        stopped_at_s: if finished {
            duration
        } else {
            seconds(state.graph.position, rate)
        },
        finished,
        xruns: state.xruns,
        events: say.lines,
    })
}

/// The output devices of a backend.
pub(crate) fn devices(backend: BackendArg) -> Result<Vec<DeviceInfo>, String> {
    let result = match backend {
        BackendArg::Default => CpalBackend::new().devices(),
        BackendArg::Null => NullBackend::new(NullConfig::new(48_000, 256)).devices(),
    };
    result.map_err(|err| err.to_string())
}
