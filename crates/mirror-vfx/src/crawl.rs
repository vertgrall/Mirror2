//! CRAWL — analog static entities crawl across the subject like possessed TV snow.

use super::ops::{hash2d, sample_rgb, rgb_to_rgba};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let swarm = p.v(1);
    let speed = p.v(2);
    let static_amt = p.v(3);

    let entity_count = (swarm * 18.0 + 4.0) as u32;
    let mut out = rgb.to_vec();
    let t = state.frame as f32 * (0.04 + speed * 0.12);

    for e in 0..entity_count {
        let seed = e as f32 * 17.3;
        let ex = (hash2d(seed, t * 0.3).sin() * 0.5 + 0.5) * w as f32;
        let ey = (hash2d(seed + 4.0, t * 0.27).cos() * 0.5 + 0.5) * h as f32;
        let er = 8.0 + hash2d(seed, seed) * 24.0 * swarm;
        let er2 = er * er;

        let y0 = (ey - er).max(0.0) as u32;
        let y1 = (ey + er).min(h as f32 - 1.0) as u32;
        let x0 = (ex - er).max(0.0) as u32;
        let x1 = (ex + er).min(w as f32 - 1.0) as u32;

        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x as f32 - ex;
                let dy = y as f32 - ey;
                let d2 = dx * dx + dy * dy;
                if d2 > er2 {
                    continue;
                }
                let falloff = 1.0 - d2.sqrt() / er;
                let i = (y as usize * ww + x as usize) * 3;

                let depth = (e % 5 + 2) as usize;
                let (sr, sg, sb) = if let Some(past) = state.get_ring(depth) {
                    let shift = (t + seed).sin() * 22.0;
                    sample_rgb(past, w, h, x as f32 + shift, y as f32)
                } else {
                    (rgb[i], rgb[i + 1], rgb[i + 2])
                };

                let n = hash2d(x as f32 * 0.5 + seed, y as f32 * 0.5 + t);
                let snow = (n * 255.0) as u8;
                let mix = falloff * (0.55 + static_amt * 0.45);

                out[i] = (out[i] as f32 * (1.0 - mix) + sr as f32 * mix * 0.5 + snow as f32 * mix * 0.5) as u8;
                out[i + 1] = (out[i + 1] as f32 * (1.0 - mix) + sg as f32 * mix * 0.4 + snow as f32 * mix * 0.6) as u8;
                out[i + 2] = (out[i + 2] as f32 * (1.0 - mix) + sb as f32 * mix * 0.3 + snow as f32 * mix * 0.7) as u8;
            }
        }
    }

    rgb_to_rgba(&out, w, h)
}
