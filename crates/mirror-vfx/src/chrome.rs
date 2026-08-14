//! CHROME — super fast random coordinate metallic/solarized patch glinting.

use super::ops::{hash2d, lum, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let jitter = p.v(1); // fast randomization rate
    let shine = p.v(2); // metallic reflectivity
    let patches = p.v(3); // coordinate patch density

    let mut out = vec![0u8; ww * hh * 3];

    let block_w = 16u32;
    let block_h = 12u32;

    for y in 0..hh {
        let by = y as u32 / block_h;
        for x in 0..ww {
            let bx = x as u32 / block_w;
            let i = (y * ww + x) * 3;

            // Fast per-frame random seed for blocks and individual coordinates
            let patch_n = hash2d(
                bx as f32 * 0.17 + state.frame as f32 * (0.8 + jitter * 1.5),
                by as f32 * 0.23,
            );

            let mut r = rgb[i] as f32;
            let mut g = rgb[i + 1] as f32;
            let mut b = rgb[i + 2] as f32;

            if patch_n > 1.0 - patches * 0.70 {
                let l = lum(rgb[i], rgb[i + 1], rgb[i + 2]);

                // Sabattier tone inversion curve -> liquid chrome metal
                let chrome_l = (l * 3.14159 * 2.0).sin().abs();
                let chrome_val = (chrome_l * (1.0 + shine * 1.8)).clamp(0.0, 1.0) * 255.0;

                r = r * 0.2 + chrome_val * 0.8;
                g = g * 0.2 + (chrome_val * 0.95 + 12.0) * 0.8;
                b = b * 0.2 + (chrome_val * 1.1 + 24.0).min(255.0) * 0.8;
            }

            out[i] = r.clamp(0.0, 255.0) as u8;
            out[i + 1] = g.clamp(0.0, 255.0) as u8;
            out[i + 2] = b.clamp(0.0, 255.0) as u8;
        }
    }
    rgb_to_rgba(&out, w, h)
}
