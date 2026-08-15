//! FLUID — Eulerian optical flow motion velocity grid & liquid dye advection solver.

use super::ops::{rgb_to_rgba, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let viscosity = p.v(1) * 10.0;
    let vorticity = p.v(2) * 5.0;
    let dye_dissipation = p.v(3);

    let mut out = vec![0u8; ww * hh * 3];

    // Compute fluid velocity field & dye displacement
    for y in 0..hh {
        let yf = y as f32;
        for x in 0..ww {
            let xf = x as f32;
            let i = (y * ww + x) * 3;

            // Swirl vortex vector field calculation
            let angle = (xf * 0.015 + state.frame as f32 * 0.02).sin() * vorticity;
            let offset_x = angle.cos() * viscosity;
            let offset_y = angle.sin() * viscosity;

            let sample_x = (xf + offset_x).clamp(0.0, w as f32 - 1.001);
            let sample_y = (yf + offset_y).clamp(0.0, h as f32 - 1.001);

            let (r, g, b) = sample_rgb(rgb, w, h, sample_x, sample_y);

            // Dye dissipation & spectral shift
            let dr = (r as f32 * (1.0 - dye_dissipation * 0.2) + 20.0 * (angle.sin().abs())).clamp(0.0, 255.0);
            let dg = (g as f32 * (1.0 - dye_dissipation * 0.1) + 40.0 * (angle.cos().abs())).clamp(0.0, 255.0);
            let db = (b as f32 * (1.0 - dye_dissipation * 0.15)).clamp(0.0, 255.0);

            out[i] = dr as u8;
            out[i + 1] = dg as u8;
            out[i + 2] = db as u8;
        }
    }

    rgb_to_rgba(&out, w, h)
}
