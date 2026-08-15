//! FILM — 35mm gate: sprockets, rebate weave, halation, layered grain, scratches.

use super::ops::{hash2d, lerp_u8, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

fn fine_grain(x: f32, y: f32, frame: f32) -> f32 {
    hash2d(x * 3.7 + frame * 0.31, y * 4.1 - frame * 0.17) - 0.5
}

fn coarse_grain(x: f32, y: f32, frame: f32) -> f32 {
    hash2d(x * 0.9 + frame * 0.07, y * 1.1 + frame * 0.05) - 0.5
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let grain = p.v(1);
    let warm = p.v(2);
    let frame_amt = p.v(3);

    let rebate = 0.075 + frame_amt * 0.085;
    let left = (w as f32 * rebate).round() as u32;
    let right = w.saturating_sub(left);
    let top = (h as f32 * (rebate * 0.55)).round() as u32;
    let bottom = h.saturating_sub(top);

    let gate_x = (state.frame as f32 * 0.11).sin() * (1.5 + frame_amt * 2.5);
    let gate_y = (state.frame as f32 * 0.083).cos() * (1.0 + frame_amt * 2.0);
    let row_jitter = |y: f32| (y * 0.07 + state.frame as f32 * 0.19).sin() * frame_amt * 1.8;

    let sprocket_pitch = 13.0 + frame_amt * 3.5;
    let sprocket_w = 3.5 + frame_amt * 2.5;
    let hole_h = 7.0 + frame_amt * 3.5;

    let mut out = vec![0u8; ww * hh * 4];
    let xmax = w as f32 - 1.001;
    let ymax = h as f32 - 1.001;
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;

    for y in 0..hh {
        let yf = y as f32;
        let row_wobble = row_jitter(yf);

        for x in 0..ww {
            let xf = x as f32;
            let o = (y * ww + x) * 4;

            let in_rebate = x < left as usize
                || x >= right as usize
                || y < top as usize
                || y >= bottom as usize;

            if in_rebate {
                let hole_row = (yf / sprocket_pitch).floor();
                let hole_phase = (yf - hole_row * sprocket_pitch) / sprocket_pitch;
                let side_left = x < left as usize;
                let side_right = x >= right as usize;
                let near_edge = if side_left {
                    xf / left as f32
                } else if side_right {
                    (w as f32 - xf) / left as f32
                } else {
                    0.0
                };

                let sprocket_hole = (side_left || side_right)
                    && frame_amt > 0.08
                    && hole_phase > 0.18
                    && hole_phase < 0.18 + hole_h / sprocket_pitch
                    && near_edge > 0.35
                    && near_edge < 0.35 + sprocket_w / left as f32;

                let base = if sprocket_hole {
                    8.0 + hash2d(hole_row, xf) * 6.0
                } else {
                    18.0 + hash2d(xf, yf) * 14.0
                };
                out[o] = base as u8;
                out[o + 1] = (base * 0.92) as u8;
                out[o + 2] = (base * 0.82) as u8;
                out[o + 3] = 255;
                continue;
            }

            let sx = (xf + gate_x + row_wobble).clamp(0.0, xmax);
            let sy = (yf + gate_y).clamp(0.0, ymax);

            // Chromatic misregistration — film transport wobble.
            let reg = frame_amt * 1.4;
            let (r, _, _) = sample_rgb(rgb, w, h, (sx - reg).clamp(0.0, xmax), sy);
            let (_, g, _) = sample_rgb(rgb, w, h, sx, sy);
            let (_, _, b) = sample_rgb(rgb, w, h, (sx + reg).clamp(0.0, xmax), sy);
            let mut rf = r as f32;
            let mut gf = g as f32;
            let mut bf = b as f32;

            rf = rf * (1.0 + warm * 0.14) + warm * 16.0;
            gf = gf * (1.0 + warm * 0.07) + warm * 9.0;
            bf = bf * (1.0 - warm * 0.12);

            let lift = 12.0 + warm * 10.0;
            rf = (rf * 0.96 + lift).min(255.0);
            gf = (gf * 0.94 + lift * 0.96).min(255.0);
            bf = (bf * 0.90 + lift * 0.82).min(255.0);

            if rf > 185.0 {
                let halation = warm * 0.22 + grain * 0.08;
                rf = (rf + halation * 35.0).min(255.0);
                gf = (gf + halation * 18.0).min(255.0);
            }

            if grain > 0.01 {
                let fine = fine_grain(xf, yf, state.frame as f32);
                let coarse = coarse_grain(xf, yf, state.frame as f32);
                let gstr = grain * 52.0;
                rf = (rf + fine * gstr + coarse * gstr * 1.6).clamp(0.0, 255.0);
                gf = (gf + fine * gstr * 0.88 + coarse * gstr * 1.4).clamp(0.0, 255.0);
                bf = (bf + fine * gstr * 0.75 + coarse * gstr * 1.2).clamp(0.0, 255.0);
            }

            // Vertical scratch lines.
            let scratch_col = (xf * 0.07 + state.frame as f32 * 0.4).floor();
            if hash2d(scratch_col, 0.0) > 1.0 - grain * 0.014 - frame_amt * 0.006 {
                if hash2d(scratch_col, yf * 0.15) > 0.25 {
                    let dim = 0.52 + hash2d(scratch_col, yf) * 0.28;
                    rf *= dim;
                    gf *= dim;
                    bf *= dim;
                }
            }

            // Dust and gate dirt.
            let dust = hash2d(xf * 1.3 + state.frame as f32 * 0.02, yf * 1.7);
            if dust > 1.0 - grain * 0.004 {
                let speck = (dust - (1.0 - grain * 0.004)) / (grain * 0.004).max(0.001);
                rf = lerp_u8(rf as u8, 40, speck * 0.7) as f32;
                gf = lerp_u8(gf as u8, 35, speck * 0.7) as f32;
                bf = lerp_u8(bf as u8, 30, speck * 0.7) as f32;
            }

            let nx = (xf - cx) / cx;
            let ny = (yf - cy) / cy;
            let vig = (nx * nx + ny * ny).min(1.0) * (0.18 + frame_amt * 0.12);
            rf *= 1.0 - vig;
            gf *= 1.0 - vig;
            bf *= 1.0 - vig * 0.9;

            out[o] = rf as u8;
            out[o + 1] = gf as u8;
            out[o + 2] = bf as u8;
            out[o + 3] = 255;
        }
    }
    out
}
