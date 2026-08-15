//! SMUDGE — drag the mouse to smear and bleed color. Finger-painting on the live feed.

use super::ops::{sample_rgb, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

const TAPS: i32 = 12;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let radius = p.v(1) * 90.0 + 24.0;
    let drag = p.v(2);
    let bleed = p.v(3);

    let px = state.pointer_x * w as f32;
    let py = state.pointer_y * h as f32;
    let active = state.pointer_down;
    let strength = if active { 1.0 } else { 0.0 };

    let prior = state.prev_rgb();
    let mut out = vec![0u8; ww * hh * 4];
    let xmax = w as f32 - 1.001;
    let ymax = h as f32 - 1.001;

    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let mut r = rgb[i] as f32;
            let mut g = rgb[i + 1] as f32;
            let mut b = rgb[i + 2] as f32;

            if strength > 0.01 {
                let dx = x as f32 - px;
                let dy = y as f32 - py;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < radius {
                    let falloff = (1.0 - dist / radius).powf(1.6);
                    let smear = drag * falloff * strength;

                    // Tangent smear — colors drag perpendicular to the radius (finger swirl).
                    let len = dist.max(1.0);
                    let tx = -dy / len;
                    let ty = dx / len;
                    let trail = radius * smear * 0.85;

                    let mut sr = 0.0f32;
                    let mut sg = 0.0f32;
                    let mut sb = 0.0f32;
                    let mut wsum = 0.0f32;

                    for tap in 0..=TAPS {
                        let t = tap as f32 / TAPS as f32;
                        let off = trail * t;
                        let sx = (x as f32 + tx * off).clamp(0.0, xmax);
                        let sy = (y as f32 + ty * off).clamp(0.0, ymax);
                        let weight = (1.0 - t).powf(1.2) * falloff;
                        let (tr, tg, tb) = sample_rgb(rgb, w, h, sx, sy);
                        sr += tr as f32 * weight;
                        sg += tg as f32 * weight;
                        sb += tb as f32 * weight;
                        wsum += weight;
                    }

                    if wsum > 0.0 {
                        r = r * (1.0 - smear) + (sr / wsum) * smear;
                        g = g * (1.0 - smear) + (sg / wsum) * smear;
                        b = b * (1.0 - smear) + (sb / wsum) * smear;
                    }

                    // Radial pull toward pointer — wet paint gathers.
                    let pull = smear * 0.35;
                    let sx = (x as f32 - dx * pull * 0.15).clamp(0.0, xmax);
                    let sy = (y as f32 - dy * pull * 0.15).clamp(0.0, ymax);
                    let (pr, pg, pb) = sample_rgb(rgb, w, h, sx, sy);
                    r = r * (1.0 - pull) + pr as f32 * pull;
                    g = g * (1.0 - pull) + pg as f32 * pull;
                    b = b * (1.0 - pull) + pb as f32 * pull;
                }
            }

            if let Some(prior) = prior {
                if prior.len() == rgb.len() && bleed > 0.01 {
                    let ghost = bleed * 0.55 * strength.max(0.15);
                    r = r * (1.0 - ghost) + prior[i] as f32 * ghost;
                    g = g * (1.0 - ghost) + prior[i + 1] as f32 * ghost;
                    b = b * (1.0 - ghost) + prior[i + 2] as f32 * ghost;
                }
            }

            let o = (y * ww + x) * 4;
            out[o] = r.clamp(0.0, 255.0) as u8;
            out[o + 1] = g.clamp(0.0, 255.0) as u8;
            out[o + 2] = b.clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;
        }
    }

    out
}
