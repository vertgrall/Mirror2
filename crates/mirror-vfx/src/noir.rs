//! NOIR — 40s Tri-X silver monochrome. Deep crushed blacks, glowing highlights, vignette.

use super::ops::{hash2d, lum, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let contrast = p.v(1);
    let grain = p.v(2);
    let vignette = p.v(3);

    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.48;

    let mut out = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        let yf = y as f32;
        let ny = (yf - cy) / cy;
        for x in 0..ww {
            let xf = x as f32;
            let nx = (xf - cx) / cx;
            let i = (y * ww + x) * 3;

            let l = lum(rgb[i], rgb[i + 1], rgb[i + 2]);

            // High contrast sigmoid-style curve
            let c = 1.0 + contrast * 3.5;
            let mut v = ((l - 0.5) * c + 0.5).clamp(0.0, 1.0);

            // Silver highlight bloom
            if v > 0.75 {
                v = (v + (v - 0.75) * 0.4).min(1.0);
            }

            // Silver halide grain
            if grain > 0.01 {
                let n = hash2d(xf * 2.3 + state.frame as f32 * 0.5, yf * 2.9) - 0.5;
                v = (v + n * grain * 0.22).clamp(0.0, 1.0);
            }

            // Dramatic lens spotlight vignette
            if vignette > 0.01 {
                let r2 = nx * nx + ny * ny;
                let vig = (r2 * 0.85 * vignette).min(0.92);
                v = (v * (1.0 - vig)).max(0.0);
            }

            let byte_v = (v * 255.0) as u8;
            out[i] = byte_v;
            out[i + 1] = byte_v;
            out[i + 2] = byte_v;
        }
    }
    rgb_to_rgba(&out, w, h)
}
