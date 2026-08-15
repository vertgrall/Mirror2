//! DATAMOSH — Macroblock motion vector calculation and P-frame pixel dragging corruption.

use super::ops::rgb_to_rgba;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let block_size = (8.0 + p.v(1) * 24.0).max(4.0) as usize;
    let mosh_amount = p.v(2);
    let persistence = p.v(3);

    let mut out = vec![0u8; ww * hh * 3];

    let prev_buf = state.prev_rgb();

    for by in (0..hh).step_by(block_size) {
        for bx in (0..ww).step_by(block_size) {
            // Compute block motion displacement
            let noise = ((bx as f32 * 0.12 + by as f32 * 0.17 + state.frame as f32 * 0.1).sin() * mosh_amount * 12.0) as i32;
            let move_x = if noise.abs() > 4 { noise } else { 0 };
            let move_y = if noise.abs() > 6 { noise / 2 } else { 0 };

            for y in by..(by + block_size).min(hh) {
                for x in bx..(bx + block_size).min(ww) {
                    let idx = (y * ww + x) * 3;

                    let src_x = (x as i32 + move_x).clamp(0, w as i32 - 1) as usize;
                    let src_y = (y as i32 + move_y).clamp(0, h as i32 - 1) as usize;
                    let src_idx = (src_y * ww + src_x) * 3;

                    let cur_r = rgb[src_idx] as f32;
                    let cur_g = rgb[src_idx + 1] as f32;
                    let cur_b = rgb[src_idx + 2] as f32;

                    if let Some(prev) = prev_buf {
                        if prev.len() == rgb.len() && move_x != 0 {
                            let pr = prev[idx] as f32;
                            let pg = prev[idx + 1] as f32;
                            let pb = prev[idx + 2] as f32;

                            out[idx] = (cur_r * (1.0 - persistence) + pr * persistence) as u8;
                            out[idx + 1] = (cur_g * (1.0 - persistence) + pg * persistence) as u8;
                            out[idx + 2] = (cur_b * (1.0 - persistence) + pb * persistence) as u8;
                            continue;
                        }
                    }

                    out[idx] = cur_r as u8;
                    out[idx + 1] = cur_g as u8;
                    out[idx + 2] = cur_b as u8;
                }
            }
        }
    }

    rgb_to_rgba(&out, w, h)
}
