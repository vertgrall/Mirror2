//! JAM — VHS transport jam: vertical bands, head-switch noise, frame freezes.

use super::ops::{hash2d, rgb_to_rgba, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let jam = p.v(1);
    let bands = p.v(2);
    let freeze = p.v(3);

    let mut out = vec![0u8; ww * hh * 3];
    let freeze_y = if freeze > 0.2 {
        (hash2d(state.frame as f32 * 0.07, jam) * hh as f32) as u32
    } else {
        u32::MAX
    };

    for y in 0..hh {
        let band_w = 12.0 + bands * 40.0;
        let band = (y as f32 / band_w).floor();
        let band_shift = if hash2d(band, state.frame as f32 * 0.15) > 1.0 - jam * 0.35 {
            ((hash2d(band + 3.0, state.frame as f32) - 0.5) * jam * 60.0) as i32
        } else {
            0
        };

        let row_y = if freeze > 0.2 && (y as u32).abs_diff(freeze_y) < (freeze * 18.0) as u32 {
            freeze_y as usize
        } else {
            y
        };

        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let sx = (x as i32 + band_shift).clamp(0, w as i32 - 1) as u32;
            let (mut r, mut g, mut b) = sample_rgb(rgb, w, h, sx as f32, row_y as f32);

            if jam > 0.1 && hash2d(x as f32 * 0.08, y as f32 * 0.02 + state.frame as f32) > 1.0 - jam * 0.12
            {
                let n = (hash2d(x as f32, y as f32) * 220.0) as u8;
                r = n;
                g = (n as f32 * 0.85) as u8;
                b = (n as f32 * 0.6) as u8;
            }

            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
        }
    }

    rgb_to_rgba(&out, w, h)
}
