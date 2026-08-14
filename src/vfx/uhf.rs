//! UHF — rabbit-ear broadcast. Snow, vertical roll, dying signal.

use super::ops::{hash2d, lerp_u8, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let snow = p.v(1);
    let roll = p.v(2);
    let tint = p.v(3);

    let roll_off =
        ((state.frame as f32 * 0.7 + roll * 40.0).sin() * roll * hh as f32 * 0.08) as i32;

    let mut out = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        let sy = ((y as i32 + roll_off).rem_euclid(hh as i32)) as usize;
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let si = (sy * ww + x) * 3;
            let mut r = rgb[si] as f32;
            let mut g = rgb[si + 1] as f32;
            let mut b = rgb[si + 2] as f32;

            let n = hash2d(x as f32 * 0.31 + state.frame as f32, y as f32 * 0.17);
            if n > 1.0 - snow * 0.22 {
                let v = if n > 1.0 - snow * 0.04 {
                    240.0
                } else {
                    28.0
                };
                r = r * (1.0 - snow * 0.85) + v * snow * 0.85;
                g = g * (1.0 - snow * 0.85) + v * snow * 0.85;
                b = b * (1.0 - snow * 0.85) + v * snow * 0.85;
            }

            let band = ((y as f32 * 0.08 + state.frame as f32 * 0.5).sin() * 0.5 + 0.5) * roll;
            r *= 1.0 - band * 0.35;
            g *= 1.0 - band * 0.35;
            b *= 1.0 - band * 0.35;

            let grey = r * 0.299 + g * 0.587 + b * 0.114;
            r = grey * (1.0 - tint * 0.35) + 40.0 * tint;
            g = grey * (1.0 - tint * 0.25) + 48.0 * tint;
            b = grey * (1.0 - tint * 0.05) + 72.0 * tint;

            out[i] = r.clamp(0.0, 255.0) as u8;
            out[i + 1] = g.clamp(0.0, 255.0) as u8;
            out[i + 2] = b.clamp(0.0, 255.0) as u8;
        }
    }

    let mut rgba = rgb_to_rgba(&out, w, h);
    for y in (0..hh).step_by(3) {
        for x in 0..ww {
            let i = (y * ww + x) * 4;
            rgba[i] = lerp_u8(rgba[i], 0, 0.06);
            rgba[i + 1] = lerp_u8(rgba[i + 1], 0, 0.06);
            rgba[i + 2] = lerp_u8(rgba[i + 2], 0, 0.06);
        }
    }
    rgba
}
