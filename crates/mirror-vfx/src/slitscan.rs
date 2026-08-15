//! SLITSCAN — Space-time slice depth remapping via circular frame history ring buffer.

use super::ops::rgb_to_rgba;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let max_depth = (p.v(1) * 45.0).max(1.0) as usize;
    let mode = p.v(2) as u32; // 0: X-axis, 1: Y-axis, 2: Radial ring
    let modulation = p.v(3);

    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    let max_r = (cx * cx + cy * cy).sqrt().max(1.0);

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        let yf = y as f32;
        for x in 0..ww {
            let xf = x as f32;
            let idx = (y * ww + x) * 3;

            // Calculate temporal depth ratio (0.0..1.0)
            let depth_ratio = match mode {
                0 => xf / (w as f32),
                1 => yf / (h as f32),
                _ => {
                    let dx = xf - cx;
                    let dy = yf - cy;
                    ((dx * dx + dy * dy).sqrt() / max_r).clamp(0.0, 1.0)
                }
            };

            // Add dynamic wave ripple modulation
            let wave = (depth_ratio * 12.0 + state.frame as f32 * 0.05).sin() * 0.1 * modulation;
            let effective_depth_ratio = (depth_ratio + wave).clamp(0.0, 1.0);
            let frame_offset = (effective_depth_ratio * (max_depth as f32)) as usize;

            let shift = (effective_depth_ratio * 35.0 * (1.0 + modulation)) as i32;
            let (sx, sy) = match mode {
                0 => ((x as i32 + shift).clamp(0, w as i32 - 1) as usize, y),
                1 => (x, (y as i32 + shift).clamp(0, h as i32 - 1) as usize),
                _ => (
                    (x as i32 + shift).clamp(0, w as i32 - 1) as usize,
                    (y as i32 + shift).clamp(0, h as i32 - 1) as usize,
                ),
            };
            let sample_idx = (sy * ww + sx) * 3;

            if let Some(hist_frame) = state.get_ring(frame_offset) {
                if hist_frame.len() == out.len() {
                    out[idx..idx + 3].copy_from_slice(&hist_frame[sample_idx..sample_idx + 3]);
                } else {
                    out[idx..idx + 3].copy_from_slice(&rgb[sample_idx..sample_idx + 3]);
                }
            } else {
                out[idx..idx + 3].copy_from_slice(&rgb[sample_idx..sample_idx + 3]);
            }
        }
    }

    rgb_to_rgba(&out, w, h)
}
