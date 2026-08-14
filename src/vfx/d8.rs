//! MiniDV — 8×8 macroblocks, first digital tape lie.

use super::ops::{hash2d, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

const BLOCK: u32 = 8;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let block = p.v(1);
    let drop = p.v(2);
    let date = p.v(3);

    let mut out = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        for x in 0..ww {
            let bx = (x as u32 / BLOCK) * BLOCK + BLOCK / 2;
            let by = (y as u32 / BLOCK) * BLOCK + BLOCK / 2;
            let bx = bx.min(w - 1) as usize;
            let by = by.min(h - 1) as usize;
            let si = (by * ww + bx) * 3;
            let mut r = rgb[si] as f32;
            let mut g = rgb[si + 1] as f32;
            let mut b = rgb[si + 2] as f32;

            let q = 1.0 + block * 24.0;
            r = (r / q).round() * q;
            g = (g / q).round() * q;
            b = (b / q).round() * q;

            let n = hash2d(bx as f32, by as f32 + state.frame as f32 * 0.3);
            if n > 1.0 - drop * 0.08 {
                r *= 0.55;
                g *= 0.55;
                b *= 0.55;
            }

            let i = (y * ww + x) * 3;
            out[i] = r.clamp(0.0, 255.0) as u8;
            out[i + 1] = g.clamp(0.0, 255.0) as u8;
            out[i + 2] = b.clamp(0.0, 255.0) as u8;
        }
    }

    let mut rgba = rgb_to_rgba(&out, w, h);
    if date > 0.01 {
        super::osd::burn_d8_date(&mut rgba, w, h, date);
    }
    rgba
}