//! STICK — drag paint that never dries until Reset.

use super::ops::sample_rgb;
use super::params::LookParams;
use super::state::VfxState;

const TAPS: i32 = 14;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &mut VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let radius = p.v(1) * 90.0 + 28.0;
    let drag = p.v(2);
    let bleed = p.v(3);

    let n = ww * hh;
    state.ensure_paint(n);

    if state.pointer_down {
        let px = state.pointer_x * w as f32;
        let py = state.pointer_y * h as f32;
        let vx = (state.pointer_x - state.pointer_prev_x) * w as f32;
        let vy = (state.pointer_y - state.pointer_prev_y) * h as f32;
        let speed = (vx * vx + vy * vy).sqrt();
        let (dir_x, dir_y) = if speed > 0.35 {
            (vx / speed, vy / speed)
        } else {
            (0.0, 0.0)
        };

        let r2 = radius * radius;
        let stamp = 0.55 + drag * 0.45;
        let trail_len = radius * (0.7 + drag * 1.0);
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
                let falloff = (1.0 - d2.sqrt() / radius).powf(1.3);
                let si = y as usize * ww + x as usize;
                let i = si * 3;

                let mut r = if state.paint_a[si] > 0.01 {
                    state.paint_r[si]
                } else {
                    rgb[i] as f32
                };
                let mut g = if state.paint_a[si] > 0.01 {
                    state.paint_g[si]
                } else {
                    rgb[i + 1] as f32
                };
                let mut b = if state.paint_a[si] > 0.01 {
                    state.paint_b[si]
                } else {
                    rgb[i + 2] as f32
                };

                if speed > 0.35 {
                    let mut sr = 0.0f32;
                    let mut sg = 0.0f32;
                    let mut sb = 0.0f32;
                    let mut wsum = 0.0f32;
                    for tap in 0..=TAPS {
                        let t = tap as f32 / TAPS as f32;
                        let off = trail_len * t;
                        let sx = (x as f32 - dir_x * off).clamp(0.0, xmax);
                        let sy = (y as f32 - dir_y * off).clamp(0.0, ymax);
                        let weight = (1.0 - t).powf(1.1) * falloff;
                        let (tr, tg, tb) = sample_rgb(rgb, w, h, sx, sy);
                        sr += tr as f32 * weight;
                        sg += tg as f32 * weight;
                        sb += tb as f32 * weight;
                        wsum += weight;
                    }
                    if wsum > 0.0 {
                        let smear = drag * falloff;
                        r = r * (1.0 - smear) + (sr / wsum) * smear;
                        g = g * (1.0 - smear) + (sg / wsum) * smear;
                        b = b * (1.0 - smear) + (sb / wsum) * smear;
                    }
                } else {
                    r = r * (1.0 - falloff) + rgb[i] as f32 * falloff;
                    g = g * (1.0 - falloff) + rgb[i + 1] as f32 * falloff;
                    b = b * (1.0 - falloff) + rgb[i + 2] as f32 * falloff;
                }

                let edge_soft = 1.0 - bleed * 0.15 * (1.0 - falloff);
                state.paint_r[si] = r * edge_soft;
                state.paint_g[si] = g * edge_soft;
                state.paint_b[si] = b * edge_soft;
                state.paint_a[si] = (state.paint_a[si] + stamp * falloff).min(1.0);
            }
        }
    }

    let mut out = vec![0u8; n * 4];
    for y in 0..hh {
        for x in 0..ww {
            let si = y * ww + x;
            let i = si * 3;
            let a = state.paint_a[si].clamp(0.0, 1.0);
            let r = rgb[i] as f32 * (1.0 - a) + state.paint_r[si] * a;
            let g = rgb[i + 1] as f32 * (1.0 - a) + state.paint_g[si] * a;
            let b = rgb[i + 2] as f32 * (1.0 - a) + state.paint_b[si] * a;
            let o = si * 4;
            out[o] = r.clamp(0.0, 255.0) as u8;
            out[o + 1] = g.clamp(0.0, 255.0) as u8;
            out[o + 2] = b.clamp(0.0, 255.0) as u8;
            out[o + 3] = 255;
        }
    }
    out
}
