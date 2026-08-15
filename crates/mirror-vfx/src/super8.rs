//! SUPER8 — home-movie 8mm: gate jitter, warm halation, coarse flickering grain.

use super::ops::{hash2d, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let gate = p.v(1);
    let warm = p.v(2);
    let flicker = p.v(3);

    let jx = (state.frame as f32 * 0.19).sin() * gate * 2.5;
    let jy = (state.frame as f32 * 0.13).cos() * gate * 2.0;
    let flick = 1.0 + (state.frame as f32 * 0.7).sin() * flicker * 0.12;
    let xmax = w as f32 - 1.001;
    let ymax = h as f32 - 1.001;

    let bar = (h as f32 * 0.04).round() as u32;

    let mut out = vec![0u8; ww * hh * 4];
    for y in 0..hh {
        let row_j = (y as f32 * 0.05 + state.frame as f32 * 0.11).sin() * gate * 1.2;
        for x in 0..ww {
            let o = (y * ww + x) * 4;

            if y < bar as usize || y >= hh - bar as usize {
                out[o] = 12;
                out[o + 1] = 10;
                out[o + 2] = 8;
                out[o + 3] = 255;
                continue;
            }

            let sx = (x as f32 + jx + row_j).clamp(0.0, xmax);
            let sy = (y as f32 + jy).clamp(0.0, ymax);
            let (r, g, b) = sample_rgb(rgb, w, h, sx, sy);

            let mut rf = r as f32 * flick * (1.0 + warm * 0.1) + warm * 10.0;
            let mut gf = g as f32 * flick * (1.0 + warm * 0.05) + warm * 6.0;
            let mut bf = b as f32 * flick * (1.0 - warm * 0.12);

            let coarse = hash2d(x as f32 * 0.6, y as f32 * 0.7 + state.frame as f32 * 0.25) - 0.5;
            let fine = hash2d(x as f32 * 3.2, y as f32 * 3.4) - 0.5;
            rf += coarse * gate * 45.0 + fine * gate * 18.0;
            gf += coarse * gate * 40.0 + fine * gate * 16.0;
            bf += coarse * gate * 35.0 + fine * gate * 14.0;

            if rf > 190.0 {
                rf = (rf + warm * 20.0).min(255.0);
            }

            out[o] = rf.clamp(0.0, 255.0) as u8;
            out[o + 1] = gf.clamp(0.0, 255.0) as u8;
            out[o + 2] = bf.clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;
        }
    }
    out
}
