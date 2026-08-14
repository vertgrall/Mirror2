//! LIVE — broadcast truck. Interlace, tally, crush.

use super::ops::lerp_u8;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let tally = p.v(1);
    let comb = p.v(2);
    let crush = p.v(3);

    let odd = (state.frame % 2) as usize;
    let mut out = vec![0u8; ww * hh * 4];

    for y in 0..hh {
        let field = if y % 2 == odd {
            1.0 - comb * 0.55
        } else {
            1.0
        };
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let o = (y * ww + x) * 4;
            let mut r = rgb[i] as f32 * field;
            let mut g = rgb[i + 1] as f32 * field;
            let mut b = rgb[i + 2] as f32 * field;

            let lo = 22.0 + crush * 18.0;
            let hi = 248.0 - crush * 40.0;
            r = ((r - lo) * (1.0 + crush * 0.4) + lo).clamp(0.0, hi);
            g = ((g - lo) * (1.0 + crush * 0.35) + lo).clamp(0.0, hi);
            b = ((b - lo) * (1.0 + crush * 0.35) + lo).clamp(0.0, hi);

            out[o] = r as u8;
            out[o + 1] = g as u8;
            out[o + 2] = b as u8;
            out[o + 3] = 255;
        }
    }

    let bar_w = (w as f32 * 0.018 * tally).max(1.0) as u32;
    for y in 0..h {
        for x in 0..bar_w.min(w) {
            let o = ((y * w + x) as usize) * 4;
            out[o] = lerp_u8(out[o], 220, tally * 0.85);
            out[o + 1] = lerp_u8(out[o + 1], 28, tally * 0.85);
            out[o + 2] = lerp_u8(out[o + 2], 28, tally * 0.85);
        }
    }
    out
}
