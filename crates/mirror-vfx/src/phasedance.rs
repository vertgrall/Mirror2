//! PHASE — slow waves invert the subject and leak glitch underneath.

use super::ops::{hash2d, lerp_u8, rgb_to_rgba, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

fn is_subject(r: u8, g: u8, b: u8) -> bool {
    let y = r as f32 * 0.2126 + g as f32 * 0.7152 + b as f32 * 0.0722;
    let skin = r > 70 && r >= g.saturating_sub(8) && r > b && (r as i16 - g as i16).abs() < 90;
    skin || (y > 70.0 && y < 230.0 && r > 40)
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let invert_amt = p.v(1);
    let warp = p.v(2);
    let glitch = p.v(3);
    let t = state.frame as f32 * 0.045;

    let mut out = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        for x in 0..ww {
            let nx = x as f32 / w as f32;
            let ny = y as f32 / h as f32;
            let wave = (ny * 6.2 + t * 0.85).sin() * 0.55 + (nx * 4.1 - t * 0.55).sin() * 0.45;
            let shift = wave * warp * 22.0;
            let sx = x as f32 + shift;
            let sy = y as f32 + wave * warp * 8.0;
            let (mut r, mut g, mut b) = sample_rgb(rgb, w, h, sx, sy);

            if is_subject(r, g, b) {
                let ir = 255 - r;
                let ig = 255 - g;
                let ib = 255 - b;
                r = lerp_u8(r, ir, invert_amt * (0.55 + wave.abs() * 0.45));
                g = lerp_u8(g, ig, invert_amt * (0.55 + wave.abs() * 0.45));
                b = lerp_u8(b, ib, invert_amt * (0.55 + wave.abs() * 0.45));
            }

            if glitch > 0.04 && wave > 0.35 {
                let block = 4 + (glitch * 10.0) as u32;
                let bx = (x as u32 / block) * block;
                let by = (y as u32 / block) * block;
                let jx = hash2d(bx as f32 + t * 3.0, by as f32) * glitch * 18.0;
                let jy = hash2d(by as f32, bx as f32 - t) * glitch * 10.0;
                let (gr, gg, gb) = sample_rgb(rgb, w, h, bx as f32 + jx, by as f32 + jy);
                let leak = ((wave - 0.35) / 0.65).clamp(0.0, 1.0) * glitch;
                r = lerp_u8(r, gr.saturating_add(20), leak);
                g = lerp_u8(g, gg, leak * 0.7);
                b = lerp_u8(b, gb.saturating_add(30), leak);
            }

            let i = (y * ww + x) * 3;
            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
        }
    }
    rgb_to_rgba(&out, w, h)
}
