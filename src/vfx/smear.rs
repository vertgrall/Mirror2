//! SMEAR — motion-directed color drag. Heavy temporal blur + warm/cold trail.

use super::ops::{lum, sample_rgb, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

const TAPS: i32 = 16;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let warm = p.v(1);
    let lag = p.v(2);
    let spread = p.v(3);

    let Some(prior) = state.prev_rgb() else {
        return rgb_to_rgba(rgb, w, h);
    };
    if prior.len() != rgb.len() {
        return rgb_to_rgba(rgb, w, h);
    }

    // spread 1.0 ≈ 140px trail at preview res
    let spread_px = spread * 140.0;
    let lag_strength = lag * 0.96;

    let mut motion = vec![0f32; ww * hh];
    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let cr = rgb[i] as f32;
            let cg = rgb[i + 1] as f32;
            let cb = rgb[i + 2] as f32;
            let pr = prior[i] as f32;
            let pg = prior[i + 1] as f32;
            let pb = prior[i + 2] as f32;
            motion[y * ww + x] =
                ((cr - pr).abs() + (cg - pg).abs() + (cb - pb).abs()) / (255.0 * 3.0);
        }
    }

    let mut out = vec![0u8; ww * hh * 4];
    let xmax = w as f32 - 1.001;
    let ymax = h as f32 - 1.001;
    let fallback_angle = (state.frame as f32 * 0.018).sin() * 0.35;
    let fb_dx = fallback_angle.cos() * 0.9 + 0.1;
    let fb_dy = fallback_angle.sin() * 0.3;

    for y in 1..hh - 1 {
        for x in 1..ww - 1 {
            let idx = y * ww + x;
            let i = idx * 3;
            let cr = rgb[i] as f32;
            let cg = rgb[i + 1] as f32;
            let cb = rgb[i + 2] as f32;

            let m = motion[idx].min(1.0);
            let gx = motion[idx + 1] - motion[idx - 1];
            let gy = motion[idx + ww] - motion[idx - ww];
            let gl = (gx * gx + gy * gy).sqrt();
            let (vx, vy) = if gl > 0.002 {
                (gx / gl, gy / gl)
            } else {
                (fb_dx, fb_dy)
            };

            // Motion opens the shutter longer; still blur a little when still.
            let trail = spread_px * (0.55 + m * 2.8);

            let mut r = 0.0f32;
            let mut g = 0.0f32;
            let mut b = 0.0f32;
            let mut wsum = 0.0f32;

            for tap in 0..=TAPS {
                let t = tap as f32 / TAPS as f32;
                let off = trail * t;
                let sx = (x as f32 - vx * off).clamp(0.0, xmax);
                let sy = (y as f32 - vy * off).clamp(0.0, ymax);
                let weight = (1.0 - t).powf(1.35);
                let curr_w = weight * (0.55 + m * 0.35);
                let prev_w = weight * (0.45 + lag_strength * 0.55);

                let (tr, tg, tb) = sample_rgb(rgb, w, h, sx, sy);
                let (pr, pg, pb) = sample_rgb(prior, w, h, sx, sy);

                r += tr as f32 * curr_w + pr as f32 * prev_w;
                g += tg as f32 * curr_w + pg as f32 * prev_w;
                b += tb as f32 * curr_w + pb as f32 * prev_w;
                wsum += curr_w + prev_w;
            }

            if wsum > 0.0 {
                r /= wsum;
                g /= wsum;
                b /= wsum;
            } else {
                r = cr;
                g = cg;
                b = cb;
            }

            // Hard ghost of last frame on top of the streak.
            let pr = prior[i] as f32;
            let pg = prior[i + 1] as f32;
            let pb = prior[i + 2] as f32;
            let ghost = lag_strength * (0.42 + m * 0.58);
            r = r * (1.0 - ghost) + pr * ghost;
            g = g * (1.0 - ghost) + pg * ghost;
            b = b * (1.0 - ghost) + pb * ghost;

            let smear = (ghost + m * lag_strength * 0.35).min(1.0);
            let warm_amt = warm * smear * 34.0;
            let cold_amt = (1.0 - warm) * smear * 34.0;
            r = (r + warm_amt - cold_amt * 0.45).clamp(0.0, 255.0);
            g = (g + warm_amt * 0.38).clamp(0.0, 255.0);
            b = (b + cold_amt - warm_amt * 0.38).clamp(0.0, 255.0);

            // Slight streak softening — crushed halation on bright motion.
            let l = lum(r as u8, g as u8, b as u8);
            if l > 0.55 && m > 0.08 {
                let bloom = (l - 0.55) * smear * 0.22;
                r = (r + bloom * 55.0).min(255.0);
                g = (g + bloom * 40.0).min(255.0);
                b = (b + bloom * 18.0).min(255.0);
            }

            let o = idx * 4;
            out[o] = r as u8;
            out[o + 1] = g as u8;
            out[o + 2] = b as u8;
            out[o + 3] = 255;
        }
    }

    // Copy border pixels untouched (inner loop skips edges).
    for y in 0..hh {
        for x in 0..ww {
            if y > 0 && y + 1 < hh && x > 0 && x + 1 < ww {
                continue;
            }
            let i = (y * ww + x) * 3;
            let o = (y * ww + x) * 4;
            out[o] = rgb[i];
            out[o + 1] = rgb[i + 1];
            out[o + 2] = rgb[i + 2];
            out[o + 3] = 255;
        }
    }

    out
}
