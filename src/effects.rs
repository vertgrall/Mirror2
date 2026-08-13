//! Ways of seeing — art processes, not carnival filters.
//!
//! Photo Booth had Thermal, X-Ray, Light Tunnel. Those stay in the booth.
//! These are copy-shop, school-desk, and ballpoint processes.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    Plain,
    Xerox,
    Carbon,
    Ballpoint,
    Ruled,
}

impl Look {
    pub const ALL: [Self; 5] = [
        Self::Plain,
        Self::Xerox,
        Self::Carbon,
        Self::Ballpoint,
        Self::Ruled,
    ];

    pub fn id(self) -> u8 {
        match self {
            Self::Plain => 0,
            Self::Xerox => 1,
            Self::Carbon => 2,
            Self::Ballpoint => 3,
            Self::Ruled => 4,
        }
    }

    pub fn from_id(id: u8) -> Self {
        match id {
            1 => Self::Xerox,
            2 => Self::Carbon,
            3 => Self::Ballpoint,
            4 => Self::Ruled,
            _ => Self::Plain,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Plain => "PLAIN",
            Self::Xerox => "XEROX",
            Self::Carbon => "CARBON",
            Self::Ballpoint => "BALLPOINT",
            Self::Ruled => "RULED",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Plain => "as you are",
            Self::Xerox => "the copy machine",
            Self::Carbon => "the tissue underneath",
            Self::Ballpoint => "cheap pen, cheap paper",
            Self::Ruled => "the composition book",
        }
    }
}

/// RGB (3 bytes/pixel) → RGBA. `src` is already mirrored.
pub fn apply(look: Look, rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let n = (w as usize) * (h as usize);
    assert_eq!(rgb.len(), n * 3);
    match look {
        Look::Plain => plain(rgb, w, h),
        Look::Xerox => xerox(rgb, w, h),
        Look::Carbon => carbon(rgb, w, h),
        Look::Ballpoint => ballpoint(rgb, w, h),
        Look::Ruled => ruled(rgb, w, h),
    }
}

fn lum(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 * 77 + g as u16 * 150 + b as u16 * 29) >> 8) as u8
}

fn hash32(x: u32, y: u32) -> u32 {
    let mut n = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}

fn grain(x: u32, y: u32) -> i16 {
    (hash32(x, y) % 11) as i16 - 5
}

fn plain(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) as usize) * 3;
            let o = ((y * w + x) as usize) * 4;
            out[o] = rgb[i];
            out[o + 1] = rgb[i + 1];
            out[o + 2] = rgb[i + 2];
            out[o + 3] = 255;
        }
    }
    out
}

fn xerox(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) as usize) * 3;
            let o = ((y * w + x) as usize) * 4;
            let mut yv = lum(rgb[i], rgb[i + 1], rgb[i + 2]) as i16;
            yv += grain(x, y);
            // Soft-knee crush — a tired office copier, not a clean bitmap.
            let ink = if yv < 92 {
                18
            } else if yv > 168 {
                242
            } else {
                let t = (yv - 92) as f32 / 76.0;
                (18.0 + t * t * 224.0) as u8
            };
            let mut r = ink;
            let mut g = ink.saturating_sub(2);
            let mut b = ink.saturating_sub(6);
            let speck = hash32(x.wrapping_mul(3), y.wrapping_mul(7));
            if speck % 1103 == 0 {
                r = 12;
                g = 10;
                b = 8;
            } else if speck % 1409 == 0 {
                r = 250;
                g = 248;
                b = 242;
            }
            out[o] = r;
            out[o + 1] = g;
            out[o + 2] = b;
            out[o + 3] = 255;
        }
    }
    out
}

fn carbon(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) as usize) * 3;
            let o = ((y * w + x) as usize) * 4;
            let yv = lum(rgb[i], rgb[i + 1], rgb[i + 2]) as f32 / 255.0;
            let ink = 1.0 - yv;
            // Carbon paper: indigo on pale tissue.
            let r = (210.0 - ink * 175.0) as u8;
            let g = (214.0 - ink * 95.0) as u8;
            let b = (232.0 - ink * 28.0) as u8;
            out[o] = r;
            out[o + 1] = g;
            out[o + 2] = b;
            out[o + 3] = 255;
        }
    }
    out
}

fn ballpoint(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let mut gray = vec![0u8; ww * hh];
    for i in 0..ww * hh {
        gray[i] = lum(rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]);
    }

    let mut out = vec![0u8; ww * hh * 4];
    let paper = (236u8, 224u8, 198u8);
    for y in 0..hh {
        for x in 0..ww {
            let i = y * ww + x;
            let l = gray[i] as i16;
            let gx = {
                let xl = gray[y * ww + x.saturating_sub(1)] as i16;
                let xr = gray[y * ww + (x + 1).min(ww - 1)] as i16;
                xr - xl
            };
            let gy = {
                let yu = gray[y.saturating_sub(1) * ww + x] as i16;
                let yd = gray[(y + 1).min(hh - 1) * ww + x] as i16;
                yd - yu
            };
            let edge = ((gx.abs() + gy.abs()) / 2) as u8;

            let hatch_a = ((x + y) % 4) == 0;
            let hatch_b = ((x + y * 2) % 6) == 0;
            let hatch_c = ((x * 2 + y) % 8) == 0;

            let mut ink = 0.0f32;
            if edge > 36 {
                ink = (edge as f32 / 180.0).clamp(0.35, 1.0);
            } else if l < 70 && hatch_a {
                ink = 0.82;
            } else if l < 120 && hatch_b {
                ink = 0.55;
            } else if l < 170 && hatch_c {
                ink = 0.28;
            } else if l < 200 && ((x + y) % 11 == 0) {
                ink = 0.12;
            }

            let o = i * 4;
            out[o] = lerp(paper.0, 22, ink);
            out[o + 1] = lerp(paper.1, 58, ink);
            out[o + 2] = lerp(paper.2, 132, ink);
            out[o + 3] = 255;
        }
    }
    out
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

fn ruled(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    let line_every = (h as f32 / 18.0).max(10.0) as u32;
    let margin_x = (w as f32 * 0.12) as u32;
    for y in 0..h {
        let on_rule = y > line_every / 2 && y % line_every == 0;
        for x in 0..w {
            let i = ((y * w + x) as usize) * 3;
            let o = ((y * w + x) as usize) * 4;
            let yv = lum(rgb[i], rgb[i + 1], rgb[i + 2]) as f32 / 255.0;
            let ink = (1.0 - yv).powf(1.15);
            // Blue-black school ink on cheap ruled stock.
            let mut r = lerp(244, 24, ink);
            let mut g = lerp(240, 36, ink);
            let mut b = lerp(226, 78, ink);
            if on_rule && ink < 0.55 {
                r = r.saturating_sub(8);
                g = g.saturating_sub(18);
                b = 210;
            }
            if x == margin_x || x == margin_x + 1 {
                r = 196;
                g = 64;
                b = 72;
            }
            out[o] = r;
            out[o + 1] = g;
            out[o + 2] = b;
            out[o + 3] = 255;
        }
    }
    out
}

/// Crude drawn witness used when the machine is blind (no camera).
pub fn standin_rgb(w: u32, h: u32, t: f32) -> Vec<u8> {
    let mut rgb = vec![0u8; (w as usize) * (h as usize) * 3];
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.48;
    let rx = w as f32 * 0.22;
    let ry = h as f32 * 0.30;
    let blink = ((t * 0.35).sin().abs() < 0.04) as i32;
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) as usize) * 3;
            let nx = (x as f32 - cx) / rx;
            let ny = (y as f32 - cy) / ry;
            let in_head = nx * nx + ny * ny < 1.0;
            let eye_y = cy - ry * 0.18;
            let eye_dx = rx * 0.38;
            let left = dist(x as f32, y as f32, cx - eye_dx, eye_y) < rx * 0.07 && blink == 0;
            let right = dist(x as f32, y as f32, cx + eye_dx, eye_y) < rx * 0.07 && blink == 0;
            let mouth = {
                let mx = (x as f32 - cx) / (rx * 0.35);
                let my = (y as f32 - (cy + ry * 0.28)) / (ry * 0.06);
                mx.abs() < 1.0 && my.abs() < 1.0 && my > -0.2
            };
            let (r, g, b) = if left || right {
                (28, 24, 18)
            } else if mouth {
                (48, 32, 28)
            } else if in_head {
                (214, 186, 158)
            } else {
                let g = 198 + ((hash32(x, y) % 7) as u8);
                (228, g, 170)
            };
            rgb[i] = r;
            rgb[i + 1] = g;
            rgb[i + 2] = b;
        }
    }
    rgb
}

fn dist(x: f32, y: f32, ox: f32, oy: f32) -> f32 {
    let dx = x - ox;
    let dy = y - oy;
    (dx * dx + dy * dy).sqrt()
}

pub fn mirror_rgb(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let ww = w as usize;
    let hh = h as usize;
    let mut out = vec![0u8; ww * hh * 3];
    for y in 0..hh {
        for x in 0..ww {
            let si = (y * ww + (ww - 1 - x)) * 3;
            let di = (y * ww + x) * 3;
            out[di..di + 3].copy_from_slice(&rgb[si..si + 3]);
        }
    }
    out
}

pub fn downscale_rgba(src: &[u8], sw: u32, sh: u32, max_w: u32) -> (u32, u32, Vec<u8>) {
    if sw <= max_w {
        return (sw, sh, src.to_vec());
    }
    let tw = max_w;
    let th = ((sh as f32) * (tw as f32) / (sw as f32)).round().max(1.0) as u32;
    let mut out = vec![0u8; (tw as usize) * (th as usize) * 4];
    for y in 0..th {
        let sy = y * sh / th;
        for x in 0..tw {
            let sx = x * sw / tw;
            let si = ((sy * sw + sx) as usize) * 4;
            let di = ((y * tw + x) as usize) * 4;
            out[di..di + 4].copy_from_slice(&src[si..si + 4]);
        }
    }
    (tw, th, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rgb(w: u32, h: u32) -> Vec<u8> {
        let mut v = vec![0u8; (w * h * 3) as usize];
        for i in 0..w * h {
            let i = i as usize;
            v[i * 3] = (i % 256) as u8;
            v[i * 3 + 1] = 80;
            v[i * 3 + 2] = 160;
        }
        v
    }

    #[test]
    fn every_look_preserves_size() {
        let rgb = sample_rgb(32, 24);
        for look in Look::ALL {
            let out = apply(look, &rgb, 32, 24);
            assert_eq!(out.len(), 32 * 24 * 4, "{look:?}");
            assert!(out.chunks(4).all(|p| p[3] == 255), "{look:?} alpha");
        }
    }

    #[test]
    fn xerox_is_high_contrast() {
        let rgb = sample_rgb(48, 32);
        let out = apply(Look::Xerox, &rgb, 48, 32);
        let extremes = out.chunks(4).filter(|p| p[0] < 40 || p[0] > 220).count();
        assert!(
            extremes > 200,
            "xerox should crush most tones, got {extremes}"
        );
    }

    #[test]
    fn mirror_flips_horizontally() {
        let mut rgb = vec![0u8; 4 * 1 * 3];
        rgb[0] = 255;
        let out = mirror_rgb(&rgb, 4, 1);
        assert_eq!(out[9], 255);
        assert_eq!(out[0], 0);
    }
}
