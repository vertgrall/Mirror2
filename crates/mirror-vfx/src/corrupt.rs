//! CORRUPT — brutal digital corruption: block shuffles, channel swaps, inverted tiles.

use super::ops::{hash2d, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let tear = p.v(1);
    let blocks = p.v(2);
    let chaos = p.v(3);

    let block_size = (8.0 + blocks * 40.0) as u32;
    let mut out = rgb.to_vec();

    // Macroblock channel scramble
    for by in (0..hh as u32).step_by(block_size as usize) {
        for bx in (0..ww as u32).step_by(block_size as usize) {
            let bh = hash2d(bx as f32 * 0.09, by as f32 * 0.11 + state.frame as f32 * 0.07);
            if bh > 1.0 - chaos * 0.45 {
                let mode = (bh * 17.0) as u32 % 4;
                for y in by..(by + block_size).min(h as u32) {
                    for x in bx..(bx + block_size).min(w as u32) {
                        let i = (y as usize * ww + x as usize) * 3;
                        let (r, g, b) = (out[i], out[i + 1], out[i + 2]);
                        match mode {
                            0 => {
                                out[i] = 255 - r;
                                out[i + 1] = 255 - g;
                                out[i + 2] = 255 - b;
                            }
                            1 => {
                                out[i] = g;
                                out[i + 1] = b;
                                out[i + 2] = r;
                            }
                            2 => {
                                out[i] = b;
                                out[i + 1] = r;
                                out[i + 2] = g;
                            }
                            _ => {
                                let n = (hash2d(x as f32, y as f32 + state.frame as f32) * 255.0) as u8;
                                out[i] = n;
                                out[i + 1] = n;
                                out[i + 2] = n;
                            }
                        }
                    }
                }
            }
        }
    }

    // Horizontal tear bands
    let mut torn = out.clone();
    for y in 0..hh {
        let line_hash = hash2d(y as f32 * 0.17, state.frame as f32 * 0.55);
        if line_hash > 1.0 - tear * 0.4 {
            let shift = ((line_hash - 0.5) * tear * 180.0) as i32;
            for x in 0..ww {
                let sx = (x as i32 + shift).clamp(0, w as i32 - 1) as usize;
                let i = (y * ww + x) * 3;
                let si = (y * ww + sx) * 3;
                torn[i] = out[si];
                torn[i + 1] = out[si + 1];
                torn[i + 2] = out[si + 2];
            }
        }
    }
    out = torn;

    // Sparse pixel death
    if chaos > 0.05 {
        for y in 0..hh {
            for x in 0..ww {
                let n = hash2d(x as f32 * 0.31, y as f32 * 0.29 + state.frame as f32 * 0.8);
                if n > 1.0 - chaos * 0.08 {
                    let i = (y * ww + x) * 3;
                    out[i] = 0;
                    out[i + 1] = (n * 80.0) as u8;
                    out[i + 2] = (n * 255.0) as u8;
                }
            }
        }
    }

    rgb_to_rgba(&out, w, h)
}
