//! CYBER — 90s Arcade Trinitron CRT. RGB triad shadow mask, phosphor beam bleed & matrix tint.

use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, _state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let grid_intensity = p.v(1); // 0.0 .. 1.0
    let beam_bleed = p.v(2); // 0.0 .. 1.0
    let tint_mode = p.v(3); // 0.0..0.33 Trinitron RGB, 0.33..0.66 Amber Terminal, 0.66..1.0 Matrix Green

    let mut out = vec![0u8; ww * hh * 4];
    let shift_offset = (beam_bleed * 4.0) as usize;

    for y in 0..hh {
        // Horizontal CRT scanline dampening
        let scanline = if y % 2 == 0 {
            1.0 - grid_intensity * 0.35
        } else {
            1.0
        };

        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let o = (y * ww + x) * 4;

            // Chromatic shift with beam bleed
            let r_idx = (y * ww + (x + shift_offset).min(ww - 1)) * 3;
            let b_idx = (y * ww + x.saturating_sub(shift_offset)) * 3;

            let mut r = rgb[r_idx] as f32 / 255.0;
            let mut g = rgb[i + 1] as f32 / 255.0;
            let mut b = rgb[b_idx + 2] as f32 / 255.0;

            // RGB triad pixel shadow mask
            if grid_intensity > 0.01 {
                let sub_column = x % 3;
                let mask = match sub_column {
                    0 => (1.0, 1.0 - grid_intensity * 0.6, 1.0 - grid_intensity * 0.6), // Red phosphor stripe
                    1 => (1.0 - grid_intensity * 0.6, 1.0, 1.0 - grid_intensity * 0.6), // Green phosphor stripe
                    _ => (1.0 - grid_intensity * 0.6, 1.0 - grid_intensity * 0.6, 1.0), // Blue phosphor stripe
                };
                r *= mask.0 * scanline;
                g *= mask.1 * scanline;
                b *= mask.2 * scanline;
            }

            // Apply tint modes
            if tint_mode >= 0.35 && tint_mode < 0.70 {
                // Amber Terminal Phosphor
                let luma = 0.299 * r + 0.587 * g + 0.114 * b;
                r = luma * 1.0;
                g = luma * 0.68;
                b = luma * 0.08;
            } else if tint_mode >= 0.70 {
                // Matrix Emerald Terminal Phosphor
                let luma = 0.299 * r + 0.587 * g + 0.114 * b;
                r = luma * 0.15;
                g = luma * 1.0;
                b = luma * 0.35;
            }

            out[o] = (r * 255.0).clamp(0.0, 255.0) as u8;
            out[o + 1] = (g * 255.0).clamp(0.0, 255.0) as u8;
            out[o + 2] = (b * 255.0).clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;
        }
    }

    out
}
