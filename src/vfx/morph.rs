//! Morphological edge network — gradient → dilate → ink.
//!
//! Base pass is the original fixed operator (double dilate3).
//! Sliders: wet (mix back to photo), edge threshold, ink, fill.

use super::ops::{dilate3, gray, lerp_u8};
use super::params::LookParams;

pub fn apply(rgb: &[u8], w: u32, h: u32, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let g = gray(rgb, w, h);

    let threshold = p.v(1);
    let ink = p.v(2);
    let fill_cap = p.v(3);

    // Original morph: interior gradient magnitude → threshold mask.
    let mut mask = vec![0u8; ww * hh];
    for y in 1..hh - 1 {
        for x in 1..ww - 1 {
            let i = y * ww + x;
            let gx = g[i + 1] - g[i - 1];
            let gy = g[i + ww] - g[i - ww];
            let mag = (gx * gx + gy * gy).sqrt();
            mask[i] = if mag > threshold { 255 } else { 0 };
        }
    }

    // Original signature: always two 3×3 dilations.
    let edges = dilate3(&dilate3(&mask, w, h), w, h);

    let mut out = vec![0u8; ww * hh * 4];
    let paper = (242u8, 240u8, 234u8);
    let ink_c = (18u8, 20u8, 28u8);

    for y in 0..hh {
        for x in 0..ww {
            let i = y * ww + x;
            let edge = edges[i] as f32 / 255.0;
            let shade = g[i];
            let fill = (shade * 0.35).min(fill_cap);
            let t = (edge * ink + fill).clamp(0.0, 1.0);
            let o = i * 4;
            out[o] = lerp_u8(paper.0, ink_c.0, t);
            out[o + 1] = lerp_u8(paper.1, ink_c.1, t);
            out[o + 2] = lerp_u8(paper.2, ink_c.2, t);
            out[o + 3] = 255;
        }
    }
    out
}
