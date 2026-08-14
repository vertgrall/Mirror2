//! WAVES — sepia silver print. Slow horizontal film-gate ripples + grain.

use super::ops::{hash2d, lerp_u8, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

fn sepia_tone(r: f32, g: f32, b: f32, amount: f32) -> (f32, f32, f32) {
    let sr = (r * 0.393 + g * 0.769 + b * 0.189).clamp(0.0, 255.0);
    let sg = (r * 0.349 + g * 0.686 + b * 0.168).clamp(0.0, 255.0);
    let sb = (r * 0.272 + g * 0.534 + b * 0.131).clamp(0.0, 255.0);
    let t = amount.clamp(0.0, 1.0);
    (
        r * (1.0 - t) + sr * t,
        g * (1.0 - t) + sg * t,
        b * (1.0 - t) + sb * t,
    )
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let sepia = p.v(1);
    let wave = p.v(2);
    let grain = p.v(3);

    let amp = wave * 22.0;
    let phase = state.frame as f32 * (0.04 + wave * 0.06);

    let mut out = vec![0u8; ww * hh * 4];
    let xmax = w as f32 - 1.001;
    let ymax = h as f32 - 1.001;
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.48;

    for y in 0..hh {
        let yf = y as f32;
        let row_roll = (yf * 0.038 + phase).sin() * amp;
        let row_roll2 = (yf * 0.015 + phase * 0.73).sin() * amp * 0.45;
        let ripple = (yf * 0.09 + phase * 1.4).sin() * wave * 3.5;

        for x in 0..ww {
            let xf = x as f32;
            let dx = row_roll + row_roll2 + ripple;
            let dy = (xf * 0.012 + phase * 0.5).sin() * wave * 2.2;
            let sx = (xf + dx).clamp(0.0, xmax);
            let sy = (yf + dy).clamp(0.0, ymax);

            let (mut r, mut g, mut b) = sample_rgb(rgb, w, h, sx, sy);
            let mut rf = r as f32;
            let mut gf = g as f32;
            let mut bf = b as f32;

            (rf, gf, bf) = sepia_tone(rf, gf, bf, sepia);

            let lift = 8.0 + sepia * 10.0;
            rf = (rf * 0.94 + lift).min(255.0);
            gf = (gf * 0.92 + lift * 0.92).min(255.0);
            bf = (bf * 0.88 + lift * 0.75).min(255.0);

            if grain > 0.01 {
                let n = hash2d(xf * 2.1 + state.frame as f32, yf * 2.7) - 0.5;
                let gstr = grain * 38.0;
                rf = (rf + n * gstr).clamp(0.0, 255.0);
                gf = (gf + n * gstr * 0.85).clamp(0.0, 255.0);
                bf = (bf + n * gstr * 0.7).clamp(0.0, 255.0);
            }

            let nx = (xf - cx) / cx;
            let ny = (yf - cy) / cy;
            let vig = (nx * nx + ny * ny).min(1.0) * sepia * 0.22;
            rf = rf * (1.0 - vig);
            gf = gf * (1.0 - vig);
            bf = bf * (1.0 - vig * 0.85);

            if wave > 0.05 && y % 3 == 0 {
                let scan = wave * 0.06;
                rf = lerp_u8(rf as u8, 0, scan) as f32;
                gf = lerp_u8(gf as u8, 0, scan) as f32;
                bf = lerp_u8(bf as u8, 0, scan) as f32;
            }

            let o = (y * ww + x) * 4;
            out[o] = rf as u8;
            out[o + 1] = gf as u8;
            out[o + 2] = bf as u8;
            out[o + 3] = 255;
        }
    }
    out
}
