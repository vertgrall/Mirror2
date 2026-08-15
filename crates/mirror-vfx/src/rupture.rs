//! RUPTURE — marquee a region, then glitch / shear / corrupt inside it.

use super::ops::{hash2d, pixel_in_selection, rgb_to_rgba, selection_bounds};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let shear = p.v(1);
    let blocks = p.v(2);
    let chaos = p.v(3);

    let mut out = rgb.to_vec();
    let Some((x0, y0, x1, y1)) = selection_bounds(state) else {
        return rgb_to_rgba(&out, w, h);
    };

    let x_min = (x0 * w as f32).floor() as u32;
    let x_max = (x1 * w as f32).ceil() as u32;
    let y_min = (y0 * h as f32).floor() as u32;
    let y_max = (y1 * h as f32).ceil() as u32;

    for y in y_min..=y_max.min(h - 1) {
        let row_shift = if hash2d(y as f32 * 0.17, state.frame as f32) > 1.0 - shear * 0.4 {
            ((hash2d(y as f32, state.frame as f32 * 0.3) - 0.5) * shear * 80.0) as i32
        } else {
            0
        };

        for x in x_min..=x_max.min(w - 1) {
            if !pixel_in_selection(state, x, y, w, h) {
                continue;
            }
            let i = (y as usize * ww + x as usize) * 3;

            let block = 8.0 + blocks * 24.0;
            let bx = (x as f32 / block).floor() * block;
            let by = (y as f32 / block).floor() * block;
            let bh = hash2d(bx, by + state.frame as f32 * 0.2);
            if bh > 1.0 - blocks * 0.25 {
                let nx = (hash2d(bx + 1.0, by) * 255.0) as u8;
                out[i] = nx;
                out[i + 1] = (nx as f32 * 0.7) as u8;
                out[i + 2] = (nx as f32 * 0.4) as u8;
                continue;
            }

            let sx = (x as i32 + row_shift).clamp(x_min as i32, x_max as i32) as u32;
            let si = (y as usize * ww + sx as usize) * 3;
            let r = rgb[si];
            let g = rgb[si + 1];
            let b = rgb[si + 2];

            if chaos > 0.05 && hash2d(x as f32 * 0.4, y as f32 * 0.4) > 1.0 - chaos * 0.2 {
                out[i] = b;
                out[i + 1] = r;
                out[i + 2] = g;
            } else {
                out[i] = r;
                out[i + 1] = g;
                out[i + 2] = b;
            }
        }
    }

    rgb_to_rgba(&out, w, h)
}
