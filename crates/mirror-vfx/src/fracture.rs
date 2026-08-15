//! FRACTURE — marquee a region, then tile / mirror / kaleidoscope it.

use super::ops::{hash2d, pixel_in_selection, rgb_to_rgba, sample_rgb, selection_bounds};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let tiles = 2.0 + p.v(1) * 10.0;
    let mirror = p.v(2);
    let spin = p.v(3) * 0.8;

    let mut out = vec![0u8; ww * hh * 3];
    let Some((x0, y0, x1, y1)) = selection_bounds(state) else {
        out.copy_from_slice(rgb);
        return rgb_to_rgba(&out, w, h);
    };

    let rw = (x1 - x0).max(0.02);
    let rh = (y1 - y0).max(0.02);
    let phase = state.frame as f32 * 0.04 * (1.0 + spin);

    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            if !pixel_in_selection(state, x as u32, y as u32, w, h) {
                out[i] = rgb[i];
                out[i + 1] = rgb[i + 1];
                out[i + 2] = rgb[i + 2];
                continue;
            }

            let nx = x as f32 / w as f32;
            let ny = y as f32 / h as f32;
            let mut lx = (nx - x0) / rw;
            let mut ly = (ny - y0) / rh;

            let t = tiles;
            lx = (lx * t).fract();
            ly = (ly * t).fract();

            if mirror > 0.2 {
                let mx = (lx * 2.0).fract();
                let my = (ly * 2.0).fract();
                lx = if mx > 1.0 - mirror * 0.5 { 1.0 - mx } else { mx * 0.5 };
                ly = if my > 1.0 - mirror * 0.5 { 1.0 - my } else { my * 0.5 };
            }

            if spin > 0.01 {
                let cx = 0.5;
                let cy = 0.5;
                let dx = lx - cx;
                let dy = ly - cy;
                let c = phase.cos();
                let s = phase.sin();
                lx = (dx * c - dy * s + cx).fract();
                ly = (dx * s + dy * c + cy).fract();
            }

            let sx = (x0 + lx * rw) * w as f32;
            let sy = (y0 + ly * rh) * h as f32;
            let jitter = hash2d(x as f32, y as f32 + phase) * spin * 3.0;
            let (r, g, b) = sample_rgb(rgb, w, h, sx + jitter, sy - jitter);
            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
        }
    }

    rgb_to_rgba(&out, w, h)
}
