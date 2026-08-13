//! Sony-style consumer camcorder — warm, interlaced, date stamp.

use super::osd;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let warmth = p.v(1);
    let interlace = p.v(2);
    let stamp = p.v(3);

    let pump = 1.0 + (state.frame as f32 * 0.04).sin() * warmth * 0.10;
    let odd = (state.frame % 2) as usize;

    let mut out = vec![0u8; ww * hh * 4];
    for y in 0..hh {
        let comb = if y % 2 == odd {
            1.0 - interlace * 0.52
        } else {
            1.0
        };
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let o = (y * ww + x) * 4;
            let mut r = rgb[i] as f32 * pump * comb;
            let mut g = rgb[i + 1] as f32 * pump * comb;
            let mut b = rgb[i + 2] as f32 * pump * comb;
            r = r * (1.0 + warmth * 0.28) + warmth * 32.0;
            g = g * (1.0 + warmth * 0.12) + warmth * 16.0;
            b = b * (1.0 - warmth * 0.22);
            r = r.min(250.0);
            g = g.min(248.0);
            if g > 210.0 {
                g = 210.0 + (g - 210.0) * 0.4;
            }
            out[o] = r as u8;
            out[o + 1] = g as u8;
            out[o + 2] = b as u8;
            out[o + 3] = 255;
        }
    }

    osd::burn_gx_stamp(&mut out, w, h, stamp);
    out
}
