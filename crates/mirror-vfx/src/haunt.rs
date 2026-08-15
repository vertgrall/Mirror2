//! HAUNT — hero look: drag to smear wet paint while ghost copies lurk and burn in.

use super::ops::{lum, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

const TAPS: i32 = 12;
const GHOST_DEPTHS: [usize; 4] = [6, 14, 24, 36];

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &mut VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let n = ww * hh;
    let smear = p.v(1);
    let ghosts = (p.v(2) * 3.0 + 1.0).round() as usize;
    let burn_amt = p.v(3);

    let radius = 36.0 + smear * 88.0;
    let r2 = radius * radius;

    state.ensure_paint(n);
    state.ensure_burn(n);

    let dry = 0.992 - smear * 0.012;
    let burn_decay = 1.0 - burn_amt * 0.025;
    for i in 0..n {
        state.paint_r[i] *= dry;
        state.paint_g[i] *= dry;
        state.paint_b[i] *= dry;
        state.paint_a[i] *= dry;
        state.burn_r[i] *= burn_decay;
        state.burn_g[i] *= burn_decay;
        state.burn_b[i] *= burn_decay;
    }

    // Ghost lurkers behind the live feed.
    let mut base = vec![0u8; n * 3];
    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let mut r = rgb[i] as f32;
            let mut g = rgb[i + 1] as f32;
            let mut b = rgb[i + 2] as f32;

            for layer in 0..ghosts.min(GHOST_DEPTHS.len()) {
                let depth = GHOST_DEPTHS[layer];
                let Some(past) = state.get_ring(depth) else {
                    continue;
                };
                if past.len() != rgb.len() {
                    continue;
                }
                let phase = state.frame as f32 * 0.028 + layer as f32 * 1.4;
                let ox = phase.sin() * (10.0 + smear * 18.0);
                let oy = phase.cos() * (8.0 + smear * 16.0);
                let (gr, gg, gb) = sample_rgb(
                    past,
                    w,
                    h,
                    (x as f32 + ox).clamp(0.0, w as f32 - 1.001),
                    (y as f32 + oy).clamp(0.0, h as f32 - 1.001),
                );
                let subject = (lum(gr, gg, gb) - 0.1).max(0.0) / 0.9;
                let alpha = subject * (0.46 - layer as f32 * 0.08);
                r = r * (1.0 - alpha) + gr as f32 * alpha;
                g = g * (1.0 - alpha) + gg as f32 * alpha;
                b = b * (1.0 - alpha) + gb as f32 * alpha;
            }

            base[i] = r as u8;
            base[i + 1] = g as u8;
            base[i + 2] = b as u8;
        }
    }

    if state.pointer_down {
        let px = state.pointer_x * w as f32;
        let py = state.pointer_y * h as f32;
        let vx = (state.pointer_x - state.pointer_prev_x) * w as f32;
        let vy = (state.pointer_y - state.pointer_prev_y) * h as f32;
        let speed = (vx * vx + vy * vy).sqrt();
        let (dir_x, dir_y) = if speed > 0.25 {
            (vx / speed, vy / speed)
        } else {
            (0.0, 0.0)
        };
        let trail = radius * (0.7 + smear * 0.9);
        let stamp = 0.5 + smear * 0.45;
        let xmax = w as f32 - 1.001;
        let ymax = h as f32 - 1.001;

        let y0 = ((py - radius).floor() as i32).max(0) as u32;
        let y1 = ((py + radius).ceil() as i32).min(h as i32 - 1) as u32;
        let x0 = ((px - radius).floor() as i32).max(0) as u32;
        let x1 = ((px + radius).ceil() as i32).min(w as i32 - 1) as u32;

        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x as f32 - px;
                let dy = y as f32 - py;
                let d2 = dx * dx + dy * dy;
                if d2 > r2 {
                    continue;
                }
                let falloff = (1.0 - d2.sqrt() / radius).powf(1.25);
                let si = y as usize * ww + x as usize;
                let i = si * 3;

                let mut r = if state.paint_a[si] > 0.02 {
                    state.paint_r[si]
                } else {
                    base[i] as f32
                };
                let mut g = if state.paint_a[si] > 0.02 {
                    state.paint_g[si]
                } else {
                    base[i + 1] as f32
                };
                let mut b = if state.paint_a[si] > 0.02 {
                    state.paint_b[si]
                } else {
                    base[i + 2] as f32
                };

                if speed > 0.25 {
                    let mut sr = 0.0f32;
                    let mut sg = 0.0f32;
                    let mut sb = 0.0f32;
                    let mut wsum = 0.0f32;
                    for tap in 0..=TAPS {
                        let t = tap as f32 / TAPS as f32;
                        let off = trail * t;
                        let sx = (x as f32 - dir_x * off).clamp(0.0, xmax);
                        let sy = (y as f32 - dir_y * off).clamp(0.0, ymax);
                        let weight = (1.0 - t).powf(1.1) * falloff;
                        let (tr, tg, tb) = sample_rgb(&base, w, h, sx, sy);
                        sr += tr as f32 * weight;
                        sg += tg as f32 * weight;
                        sb += tb as f32 * weight;
                        wsum += weight;
                    }
                    if wsum > 0.0 {
                        let mix = smear * falloff;
                        r = r * (1.0 - mix) + (sr / wsum) * mix;
                        g = g * (1.0 - mix) + (sg / wsum) * mix;
                        b = b * (1.0 - mix) + (sb / wsum) * mix;
                    }
                }

                state.paint_r[si] = r;
                state.paint_g[si] = g;
                state.paint_b[si] = b;
                state.paint_a[si] = (state.paint_a[si] + stamp * falloff).min(1.0);

                let add = burn_amt * falloff * 0.65;
                state.burn_r[si] = (state.burn_r[si] + base[i] as f32 * add).min(255.0);
                state.burn_g[si] = (state.burn_g[si] + base[i + 1] as f32 * add).min(255.0);
                state.burn_b[si] = (state.burn_b[si] + base[i + 2] as f32 * add).min(255.0);
            }
        }
    }

    let mut out = vec![0u8; n * 4];
    for y in 0..hh {
        for x in 0..ww {
            let si = y * ww + x;
            let i = si * 3;
            let pa = state.paint_a[si].clamp(0.0, 1.0);
            let mut r = base[i] as f32 * (1.0 - pa) + state.paint_r[si] * pa;
            let mut g = base[i + 1] as f32 * (1.0 - pa) + state.paint_g[si] * pa;
            let mut b = base[i + 2] as f32 * (1.0 - pa) + state.paint_b[si] * pa;

            let br = state.burn_r[si];
            let bg = state.burn_g[si];
            let bb = state.burn_b[si];
            let burn_l = br.max(bg).max(bb) / 255.0;
            let mix = (burn_l * (0.9 + burn_amt * 0.8)).clamp(0.0, 0.85);
            r = r * (1.0 - mix) + br * mix;
            g = g * (1.0 - mix) + bg * mix;
            b = b * (1.0 - mix) + bb * mix;

            if burn_l > 0.08 {
                let bloom = burn_amt * burn_l * 35.0;
                r = (r + bloom).min(255.0);
                g = (g + bloom * 0.55).min(255.0);
            }

            let o = si * 4;
            out[o] = r as u8;
            out[o + 1] = g as u8;
            out[o + 2] = b as u8;
            out[o + 3] = 255;
        }
    }
    out
}
