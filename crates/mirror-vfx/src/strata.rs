//! STRATA — Recursive homography video feedback infinity tunnel with luma threshold light trails.

use super::ops::{rgb_to_rgba, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let zoom = 1.0 + p.v(1) * 0.08;
    let rot = (p.v(2) - 0.5) * 0.1;
    let decay = p.v(3).clamp(0.1, 0.98);

    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;

    let mut out = vec![0u8; ww * hh * 3];

    let cos_r = rot.cos();
    let sin_r = rot.sin();

    for y in 0..hh {
        let yf = y as f32 - cy;
        for x in 0..ww {
            let xf = x as f32 - cx;
            let i = (y * ww + x) * 3;

            // Affine inverse transform coordinate for feedback loop
            let rx = (xf * cos_r - yf * sin_r) / zoom + cx;
            let ry = (xf * sin_r + yf * cos_r) / zoom + cy;

            let (prev_r, prev_g, prev_b) = if rx >= 0.0 && rx < w as f32 && ry >= 0.0 && ry < h as f32 {
                if let Some(prev) = state.prev_rgb() {
                    if prev.len() == rgb.len() {
                        sample_rgb(prev, w, h, rx, ry)
                    } else {
                        (0, 0, 0)
                    }
                } else {
                    (0, 0, 0)
                }
            } else {
                (0, 0, 0)
            };

            let cur_r = rgb[i] as f32;
            let cur_g = rgb[i + 1] as f32;
            let cur_b = rgb[i + 2] as f32;

            // Composite live input with transformed decay feedback frame
            let blended_r = (cur_r * (1.0 - decay * 0.5) + (prev_r as f32) * decay).min(255.0);
            let blended_g = (cur_g * (1.0 - decay * 0.5) + (prev_g as f32) * decay).min(255.0);
            let blended_b = (cur_b * (1.0 - decay * 0.5) + (prev_b as f32) * decay).min(255.0);

            out[i] = blended_r as u8;
            out[i + 1] = blended_g as u8;
            out[i + 2] = blended_b as u8;
        }
    }

    rgb_to_rgba(&out, w, h)
}
