//! TOPO — 2.5D heightmap scanline topographic contour landscape projection.

use super::ops::rgb_to_rgba;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let altitude = p.v(1) * 40.0;
    let contour_freq = 10.0 + p.v(2) * 40.0;
    let wireframe = p.v(3);

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;

            let r = rgb[i] as f32;
            let g = rgb[i + 1] as f32;
            let b = rgb[i + 2] as f32;

            let luma = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;

            // Heightmap elevation displacement offset
            let height_offset = (luma * altitude) as i32;
            let target_y = (y as i32 - height_offset).clamp(0, h as i32 - 1) as usize;
            let target_i = (target_y * ww + x) * 3;

            // Calculate iso-contour elevation bands
            let contour_val = (luma * contour_freq + state.frame as f32 * 0.02).fract();
            let is_contour = contour_val < 0.08;

            let (cr, cg, cb) = if is_contour {
                (0, 240, 210) // Topographic bright cyan-emerald contour line
            } else if wireframe > 0.5 && (y % 4 == 0) {
                ((r * 0.4) as u8, (g * 0.9 + 50.0) as u8, (b * 0.6) as u8)
            } else {
                ((r * 0.8) as u8, (g * 0.8) as u8, (b * 0.9) as u8)
            };

            out[target_i] = cr;
            out[target_i + 1] = cg;
            out[target_i + 2] = cb;
        }
    }

    rgb_to_rgba(&out, w, h)
}
