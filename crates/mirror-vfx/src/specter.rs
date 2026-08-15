//! SPECTER — hue-shifted ghost copies shear across the frame like a séance double exposure.

use super::ops::{sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

fn rotate_hue(r: f32, g: f32, b: f32, angle: f32) -> (f32, f32, f32) {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let nr = r * (0.667 + cos_a * 0.333) + g * (0.667 - cos_a * 0.333 + sin_a * 0.577) + b * (0.667 - cos_a * 0.333 - sin_a * 0.577);
    let ng = r * (0.667 - cos_a * 0.333 - sin_a * 0.577) + g * (0.667 + cos_a * 0.333) + b * (0.667 - cos_a * 0.333 + sin_a * 0.577);
    let nb = r * (0.667 - cos_a * 0.333 + sin_a * 0.577) + g * (0.667 - cos_a * 0.333 - sin_a * 0.577) + b * (0.667 + cos_a * 0.333);
    (nr, ng, nb)
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let ghosts = (p.v(1) * 3.0 + 1.0).round() as usize;
    let hue = p.v(2) * 6.28;
    let shear = p.v(3) * 48.0;

    let depths = [6usize, 14, 24, 34];
    let mut out = vec![0u8; ww * hh * 4];

    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let mut r = rgb[i] as f32;
            let mut g = rgb[i + 1] as f32;
            let mut b = rgb[i + 2] as f32;

            for layer in 0..ghosts.min(depths.len()) {
                let depth = depths[layer];
                let Some(past) = state.get_ring(depth) else {
                    continue;
                };
                if past.len() != rgb.len() {
                    continue;
                }

                let yf = y as f32 / h as f32;
                let shear_off = (yf - 0.5) * shear * (layer as f32 + 1.0);
                let drift = (state.frame as f32 * 0.02 + layer as f32).sin() * 12.0;

                let sx = (x as f32 + shear_off + drift).clamp(0.0, w as f32 - 1.001);
                let sy = (y as f32 - layer as f32 * 3.0).clamp(0.0, h as f32 - 1.001);
                let (gr, gg, gb) = sample_rgb(past, w, h, sx, sy);

                let angle = hue * (layer as f32 + 1.0);
                let (hr, hg, hb) = rotate_hue(gr as f32, gg as f32, gb as f32, angle);
                let alpha = 0.55 / (layer as f32 + 1.2);

                r = r * (1.0 - alpha) + hr * alpha;
                g = g * (1.0 - alpha) + hg * alpha;
                b = b * (1.0 - alpha) + hb * alpha;
            }

            let o = (y * ww + x) * 4;
            out[o] = r.clamp(0.0, 255.0) as u8;
            out[o + 1] = g.clamp(0.0, 255.0) as u8;
            out[o + 2] = b.clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;
        }
    }
    out
}
