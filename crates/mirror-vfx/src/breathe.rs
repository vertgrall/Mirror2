//! BREATHE — whole-frame inhale/exhale with drifting black particle balls.

use super::ops::{hash2d, lerp_u8, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

const TAU: f32 = std::f32::consts::TAU;

/// Smooth inhale (0→1) / exhale (1→0) over one cycle.
fn breath_wave(phase: f32, hold: f32) -> f32 {
    let s = phase.sin();
    if hold < 0.02 {
        return s;
    }
    let top = (1.0 - hold * 0.45).max(0.08);
    if s >= 0.0 {
        (s / top).min(1.0)
    } else {
        s
    }
}

fn cluster_anchor(seed: u32, cluster_id: usize, w: f32, h: f32) -> (f32, f32) {
    let fi = cluster_id as f32;
    let sx = hash2d(fi + seed as f32 * 0.0019, seed as f32 * 0.0027);
    let sy = hash2d(seed as f32 * 0.0031, fi + seed as f32 * 0.0043);
    let margin_x = w * 0.06;
    let margin_y = h * 0.06;
    (
        margin_x + sx * (w - margin_x * 2.0),
        margin_y + sy * (h - margin_y * 2.0),
    )
}

fn ball_center(
    id: usize,
    w: f32,
    h: f32,
    span: f32,
    frame: f32,
    breath: f32,
    speed: f32,
    spread: f32,
    layout_seed: u32,
    cluster_count: usize,
) -> (f32, f32) {
    let fi = id as f32;
    let spread_t = spread.clamp(0.0, 1.0);
    let spread_curve = spread_t * spread_t;

    let cluster_id =
        (hash2d(fi * 0.83, layout_seed as f32) * cluster_count as f32) as usize;
    let (anchor_x, anchor_y) = cluster_anchor(layout_seed, cluster_id, w, h);

    let angle = hash2d(fi * 1.71, layout_seed as f32 + 8.2) * TAU;
    let ring = 0.08 + hash2d(fi * 0.53, layout_seed as f32 + 3.4).powf(0.65);
    let sep = span * (0.008 + spread_curve * 0.98) * ring;

    let spin_rate = 0.01 + speed.clamp(0.0, 1.0) * 0.07;
    let spin = (frame * spin_rate + angle).rem_euclid(TAU);
    let drift = span * speed * 0.035;

    let px = anchor_x + spin.cos() * sep + breath * drift * angle.cos();
    let py = anchor_y + spin.sin() * sep * 0.92 + breath * drift * angle.sin() * 0.85;

    (px, py)
}

fn stamp_ball(out: &mut [u8], ww: usize, hh: usize, px: f32, py: f32, radius: f32) {
    if ww == 0
        || hh == 0
        || radius < 0.75
        || !radius.is_finite()
        || !px.is_finite()
        || !py.is_finite()
    {
        return;
    }

    let xmax = ww as i32 - 1;
    let ymax = hh as i32 - 1;
    if xmax < 0 || ymax < 0 {
        return;
    }

    let r2 = radius * radius;
    let x0 = ((px - radius - 1.0).floor() as i32).clamp(0, xmax) as u32;
    let y0 = ((py - radius - 1.0).floor() as i32).clamp(0, ymax) as u32;
    let x1 = ((px + radius + 1.0).ceil() as i32).clamp(0, xmax) as u32;
    let y1 = ((py + radius + 1.0).ceil() as i32).clamp(0, ymax) as u32;
    if x0 > x1 || y0 > y1 {
        return;
    }

    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 - px;
            let dy = y as f32 - py;
            let d2 = dx * dx + dy * dy;
            if d2 > r2 {
                continue;
            }
            let t = (1.0 - (d2.sqrt() / radius).powf(1.35)).clamp(0.0, 1.0);
            let o = (y as usize * ww + x as usize) * 4;
            out[o] = lerp_u8(out[o], 0, t);
            out[o + 1] = lerp_u8(out[o + 1], 0, t);
            out[o + 2] = lerp_u8(out[o + 2], 0, t);
        }
    }
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let depth = p.v(1).clamp(0.0, 1.0);
    let pace = p.v(2).clamp(0.0, 1.0);
    let hold = p.v(3).clamp(0.0, 1.0);
    let ball_size = p.v(4).clamp(0.0, 1.0);
    let ball_speed = p.v(5).clamp(0.0, 1.0);
    let ball_spread = p.v(6).clamp(0.0, 1.0);

    let phase = state.frame as f32 * (0.028 + pace * 0.07);
    let breath = breath_wave(phase, hold);

    let inhale = breath.max(0.0);
    let exhale = (-breath).max(0.0);
    let scale = 1.0 + depth * 0.26 * inhale - depth * 0.14 * exhale;

    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.46;
    let span = w.min(h) as f32;
    let wf = w as f32;
    let hf = h as f32;
    let xmax = wf - 1.001;
    let ymax = hf - 1.001;

    let mut out = vec![0u8; ww * hh * 4];
    for y in 0..hh {
        for x in 0..ww {
            let xf = x as f32;
            let yf = y as f32;
            let sx = (xf - cx) / scale + cx;
            let sy = (yf - cy) / scale + cy;

            let edge_falloff = if sx < 0.0 || sx > xmax || sy < 0.0 || sy > ymax {
                let ox = if sx < 0.0 {
                    -sx
                } else if sx > xmax {
                    sx - xmax
                } else {
                    0.0
                };
                let oy = if sy < 0.0 {
                    -sy
                } else if sy > ymax {
                    sy - ymax
                } else {
                    0.0
                };
                (ox + oy).min(40.0) / 40.0
            } else {
                0.0
            };

            let (r, g, b) = if edge_falloff >= 1.0 {
                (0u8, 0u8, 0u8)
            } else if edge_falloff > 0.0 {
                let (sr, sg, sb) = sample_rgb(
                    rgb,
                    w,
                    h,
                    sx.clamp(0.0, xmax),
                    sy.clamp(0.0, ymax),
                );
                let t = edge_falloff;
                (
                    lerp_u8(sr, 0, t),
                    lerp_u8(sg, 0, t),
                    lerp_u8(sb, 0, t),
                )
            } else {
                sample_rgb(rgb, w, h, sx, sy)
            };

            let glow = 1.0 + depth * 0.22 * inhale - depth * 0.12 * exhale;
            let mut rf = (r as f32 * glow).clamp(0.0, 255.0);
            let mut gf = (g as f32 * glow).clamp(0.0, 255.0);
            let mut bf = (b as f32 * glow).clamp(0.0, 255.0);

            let dx = if cx > 0.0 { (xf - cx) / cx } else { 0.0 };
            let dy = if cy > 0.0 { (yf - cy) / cy } else { 0.0 };
            let dist = (dx * dx + dy * dy).sqrt();
            let vignette = depth * (0.08 + exhale * 0.28 + inhale * 0.06) * dist;
            rf = lerp_u8(rf as u8, 0, vignette) as f32;
            gf = lerp_u8(gf as u8, 0, vignette) as f32;
            bf = lerp_u8(bf as u8, 0, vignette * 0.85) as f32;

            let o = (y * ww + x) * 4;
            out[o] = rf as u8;
            out[o + 1] = gf as u8;
            out[o + 2] = bf as u8;
            out[o + 3] = 255;
        }
    }

    if ball_size > 0.01 && ww > 0 && hh > 0 {
        let count = (10.0 + ball_size * 34.0) as usize;
        let base_r = 3.0 + ball_size * 22.0;
        let frame = state.frame as f32;
        let cycle = (phase / TAU).floor() as u32;
        let layout_seed = state
            .breathe_seed
            .wrapping_add(cycle.wrapping_mul(0x9E37_79B9));
        let cluster_count = (1.0 + ball_spread * 4.0).round().clamp(1.0, 5.0) as usize;

        for id in 0..count {
            let (px, py) = ball_center(
                id,
                wf,
                hf,
                span,
                frame,
                breath,
                ball_speed,
                ball_spread,
                layout_seed,
                cluster_count,
            );
            let jitter = 0.75 + hash2d(id as f32 * 0.41, layout_seed as f32) * 0.55;
            stamp_ball(&mut out, ww, hh, px, py, base_r * jitter);
        }
    }

    out
}
