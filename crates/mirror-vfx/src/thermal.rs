//! THERMAL — FLIR / Heat Vision. False-color ironbow & rainbow palette mapping.

use super::ops::hash2d;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let heat_bias = p.v(1); // 0.0 .. 1.0 (contrast / threshold shift)
    let palette_mode = p.v(2); // 0.0..0.33 Ironbow, 0.33..0.66 Rainbow, 0.66..1.0 Cold-IR
    let noise_amount = p.v(3); // 0.0 .. 1.0 sensor noise

    let mut out = vec![0u8; ww * hh * 4];
    let frame_seed = (state.frame as u32).wrapping_mul(1103515245);

    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let o = (y * ww + x) * 4;

            let r_in = rgb[i] as f32 / 255.0;
            let g_in = rgb[i + 1] as f32 / 255.0;
            let b_in = rgb[i + 2] as f32 / 255.0;

            // Perceptual luma
            let mut luma = 0.299 * r_in + 0.587 * g_in + 0.114 * b_in;

            // Apply heat contrast bias
            luma = ((luma - 0.5) * (1.0 + heat_bias * 1.5) + 0.5 + (heat_bias - 0.5) * 0.2).clamp(0.0, 1.0);

            // Add sensor thermal noise
            if noise_amount > 0.01 {
                let n = hash2d(x as f32 + frame_seed as f32 * 0.001, y as f32);
                luma = (luma + (n - 0.5) * noise_amount * 0.15).clamp(0.0, 1.0);
            }

            let (r, g, b) = if palette_mode < 0.35 {
                // Ironbow Palette (Navy -> Blue -> Violet -> Crimson -> Amber -> White)
                ironbow(luma)
            } else if palette_mode < 0.70 {
                // Rainbow Thermal (Black -> Blue -> Cyan -> Green -> Yellow -> Red -> White)
                rainbow(luma)
            } else {
                // Cold Infrared (Dark Cyan -> Blue -> Purple -> Hot Magenta -> White)
                cold_ir(luma)
            };

            out[o] = (r * 255.0).clamp(0.0, 255.0) as u8;
            out[o + 1] = (g * 255.0).clamp(0.0, 255.0) as u8;
            out[o + 2] = (b * 255.0).clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;
        }
    }

    out
}

fn ironbow(t: f32) -> (f32, f32, f32) {
    if t < 0.25 {
        let s = t / 0.25;
        (0.04 + s * 0.08, 0.02 + s * 0.10, 0.18 + s * 0.45)
    } else if t < 0.50 {
        let s = (t - 0.25) / 0.25;
        (0.12 + s * 0.65, 0.12 - s * 0.08, 0.63 - s * 0.25)
    } else if t < 0.75 {
        let s = (t - 0.50) / 0.25;
        (0.77 + s * 0.23, 0.04 + s * 0.60, 0.38 - s * 0.35)
    } else {
        let s = (t - 0.75) / 0.25;
        (1.0, 0.64 + s * 0.36, 0.03 + s * 0.97)
    }
}

fn rainbow(t: f32) -> (f32, f32, f32) {
    if t < 0.20 {
        let s = t / 0.20;
        (0.0, 0.0, s * 0.8)
    } else if t < 0.40 {
        let s = (t - 0.20) / 0.20;
        (0.0, s * 0.9, 0.8 + s * 0.2)
    } else if t < 0.60 {
        let s = (t - 0.40) / 0.20;
        (s * 0.9, 0.9 + s * 0.1, 1.0 - s * 1.0)
    } else if t < 0.80 {
        let s = (t - 0.60) / 0.20;
        (0.9 + s * 0.1, 1.0 - s * 0.9, 0.0)
    } else {
        let s = (t - 0.80) / 0.20;
        (1.0, s * 1.0, s * 1.0)
    }
}

fn cold_ir(t: f32) -> (f32, f32, f32) {
    if t < 0.33 {
        let s = t / 0.33;
        (0.02 + s * 0.10, 0.15 + s * 0.45, 0.35 + s * 0.55)
    } else if t < 0.66 {
        let s = (t - 0.33) / 0.33;
        (0.12 + s * 0.80, 0.60 - s * 0.45, 0.90 - s * 0.20)
    } else {
        let s = (t - 0.66) / 0.34;
        (0.92 + s * 0.08, 0.15 + s * 0.85, 0.70 + s * 0.30)
    }
}
