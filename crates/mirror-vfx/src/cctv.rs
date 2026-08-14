//! 80s CCTV — blocky, green-grey, crushed contrast, timestamp.

use super::ops::{lerp_u8, lum, rgb_to_rgba};
use super::osd;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let block = p.v(1).round().clamp(2.0, 8.0) as u32;
    let green = p.v(2);
    let contrast = p.v(3);
    let fisheye = 0.22;

    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;

    let mut tmp = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        for x in 0..ww {
            let (sx, sy) = if fisheye > 0.01 {
                let nx = (x as f32 - cx) / cx;
                let ny = (y as f32 - cy) / cy;
                let r2 = nx * nx + ny * ny;
                let k = 1.0 + fisheye * r2 * 0.85;
                (
                    (cx + nx * cx / k).clamp(0.0, w as f32 - 1.0) as u32,
                    (cy + ny * cy / k).clamp(0.0, h as f32 - 1.0) as u32,
                )
            } else {
                (x as u32, y as u32)
            };
            let bx = (sx / block) * block;
            let by = (sy / block) * block;
            let si = ((by as usize * ww + bx as usize)) * 3;
            let i = (y * ww + x) * 3;
            tmp[i..i + 3].copy_from_slice(&rgb[si..si + 3]);
        }
    }

    let mut rgba = rgb_to_rgba(&tmp, w, h);
    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 4;
            let l = lum(rgba[i], rgba[i + 1], rgba[i + 2]);
            let crushed = ((l - 0.5) * (1.0 + contrast * 3.2) + 0.5).clamp(0.0, 1.0);
            let gv = (crushed * 180.0 + 20.0) as u8;
            let gg = (crushed * 220.0 * (0.7 + green * 0.3) + 30.0) as u8;
            let gb = (crushed * 120.0 + 15.0) as u8;
            rgba[i] = lerp_u8(gv, gg, green * 0.3);
            rgba[i + 1] = gg;
            rgba[i + 2] = gb;
            if y % 2 == 0 {
                rgba[i] = lerp_u8(rgba[i], 0, 0.12);
                rgba[i + 1] = lerp_u8(rgba[i + 1], 0, 0.12);
                rgba[i + 2] = lerp_u8(rgba[i + 2], 0, 0.12);
            }
        }
    }

    // Stutter feel — hold frame
    if state.frame % 4 != 0 && state.frame % 4 != 1 {
        // slight dim on "duplicate" frames
        for px in rgba.chunks_mut(4) {
            px[0] = lerp_u8(px[0], 0, 0.06);
            px[1] = lerp_u8(px[1], 0, 0.06);
            px[2] = lerp_u8(px[2], 0, 0.06);
        }
    }

    osd::burn_cctv_stamp(&mut rgba, w, h, state.frame);
    rgba
}
