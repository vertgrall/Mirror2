//! GLITCH — analog sync tear, horizontal line displacement, RGB channel shear, static noise.

use super::ops::{hash2d, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let tear = p.v(1);
    let shear = p.v(2);
    let glitch = p.v(3);

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        // Line displacement from sync tear
        let line_hash = hash2d(y as f32 * 0.12, state.frame as f32 * 0.4);
        let shift = if line_hash > 1.0 - tear * 0.35 {
            ((line_hash - 0.5) * tear * 140.0) as i32
        } else {
            0
        };

        let red_offset = (shear * 18.0) as i32;
        let blue_offset = -(shear * 14.0) as i32;

        for x in 0..ww {
            let i = (y * ww + x) * 3;

            let rx = (x as i32 + shift + red_offset).clamp(0, w as i32 - 1) as usize;
            let gx = (x as i32 + shift).clamp(0, w as i32 - 1) as usize;
            let bx = (x as i32 + shift + blue_offset).clamp(0, w as i32 - 1) as usize;

            let r_si = (y * ww + rx) * 3;
            let g_si = (y * ww + gx) * 3;
            let b_si = (y * ww + bx) * 3;

            let mut r = rgb[r_si];
            let mut g = rgb[g_si + 1];
            let mut b = rgb[b_si + 2];

            // Random digital static breakup blocks
            if glitch > 0.01 {
                let static_n = hash2d(x as f32 * 0.25, y as f32 * 0.25 + state.frame as f32);
                if static_n > 1.0 - glitch * 0.15 {
                    let noise = (static_n * 255.0) as u8;
                    r = noise;
                    g = noise;
                    b = noise;
                }
            }

            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
        }
    }
    rgb_to_rgba(&out, w, h)
}
