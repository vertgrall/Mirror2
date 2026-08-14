//! FILM — 35mm stock, warm tone, grain, halation.

use super::ops::hash2d;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let grain = p.v(1);
    let warm = p.v(2);
    let frame = p.v(3);

    let mut out = vec![0u8; ww * hh * 4];
    for y in 0..hh {
        for x in 0..ww {
            let o = (y * ww + x) * 4;
            let i = (y * ww + x) * 3;
            let mut r = rgb[i] as f32;
            let mut g = rgb[i + 1] as f32;
            let mut b = rgb[i + 2] as f32;

            r = r * (1.0 + warm * 0.12) + warm * 14.0;
            g = g * (1.0 + warm * 0.06) + warm * 8.0;
            b = b * (1.0 - warm * 0.08);

            let lift = 10.0 + warm * 8.0 + frame * 6.0;
            r = (r + lift).min(255.0);
            g = (g + lift * 0.95).min(255.0);
            b = (b + lift * 0.85).min(255.0);

            if grain > 0.01 {
                let n = hash2d(x as f32 * 1.7 + state.frame as f32, y as f32 * 2.1) - 0.5;
                let gstr = grain * 42.0;
                r = (r + n * gstr).clamp(0.0, 255.0);
                g = (g + n * gstr * 0.9).clamp(0.0, 255.0);
                b = (b + n * gstr * 0.8).clamp(0.0, 255.0);
            }

            let halation = warm * 0.08;
            if r > 200.0 {
                r = (r + halation * 20.0).min(255.0);
            }

            out[o] = r as u8;
            out[o + 1] = g as u8;
            out[o + 2] = b as u8;
            out[o + 3] = 255;
        }
    }
    out
}
