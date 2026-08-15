//! QUANTUM — Spatial light phase modulation & 2D wave interference holographic iridescence.

use super::ops::rgb_to_rgba;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let wave_freq = 0.05 + p.v(1) * 0.2;
    let phase_shift = p.v(2) * 6.28;
    let iridescence = p.v(3);

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        let yf = y as f32;
        for x in 0..ww {
            let xf = x as f32;
            let i = (y * ww + x) * 3;

            let r = rgb[i] as f32;
            let g = rgb[i + 1] as f32;
            let b = rgb[i + 2] as f32;

            let luma = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;

            // 2D Quantum wave superposition equation
            let wave1 = (xf * wave_freq + yf * wave_freq * 0.7 + phase_shift + state.frame as f32 * 0.04).sin();
            let wave2 = (xf * wave_freq * 1.3 - yf * wave_freq + luma * 10.0).cos();
            let interference = (wave1 + wave2) * 0.5;

            // Iridescent spectral phase mapping (Red/Green/Blue phase separation)
            let pr = ((interference + phase_shift).sin() * 0.5 + 0.5) * 255.0;
            let pg = ((interference + phase_shift + 2.094).sin() * 0.5 + 0.5) * 255.0; // +120 deg
            let pb = ((interference + phase_shift + 4.188).sin() * 0.5 + 0.5) * 255.0; // +240 deg

            let final_r = (r * (1.0 - iridescence * 0.7) + pr * iridescence).clamp(0.0, 255.0);
            let final_g = (g * (1.0 - iridescence * 0.7) + pg * iridescence).clamp(0.0, 255.0);
            let final_b = (b * (1.0 - iridescence * 0.7) + pb * iridescence).clamp(0.0, 255.0);

            out[i] = final_r as u8;
            out[i + 1] = final_g as u8;
            out[i + 2] = final_b as u8;
        }
    }

    rgb_to_rgba(&out, w, h)
}
