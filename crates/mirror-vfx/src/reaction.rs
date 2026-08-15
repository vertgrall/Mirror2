//! REACTION — Gray-Scott Turing patterns fed by live luminance, grown over time.

use super::ops::{lum, rgb_to_rgba, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

const DU: f32 = 0.2097;
const DV: f32 = 0.105;

fn sim_dims(w: u32, h: u32) -> (usize, usize) {
    let sw = ((w as usize).max(64)).min(320);
    let sh = ((h as usize).max(48)).min(240);
    let sw = sw / 2;
    let sh = sh / 2;
    (sw.max(80), sh.max(60))
}

fn idx(x: usize, y: usize, sw: usize) -> usize {
    y * sw + x
}

fn laplacian(field: &[f32], x: usize, y: usize, sw: usize, sh: usize) -> f32 {
    let i = idx(x, y, sw);
    let l = if x > 0 { field[i - 1] } else { field[i] };
    let r = if x + 1 < sw { field[i + 1] } else { field[i] };
    let u = if y > 0 { field[i - sw] } else { field[i] };
    let d = if y + 1 < sh { field[i + sw] } else { field[i] };
    l + r + u + d - 4.0 * field[i]
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &mut VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let feed = 0.052 + p.v(1) * 0.022;
    let kill = 0.054 + p.v(2) * 0.016;
    let luma_gain = 0.45 + p.v(3) * 0.95;

    let (sw, sh) = sim_dims(w, h);
    let n = sw * sh;
    state.ensure_rd(n);

    let u = &mut state.rd_u;
    let v = &mut state.rd_v;
    let mut u_next = vec![0.0f32; n];
    let mut v_next = vec![0.0f32; n];

    // Seed chemical V from subject luminance each frame.
    for sy in 0..sh {
        let yf = (sy as f32 + 0.5) / sh as f32 * h as f32;
        for sx in 0..sw {
            let xf = (sx as f32 + 0.5) / sw as f32 * w as f32;
            let si = idx(sx, sy, sw);
            let (r, g, b) = sample_rgb(rgb, w, h, xf, yf);
            let subject = lum(r, g, b);
            let inject = (subject - 0.10).max(0.0) * luma_gain * 0.52;
            v[si] = (v[si] + inject).min(1.0);
            if subject > 0.38 && hash_jitter(sx, sy, state.frame) > 0.984 {
                v[si] = 1.0;
                u[si] = 0.5;
            }
        }
    }

    let steps = 7 + (p.v(1) * 6.0) as usize;
    for _ in 0..steps {
        for y in 0..sh {
            for x in 0..sw {
                let i = idx(x, y, sw);
                let ui = u[i];
                let vi = v[i];
                let lap_u = laplacian(u, x, y, sw, sh);
                let lap_v = laplacian(v, x, y, sw, sh);
                let reaction = ui * vi * vi;
                u_next[i] = (ui + (DU * lap_u - reaction + feed * (1.0 - ui))).clamp(0.0, 1.0);
                v_next[i] =
                    (vi + (DV * lap_v + reaction - (feed + kill) * vi)).clamp(0.0, 1.0);
            }
        }
        u.copy_from_slice(&u_next);
        v.copy_from_slice(&v_next);
    }

    let mut out = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let sx = ((x as f32 / ww as f32) * sw as f32) as usize;
            let sy = ((y as f32 / hh as f32) * sh as f32) as usize;
            let sx = sx.min(sw - 1);
            let sy = sy.min(sh - 1);
            let si = idx(sx, sy, sw);

            let ui = u[si];
            let vi = v[si];
            let density = vi / (ui + vi + 0.08);

            let (br, bg, bb) = sample_rgb(rgb, w, h, x as f32, y as f32);
            let subject = lum(br, bg, bb);

            // Bioluminescent Turing palette — visible even at default wet.
            let pat_r = (20.0 + density * 240.0 + ui * 40.0).min(255.0);
            let pat_g = (10.0 + (1.0 - density) * 180.0 + vi * 200.0).min(255.0);
            let pat_b = (30.0 + density * 120.0 + (1.0 - ui) * 160.0).min(255.0);

            let mix = (0.58 + density * 0.42) * (0.50 + subject * 0.50) * luma_gain;
            let mix = mix.clamp(0.0, 0.96);

            let r = br as f32 * (1.0 - mix) + pat_r * mix;
            let g = bg as f32 * (1.0 - mix) + pat_g * mix;
            let b = bb as f32 * (1.0 - mix) + pat_b * mix;

            out[i] = r as u8;
            out[i + 1] = g as u8;
            out[i + 2] = b as u8;
        }
    }

    rgb_to_rgba(&out, w, h)
}

fn hash_jitter(x: usize, y: usize, frame: u64) -> f32 {
    let n = (x as f32 * 12.9898 + y as f32 * 78.233 + frame as f32 * 0.17).sin() * 43758.5453;
    n.fract().abs()
}
