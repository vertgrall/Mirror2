//! B-OMEN — magic wand at the tap, then drag that copy as an omen.

use super::ops::lerp_u8;
use super::params::LookParams;
use super::state::VfxState;

fn color_dist(rgb: &[u8], i: usize, j: usize) -> f32 {
    let dr = rgb[i] as f32 - rgb[j] as f32;
    let dg = rgb[i + 1] as f32 - rgb[j + 1] as f32;
    let db = rgb[i + 2] as f32 - rgb[j + 2] as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

fn reach_px(w: u32, h: u32, reach: f32) -> f32 {
    (w.min(h) as f32 * (0.05 + reach.clamp(0.0, 1.0) * 0.40)).max(10.0)
}

fn flood_mask(
    rgb: &[u8],
    w: u32,
    h: u32,
    sx: u32,
    sy: u32,
    match_amt: f32,
    reach: f32,
) -> Vec<f32> {
    let ww = w as usize;
    let hh = h as usize;
    let n = ww * hh;
    let mut mask = vec![0f32; n];
    let thresh = 12.0 + match_amt.clamp(0.0, 1.0) * 90.0;
    let radius = reach_px(w, h, reach);
    let r2 = radius * radius;
    let seed = (sy as usize * ww + sx as usize) * 3;
    let cx = sx as f32;
    let cy = sy as f32;
    let mut stack = vec![sy as usize * ww + sx as usize];
    mask[sy as usize * ww + sx as usize] = 1.0;

    while let Some(idx) = stack.pop() {
        let x = idx % ww;
        let y = idx / ww;
        let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];
        for (nx, ny) in neighbors {
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
            let pi = ni * 3;
            if color_dist(rgb, seed, pi) <= thresh {
                let falloff = (1.0 - (dx * dx + dy * dy).sqrt() / radius).clamp(0.25, 1.0);
                mask[ni] = falloff;
                stack.push(ni);
            }
        }
    }

    // Always keep a visible disk at the exact tap so grain can't starve the wand.
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
                let falloff = (1.0 - d2.sqrt() / radius).clamp(0.20, 1.0);
                let i = y * ww + x;
                if mask[i] < falloff {
                    mask[i] = falloff;
                }
            }
        }
    }
    mask
}

fn capture_source(
    rgb: &[u8],
    w: u32,
    h: u32,
    state: &mut VfxState,
    nx: f32,
    ny: f32,
    match_amt: f32,
    reach: f32,
) {
    let sx = (nx * w as f32).round().clamp(0.0, w as f32 - 1.0) as u32;
    let sy = (ny * h as f32).round().clamp(0.0, h as f32 - 1.0) as u32;
    state.bomen_mask = flood_mask(rgb, w, h, sx, sy, match_amt, reach);
    if state.bomen_src_rgb.len() != rgb.len() {
        state.bomen_src_rgb = vec![0u8; rgb.len()];
    }
    state.bomen_src_rgb.copy_from_slice(rgb);
    state.bomen_tap_x = nx;
    state.bomen_tap_y = ny;
    state.bomen_src_w = w;
    state.bomen_src_h = h;
    state.bomen_has_source = true;
}

fn recapture_at_size(
    rgb: &[u8],
    w: u32,
    h: u32,
    state: &mut VfxState,
    match_amt: f32,
    reach: f32,
) {
    let nx = state.bomen_tap_x;
    let ny = state.bomen_tap_y;
    capture_source(rgb, w, h, state, nx, ny, match_amt, reach);
}

fn stamp_reticle(out: &mut [u8], w: u32, h: u32, nx: f32, ny: f32, pulse: f32) {
    let cx = (nx * w as f32).round() as i32;
    let cy = (ny * h as f32).round() as i32;
    let ww = w as i32;
    let hh = h as i32;
    let arm = 7i32;
    let put = |out: &mut [u8], x: i32, y: i32, a: f32| {
        if x < 0 || y < 0 || x >= ww || y >= hh {
            return;
        }
        let o = ((y * ww + x) * 4) as usize;
        out[o] = lerp_u8(out[o], 255, a);
        out[o + 1] = lerp_u8(out[o + 1], 240, a);
        out[o + 2] = lerp_u8(out[o + 2], 80, a);
    };
    for d in -arm..=arm {
        put(out, cx + d, cy, pulse);
        put(out, cx, cy + d, pulse);
    }
    let ring = 5i32;
    for a in 0..24 {
        let t = a as f32 * std::f32::consts::TAU / 24.0;
        put(
            out,
            cx + (t.cos() * ring as f32).round() as i32,
            cy + (t.sin() * ring as f32).round() as i32,
            pulse * 0.85,
        );
    }
}

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &mut VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let n = ww * hh;
    let reach = p.v(1);
    let match_amt = p.v(2);
    let pulse_amt = p.v(3);
    let echo_amt = p.v(4);
    let drift_amt = p.v(5);

    state.ensure_bomen(n);

    if state.bomen_tap_pending {
        state.bomen_tap_pending = false;
        capture_source(
            rgb,
            w,
            h,
            state,
            state.bomen_tap_x,
            state.bomen_tap_y,
            match_amt,
            reach,
        );
        state.bomen_clone_placed = false;
        state.bomen_placing = false;
    }

    if state.pointer_down {
        if !state.bomen_placing {
            // New press = new wand sample at this exact tap.
            capture_source(
                rgb,
                w,
                h,
                state,
                state.pointer_x,
                state.pointer_y,
                match_amt,
                reach,
            );
            state.bomen_placing = true;
            state.bomen_clone_x = state.pointer_x;
            state.bomen_clone_y = state.pointer_y;
        }
        state.bomen_clone_x = state.pointer_x;
        state.bomen_clone_y = state.pointer_y;
        state.bomen_clone_placed = true;
    } else {
        state.bomen_placing = false;
    }

    // Shutter is full-res; preview is ~640px. Rebuild the wand at this size
    // without moving the omen the user already placed.
    if state.bomen_has_source && (state.bomen_src_w != w || state.bomen_src_h != h) {
        recapture_at_size(rgb, w, h, state, match_amt, reach);
    }

    let mut out = vec![0u8; n * 4];
    let t = state.frame as f32 * 0.07;
    let highlight_pulse = 0.45 + pulse_amt * 0.35 * (1.0 + t.sin()) * 0.5;

    for y in 0..hh {
        for x in 0..ww {
            let si = y * ww + x;
            let i = si * 3;
            let o = si * 4;
            let mut r = rgb[i] as f32;
            let mut g = rgb[i + 1] as f32;
            let mut b = rgb[i + 2] as f32;

            if state.bomen_has_source && state.bomen_mask[si] > 0.15 {
                let m = state.bomen_mask[si] * highlight_pulse;
                r = lerp_u8(r as u8, 120, m * 0.55) as f32;
                g = lerp_u8(g as u8, 220, m * 0.75) as f32;
                b = lerp_u8(b as u8, 255, m * 0.85) as f32;
            }

            out[o] = r as u8;
            out[o + 1] = g as u8;
            out[o + 2] = b as u8;
            out[o + 3] = 255;
        }
    }

    if state.bomen_has_source && (state.bomen_placing || state.bomen_clone_placed) {
        let src_cx = state.bomen_tap_x * w as f32;
        let src_cy = state.bomen_tap_y * h as f32;
        let live = if state.bomen_placing { 0.0 } else { 1.0 };
        let drift_x =
            drift_amt * live * 14.0 * (t * 0.9).sin() + drift_amt * live * 9.0 * (t * 1.3).cos();
        let drift_y =
            drift_amt * live * 12.0 * (t * 1.1).cos() + drift_amt * live * 8.0 * (t * 0.7).sin();
        let scale = (1.0 + pulse_amt * 0.14 * t.sin()).max(0.35);
        let base_px = state.bomen_clone_x * w as f32;
        let base_py = state.bomen_clone_y * h as f32;

        let layers = [
            (echo_amt * 0.35, -12.0 * scale, -8.0 * scale, 0.55),
            (echo_amt * 0.55, -6.0 * scale, -4.0 * scale, 0.75),
            (1.0, 0.0, 0.0, 1.0),
        ];

        for (layer_a, off_x, off_y, layer_s) in layers {
            if layer_a < 0.02 {
                continue;
            }
            let clone_cx = base_px + drift_x + off_x;
            let clone_cy = base_py + drift_y + off_y;

            for y in 0..hh {
                for x in 0..ww {
                    let dx = (x as f32 - clone_cx) / scale;
                    let dy = (y as f32 - clone_cy) / scale;
                    let sx = (src_cx + dx).round() as i32;
                    let sy = (src_cy + dy).round() as i32;
                    if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 {
                        continue;
                    }
                    let src = sy as usize * ww + sx as usize;
                    let ma = state.bomen_mask[src] * layer_a * layer_s;
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
    }

    if state.bomen_has_source && !state.exporting {
        stamp_reticle(
            &mut out,
            w,
            h,
            state.bomen_tap_x,
            state.bomen_tap_y,
            0.75 + pulse_amt * 0.2 * t.sin().abs(),
        );
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flood_selects_connected_region() {
        let w = 16u32;
        let h = 16u32;
        let mut rgb = vec![10u8; (w * h * 3) as usize];
        for y in 4..10 {
            for x in 4..10 {
                let i = ((y * w + x) * 3) as usize;
                rgb[i] = 200;
                rgb[i + 1] = 200;
                rgb[i + 2] = 200;
            }
        }
        let mask = flood_mask(&rgb, w, h, 6, 6, 0.5, 0.35);
        assert!(mask[(6 * 16 + 6) as usize] > 0.0);
        assert_eq!(mask[15], 0.0, "far corner should stay unselected");
    }

    #[test]
    fn flood_stays_near_the_tap() {
        let w = 64u32;
        let h = 48u32;
        let rgb = vec![180u8; (w * h * 3) as usize];
        let mask = flood_mask(&rgb, w, h, 48, 24, 1.0, 0.15);
        assert!(mask[(24 * 64 + 48) as usize] > 0.0);
        assert_eq!(mask[24 * 64 + 4], 0.0, "left edge must not be selected");
    }
}
