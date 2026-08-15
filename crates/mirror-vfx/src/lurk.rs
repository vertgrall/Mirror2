//! LURK — ghost images of the subject linger and drift at the edges of vision.

use super::ops::{lum, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let count = (p.v(1) * 4.0 + 1.0).round() as usize;
    let drift = p.v(2) * 28.0;
    let fade = p.v(3) * 0.85 + 0.05;

    let depths = [4usize, 10, 18, 28, 38];
    let mut out = vec![0u8; ww * hh * 4];

    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let mut r = rgb[i] as f32;
            let mut g = rgb[i + 1] as f32;
            let mut b = rgb[i + 2] as f32;

            for layer in 0..count.min(depths.len()) {
                let depth = depths[layer];
                let Some(past) = state.get_ring(depth) else {
                    continue;
                };
                if past.len() != rgb.len() {
                    continue;
                }

                let phase = state.frame as f32 * 0.03 + layer as f32 * 1.7;
                let ox = phase.sin() * drift * (layer as f32 * 0.35 + 0.5);
                let oy = phase.cos() * drift * (layer as f32 * 0.4 + 0.45);

                let sx = (x as f32 + ox).clamp(0.0, w as f32 - 1.001);
                let sy = (y as f32 + oy).clamp(0.0, h as f32 - 1.001);
                let (gr, gg, gb) = sample_rgb(past, w, h, sx, sy);

                let l = lum(gr, gg, gb);
                let subject = (l - 0.12).max(0.0) / 0.88;
                let alpha = fade * (1.0 - layer as f32 / count as f32) * subject * 0.72;

                r = r * (1.0 - alpha) + gr as f32 * alpha;
                g = g * (1.0 - alpha) + gg as f32 * alpha;
                b = b * (1.0 - alpha) + gb as f32 * alpha;
            }

            let o = (y * ww + x) * 4;
            out[o] = r as u8;
            out[o + 1] = g as u8;
            out[o + 2] = b as u8;
            out[o + 3] = 255;
        }
    }

    out
}
