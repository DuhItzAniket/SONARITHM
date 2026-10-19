// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spectral helpers for the crate's tests: a radix-2 FFT and amplitude spectra.

/// Single-sided amplitude spectrum with a rectangular window: bin `k` holds the amplitude
/// of a sinusoid centred on that bin. `signal.len()` must be a power of two.
pub(crate) fn spectrum(signal: &[f32]) -> Vec<f64> {
    let samples: Vec<f64> = signal.iter().map(|s| f64::from(*s)).collect();
    amplitudes(&samples, 1.0)
}

/// Single-sided amplitude spectrum with a 4-term Blackman–Harris window (side lobes
/// near −92 dB), scaled by the window's coherent gain.
pub(crate) fn windowed_spectrum(signal: &[f32]) -> Vec<f64> {
    let n = signal.len() as f64;
    let samples: Vec<f64> = signal
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let x = core::f64::consts::TAU * i as f64 / n;
            let w = 0.35875 - 0.48829 * libm::cos(x) + 0.14128 * libm::cos(2.0 * x)
                - 0.01168 * libm::cos(3.0 * x);
            f64::from(*s) * w
        })
        .collect();
    amplitudes(&samples, 0.35875)
}

/// An amplitude in dB re 1.
pub(crate) fn magnitude_db(amplitude: f64) -> f64 {
    20.0 * libm::log10(amplitude.max(1e-30))
}

fn amplitudes(samples: &[f64], coherent_gain: f64) -> Vec<f64> {
    let n = samples.len();
    assert!(n.is_power_of_two(), "FFT length must be a power of two");
    let mut re = samples.to_vec();
    let mut im = vec![0.0; n];
    // Bit-reversal permutation.
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let angle = -core::f64::consts::TAU / len as f64;
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (wr, wi) = (libm::cos(angle * k as f64), libm::sin(angle * k as f64));
                let (a, b) = (start + k, start + k + len / 2);
                let tr = re[b] * wr - im[b] * wi;
                let ti = re[b] * wi + im[b] * wr;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
            }
        }
        len <<= 1;
    }
    let scale = 2.0 / (n as f64 * coherent_gain);
    (0..n / 2)
        .map(|k| libm::hypot(re[k], im[k]) * scale)
        .collect()
}
