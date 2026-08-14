//! HOLO — cyberpunk holographic projection, cyan/magenta laser scanlines, beam power instability.

use super::ops::{hash2d, lerp_u8, lum, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let beams = p.v(1);
    let fringe = p.v(2);
    let flicker = p.v(3);

    // Power flicker instability
    let p_flicker = if flicker > 0.01 {
        let f = hash2d(state.frame as f32 * 0.4, 1.0);
        1.0 - (f * flicker * 0.35)
    } else {
        1.0
    };

    let shift_x = (fringe * 16.0) as i32;

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        // Laser scanline grid
        let beam_freq = 0.2 + beams * 0.8;
        let line_val = (y as f32 * beam_freq + state.frame as f32 * 0.15).sin();
        let scan_darken = 1.0 - (line_val.max(0.0) * 0.3 * beams);

        for x in 0..ww {
            let i = (y * ww + x) * 3;

            let rx = (x as i32 + shift_x).clamp(0, w as i32 - 1) as usize;
            let bx = (x as i32 - shift_x).clamp(0, w as i32 - 1) as usize;

            let r_si = (y * ww + rx) * 3;
            let b_si = (y * ww + bx) * 3;

            let l = lum(rgb[i], rgb[i + 1], rgb[i + 2]);

            // Cyan / Magenta holographic phosphor tinting
            let mut r = (rgb[r_si] as f32 * 0.3 + l * 180.0) * p_flicker * scan_darken;
            let mut g = (rgb[i + 1] as f32 * 0.7 + l * 220.0) * p_flicker * scan_darken;
            let mut b = (rgb[b_si + 2] as f32 * 0.9 + l * 240.0) * p_flicker * scan_darken;

            // Electric laser blue edge boost
            if g > 160.0 {
                b = (b * 1.15).min(255.0);
            }

            out[i] = r.clamp(0.0, 255.0) as u8;
            out[i + 1] = g.clamp(0.0, 255.0) as u8;
            out[i + 2] = b.clamp(0.0, 255.0) as u8;
        }
    }
    rgb_to_rgba(&out, w, h)
}
