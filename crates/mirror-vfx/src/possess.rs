//! POSSESS — drag the mouse to burn afterimages into the frame. Colors haunt where you touch.

use super::ops::{sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &mut VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let radius = p.v(1) * 80.0 + 18.0;
    let decay = 1.0 - p.v(2) * 0.12;
    let glow = p.v(3);

    let n = ww * hh;
    state.ensure_burn(n);

    for i in 0..n {
        state.burn_r[i] *= decay;
        state.burn_g[i] *= decay;
        state.burn_b[i] *= decay;
    }

    if state.pointer_down {
        let px = state.pointer_x * w as f32;
        let py = state.pointer_y * h as f32;
        let r2 = radius * radius;

        let y0 = ((py - radius).floor() as i32).max(0) as u32;
        let y1 = ((py + radius).ceil() as i32).min(h as i32 - 1) as u32;
        let x0 = ((px - radius).floor() as i32).max(0) as u32;
        let x1 = ((px + radius).ceil() as i32).min(w as i32 - 1) as u32;

        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x as f32 - px;
                let dy = y as f32 - py;
                let d2 = dx * dx + dy * dy;
                if d2 > r2 {
                    continue;
                }
                let falloff = (1.0 - d2.sqrt() / radius).powf(1.4);
                let i = (y as usize * ww + x as usize) * 3;
                let si = y as usize * ww + x as usize;
                state.burn_r[si] = (state.burn_r[si] + rgb[i] as f32 * falloff * 0.35).min(255.0);
                state.burn_g[si] = (state.burn_g[si] + rgb[i + 1] as f32 * falloff * 0.35).min(255.0);
                state.burn_b[si] = (state.burn_b[si] + rgb[i + 2] as f32 * falloff * 0.35).min(255.0);
            }
        }
    }

    let mut out = vec![0u8; n * 4];
    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let si = y * ww + x;
            let mut r = rgb[i] as f32;
            let mut g = rgb[i + 1] as f32;
            let mut b = rgb[i + 2] as f32;

            let br = state.burn_r[si];
            let bg = state.burn_g[si];
            let bb = state.burn_b[si];
            let burn_l = (br + bg + bb) / (255.0 * 3.0);
            let mix = (burn_l * 1.8).min(1.0);

            r = r * (1.0 - mix) + br * mix;
            g = g * (1.0 - mix) + bg * mix;
            b = b * (1.0 - mix) + bb * mix;

            if glow > 0.01 && burn_l > 0.05 {
                let bloom = glow * burn_l * 40.0;
                r = (r + bloom).min(255.0);
                g = (g + bloom * 0.7).min(255.0);
                b = (b + bloom * 0.4).min(255.0);
            }

            // Slight spectral fringe on hot burns
            if burn_l > 0.2 {
                let sx = (x as f32 + glow * 6.0).clamp(0.0, w as f32 - 1.001);
                let (fr, _, _) = sample_rgb(rgb, w, h, sx, y as f32);
                r = r * 0.85 + fr as f32 * 0.15;
            }

            let o = si * 4;
            out[o] = r as u8;
            out[o + 1] = g as u8;
            out[o + 2] = b as u8;
            out[o + 3] = 255;
        }
    }
    out
}
