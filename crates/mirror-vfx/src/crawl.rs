//! CRAWL — analog static entities crawl across the subject like possessed TV snow.

use super::ops::{hash2d, rgb_to_rgba, sample_rgb};
use super::params::LookParams;
use super::state::VfxState;

pub fn apply(rgb: &[u8], w: u32, h: u32, state: &VfxState, p: &LookParams) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let swarm = p.v(1);
    let speed = p.v(2);
    let static_amt = p.v(3);

    let block = (12.0 + swarm * 36.0) as u32;
    let t = state.frame as f32 * (0.06 + speed * 0.18);
    let entity_count = (swarm * 24.0 + 8.0) as u32;

    let mut out = rgb.to_vec();

    // Crawling macroblock smears from ring history.
    for by in (0..h).step_by(block as usize) {
        for bx in (0..w).step_by(block as usize) {
            let seed = bx as f32 * 0.07 + by as f32 * 0.11;
            let crawl_x = (t * 1.3 + seed).sin() * (18.0 + swarm * 40.0);
            let crawl_y = (t * 0.9 + seed * 1.7).cos() * (12.0 + swarm * 28.0);
            let corrupt = hash2d(seed, t * 0.4);
            if corrupt < 0.35 - swarm * 0.2 {
                continue;
            }

            let depth = ((seed.abs() as u32) % 12 + 4) as usize;
            let mix = (0.35 + corrupt * 0.55) * (0.5 + static_amt * 0.5);

            for y in by..(by + block).min(h) {
                for x in bx..(bx + block).min(w) {
                    let i = (y as usize * ww + x as usize) * 3;
                    let (sr, sg, sb) = if let Some(past) = state.get_ring(depth) {
                        sample_rgb(
                            past,
                            w,
                            h,
                            x as f32 + crawl_x,
                            y as f32 + crawl_y,
                        )
                    } else {
                        (rgb[i], rgb[i + 1], rgb[i + 2])
                    };

                    out[i] = (out[i] as f32 * (1.0 - mix) + sr as f32 * mix) as u8;
                    out[i + 1] = (out[i + 1] as f32 * (1.0 - mix) + sg as f32 * mix) as u8;
                    out[i + 2] = (out[i + 2] as f32 * (1.0 - mix) + sb as f32 * mix) as u8;
                }
            }
        }
    }

    // Roaming static blobs.
    for e in 0..entity_count {
        let seed = e as f32 * 19.7 + 3.0;
        let ex = ((t * 0.55 + seed).sin() * 0.5 + 0.5) * w as f32;
        let ey = ((t * 0.47 + seed * 1.3).cos() * 0.5 + 0.5) * h as f32;
        let er = 16.0 + hash2d(seed, seed * 2.0) * 48.0 * (0.4 + swarm * 0.6);
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

                let n = hash2d(x as f32 * 0.41 + seed, y as f32 * 0.37 + t);
                let snow = (n * 255.0) as u8;
                let mix = falloff * (0.45 + static_amt * 0.55);

                out[i] = (out[i] as f32 * (1.0 - mix) + snow as f32 * mix) as u8;
                out[i + 1] = (out[i + 1] as f32 * (1.0 - mix) + snow as f32 * mix * 0.92) as u8;
                out[i + 2] = (out[i + 2] as f32 * (1.0 - mix) + snow as f32 * mix * 0.85) as u8;
            }
        }
    }

    // Full-frame TV grain.
    let grain = static_amt * 0.22 + 0.06;
    for y in 0..hh {
        for x in 0..ww {
            let i = (y * ww + x) * 3;
            let n = hash2d(x as f32 * 0.8, y as f32 * 0.8 + t * 2.0);
            if n > 1.0 - grain {
                let snow = (n * 255.0) as u8;
                let mix = (n - (1.0 - grain)) / grain;
                out[i] = (out[i] as f32 * (1.0 - mix) + snow as f32 * mix) as u8;
                out[i + 1] = (out[i + 1] as f32 * (1.0 - mix) + snow as f32 * mix) as u8;
                out[i + 2] = (out[i + 2] as f32 * (1.0 - mix) + snow as f32 * mix * 0.9) as u8;
            }
        }
    }

    rgb_to_rgba(&out, w, h)
}
