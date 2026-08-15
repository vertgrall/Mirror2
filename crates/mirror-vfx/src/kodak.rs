//! KODAK — Kodachrome slide era: saturated reds, golden highlights, deep blues.

use super::ops::{hash2d, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let chrome = p.v(1);
    let red = p.v(2);
    let grain = p.v(3);

    let mut out = vec![0u8; ww * hh * 4];
    for y in 0..hh {
        for x in 0..ww {
            let (r, g, b) = sample_rgb(rgb, w, h, x as f32, y as f32);
            let l = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0;

            let mut rf = r as f32 * (1.0 + red * 0.22) + red * 12.0;
            let mut gf = g as f32 * (1.0 + chrome * 0.08) + chrome * 6.0;
            let mut bf = b as f32 * (1.0 + chrome * 0.18) - red * 8.0;

            if l > 0.72 {
                let gold = (l - 0.72) * chrome * 120.0;
                rf = (rf + gold).min(255.0);
                gf = (gf + gold * 0.85).min(255.0);
            }

            let sat = 1.0 + chrome * 0.35;
            let avg = (rf + gf + bf) / 3.0;
            rf = avg + (rf - avg) * sat;
            gf = avg + (gf - avg) * sat;
            bf = avg + (bf - avg) * sat;

            let gnoise = hash2d(x as f32 * 1.7, y as f32 * 1.9 + state.frame as f32 * 0.08) - 0.5;
            rf += gnoise * grain * 38.0;
            gf += gnoise * grain * 34.0;
            bf += gnoise * grain * 30.0;

            let o = (y * ww + x) * 4;
            out[o] = rf.clamp(0.0, 255.0) as u8;
            out[o + 1] = gf.clamp(0.0, 255.0) as u8;
            out[o + 2] = bf.clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;
        }
    }
    out
}
