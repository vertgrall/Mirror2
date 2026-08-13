//! RIPPLE — clear VHS camcorder through water: radial waves + light tape.

use super::ops::{lerp_u8, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let swell = p.v(1);
    let rings = p.v(2);
    let tape = p.v(3);

    let amp = swell * 14.0;
    let freq = 0.028 + rings * 0.085;
    let phase = state.frame as f32 * (0.07 + rings * 0.05);

    let cx = w as f32 * 0.50;
    let cy = h as f32 * 0.46;
    let cx2 = w as f32 * 0.36;
    let cy2 = h as f32 * 0.68;
    let phase2 = phase * 0.82 + 2.1;

    let mut out = vec![0u8; ww * hh * 4];
    let xmax = w as f32 - 1.001;
    let ymax = h as f32 - 1.001;

    for y in 0..hh {
        let jitter = if tape > 0.02 {
            ((y as f32 * 0.19 + state.frame as f32 * 0.28).sin() * tape * 2.4) as f32
        } else {
            0.0
        };
        let scan = if tape > 0.01 && y % 2 == 0 {
            0.06 + tape * 0.10
        } else {
            0.0
        };
        for x in 0..ww {
            let xf = x as f32;
            let yf = y as f32;
            let (dx, dy) = (xf - cx, yf - cy);
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            let (dx2, dy2) = (xf - cx2, yf - cy2);
            let d2 = (dx2 * dx2 + dy2 * dy2).sqrt().max(1.0);

            let wave = (d * freq - phase).sin() + (d2 * freq * 1.18 - phase2).sin() * 0.55;
            let mag = wave * amp;
            let sx = (xf + dx / d * mag + jitter).clamp(0.0, xmax);
            let sy = (yf + dy / d * mag).clamp(0.0, ymax);

            let (mut r, mut g, mut b) = sample_rgb(rgb, w, h, sx, sy);

            if tape > 0.02 && x > 0 && x + 1 < ww {
                let (rr, _, _) = sample_rgb(rgb, w, h, (sx + 1.6 * tape).clamp(0.0, xmax), sy);
                let (_, _, bb) = sample_rgb(rgb, w, h, (sx - 1.4 * tape).clamp(0.0, xmax), sy);
                let t = tape * 0.55;
                r = lerp_u8(r, rr, t);
                b = lerp_u8(b, bb, t);
            }

            if tape > 0.01 {
                r = ((r as f32) * 0.94 + 12.0 + tape * 8.0).min(252.0) as u8;
                g = ((g as f32) * 0.93 + 10.0 + tape * 4.0).min(250.0) as u8;
                b = ((b as f32) * 0.90 + 8.0).min(248.0) as u8;
            }
            if scan > 0.0 {
                r = lerp_u8(r, 0, scan);
                g = lerp_u8(g, 0, scan);
                b = lerp_u8(b, 0, scan);
            }

            let o = (y * ww + x) * 4;
            out[o] = r;
            out[o + 1] = g;
            out[o + 2] = b;
            out[o + 3] = 255;
        }
    }
    out
}
