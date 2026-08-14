//! MOSH — datamosh video compression artifacts, motion vector macroblock bleeding, P-frame drops.

use super::ops::{hash2d, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let smear = p.v(1);
    let block_size = (p.v(2).clamp(0.0, 1.0) * 24.0 + 8.0) as u32;
    let drop_rate = p.v(3);

    let prev = state.prev_rgb();

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        let by = (y as u32 / block_size) * block_size;
        for x in 0..ww {
            let bx = (x as u32 / block_size) * block_size;
            let i = (y * ww + x) * 3;

            // Pseudo motion vector for macroblock
            let m_angle = hash2d(bx as f32 * 0.1, by as f32 * 0.1 + state.frame as f32 * 0.05) * 6.28;
            let m_dist = smear * 45.0;

            let vx = (m_angle.cos() * m_dist) as i32;
            let vy = (m_angle.sin() * m_dist) as i32;

            let sx = (x as i32 + vx).clamp(0, w as i32 - 1) as usize;
            let sy = (y as i32 + vy).clamp(0, h as i32 - 1) as usize;
            let si = (sy * ww + sx) * 3;

            let mut r = rgb[si] as f32;
            let mut g = rgb[si + 1] as f32;
            let mut b = rgb[si + 2] as f32;

            // P-frame hold/bleed from prior frame
            if let Some(prior) = prev {
                let drop_hash = hash2d(bx as f32 * 0.3, by as f32 * 0.3 + (state.frame / 4) as f32);
                if drop_hash < drop_rate * 0.65 {
                    let pi = (y * ww + x) * 3;
                    r = r * 0.2 + prior[pi] as f32 * 0.8;
                    g = g * 0.2 + prior[pi + 1] as f32 * 0.8;
                    b = b * 0.2 + prior[pi + 2] as f32 * 0.8;
                }
            }

            out[i] = r.clamp(0.0, 255.0) as u8;
            out[i + 1] = g.clamp(0.0, 255.0) as u8;
            out[i + 2] = b.clamp(0.0, 255.0) as u8;
        }
    }
    rgb_to_rgba(&out, w, h)
}
