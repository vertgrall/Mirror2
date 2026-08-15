//! POLAR — 1970s instant film: soft fade, cyan shadow, thick white rebate.

use super::ops::{hash2d, lerp_u8, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let fade = p.v(1);
    let warmth = p.v(2);
    let frame_amt = p.v(3);

    let border = 0.05 + frame_amt * 0.06;
    let left = (w as f32 * border) as u32;
    let right = w - left;
    let top = (h as f32 * border * 1.1) as u32;
    let bottom = h - top;

    let mut out = vec![0u8; ww * hh * 4];
    for y in 0..hh {
        for x in 0..ww {
            let o = (y * ww + x) * 4;
            let in_border = x < left as usize
                || x >= right as usize
                || y < top as usize
                || y >= bottom as usize;

            if in_border {
                let base = 242.0 - hash2d(x as f32, y as f32) * 8.0;
                out[o] = base as u8;
                out[o + 1] = (base - 2.0) as u8;
                out[o + 2] = (base - 6.0) as u8;
                out[o + 3] = 255;
                continue;
            }

            let (r, g, b) = sample_rgb(rgb, w, h, x as f32, y as f32);
            let l = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0;

            let mut rf = r as f32 * (1.0 + warmth * 0.12) + warmth * 14.0;
            let mut gf = g as f32 * (1.0 + warmth * 0.06) + warmth * 8.0;
            let mut bf = b as f32 * (1.0 - warmth * 0.08) + (1.0 - l) * warmth * 18.0;

            let lift = fade * 28.0;
            rf = rf * (1.0 - fade * 0.08) + lift;
            gf = gf * (1.0 - fade * 0.06) + lift * 0.98;
            bf = bf * (1.0 - fade * 0.04) + lift * 0.92;

            let grain = hash2d(x as f32 * 2.1, y as f32 * 2.3 + state.frame as f32 * 0.2) - 0.5;
            rf += grain * fade * 22.0;
            gf += grain * fade * 20.0;
            bf += grain * fade * 18.0;

            out[o] = rf.clamp(0.0, 255.0) as u8;
            out[o + 1] = gf.clamp(0.0, 255.0) as u8;
            out[o + 2] = bf.clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;

            if l < 0.35 {
                out[o + 2] = lerp_u8(out[o + 2], 180, fade * 0.35);
            }
        }
    }
    out
}
