//! ECHO — temporal feedback infinity mirror tunnel. Recursive scaling, rotation, and color trail decay.

use super::ops::{lerp_u8, sample_rgb, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let zoom = 0.96 - p.v(1) * 0.12; // 0.84 to 0.96 scale factor
    let spin = (p.v(2) - 0.5) * 0.08; // subtle rotational twist angle
    let decay = p.v(3) * 0.75 + 0.15; // feedback decay rate

    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;

    let prev = state.prev_rgb();

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        let yf = y as f32 - cy;
        for x in 0..ww {
            let xf = x as f32 - cx;
            let i = (y * ww + x) * 3;

            let cur_r = rgb[i];
            let cur_g = rgb[i + 1];
            let cur_b = rgb[i + 2];

            if let Some(prior) = prev {
                // Apply reverse scale and rotation to sample recursive feedback frame
                let cos_a = spin.cos();
                let sin_a = spin.sin();

                let rx = (xf * cos_a - yf * sin_a) / zoom + cx;
                let ry = (xf * sin_a + yf * cos_a) / zoom + cy;

                let sx = rx.clamp(0.0, w as f32 - 1.001);
                let sy = ry.clamp(0.0, h as f32 - 1.001);

                let (pr, pg, pb) = sample_rgb(prior, w, h, sx, sy);

                // Blend live frame with temporal feedback trail
                out[i] = lerp_u8(cur_r, pr, decay);
                out[i + 1] = lerp_u8(cur_g, pg, decay);
                out[i + 2] = lerp_u8(cur_b, pb, decay);
            } else {
                out[i] = cur_r;
                out[i + 1] = cur_g;
                out[i + 2] = cur_b;
            }
        }
    }
    rgb_to_rgba(&out, w, h)
}
