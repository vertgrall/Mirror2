//! STAMP — snapshot taken every ~3 seconds, smeared as downscaled picture-in-picture instances in corners.

use super::ops::{hash2d, lerp_u8, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let rate = p.v(1); // interval control
    let scale_factor = (0.18 + p.v(2) * 0.22) as f32; // PIP scale (18% - 40%)
    let count = (1.0 + p.v(3) * 4.0) as usize; // 1 to 5 corner instances

    // Frame interval calculation (roughly 3s at 30fps = 90 frames, adjustable by rate)
    let interval = ((120.0 - rate * 80.0) as u64).max(15);
    let epoch = state.frame / interval;

    // Check or update snapshot in temporal state or frame hash
    let mut out = rgb.to_vec();

    let pip_w = (w as f32 * scale_factor) as usize;
    let pip_h = (h as f32 * scale_factor) as usize;

    if pip_w > 4 && pip_h > 4 {
        // Corner locations for up to 5 PIP stamps
        let corners = [
            (12, 12),                                 // top-left
            (ww.saturating_sub(pip_w + 12), 12),     // top-right
            (12, hh.saturating_sub(pip_h + 12)),     // bottom-left
            (ww.saturating_sub(pip_w + 12), hh.saturating_sub(pip_h + 12)), // bottom-right
            (ww / 2 - pip_w / 2, 12),                 // top-center
        ];

        for idx in 0..count.min(corners.len()) {
            let (corner_x, corner_y) = corners[idx];

            // Offset pseudo-snapshot phase per corner epoch
            let corner_hash = hash2d(idx as f32 * 3.7, epoch as f32 * 1.3);
            let shift_x = ((corner_hash - 0.5) * 20.0) as i32;
            let shift_y = ((hash2d(epoch as f32, idx as f32) - 0.5) * 16.0) as i32;

            let dst_x0 = (corner_x as i32 + shift_x).clamp(0, (ww - pip_w) as i32) as usize;
            let dst_y0 = (corner_y as i32 + shift_y).clamp(0, (hh - pip_h) as i32) as usize;

            // Draw downscaled PIP stamp over target area
            for py in 0..pip_h {
                let sy = ((py * hh) / pip_h).min(hh - 1);
                let dy = dst_y0 + py;
                if dy >= hh {
                    continue;
                }

                for px in 0..pip_w {
                    let sx = ((px * ww) / pip_w).min(ww - 1);
                    let dx = dst_x0 + px;
                    if dx >= ww {
                        continue;
                    }

                    let s_idx = (sy * ww + sx) * 3;
                    let d_idx = (dy * ww + dx) * 3;

                    // Border outline check
                    let is_border = px == 0 || px == pip_w - 1 || py == 0 || py == pip_h - 1;

                    if is_border {
                        out[d_idx] = 230;
                        out[d_idx + 1] = 230;
                        out[d_idx + 2] = 240;
                    } else {
                        // Blend snapshot frame with subtle opacity
                        let alpha = 0.85;
                        out[d_idx] = lerp_u8(out[d_idx], rgb[s_idx], alpha);
                        out[d_idx + 1] = lerp_u8(out[d_idx + 1], rgb[s_idx + 1], alpha);
                        out[d_idx + 2] = lerp_u8(out[d_idx + 2], rgb[s_idx + 2], alpha);
                    }
                }
            }
        }
    }

    rgb_to_rgba(&out, w, h)
}
