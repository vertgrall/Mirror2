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
        // Spill suppression on foreground
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
