//! VORONOI — Dynamic Delaunay / Voronoi mosaic tessellation glass shatter effect.

use super::ops::{rgb_to_rgba, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let seed_count = (12.0 + p.v(1) * 60.0) as usize;
    let jitter = p.v(2);
    let stroke = p.v(3);

    // Deterministically generate seeds based on frame & video layout
    let mut seeds = Vec::with_capacity(seed_count);
    for i in 0..seed_count {
        let fi = i as f32;
        let base_x = (fi * 137.5).sin() * 0.45 + 0.5;
        let base_y = (fi * 293.7).cos() * 0.45 + 0.5;

        let move_x = (state.frame as f32 * 0.02 + fi).sin() * 0.05 * jitter;
        let move_y = (state.frame as f32 * 0.03 + fi * 2.0).cos() * 0.05 * jitter;

        let sx = ((base_x + move_x).clamp(0.02, 0.98)) * w as f32;
        let sy = ((base_y + move_y).clamp(0.02, 0.98)) * h as f32;
        seeds.push((sx, sy));
    }

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        let yf = y as f32;
        for x in 0..ww {
            let xf = x as f32;
            let i = (y * ww + x) * 3;

            // Find 2 nearest seeds for Voronoi cell calculation & edge detection
            let mut min_d1 = f32::MAX;
            let mut min_d2 = f32::MAX;
            let mut nearest_seed = (0.0f32, 0.0f32);

            for &(sx, sy) in &seeds {
                let dx = xf - sx;
                let dy = yf - sy;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < min_d1 {
                    min_d2 = min_d1;
                    min_d1 = dist_sq;
                    nearest_seed = (sx, sy);
                } else if dist_sq < min_d2 {
                    min_d2 = dist_sq;
                }
            }

            let d1 = min_d1.sqrt();
            let d2 = min_d2.sqrt();

            // Voronoi edge stroke line threshold
            if stroke > 0.05 && (d2 - d1) < (stroke * 3.5 + 0.5) {
                out[i] = 20;
                out[i + 1] = 25;
                out[i + 2] = 35;
            } else {
                // Sample color from seed center coordinate
                let (sr, sg, sb) = sample_rgb(rgb, w, h, nearest_seed.0, nearest_seed.1);
                out[i] = sr;
                out[i + 1] = sg;
                out[i + 2] = sb;
            }
        }
    }

    rgb_to_rgba(&out, w, h)
}
