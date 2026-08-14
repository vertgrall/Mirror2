//! SAT — dish feed. Macro rain fade & satellite color/macro block pixelation.

use super::ops::{hash2d, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

const BLOCK: u32 = 12;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let rain = p.v(1);
    let block = p.v(2);
    let sat = p.v(3);

    let mut out = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        let rain_band = hash2d(0.0, y as f32 * 0.04 + state.frame as f32 * 0.2);
        let rain_dim = if rain_band > 1.0 - rain * 0.25 {
            0.35 + rain * 0.25
        } else {
            1.0
        };

        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let bx = (x as u32 / BLOCK) * BLOCK + BLOCK / 2;
            let by = (y as u32 / BLOCK) * BLOCK + BLOCK / 2;
            let bx = bx.min(w - 1) as usize;
            let by = by.min(h - 1) as usize;
            let si = (by * ww + bx) * 3;

            let mut r = rgb[si] as f32;
            let mut g = rgb[si + 1] as f32;
            let mut b = rgb[si + 2] as f32;

            let q = 1.0 + block * 20.0;
            r = (r / q).round() * q;
            g = (g / q).round() * q;
            b = (b / q).round() * q;

            r = (r * (1.0 + sat * 0.35)).min(255.0);
            g = (g * (1.0 + sat * 0.15)).min(255.0);
            b = (b * (1.0 - sat * 0.08)).max(0.0);

            r *= rain_dim;
            g *= rain_dim;
            b *= rain_dim;

            out[i] = r as u8;
            out[i + 1] = g as u8;
            out[i + 2] = b as u8;
        }
    }
    rgb_to_rgba(&out, w, h)
}
