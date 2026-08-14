//! Chroma-key composite over a background plate.

use super::bg::BackgroundParams;

pub fn apply(
    fg: &[u8],
    w: u32,
    h: u32,
    plate: Option<&[u8]>,
    params: &BackgroundParams,
) -> Vec<u8> {
    let n = (w as usize) * (h as usize);
    if !params.enabled || plate.is_none() {
        return fg.to_vec();
    }
    let bg = plate.unwrap();
    assert_eq!(fg.len(), n * 3);
    assert_eq!(bg.len(), n * 3);

    if params.auto_key {
        return apply_color_key(fg, bg, w, h, params);
    }
    apply_hue_key(fg, bg, n, params)
}

/// Corner-sampled RGB distance key — keys uniform walls and green screen alike.
fn apply_color_key(fg: &[u8], bg: &[u8], w: u32, h: u32, params: &BackgroundParams) -> Vec<u8> {
    let n = (w as usize) * (h as usize);
    let (kr, kg, kb) = corner_key_rgb(fg, w, h);
    let mut out = vec![0u8; n * 3];
    // width slider scales how far from the corner color counts as background.
    let thresh = (0.06 + params.key_width * 0.55).max(0.04);
    let feather = (0.015 + params.feather * 0.12).max(0.005);
    let spill = params.spill;

    for i in 0..n {
        let o = i * 3;
        let r = fg[o] as f32 / 255.0;
        let g = fg[o + 1] as f32 / 255.0;
        let b = fg[o + 2] as f32 / 255.0;
        let dr = r - kr;
        let dg = g - kg;
        let db = b - kb;
        let dist = (dr * dr + dg * dg + db * db).sqrt();
        let mut matte = ((dist - thresh) / feather).clamp(0.0, 1.0);
        // Never eat very dark pixels (hair/shadows).
        if r.max(g).max(b) < 0.06 {
            matte = 1.0;
        }
        let mut fr = r;
        let mut fg_g = g;
        let mut fb = b;
        if matte < 1.0 {
            let spill_amt = (1.0 - matte) * spill;
            fr = r * (1.0 - spill_amt) + r.min(b) * spill_amt;
            fg_g = g * (1.0 - spill_amt * 1.2);
            fb = b * (1.0 - spill_amt) + r.min(b) * spill_amt;
        }
        let br = bg[o] as f32 / 255.0;
        let bg_g = bg[o + 1] as f32 / 255.0;
        let bb = bg[o + 2] as f32 / 255.0;
        out[o] = ((fr * matte + br * (1.0 - matte)).clamp(0.0, 1.0) * 255.0) as u8;
        out[o + 1] = ((fg_g * matte + bg_g * (1.0 - matte)).clamp(0.0, 1.0) * 255.0) as u8;
        out[o + 2] = ((fb * matte + bb * (1.0 - matte)).clamp(0.0, 1.0) * 255.0) as u8;
    }
    out
}

fn apply_hue_key(fg: &[u8], bg: &[u8], n: usize, params: &BackgroundParams) -> Vec<u8> {
    let mut out = vec![0u8; n * 3];
    let hue_c = params.key_hue;
    let width = params.key_width;
    let feather = params.feather.max(0.001);
    let spill = params.spill;

    for i in 0..n {
        let o = i * 3;
        let r = fg[o] as f32 / 255.0;
        let g = fg[o + 1] as f32 / 255.0;
        let b = fg[o + 2] as f32 / 255.0;
        let (h, s, v) = rgb_to_hsv(r, g, b);
        let dh = hue_dist(h, hue_c);
        let mut matte = ((dh - width * 0.5) / feather).clamp(0.0, 1.0);
        if s < 0.12 || v < 0.08 {
            matte = 1.0;
        }
        let mut fr = r;
        let mut fg_g = g;
        let mut fb = b;
        if matte < 1.0 {
            let spill_amt = (1.0 - matte) * spill;
            fr = r * (1.0 - spill_amt) + r.min(b) * spill_amt;
            fg_g = g * (1.0 - spill_amt * 1.2);
            fb = b * (1.0 - spill_amt) + r.min(b) * spill_amt;
        }
        let br = bg[o] as f32 / 255.0;
        let bg_g = bg[o + 1] as f32 / 255.0;
        let bb = bg[o + 2] as f32 / 255.0;
        out[o] = ((fr * matte + br * (1.0 - matte)).clamp(0.0, 1.0) * 255.0) as u8;
        out[o + 1] = ((fg_g * matte + bg_g * (1.0 - matte)).clamp(0.0, 1.0) * 255.0) as u8;
        out[o + 2] = ((fb * matte + bb * (1.0 - matte)).clamp(0.0, 1.0) * 255.0) as u8;
    }
    out
}

fn corner_key_rgb(fg: &[u8], w: u32, h: u32) -> (f32, f32, f32) {
    let ww = w as usize;
    let hh = h as usize;
    if ww == 0 || hh == 0 {
        return (0.0, 0.7, 0.0);
    }
    let margin = (ww.min(hh) / 12).clamp(3, 24);
    let mut rs = 0f64;
    let mut gs = 0f64;
    let mut bs = 0f64;
    let mut n = 0f64;
    let corners = [
        (0usize, 0usize),
        (ww.saturating_sub(margin), 0),
        (0, hh.saturating_sub(margin)),
        (ww.saturating_sub(margin), hh.saturating_sub(margin)),
    ];
    for (cx, cy) in corners {
        for dy in 0..margin {
            for dx in 0..margin {
                let x = cx + dx;
                let y = cy + dy;
                if x >= ww || y >= hh {
                    continue;
                }
                let i = (y * ww + x) * 3;
                rs += fg[i] as f64;
                gs += fg[i + 1] as f64;
                bs += fg[i + 2] as f64;
                n += 1.0;
            }
        }
    }
    if n < 1.0 {
        return (0.0, 0.7, 0.0);
    }
    (
        (rs / n / 255.0) as f32,
        (gs / n / 255.0) as f32,
        (bs / n / 255.0) as f32,
    )
}

fn hue_dist(a: f32, b: f32) -> f32 {
    let d = (a - b).abs();
    d.min(1.0 - d)
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d < 0.00001 {
        0.0
    } else if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    let s = if max < 0.00001 { 0.0 } else { d / max };
    (h.fract(), s, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backgrounds::{builtin_path, load_rgb};
    use crate::vfx::bg::BackgroundParams;

    #[test]
    fn auto_key_replaces_green_corners_with_plate() {
        let w = 64u32;
        let h = 48u32;
        let mut fg = vec![0u8; (w * h * 3) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 3) as usize;
                let in_face = {
                    let nx = (x as f32 - 32.0) / 14.0;
                    let ny = (y as f32 - 24.0) / 18.0;
                    nx * nx + ny * ny < 1.0
                };
                if in_face {
                    fg[i] = 210;
                    fg[i + 1] = 180;
                    fg[i + 2] = 150;
                } else {
                    fg[i] = 40;
                    fg[i + 1] = 180;
                    fg[i + 2] = 50;
                }
            }
        }
        let plate = load_rgb(&builtin_path("sky"), w, h).expect("sky plate");
        let params = BackgroundParams {
            enabled: true,
            auto_key: true,
            ..Default::default()
        };
        let out = apply(&fg, w, h, Some(&plate), &params);
        let corner = (out[0], out[1], out[2]);
        let face = (
            out[((24 * w + 32) * 3) as usize],
            out[((24 * w + 32) * 3 + 1) as usize],
            out[((24 * w + 32) * 3 + 2) as usize],
        );
        assert_ne!(corner, (fg[0], fg[1], fg[2]), "green corner should become plate");
        assert!(corner.2 > corner.0, "sky plate is blue at corners");
        assert!(face.0 > face.2, "face stays warm in the center");
    }
}
