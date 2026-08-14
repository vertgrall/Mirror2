//! DRIFT — slow undulating random LFO waveform distortion across the screen.

use super::ops::{sample_rgb, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let size = p.v(1); // wave distortion amplitude
    let raw_speed = p.v(2); // LFO speed
    let freq = p.v(3); // spatial wavelength frequency

    // Cap max speed so it remains a very slow, subtle, hypnotic drift
    let slow_speed = raw_speed * 0.015 + 0.003;
    let lfo_time = state.frame as f32 * slow_speed;

    let amplitude = size * 28.0;
    let spatial_freq = 0.008 + freq * 0.025;

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        let yf = y as f32;
        for x in 0..ww {
            let xf = x as f32;
            let i = (y * ww + x) * 3;

            // Dual harmonic slow LFO wave formulas
            let wave_x = (yf * spatial_freq + lfo_time).sin() * amplitude
                + (yf * spatial_freq * 2.1 + lfo_time * 0.7).cos() * (amplitude * 0.4);

            let wave_y = (xf * spatial_freq * 0.8 + lfo_time * 1.2).cos() * (amplitude * 0.7)
                + (xf * spatial_freq * 1.7 - lfo_time * 0.5).sin() * (amplitude * 0.3);

            let sx = (xf + wave_x).clamp(0.0, w as f32 - 1.001);
            let sy = (yf + wave_y).clamp(0.0, h as f32 - 1.001);

            let (r, g, b) = sample_rgb(rgb, w, h, sx, sy);

            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
        }
    }
    rgb_to_rgba(&out, w, h)
}
