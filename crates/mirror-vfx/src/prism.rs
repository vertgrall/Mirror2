//! PRISM — multi-spectrum glass prism refraction. Triadic RGB spectral shear and glass flare.

use super::ops::{sample_rgb, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let split = p.v(1) * 35.0; // prismatic offset distance
    let angle = (p.v(2) * 6.28) + state.frame as f32 * 0.02; // rotation angle
    let chroma = p.v(3); // spectral tint boost

    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;

    let dx = angle.cos() * split;
    let dy = angle.sin() * split;

    let mut out = vec![0u8; ww * hh * 3];

    for y in 0..hh {
        let yf = y as f32;
        for x in 0..ww {
            let xf = x as f32;
            let i = (y * ww + x) * 3;

            // Red channel sample (shifted +dx, +dy)
            let rx = (xf + dx).clamp(0.0, w as f32 - 1.001);
            let ry = (yf + dy).clamp(0.0, h as f32 - 1.001);
            let (r_val, _, _) = sample_rgb(rgb, w, h, rx, ry);

            // Green channel sample (center)
            let (_, g_val, _) = sample_rgb(rgb, w, h, xf, yf);

            // Blue channel sample (shifted -dx, -dy)
            let bx = (xf - dx).clamp(0.0, w as f32 - 1.001);
            let by = (yf - dy).clamp(0.0, h as f32 - 1.001);
            let (_, _, b_val) = sample_rgb(rgb, w, h, bx, by);

            let mut r = r_val as f32 * (1.0 + chroma * 0.25);
            let mut g = g_val as f32 * (1.0 + chroma * 0.15);
            let mut b = b_val as f32 * (1.0 + chroma * 0.35);

            // Prism glass edge reflection glare
            let dist_center = ((xf - cx) * (xf - cx) + (yf - cy) * (yf - cy)).sqrt();
            if (dist_center - split * 2.0).abs() < 12.0 {
                r = (r + 40.0 * chroma).min(255.0);
                g = (g + 50.0 * chroma).min(255.0);
                b = (b + 60.0 * chroma).min(255.0);
            }

            out[i] = r.clamp(0.0, 255.0) as u8;
            out[i + 1] = g.clamp(0.0, 255.0) as u8;
            out[i + 2] = b.clamp(0.0, 255.0) as u8;
        }
    }
    rgb_to_rgba(&out, w, h)
}
