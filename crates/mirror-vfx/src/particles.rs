//! PARTICLES — disperses camera image into floating pixel dust particles with volume, size, and swirl controls.

use super::ops::{hash2d, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let volume = p.v(1); // particle count / density
    let size = (p.v(2).clamp(0.0, 1.0) * 5.0 + 1.0) as i32; // particle point size
    let swirl = p.v(3); // particle drift & swirl velocity

    let mut out = vec![12u8; ww * hh * 3]; // dark canvas backdrop

    let step = ((1.0 - volume * 0.85) * 8.0).max(1.0) as u32;

    let time = state.frame as f32 * (0.05 + swirl * 0.12);

    for y in (0..h).step_by(step as usize) {
        for x in (0..w).step_by(step as usize) {
            let si = (y as usize * ww + x as usize) * 3;
            let r = rgb[si];
            let g = rgb[si + 1];
            let b = rgb[si + 2];

            // Calculate particle displacement & swirl
            let n = hash2d(x as f32 * 0.05, y as f32 * 0.05);
            let angle = n * 6.28 + time;
            let dist = (hash2d(n, time * 0.1) * swirl * 35.0);

            let px = (x as f32 + angle.cos() * dist) as i32;
            let py = (y as f32 + angle.sin() * dist) as i32;

            // Draw particle square point
            for dy in 0..size {
                let ty = py + dy;
                if ty < 0 || ty >= h as i32 {
                    continue;
                }
                for dx in 0..size {
                    let tx = px + dx;
                    if tx < 0 || tx >= w as i32 {
                        continue;
                    }
                    let di = (ty as usize * ww + tx as usize) * 3;
                    out[di] = r;
                    out[di + 1] = g;
                    out[di + 2] = b;
                }
            }
        }
    }
    rgb_to_rgba(&out, w, h)
}
