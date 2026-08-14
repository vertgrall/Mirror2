//! Shared imaging ops — convolutions, sampling, morphology building blocks.

pub fn rgb_to_rgba(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let n = (w as usize) * (h as usize);
    let mut out = vec![0u8; n * 4];
    for i in 0..n {
        out[i * 4] = rgb[i * 3];
        out[i * 4 + 1] = rgb[i * 3 + 1];
        out[i * 4 + 2] = rgb[i * 3 + 2];
        out[i * 4 + 3] = 255;
    }
    out
}

/// Blend look RGBA toward dry RGB. `wet` 0 = camera, 1 = full look.
pub fn mix_look_over_rgb(look_rgba: &mut [u8], rgb: &[u8], wet: f32) {
    let t = wet.clamp(0.0, 1.0);
    if t > 0.99 {
        return;
    }
    let dry = 1.0 - t;
    let n = rgb.len() / 3;
    for i in 0..n {
        let o = i * 4;
        let s = i * 3;
        look_rgba[o] = (look_rgba[o] as f32 * t + rgb[s] as f32 * dry) as u8;
        look_rgba[o + 1] = (look_rgba[o + 1] as f32 * t + rgb[s + 1] as f32 * dry) as u8;
        look_rgba[o + 2] = (look_rgba[o + 2] as f32 * t + rgb[s + 2] as f32 * dry) as u8;
    }
}

pub fn lum(r: u8, g: u8, b: u8) -> f32 {
    (r as f32 * 0.2126 + g as f32 * 0.7152 + b as f32 * 0.0722) / 255.0
}

pub fn gray(rgb: &[u8], w: u32, h: u32) -> Vec<f32> {
    let n = (w * h) as usize;
    let mut g = vec![0f32; n];
    for i in 0..n {
        g[i] = lum(rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]);
    }
    g
}

pub fn sample_rgb(rgb: &[u8], w: u32, h: u32, fx: f32, fy: f32) -> (u8, u8, u8) {
    let x = fx.clamp(0.0, w as f32 - 1.001);
    let y = fy.clamp(0.0, h as f32 - 1.001);
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;

    let i00 = ((y0 * w + x0) as usize) * 3;
    let i10 = ((y0 * w + x1) as usize) * 3;
    let i01 = ((y1 * w + x0) as usize) * 3;
    let i11 = ((y1 * w + x1) as usize) * 3;

    let mut c = [0f32; 3];
    for ch in 0..3 {
        let v = rgb[i00 + ch] as f32 * (1.0 - tx) * (1.0 - ty)
            + rgb[i10 + ch] as f32 * tx * (1.0 - ty)
            + rgb[i01 + ch] as f32 * (1.0 - tx) * ty
            + rgb[i11 + ch] as f32 * tx * ty;
        c[ch] = v;
    }
    (c[0] as u8, c[1] as u8, c[2] as u8)
}

#[allow(dead_code)]
pub fn gradient(g: &[f32], w: u32, h: u32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let ww = w as usize;
    let hh = h as usize;
    let n = ww * hh;
    let mut gx = vec![0f32; n];
    let mut gy = vec![0f32; n];
    let mut mag = vec![0f32; n];
    for y in 1..hh - 1 {
        for x in 1..ww - 1 {
            let i = y * ww + x;
            let dx = g[i + 1] - g[i - 1];
            let dy = g[i + ww] - g[i - ww];
            gx[i] = dx;
            gy[i] = dy;
            mag[i] = (dx * dx + dy * dy).sqrt();
        }
    }
    (gx, gy, mag)
}


#[allow(dead_code)]
pub fn dilate_n(mask: &[u8], w: u32, h: u32, passes: u32) -> Vec<u8> {
    if passes == 0 {
        return mask.to_vec();
    }
    let mut out = mask.to_vec();
    for _ in 0..passes {
        out = dilate3(&out, w, h);
    }
    out
}

#[allow(dead_code)]
pub fn erode_n(mask: &[u8], w: u32, h: u32, passes: u32) -> Vec<u8> {
    if passes == 0 {
        return mask.to_vec();
    }
    let mut out = mask.to_vec();
    for _ in 0..passes {
        out = erode3(&out, w, h);
    }
    out
}

/// 3×3 binary erosion
#[allow(dead_code)]
pub fn erode3(mask: &[u8], w: u32, h: u32) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let mut out = vec![0u8; ww * hh];
    for y in 1..hh - 1 {
        for x in 1..ww - 1 {
            let mut ok = true;
            'neigh: for dy in -1..=1 {
                for dx in -1..=1 {
                    if mask[(y as i32 + dy) as usize * ww + (x as i32 + dx) as usize] == 0 {
                        ok = false;
                        break 'neigh;
                    }
                }
            }
            out[y * ww + x] = if ok { 255 } else { 0 };
        }
    }
    out
}

/// 3×3 binary dilation
pub fn dilate3(mask: &[u8], w: u32, h: u32) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let mut out = vec![0u8; ww * hh];
    for y in 1..hh - 1 {
        for x in 1..ww - 1 {
            let mut any = false;
            'neigh: for dy in -1..=1 {
                for dx in -1..=1 {
                    if mask[(y as i32 + dy) as usize * ww + (x as i32 + dx) as usize] > 127 {
                        any = true;
                        break 'neigh;
                    }
                }
            }
            out[y * ww + x] = if any { 255 } else { 0 };
        }
    }
    out
}

pub fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}

/// Top bar height and active picture height for 16:9 content in a 4:3 frame.
pub fn letterbox_bars_16x9(h: u32) -> (usize, usize) {
    let active = ((h as f32) * 9.0 / 12.0).round() as usize;
    let bar = ((h as usize).saturating_sub(active)) / 2;
    (bar, active)
}

pub fn hash2d(x: f32, y: f32) -> f32 {
    let mut n = x.sin() * 43758.5453 + y.cos() * 23421.631;
    n = (n.fract().abs() * 43758.5453).fract();
    n
}

#[allow(dead_code)]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h.fract().max(0.0);
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    (
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}
