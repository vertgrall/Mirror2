//! BREATHE — whole-frame inhale/exhale. Uniform scale from center.

use super::ops::{lerp_u8, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

fn breath_ease(raw: f32, hold: f32) -> f32 {
    if hold < 0.02 {
        return raw;
    }
    let flat = hold * 0.88;
    if raw.abs() < flat {
        raw.signum() * flat
    } else {
        raw
    }
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let depth = p.v(1);
    let pace = p.v(2);
    let hold = p.v(3);

    let phase = state.frame as f32 * (0.018 + pace * 0.055);
    let eased = breath_ease(phase.sin(), hold);
    let scale = 1.0 + depth * 0.13 * eased;

    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.48;

    let mut out = vec![0u8; ww * hh * 4];
    for y in 0..hh {
        for x in 0..ww {
            let xf = x as f32;
            let yf = y as f32;
            let sx = ((xf - cx) / scale + cx).clamp(0.0, w as f32 - 1.001);
            let sy = ((yf - cy) / scale + cy).clamp(0.0, h as f32 - 1.001);

            let (mut r, mut g, mut b) = sample_rgb(rgb, w, h, sx, sy);

            let inhale = eased.max(0.0);
            let vig = depth * 0.11 * inhale;
            r = lerp_u8(r, 0, vig);
            g = lerp_u8(g, 0, vig);
            b = lerp_u8(b, 0, vig * 0.8);

            let o = (y * ww + x) * 4;
            out[o] = r;
            out[o + 1] = g;
            out[o + 2] = b;
            out[o + 3] = 255;
        }
    }
    out
}
