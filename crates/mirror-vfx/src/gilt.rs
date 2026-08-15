//! GILT — kintsugi. Live edges crack; gold leaf fills the seams.

use super::ops::{hash2d, lerp_u8, lum, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

fn luma_at(rgb: &[u8], w: u32, h: u32, x: i32, y: i32) -> f32 {
    let x = x.clamp(0, w as i32 - 1) as u32;
    let y = y.clamp(0, h as i32 - 1) as u32;
    let i = ((y * w + x) * 3) as usize;
    lum(rgb[i], rgb[i + 1], rgb[i + 2])
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let cracks = p.v(1);
    let gold_amt = p.v(2);
    let flow = p.v(3);
    let t = state.frame as f32 * (0.04 + flow * 0.10);

    let thresh = 0.045 + (1.0 - cracks) * 0.12;
    let mut mag = vec![0f32; ww * hh];
    for y in 0..hh {
        for x in 0..ww {
            let gx = luma_at(rgb, w, h, x as i32 + 1, y as i32)
                - luma_at(rgb, w, h, x as i32 - 1, y as i32);
            let gy = luma_at(rgb, w, h, x as i32, y as i32 + 1)
                - luma_at(rgb, w, h, x as i32, y as i32 - 1);
            mag[y * ww + x] = (gx * gx + gy * gy).sqrt();
        }
    }

    let mut out = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let m = mag[y * ww + x];
            let vein = ((m - thresh) / 0.18).clamp(0.0, 1.0).powf(0.55);
            let n = hash2d(x as f32 * 0.37 + t * 2.1, y as f32 * 0.29 - t);
            let shimmer = 0.55 + 0.45 * ((t * 1.7 + n * 6.3).sin() * 0.5 + 0.5);
            let a = vein * gold_amt * (0.65 + n * 0.35) * shimmer;

            let r = rgb[i];
            let g = rgb[i + 1];
            let b = rgb[i + 2];
            let dry = 1.0 - gold_amt * 0.18;
            let base_r = (r as f32 * dry) as u8;
            let base_g = (g as f32 * dry * 0.96) as u8;
            let base_b = (b as f32 * dry * 0.90) as u8;

            let gr = lerp_u8(212, 255, n);
            let gg = lerp_u8(150, 210, shimmer);
            let gb = lerp_u8(42, 88, 1.0 - n);

            out[i] = lerp_u8(base_r, gr, a);
            out[i + 1] = lerp_u8(base_g, gg, a);
            out[i + 2] = lerp_u8(base_b, gb, a);
        }
    }
    rgb_to_rgba(&out, w, h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Look, LookParams, VfxState};

    #[test]
    fn gold_reads_on_a_hard_edge() {
        let w = 48u32;
        let h = 32u32;
        let mut rgb = vec![30u8; (w * h * 3) as usize];
        for y in 0..h {
            for x in 24..w {
                let i = ((y * w + x) * 3) as usize;
                rgb[i] = 220;
                rgb[i + 1] = 200;
                rgb[i + 2] = 180;
            }
        }
        let out = apply(&rgb, w, h, &VfxState::default(), &LookParams::defaults(Look::Gilt));
        let seam = ((16 * w + 24) * 4) as usize;
        let flat = ((16 * w + 8) * 4) as usize;
        assert!(
            out[seam] > out[flat] + 20,
            "kintsugi seam should run hotter than the field (seam {} field {})",
            out[seam],
            out[flat]
        );
    }
}
