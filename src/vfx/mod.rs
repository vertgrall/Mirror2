//! VFX pipeline — composite → look → atmosphere.

mod atmo;
mod bg;
mod cctv;
mod composite;
mod gx;
mod morph;
mod ops;
mod osd;
mod params;
mod ripple;
mod state;
mod vhs;

pub use bg::{
    cycle_background, current_path, param_defs as bg_param_defs, params_from_values,
    set_params as set_background, BackgroundParams,
};
pub use atmo::{
    param_defs as atmo_param_defs, set_params as set_atmosphere, AtmosphereParams,
};
pub use params::{current_params, set_params, LookParams, ParamDef};
pub use state::VfxState;

use std::sync::{Mutex, OnceLock};

static STATE: OnceLock<Mutex<VfxState>> = OnceLock::new();

fn vfx_state() -> &'static Mutex<VfxState> {
    STATE.get_or_init(|| Mutex::new(VfxState::default()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    None,
    Morph,
    Vhs,
    Gx,
    Cctv,
    Ripple,
}

impl Look {
    pub const RAIL: [Self; 6] = [
        Self::None,
        Self::Morph,
        Self::Vhs,
        Self::Gx,
        Self::Cctv,
        Self::Ripple,
    ];

    pub fn id(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Morph => 1,
            Self::Vhs => 2,
            Self::Gx => 3,
            Self::Cctv => 4,
            Self::Ripple => 5,
        }
    }

    pub fn from_id(id: u8) -> Self {
        match id {
            1 => Self::Morph,
            2 => Self::Vhs,
            3 => Self::Gx,
            4 => Self::Cctv,
            5 => Self::Ripple,
            _ => Self::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "OFF",
            Self::Morph => "MORPH",
            Self::Vhs => "VHS",
            Self::Gx => "GX",
            Self::Cctv => "CCTV",
            Self::Ripple => "RIPPLE",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::None => "clean camera · pick a look",
            Self::Morph => "ink drawing · wet mixes back to photo",
            Self::Vhs => "tracking · chroma bleed · tape wear",
            Self::Gx => "Hi8 warmth · comb · MAR 14 1994",
            Self::Cctv => "blocky green-grey · crushed",
            Self::Ripple => "water rings · VHS cam through a puddle",
        }
    }

    /// One line on a sheet tile — what the look does, not a brand name.
    pub fn tile_line(self) -> &'static str {
        match self {
            Self::None => "clean camera",
            Self::Morph => "ink drawing",
            Self::Vhs => "tracking · wear",
            Self::Gx => "Hi8 · 1994",
            Self::Cctv => "blocky · crushed",
            Self::Ripple => "water rings",
        }
    }

    pub fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

/// RGB (mirrored) → RGBA through full pipeline.
pub fn apply(look: Look, rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let n = (w as usize) * (h as usize);
    assert_eq!(rgb.len(), n * 3);

    let look_params = current_params();
    let bg_params = bg::current_params();
    let atmo_params = atmo::current_params();
    let plate = bg::plate_for(w, h);

    let Ok(mut state) = vfx_state().lock() else {
        return ops::rgb_to_rgba(rgb, w, h);
    };
    state.tick(rgb, w, h);

    let composited = composite::apply(rgb, w, h, plate.as_deref(), &bg_params);
    let wet = look_params.wet();
    let rgba = if look.is_none() || wet < 0.01 {
        ops::rgb_to_rgba(&composited, w, h)
    } else {
        let mut looked = apply_look(look, &composited, w, h, &state, &look_params);
        if wet < 0.99 {
            ops::mix_look_over_rgb(&mut looked, &composited, wet);
        }
        looked
    };
    atmo::apply(&rgba, w, h, &state, &atmo_params)
}

fn apply_look(
    look: Look,
    rgb: &[u8],
    w: u32,
    h: u32,
    state: &VfxState,
    params: &LookParams,
) -> Vec<u8> {
    match look {
        Look::None => ops::rgb_to_rgba(rgb, w, h),
        Look::Morph => morph::apply(rgb, w, h, params),
        Look::Vhs => vhs::apply(rgb, w, h, state, params),
        Look::Gx => gx::apply(rgb, w, h, state, params),
        Look::Cctv => cctv::apply(rgb, w, h, state, params),
        Look::Ripple => ripple::apply(rgb, w, h, state, params),
    }
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

pub fn downscale_rgb(src: &[u8], sw: u32, sh: u32, max_w: u32) -> (u32, u32, Vec<u8>) {
    if sw <= max_w {
        return (sw, sh, src.to_vec());
    }
    let tw = max_w;
    let th = ((sh as f32) * (tw as f32) / (sw as f32)).round().max(1.0) as u32;
    let mut out = vec![0u8; (tw as usize) * (th as usize) * 3];
    for y in 0..th {
        let sy = y * sh / th;
        for x in 0..tw {
            let sx = x * sw / tw;
            let si = ((sy * sw + sx) as usize) * 3;
            let di = ((y * tw + x) as usize) * 3;
            out[di..di + 3].copy_from_slice(&src[si..si + 3]);
        }
    }
    (tw, th, out)
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
                // Green screen backdrop for compositing demos
                let n = (hash32(x, y) % 5) as u8;
                (40 + n, 180 + n, 50 + n)
            };
            rgb[i] = r;
            rgb[i + 1] = g;
            rgb[i + 2] = b;
        }
    }
    rgb
}

fn hash32(x: u32, y: u32) -> u32 {
    let mut n = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}

fn dist(x: f32, y: f32, ox: f32, oy: f32) -> f32 {
    let dx = x - ox;
    let dy = y - oy;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use atmo::set_params as set_atmo;

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
        set_atmo(AtmosphereParams::default());
        let rgb = sample_rgb(32, 24);
        for look in [
            Look::None,
            Look::Morph,
            Look::Vhs,
            Look::Gx,
            Look::Cctv,
            Look::Ripple,
        ] {
            set_params(LookParams::defaults(look));
            let out = apply(look, &rgb, 32, 24);
            assert_eq!(out.len(), 32 * 24 * 4, "{look:?}");
            assert!(out.chunks(4).all(|p| p[3] == 255), "{look:?} alpha");
        }
    }

    #[test]
    fn morph_produces_edges() {
        let mut rgb = vec![255u8; 32 * 24 * 3];
        for y in 8..16 {
            for x in 8..24 {
                let i = (y * 32 + x) * 3;
                rgb[i] = 20;
                rgb[i + 1] = 20;
                rgb[i + 2] = 20;
            }
        }
        set_params(LookParams::defaults(Look::Morph));
        set_atmo(AtmosphereParams { smoke: 0.0, ..Default::default() });
        let out = apply(Look::Morph, &rgb, 32, 24);
        let dark = out.chunks(4).filter(|p| p[0] < 80).count();
        assert!(dark > 40, "morph should ink edges, got {dark}");
    }

    #[test]
    fn vhs_chroma_survives_edge_jitter() {
        set_params(LookParams::defaults(Look::Vhs));
        set_atmo(AtmosphereParams { smoke: 0.0, ..Default::default() });
        // Realistic preview size + default tracking/chroma — previously OOB on jittered sx.
        let rgb = vec![128u8; 640 * 480 * 3];
        let out = apply(Look::Vhs, &rgb, 640, 480);
        assert_eq!(out.len(), 640 * 480 * 4);
    }

    #[test]
    fn wet_zero_is_dry_camera() {
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        let rgb = sample_rgb(32, 24);
        let mut p = LookParams::defaults(Look::Morph);
        p.values[0] = 0.0;
        set_params(p);
        let out = apply(Look::Morph, &rgb, 32, 24);
        assert_eq!(out, ops::rgb_to_rgba(&rgb, 32, 24));
    }

    #[test]
    fn ripple_stays_in_bounds() {
        set_params(LookParams::defaults(Look::Ripple));
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        let rgb = vec![90u8; 640 * 480 * 3];
        let out = apply(Look::Ripple, &rgb, 640, 480);
        assert_eq!(out.len(), 640 * 480 * 4);
        assert!(out.chunks(4).all(|px| px[3] == 255));
    }

    #[test]
    fn none_look_is_clean_passthrough() {
        set_atmo(AtmosphereParams::default());
        set_params(LookParams::defaults(Look::None));
        let rgb = sample_rgb(32, 24);
        let out = apply(Look::None, &rgb, 32, 24);
        assert_eq!(out, ops::rgb_to_rgba(&rgb, 32, 24));
    }

    #[test]
    fn app_defaults_are_no_fx() {
        assert!(Look::None.is_none());
        assert_eq!(Look::from_id(0), Look::None);
        assert!(!BackgroundParams::default().enabled);
        assert!(AtmosphereParams::default().smoke < 0.01);
        assert!(Look::None.param_defs().is_empty());
    }

    #[test]
    fn haze_modifies_rgba() {
        let rgb = sample_rgb(32, 24);
        let rgba = ops::rgb_to_rgba(&rgb, 32, 24);
        let state = VfxState::default();
        let out = atmo::apply(
            &rgba,
            32,
            24,
            &state,
            &AtmosphereParams {
                smoke: 0.8,
                density: 0.8,
                drift: 0.4,
                scale: 0.55,
            },
        );
        assert_ne!(out, rgba);
    }

    #[test]
    fn wet_pct_changes_morph_output() {
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        let rgb = sample_rgb(32, 24);
        let def = Look::Morph.param_defs()[0];
        let mut dry = LookParams::defaults(Look::Morph);
        assert!(dry.apply_pct(0, def, 0.0));
        set_params(dry);
        let a = apply(Look::Morph, &rgb, 32, 24);

        let mut wet = LookParams::defaults(Look::Morph);
        assert!(
            !wet.apply_pct(0, def, 100.0),
            "Morph wet default is already 100%"
        );
        set_params(wet);
        let b = apply(Look::Morph, &rgb, 32, 24);

        assert_ne!(a, b, "wet 0 vs 100 must change the image");
        assert!((dry.v(0) - 0.0).abs() < 0.001);
        assert!((wet.v(0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn gx_stamp_is_frozen_1994() {
        assert_eq!(osd::GX_DATE, "MAR 14 1994");
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
