// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A direct WASAPI shared-mode renderer, used only to explain the cpal numbers: it can open
//! the stream in RAW mode (no OEM enhancement APOs) and with an `IAudioClient3` engine
//! period, two things cpal 0.18 cannot do (cpal #1173, #1343).

use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::JoinHandle;

use windows::Win32::Media::Audio::{
    AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_EVENTCALLBACK, AUDCLNT_STREAMOPTIONS_NONE,
    AUDCLNT_STREAMOPTIONS_RAW, AudioCategory_Other, AudioClientProperties, IAudioClient3,
    IAudioClock, IAudioRenderClient, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::core::PCWSTR;

/// QPC in nanoseconds on cpal's 100 ns grid, so values compare with cpal `StreamInstant`s.
pub fn qpc_ns() -> u64 {
    let (mut c, mut f) = (0i64, 0i64);
    // SAFETY: both calls only write to the provided integers and cannot fail on Windows XP+.
    unsafe {
        let _ = QueryPerformanceCounter(&raw mut c);
        let _ = QueryPerformanceFrequency(&raw mut f);
    }
    (c as u128 * 10_000_000 / f.max(1) as u128 * 100) as u64
}

/// What the opened stream reports.
#[derive(Debug, Clone)]
pub struct Info {
    pub period_frames: u32,
    pub buffer_frames: u32,
    pub engine_periods: (u32, u32, u32, u32),
}

/// Renders on its own event-driven thread until `stop` is set. `fill(data, callback_ns,
/// reported_ns)` gets the interleaved f32 buffer, the QPC time of the clock snapshot and the
/// predicted `playback − callback` (queued frames plus stream latency), as cpal computes it.
pub fn run<F>(
    device_id: &str,
    raw: bool,
    period: Option<u32>,
    stop: Arc<AtomicBool>,
    mut fill: F,
) -> Result<(JoinHandle<()>, Info), Box<dyn Error>>
where
    F: FnMut(&mut [f32], usize, u64, u64) + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<Result<Info, String>>();
    let id: Vec<u16> = device_id.encode_utf16().chain(Some(0)).collect();
    let handle = std::thread::spawn(move || {
        // SAFETY: plain COM/WASAPI calls on this thread; every pointer comes from the API and
        // the render buffer is used only between GetBuffer and ReleaseBuffer.
        let result = unsafe {
            (|| -> windows::core::Result<()> {
                CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
                let en: IMMDeviceEnumerator =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
                // cpal's WASAPI device id is the IMMDevice endpoint id.
                let client: IAudioClient3 = en
                    .GetDevice(PCWSTR(id.as_ptr()))?
                    .Activate(CLSCTX_ALL, None)?;
                let props = AudioClientProperties {
                    cbSize: size_of::<AudioClientProperties>() as u32,
                    bIsOffload: false.into(),
                    eCategory: AudioCategory_Other,
                    Options: if raw {
                        AUDCLNT_STREAMOPTIONS_RAW
                    } else {
                        AUDCLNT_STREAMOPTIONS_NONE
                    },
                };
                client.SetClientProperties(&raw const props)?;
                let fmt = client.GetMixFormat()?;
                let (channels, sr) = (usize::from((*fmt).nChannels), (*fmt).nSamplesPerSec);
                // The shared-mode mix format is 32-bit float on every current Windows build.
                if (*fmt).wBitsPerSample != 32 {
                    return Err(windows::core::Error::from_hresult(windows::core::HRESULT(
                        -1,
                    )));
                }
                let (mut def, mut fund, mut min, mut max) = (0, 0, 0, 0);
                client.GetSharedModeEnginePeriod(
                    fmt,
                    &raw mut def,
                    &raw mut fund,
                    &raw mut min,
                    &raw mut max,
                )?;
                match period {
                    Some(p) => {
                        let p = (p.clamp(min, max) / fund.max(1)) * fund.max(1);
                        client.InitializeSharedAudioStream(
                            AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                            p,
                            fmt,
                            None,
                        )?;
                    }
                    None => client.Initialize(
                        AUDCLNT_SHAREMODE_SHARED,
                        AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                        0,
                        0,
                        fmt,
                        None,
                    )?,
                }
                let event = CreateEventW(None, false, false, None)?;
                client.SetEventHandle(event)?;
                let render: IAudioRenderClient = client.GetService()?;
                let clock: IAudioClock = client.GetService()?;
                let buffer_frames = client.GetBufferSize()?;
                let latency_ns = client.GetStreamLatency()?.max(0) as u64 * 100;
                let (mut cur_fmt, mut cur) = (std::ptr::null_mut(), 0u32);
                client.GetCurrentSharedModeEnginePeriod(&raw mut cur_fmt, &raw mut cur)?;
                let _ = tx.send(Ok(Info {
                    period_frames: cur,
                    buffer_frames,
                    engine_periods: (def, fund, min, max),
                }));
                client.Start()?;
                while !stop.load(Ordering::Acquire) {
                    let _ = WaitForSingleObject(event, 200);
                    let pad = client.GetCurrentPadding()?;
                    let avail = buffer_frames - pad;
                    if avail == 0 {
                        continue;
                    }
                    let (mut pos, mut qpc) = (0u64, 0u64);
                    clock.GetPosition(&raw mut pos, Some(&raw mut qpc))?;
                    let reported = u64::from(pad) * 1_000_000_000 / u64::from(sr) + latency_ns;
                    let ptr = render.GetBuffer(avail)?.cast::<f32>();
                    let data = std::slice::from_raw_parts_mut(ptr, avail as usize * channels);
                    fill(data, channels, qpc * 100, reported);
                    render.ReleaseBuffer(avail, 0)?;
                }
                client.Stop()?;
                Ok(())
            })()
        };
        if let Err(e) = result {
            let _ = tx.send(Err(e.to_string()));
        }
    });
    let info = rx
        .recv()?
        .map_err(|e| format!("direct WASAPI stream: {e}"))?;
    Ok((handle, info))
}
