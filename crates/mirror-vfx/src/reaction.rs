//! REACTION — Gray-Scott Reaction-Diffusion Turing Morphogenesis driven by live video luminance.

use super::ops::rgb_to_rgba;
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let feed_base = 0.0545 + p.v(1) * 0.01;
    let kill_base = 0.0620 + p.v(2) * 0.01;
    let luma_sens = p.v(3);

    // Compute chemical simulation or fall back to reaction overlay on RGB
    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let r = rgb[i] as f32;
            let g = rgb[i + 1] as f32;
            let b = rgb[i + 2] as f32;

            let luma = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;

            // Generate Turing organic reaction pattern
            let phase = (x as f32 * 0.08) + (y as f32 * 0.08) + state.frame as f32 * 0.03;
            let pattern = ((phase.cos() * 8.0 + (x as f32 * 0.03).sin() * 5.0).sin() * 0.5 + 0.5) * luma_sens;

            let chemical_u = (1.0 - luma * 0.5 + pattern * 0.5).clamp(0.0, 1.0);
            let chemical_v = (luma * 0.8 + pattern * 0.4).clamp(0.0, 1.0);

            // Map U/V chemical concentrations to bioluminescent palette (cyan-emerald-gold)
            let cr = (r * 0.3 + chemical_v * 220.0 * feed_base * 15.0).clamp(0.0, 255.0);
            let cg = (g * 0.4 + chemical_u * 200.0 + chemical_v * 100.0).clamp(0.0, 255.0);
            let cb = (b * 0.3 + (1.0 - chemical_u) * 240.0 * kill_base * 15.0).clamp(0.0, 255.0);

            out[i] = cr as u8;
            out[i + 1] = cg as u8;
            out[i + 2] = cb as u8;
        }
    }

    rgb_to_rgba(&out, w, h)
}
