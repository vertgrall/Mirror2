//! Morphological ink — multiply/darken edges on the live photo.
//!
//! Color stays in the frame; edges darken like ink on gloss. Slot 0 (**lines**)
//! scales stroke weight. **trail** smears prior ink along motion.

use super::ops::{dilate3, gray, lum, open3};
use super::params::LookParams;
use super::state::VfxState;

const TAPS: i32 = 14;
const MAX_DARKEN: f32 = 0.9;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let lines = p.v(0);
    let threshold = p.v(1);
    let ink = p.v(2);
    let trail = p.v(3);

    let curr = ink_layer(rgb, w, h, threshold, ink, lines);
    let Some(prev_ink) = state.prev_morph() else {
        return curr;
    };
    if prev_ink.len() != curr.len() {
        return curr;
    }
    let Some(prior_rgb) = state.prev_rgb() else {
        return curr;
    };
    if prior_rgb.len() != rgb.len() {
        return curr;
    }

    stretch_ink(
        &curr,
        prev_ink,
        rgb,
        prior_rgb,
        w,
        h,
        trail,
        state,
    )
}

fn ink_layer(rgb: &[u8], w: u32, h: u32, threshold: f32, ink: f32, lines: f32) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let g = gray(rgb, w, h);

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

    let edges = open3(&dilate3(&mask, w, h), w, h);

    let mut out = vec![0u8; ww * hh * 4];
    let weight = lines.clamp(0.0, 1.0);

    for y in 0..hh {
        for x in 0..ww {
            let i = y * ww + x;
            let si = i * 3;
            let edge = edges[i] as f32 / 255.0;
            let t = (edge * ink * weight).clamp(0.0, 1.0);
            let factor = 1.0 - t * MAX_DARKEN;
            let o = i * 4;
            out[o] = (rgb[si] as f32 * factor).clamp(0.0, 255.0) as u8;
            out[o + 1] = (rgb[si + 1] as f32 * factor).clamp(0.0, 255.0) as u8;
            out[o + 2] = (rgb[si + 2] as f32 * factor).clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;
        }
    }
    out
}

fn stretch_ink(
    curr: &[u8],
    prev: &[u8],
    rgb: &[u8],
    prior_rgb: &[u8],
    w: u32,
    h: u32,
    trail: f32,
    state: &VfxState,
) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let spread_px = trail * 180.0;
    let lag = trail * 0.88;
    let xmax = w as f32 - 1.001;
    let ymax = h as f32 - 1.001;

    let mut motion = vec![0f32; ww * hh];
    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            motion[y * ww + x] = ((rgb[i] as f32 - prior_rgb[i] as f32).abs()
                + (rgb[i + 1] as f32 - prior_rgb[i + 1] as f32).abs()
                + (rgb[i + 2] as f32 - prior_rgb[i + 2] as f32).abs())
                / (255.0 * 3.0);
        }
    }

    let fallback_angle = (state.frame as f32 * 0.014).sin() * 0.22;
    let fb_dx = fallback_angle.cos() * 0.75 + 0.18;
    let fb_dy = fallback_angle.sin() * 0.22;

    let mut out = curr.to_vec();

    for y in 1..hh - 1 {
        for x in 1..ww - 1 {
            let idx = y * ww + x;
            let o = idx * 4;
            let si = idx * 3;
            let m = motion[idx].min(1.0);

            let gx = motion[idx + 1] - motion[idx - 1];
            let gy = motion[idx + ww] - motion[idx - ww];
            let gl = (gx * gx + gy * gy).sqrt();
            let (vx, vy) = if gl > 0.0015 {
                (gx / gl, gy / gl)
            } else {
                (fb_dx, fb_dy)
            };

            let src = (rgb[si], rgb[si + 1], rgb[si + 2]);
            let ink_now = ink_amount(curr[o], curr[o + 1], curr[o + 2], src);
            let ink_was = ink_amount(prev[o], prev[o + 1], prev[o + 2], src);
            if m < 0.004 && ink_now < 0.05 && ink_was < 0.05 {
                continue;
            }

            let trail_len = spread_px * (0.35 + m * 3.2);
            let mut r = 0.0f32;
            let mut g = 0.0f32;
            let mut b = 0.0f32;
            let mut wsum = 0.0f32;

            for tap in 0..=TAPS {
                let t = tap as f32 / TAPS as f32;
                let off = trail_len * t;
                let sx = (x as f32 - vx * off).clamp(0.0, xmax);
                let sy = (y as f32 - vy * off).clamp(0.0, ymax);
                let weight = (1.0 - t).powf(1.15);
                let cw = weight * (0.45 + m * 0.4);
                let pw = weight * (0.55 + lag * 0.5);

                let (cr, cg, cb) = sample_rgba(curr, w, h, sx, sy);
                let (pr, pg, pb) = sample_rgba(prev, w, h, sx, sy);

                r += cr as f32 * cw + pr as f32 * pw;
                g += cg as f32 * cw + pg as f32 * pw;
                b += cb as f32 * cw + pb as f32 * pw;
                wsum += cw + pw;
            }

            if wsum > 0.0 {
                r /= wsum;
                g /= wsum;
                b /= wsum;
            } else {
                r = curr[o] as f32;
                g = curr[o + 1] as f32;
                b = curr[o + 2] as f32;
            }

            let ghost = lag * (0.42 + m * 0.58);
            r = r * (1.0 - ghost) + prev[o] as f32 * ghost;
            g = g * (1.0 - ghost) + prev[o + 1] as f32 * ghost;
            b = b * (1.0 - ghost) + prev[o + 2] as f32 * ghost;

            let smeared = (
                r.clamp(0.0, 255.0) as u8,
                g.clamp(0.0, 255.0) as u8,
                b.clamp(0.0, 255.0) as u8,
            );
            let (fr, fg, fb) = pick_darker(
                (curr[o], curr[o + 1], curr[o + 2]),
                smeared,
                src,
            );
            out[o] = fr;
            out[o + 1] = fg;
            out[o + 2] = fb;
        }
    }

    out
}

fn ink_amount(r: u8, g: u8, b: u8, src: (u8, u8, u8)) -> f32 {
    let lo = lum(r, g, b);
    let hi = lum(src.0, src.1, src.2);
    if hi < 0.02 {
        return 0.0;
    }
    ((hi - lo) / hi).clamp(0.0, 1.0)
}

fn pick_darker(a: (u8, u8, u8), b: (u8, u8, u8), src: (u8, u8, u8)) -> (u8, u8, u8) {
    if ink_amount(b.0, b.1, b.2, src) >= ink_amount(a.0, a.1, a.2, src) {
        b
    } else {
        a
    }
}

fn sample_rgba(rgba: &[u8], w: u32, h: u32, fx: f32, fy: f32) -> (u8, u8, u8) {
    let x = fx.round() as i32;
    let y = fy.round() as i32;
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return (255, 255, 255);
    }
    let o = ((y as u32 * w + x as u32) as usize) * 4;
    (rgba[o], rgba[o + 1], rgba[o + 2])
}
