//! BOUNCE — frame fragments cut from live feed bounce off viewport edges, cloning & trailing.

use super::ops::{hash2d, lerp_u8, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let num_clones = (1.0 + p.v(1) * 5.0) as usize; // 1 to 6 bouncing clones
    let speed = 2.0 + p.v(2) * 14.0; // bounce velocity
    let decay = p.v(3) * 0.70; // motion trail decay

    let prev = state.prev_rgb();

    let mut out = if let Some(prior) = prev {
        let mut v = vec![0u8; ww * hh * 3];
        for j in 0..ww * hh * 3 {
            v[j] = lerp_u8(rgb[j], prior[j], decay);
        }
        v
    } else {
        rgb.to_vec()
    };

    let frag_w = (w as f32 * 0.28) as usize;
    let frag_h = (h as f32 * 0.24) as usize;

    if frag_w > 4 && frag_h > 4 {
        let time = state.frame as f32 * speed * 0.05;

        for c in 0..num_clones {
            let seed = c as f32 * 4.13;
            let vx = (hash2d(seed, 1.0) * 2.0 + 1.0) * speed;
            let vy = (hash2d(seed, 2.0) * 2.0 + 1.0) * speed;

            // Bouncing DVD-style triangle wave position
            let raw_x = (time * vx) as i32;
            let raw_y = (time * vy) as i32;

            let max_x = (ww.saturating_sub(frag_w)) as i32;
            let max_y = (hh.saturating_sub(frag_h)) as i32;

            if max_x <= 0 || max_y <= 0 {
                continue;
            }

            let bx = (raw_x % (max_x * 2)).abs();
            let by = (raw_y % (max_y * 2)).abs();

            let dst_x = if bx > max_x { max_x * 2 - bx } else { bx } as usize;
            let dst_y = if by > max_y { max_y * 2 - by } else { by } as usize;

            // Source fragment position
            let src_x = ((c * 47) % (ww - frag_w)) as usize;
            let src_y = ((c * 31) % (hh - frag_h)) as usize;

            // Composite bouncing frame fragment
            for py in 0..frag_h {
                let dy = dst_y + py;
                let sy = src_y + py;
                if dy >= hh || sy >= hh {
                    continue;
                }
                for px in 0..frag_w {
                    let dx = dst_x + px;
                    let sx = src_x + px;
                    if dx >= ww || sx >= ww {
                        continue;
                    }

                    let d_i = (dy * ww + dx) * 3;
                    let s_i = (sy * ww + sx) * 3;

                    let is_border = px == 0 || px == frag_w - 1 || py == 0 || py == frag_h - 1;

                    if is_border {
                        out[d_i] = 240;
                        out[d_i + 1] = 240;
                        out[d_i + 2] = 250;
                    } else {
                        out[d_i] = rgb[s_i];
                        out[d_i + 1] = rgb[s_i + 1];
                        out[d_i + 2] = rgb[s_i + 2];
                    }
                }
            }
        }
    }

    rgb_to_rgba(&out, w, h)
}
