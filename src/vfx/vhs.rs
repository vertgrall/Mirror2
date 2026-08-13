//! VHS — chroma bleed, line jitter, dropout, temporal ghost.

use super::ops::{hash2d, lerp_u8, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let tracking = p.v(1);
    let chroma = p.v(2);
    let wear = p.v(3);
    let ghost = wear * 0.45 + tracking * 0.15;

    let mut out = vec![0u8; ww * hh * 3];
    let prev = state.prev_rgb();

    for y in 0..hh {
        let jitter = ((y as f32 * 0.17 + state.frame as f32 * 0.31).sin() * tracking * 10.0) as i32;
        for x in 0..ww {
            let sx = (x as i32 + jitter).clamp(0, ww as i32 - 1) as usize;
            let i = (y * ww + x) * 3;
            let si = (y * ww + sx) * 3;
            let mut r = rgb[si] as f32;
            let mut g = rgb[si + 1] as f32;
            let mut b = rgb[si + 2] as f32;
            // Chroma bleed — shift red/blue from neighbors (bounds on source column sx).
            if chroma > 0.01 && sx > 0 && sx + 1 < ww {
                r = r * (1.0 - chroma * 0.72) + rgb[si + 3] as f32 * chroma * 0.72;
                b = b * (1.0 - chroma * 0.65) + rgb[si - 1] as f32 * chroma * 0.65;
            }
            // Dropout streaks
            let drop = hash2d(x as f32 * 0.1, y as f32 + state.frame as f32 * 0.7);
            if drop > 1.0 - wear * 0.16 {
                let v = if drop > 1.0 - wear * 0.04 { 255.0 } else { 18.0 };
                r = v;
                g = v;
                b = v;
            }
            // Ghost previous frame
            if let Some(prior) = prev {
                let gr = ghost * 0.35;
                r = r * (1.0 - gr) + prior[i] as f32 * gr;
                g = g * (1.0 - gr) + prior[i + 1] as f32 * gr;
                b = b * (1.0 - gr) + prior[i + 2] as f32 * gr;
            }
            // Lift blacks, soft clip
            r = (r * 0.92 + 18.0).min(252.0);
            g = (g * 0.90 + 16.0).min(250.0);
            b = (b * 0.90 + 14.0).min(248.0);
            out[i] = r as u8;
            out[i + 1] = g as u8;
            out[i + 2] = b as u8;
        }
    }

    let mut rgba = rgb_to_rgba(&out, w, h);
    // Scanlines
    for y in (0..hh).step_by(2) {
        for x in 0..ww {
            let i = (y * ww + x) * 4;
            rgba[i] = lerp_u8(rgba[i], 0, 0.08);
            rgba[i + 1] = lerp_u8(rgba[i + 1], 0, 0.08);
            rgba[i + 2] = lerp_u8(rgba[i + 2], 0, 0.08);
        }
    }
    rgba
}
