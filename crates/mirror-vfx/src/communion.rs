//! COMMUNION — magic wand, then a congregation of copies.

use super::ops::lerp_u8;
use super::params::LookParams;
use super::state::VfxState;

fn color_dist(rgb: &[u8], i: usize, j: usize) -> f32 {
    let dr = rgb[i] as f32 - rgb[j] as f32;
    let dg = rgb[i + 1] as f32 - rgb[j + 1] as f32;
    let db = rgb[i + 2] as f32 - rgb[j + 2] as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

fn flood_disk(rgb: &[u8], w: u32, h: u32, sx: u32, sy: u32, size: f32) -> Vec<f32> {
    let ww = w as usize;
    let hh = h as usize;
    let mut mask = vec![0f32; ww * hh];
    let radius = (w.min(h) as f32 * (0.06 + size.clamp(0.0, 1.0) * 0.28)).max(12.0);
    let r2 = radius * radius;
    let thresh = 40.0 + size * 50.0;
    let seed = (sy as usize * ww + sx as usize) * 3;
    let cx = sx as f32;
    let cy = sy as f32;
    let mut stack = vec![sy as usize * ww + sx as usize];
    mask[sy as usize * ww + sx as usize] = 1.0;
    while let Some(idx) = stack.pop() {
        let x = idx % ww;
        let y = idx / ww;
        for (nx, ny) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if nx >= ww || ny >= hh {
                continue;
            }
            let dx = nx as f32 - cx;
            let dy = ny as f32 - cy;
            if dx * dx + dy * dy > r2 {
                continue;
            }
            let ni = ny * ww + nx;
            if mask[ni] > 0.0 {
                continue;
            }
            if color_dist(rgb, seed, ni * 3) <= thresh {
                mask[ni] = (1.0 - (dx * dx + dy * dy).sqrt() / radius).clamp(0.25, 1.0);
                stack.push(ni);
            }
        }
    }
    let y0 = (cy - radius).floor().max(0.0) as usize;
    let y1 = (cy + radius).ceil().min(hh as f32 - 1.0) as usize;
    let x0 = (cx - radius).floor().max(0.0) as usize;
    let x1 = (cx + radius).ceil().min(ww as f32 - 1.0) as usize;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d2 = dx * dx + dy * dy;
            if d2 <= r2 {
                let f = (1.0 - d2.sqrt() / radius).clamp(0.2, 1.0);
                let i = y * ww + x;
                if mask[i] < f {
                    mask[i] = f;
                }
            }
        }
    }
    mask
}

fn stamp_copy(
    out: &mut [u8],
    w: u32,
    h: u32,
    state: &VfxState,
    dest_x: f32,
    dest_y: f32,
    scale: f32,
    alpha: f32,
) {
    let ww = w as usize;
    let hh = h as usize;
    let src_cx = state.bomen_tap_x * w as f32;
    let src_cy = state.bomen_tap_y * h as f32;
    let scale = scale.max(0.35);
    for y in 0..hh {
        for x in 0..ww {
            let dx = (x as f32 - dest_x) / scale;
            let dy = (y as f32 - dest_y) / scale;
            let sx = (src_cx + dx).round() as i32;
            let sy = (src_cy + dy).round() as i32;
            if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 {
                continue;
            }
            let src = sy as usize * ww + sx as usize;
            let ma = state.bomen_mask[src] * alpha;
            if ma < 0.04 {
                continue;
            }
            let si = src * 3;
            let o = (y * ww + x) * 4;
            out[o] = lerp_u8(out[o], state.bomen_src_rgb[si], ma);
            out[o + 1] = lerp_u8(out[o + 1], state.bomen_src_rgb[si + 1], ma);
            out[o + 2] = lerp_u8(out[o + 2], state.bomen_src_rgb[si + 2], ma);
        }
    }
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &mut VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let n = ww * hh;
    let copies = p.v(1);
    let spread = p.v(2);
    let size = p.v(3);

    state.ensure_bomen(n);

    if state.pointer_down {
        if !state.bomen_placing {
            let sx = (state.pointer_x * w as f32).round().clamp(0.0, w as f32 - 1.0) as u32;
            let sy = (state.pointer_y * h as f32).round().clamp(0.0, h as f32 - 1.0) as u32;
            state.bomen_mask = flood_disk(rgb, w, h, sx, sy, size);
            if state.bomen_src_rgb.len() != rgb.len() {
                state.bomen_src_rgb = vec![0u8; rgb.len()];
            }
            state.bomen_src_rgb.copy_from_slice(rgb);
            state.bomen_tap_x = state.pointer_x;
            state.bomen_tap_y = state.pointer_y;
            state.bomen_src_w = w;
            state.bomen_src_h = h;
            state.bomen_has_source = true;
            state.bomen_clone_placed = true;
            state.bomen_placing = true;
        }
        state.bomen_clone_x = state.pointer_x;
        state.bomen_clone_y = state.pointer_y;
    } else {
        state.bomen_placing = false;
    }

    if state.bomen_has_source && (state.bomen_src_w != w || state.bomen_src_h != h) {
        let sx = (state.bomen_tap_x * w as f32).round().clamp(0.0, w as f32 - 1.0) as u32;
        let sy = (state.bomen_tap_y * h as f32).round().clamp(0.0, h as f32 - 1.0) as u32;
        state.bomen_mask = flood_disk(rgb, w, h, sx, sy, size);
        if state.bomen_src_rgb.len() != rgb.len() {
            state.bomen_src_rgb = vec![0u8; rgb.len()];
        }
        state.bomen_src_rgb.copy_from_slice(rgb);
        state.bomen_src_w = w;
        state.bomen_src_h = h;
    }

    let mut out = vec![0u8; n * 4];
    for i in 0..n {
        out[i * 4] = rgb[i * 3];
        out[i * 4 + 1] = rgb[i * 3 + 1];
        out[i * 4 + 2] = rgb[i * 3 + 2];
        out[i * 4 + 3] = 255;
    }

    if !state.bomen_has_source {
        return out;
    }

    let count = 1 + (copies * 7.0).round() as i32;
    let scale = 0.55 + size * 0.75;
    let radius = spread * 0.28 * w.min(h) as f32;
    let t = state.frame as f32 * 0.04;
    let base_x = if state.bomen_clone_placed {
        state.bomen_clone_x
    } else {
        state.bomen_tap_x
    } * w as f32;
    let base_y = if state.bomen_clone_placed {
        state.bomen_clone_y
    } else {
        state.bomen_tap_y
    } * h as f32;

    for i in 0..count {
        let a = i as f32 * std::f32::consts::TAU / count as f32 + t;
        let dest_x = base_x + radius * a.cos();
        let dest_y = base_y + radius * a.sin() * 0.85;
        let alpha = if i == 0 { 1.0 } else { 0.72 };
        stamp_copy(&mut out, w, h, state, dest_x, dest_y, scale, alpha);
    }

    out
}
