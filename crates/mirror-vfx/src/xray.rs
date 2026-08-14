//! XRAY — Fluoroscopy radiograph. Inverted high-contrast skeletal density & phosphor glow.

use super::ops::hash2d;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let bone_contrast = p.v(1); // 0.0 .. 1.0
    let phosphor_mode = p.v(2); // 0.0..0.33 Silver, 0.33..0.66 Cyan, 0.66..1.0 Emerald
    let flicker_amount = p.v(3); // 0.0 .. 1.0

    let mut out = vec![0u8; ww * hh * 4];

    // Cathode ray exposure flicker
    let flicker = if flicker_amount > 0.01 {
        let f = hash2d(state.frame as f32, 7.0);
        1.0 + (f - 0.5) * flicker_amount * 0.12
    } else {
        1.0
    };

    for y in 0..hh {
        let center_y = (y as f32 / hh as f32) - 0.5;
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let o = (y * ww + x) * 4;

            let r_in = rgb[i] as f32 / 255.0;
            let g_in = rgb[i + 1] as f32 / 255.0;
            let b_in = rgb[i + 2] as f32 / 255.0;

            let luma = 0.299 * r_in + 0.587 * g_in + 0.114 * b_in;

            // Invert & boost high contrast (radiograph negative)
            let mut inv = (1.0 - luma).powf(0.8 + (1.0 - bone_contrast) * 0.8);
            inv = ((inv - 0.2) * (1.2 + bone_contrast * 1.5)).clamp(0.0, 1.0) * flicker;

            // Heavy tube vignette
            let center_x = (x as f32 / ww as f32) - 0.5;
            let dist_sq = center_x * center_x + center_y * center_y;
            let vig = (1.0 - dist_sq * 1.4).clamp(0.1, 1.0);
            inv *= vig;

            // Map to selected phosphor tube tint
            let (r, g, b) = if phosphor_mode < 0.35 {
                // Silver Monochromatic
                (inv * 0.9, inv * 0.92, inv * 0.96)
            } else if phosphor_mode < 0.70 {
                // Cyan Fluoroscopy (Cyan-Blue Glow)
                (inv * 0.2, inv * 0.85, inv * 0.95)
            } else {
                // Emerald Radiograph (Green Phosphor)
                (inv * 0.25, inv * 0.95, inv * 0.45)
            };

            out[o] = (r * 255.0).clamp(0.0, 255.0) as u8;
            out[o + 1] = (g * 255.0).clamp(0.0, 255.0) as u8;
            out[o + 2] = (b * 255.0).clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;
        }
    }

    out
}
