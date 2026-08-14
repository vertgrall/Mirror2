//! Betamax — sharper tape, cruel luma dropouts.

use super::ops::{hash2d, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let dropout = p.v(1);
    let luma = p.v(2);
    let edge = p.v(3);

    let mut out = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let mut r = rgb[i] as f32;
            let mut g = rgb[i + 1] as f32;
            let mut b = rgb[i + 2] as f32;

            let sharp = 1.0 + edge * 0.35;
            if x > 0 && x + 1 < ww {
                let li = (y * ww + x - 1) * 3;
                let ri = (y * ww + x + 1) * 3;
                r = r * sharp - (rgb[li] as f32 + rgb[ri] as f32) * edge * 0.12;
                g = g * sharp - (rgb[li + 1] as f32 + rgb[ri + 1] as f32) * edge * 0.12;
                b = b * sharp - (rgb[li + 2] as f32 + rgb[ri + 2] as f32) * edge * 0.12;
            }

            let yl = r * 0.2126 + g * 0.7152 + b * 0.0722;
            let crush = 0.88 + luma * 0.18;
            r = (r * crush + yl * luma * 0.08).min(252.0);
            g = (g * crush + yl * luma * 0.08).min(250.0);
            b = (b * crush + yl * luma * 0.06).min(248.0);

            let drop = hash2d(x as f32 * 0.07, y as f32 * 0.03 + state.frame as f32);
            if drop > 1.0 - dropout * 0.12 {
                let v = if (y + state.frame as usize) % 5 == 0 {
                    12.0
                } else {
                    220.0
                };
                r = v;
                g = v;
                b = v;
            }

            out[i] = r.clamp(0.0, 255.0) as u8;
            out[i + 1] = g.clamp(0.0, 255.0) as u8;
            out[i + 2] = b.clamp(0.0, 255.0) as u8;
        }
    }
    rgb_to_rgba(&out, w, h)
}
