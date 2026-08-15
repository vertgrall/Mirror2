//! TUBER — tap a region, drag a stretch; up to four persist until Reset.

use super::ops::{lerp_u8, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub const TUBER_MAX: usize = 4;
pub const TUBER_PATCH: usize = 48;

fn patch_offset(slot: usize) -> usize {
    slot * TUBER_PATCH * TUBER_PATCH * 3
}

fn grab_patch(rgb: &[u8], w: u32, h: u32, nx: f32, ny: f32, radius: f32, dest: &mut [u8]) {
    let cx = nx * w as f32;
    let cy = ny * h as f32;
    for py in 0..TUBER_PATCH {
        for px in 0..TUBER_PATCH {
            let u = (px as f32 / (TUBER_PATCH as f32 - 1.0)) * 2.0 - 1.0;
            let v = (py as f32 / (TUBER_PATCH as f32 - 1.0)) * 2.0 - 1.0;
            let (r, g, b) = sample_rgb(rgb, w, h, cx + u * radius, cy + v * radius);
            let i = (py * TUBER_PATCH + px) * 3;
            dest[i] = r;
            dest[i + 1] = g;
            dest[i + 2] = b;
        }
    }
}

fn sample_patch(patch: &[u8], u: f32, v: f32) -> (u8, u8, u8) {
    let x = ((u * 0.5 + 0.5) * (TUBER_PATCH as f32 - 1.0)).clamp(0.0, TUBER_PATCH as f32 - 1.001);
    let y = ((v * 0.5 + 0.5) * (TUBER_PATCH as f32 - 1.0)).clamp(0.0, TUBER_PATCH as f32 - 1.001);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(TUBER_PATCH - 1);
    let y1 = (y0 + 1).min(TUBER_PATCH - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let i00 = (y0 * TUBER_PATCH + x0) * 3;
    let i10 = (y0 * TUBER_PATCH + x1) * 3;
    let i01 = (y1 * TUBER_PATCH + x0) * 3;
    let i11 = (y1 * TUBER_PATCH + x1) * 3;
    let mix = |a: u8, b: u8, c: u8, d: u8| {
        let v = a as f32 * (1.0 - tx) * (1.0 - ty)
            + b as f32 * tx * (1.0 - ty)
            + c as f32 * (1.0 - tx) * ty
            + d as f32 * tx * ty;
        v as u8
    };
    (
        mix(patch[i00], patch[i10], patch[i01], patch[i11]),
        mix(patch[i00 + 1], patch[i10 + 1], patch[i01 + 1], patch[i11 + 1]),
        mix(patch[i00 + 2], patch[i10 + 2], patch[i01 + 2], patch[i11 + 2]),
    )
}

fn stamp_stretch(
    out: &mut [u8],
    w: u32,
    h: u32,
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
    radius: f32,
    pull: f32,
    patch: &[u8],
) {
    let ww = w as usize;
    let x0 = ax * w as f32;
    let y0 = ay * h as f32;
    let x1 = bx * w as f32;
    let y1 = by * h as f32;
    let vx = x1 - x0;
    let vy = y1 - y0;
    let len2 = (vx * vx + vy * vy).max(1.0);
    let grow = 1.0 + pull * 1.2;

    let min_x = x0.min(x1) - radius * grow;
    let max_x = x0.max(x1) + radius * grow;
    let min_y = y0.min(y1) - radius * grow;
    let max_y = y0.max(y1) + radius * grow;
    let xa = min_x.floor().max(0.0) as u32;
    let xb = max_x.ceil().min(w as f32 - 1.0) as u32;
    let ya = min_y.floor().max(0.0) as u32;
    let yb = max_y.ceil().min(h as f32 - 1.0) as u32;

    for y in ya..=yb {
        for x in xa..=xb {
            let px = x as f32;
            let py = y as f32;
            let t = ((px - x0) * vx + (py - y0) * vy) / len2;
            let t = t.clamp(0.0, 1.0);
            let qx = x0 + vx * t;
            let qy = y0 + vy * t;
            let dx = px - qx;
            let dy = py - qy;
            let dist = (dx * dx + dy * dy).sqrt();
            let rad = radius * (1.0 + t * pull * 1.4);
            if dist > rad {
                continue;
            }
            let u = (dx / rad.max(1.0)).clamp(-1.2, 1.2);
            let v = (dy / rad.max(1.0)).clamp(-1.2, 1.2);
            if u * u + v * v > 1.15 {
                continue;
            }
            let (r, g, b) = sample_patch(patch, u, v);
            let a = (1.0 - dist / rad).powf(0.7);
            let o = (y as usize * ww + x as usize) * 4;
            out[o] = lerp_u8(out[o], r, a);
            out[o + 1] = lerp_u8(out[o + 1], g, a);
            out[o + 2] = lerp_u8(out[o + 2], b, a);
        }
    }
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &mut VfxState, p: &LookParams) -> Vec<u8> {
    let n = (w as usize) * (h as usize);
    let size = p.v(1);
    let pull = p.v(2);
    let radius = (w.min(h) as f32 * (0.04 + size * 0.14)).max(10.0);
    let _fade = p.v(3);

    state.ensure_tuber();

    if state.pointer_down {
        if !state.tuber_drag {
            if (state.tuber_n as usize) < TUBER_MAX {
                let i = state.tuber_n as usize;
                state.tuber_ax[i] = state.pointer_x;
                state.tuber_ay[i] = state.pointer_y;
                state.tuber_bx[i] = state.pointer_x;
                state.tuber_by[i] = state.pointer_y;
                let off = patch_offset(i);
                grab_patch(
                    rgb,
                    w,
                    h,
                    state.pointer_x,
                    state.pointer_y,
                    radius,
                    &mut state.tuber_patch[off..off + TUBER_PATCH * TUBER_PATCH * 3],
                );
                state.tuber_n += 1;
            }
            state.tuber_drag = true;
        }
        if state.tuber_n > 0 {
            let i = state.tuber_n as usize - 1;
            state.tuber_bx[i] = state.pointer_x;
            state.tuber_by[i] = state.pointer_y;
        }
    } else {
        state.tuber_drag = false;
    }

    let mut out = vec![0u8; n * 4];
    for i in 0..n {
        out[i * 4] = rgb[i * 3];
        out[i * 4 + 1] = rgb[i * 3 + 1];
        out[i * 4 + 2] = rgb[i * 3 + 2];
        out[i * 4 + 3] = 255;
    }

    let count = state.tuber_n as usize;
    for i in 0..count {
        let off = patch_offset(i);
        stamp_stretch(
            &mut out,
            w,
            h,
            state.tuber_ax[i],
            state.tuber_ay[i],
            state.tuber_bx[i],
            state.tuber_by[i],
            radius,
            pull,
            &state.tuber_patch[off..off + TUBER_PATCH * TUBER_PATCH * 3],
        );
    }

    out
}
