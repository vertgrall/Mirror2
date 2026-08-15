//! POSSESS — drag the mouse to burn afterimages into the frame. Colors haunt where you touch.

use super::ops::sample_rgb;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &mut VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let radius = p.v(1) * 110.0 + 28.0;
    let decay = 1.0 - p.v(2) * 0.035;
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
        let stamp = 0.55 + glow * 0.35;

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
                let falloff = (1.0 - d2.sqrt() / radius).powf(1.1);
                let i = (y as usize * ww + x as usize) * 3;
                let si = y as usize * ww + x as usize;
                let add = stamp * falloff;
                state.burn_r[si] = (state.burn_r[si] + rgb[i] as f32 * add).min(255.0);
                state.burn_g[si] = (state.burn_g[si] + rgb[i + 1] as f32 * add).min(255.0);
                state.burn_b[si] = (state.burn_b[si] + rgb[i + 2] as f32 * add).min(255.0);
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
            let burn_l = (br.max(bg).max(bb)) / 255.0;
            let mix = (burn_l * 1.25 + glow * 0.35).clamp(0.0, 0.94);

            r = r * (1.0 - mix) + br * mix;
            g = g * (1.0 - mix) + bg * mix;
            b = b * (1.0 - mix) + bb * mix;

            if glow > 0.01 && burn_l > 0.04 {
                let bloom = glow * burn_l * 70.0;
                r = (r + bloom).min(255.0);
                g = (g + bloom * 0.65).min(255.0);
                b = (b + bloom * 0.35).min(255.0);
            }

            if burn_l > 0.15 {
                let sx = (x as f32 + glow * 10.0).clamp(0.0, w as f32 - 1.001);
                let (fr, fg, fb) = sample_rgb(rgb, w, h, sx, y as f32);
                let fringe = (burn_l * 0.35).min(0.45);
                r = r * (1.0 - fringe) + fr as f32 * fringe;
                g = g * (1.0 - fringe) + fg as f32 * fringe;
                b = b * (1.0 - fringe) + fb as f32 * fringe;
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
