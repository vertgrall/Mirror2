//! Atmosphere — smoke haze overlay.

use std::sync::{Mutex, OnceLock};

use super::ops::{hash2d, lerp_u8};
use super::state::VfxState;

#[derive(Clone, Copy, Debug)]
pub struct AtmosphereParams {
    pub smoke: f32,
    pub density: f32,
    pub drift: f32,
    pub scale: f32,
}

impl Default for AtmosphereParams {
    fn default() -> Self {
        Self {
            smoke: 0.0,
            density: 0.35,
            drift: 0.4,
            scale: 0.55,
        }
    }
}

static ATMO: OnceLock<Mutex<AtmosphereParams>> = OnceLock::new();

pub fn set_params(p: AtmosphereParams) {
    if let Ok(mut g) = ATMO.get_or_init(|| Mutex::new(AtmosphereParams::default())).lock() {
        *g = p;
    }
}

pub fn current_params() -> AtmosphereParams {
    ATMO.get_or_init(|| Mutex::new(AtmosphereParams::default()))
        .lock()
        .map(|g| *g)
        .unwrap_or_default()
}

pub fn param_defs() -> &'static [super::params::ParamDef] {
    &[
        super::params::ParamDef {
            label: "haze",
            min: 0.0,
            max: 1.0,
            default: 0.0,
        },
        super::params::ParamDef {
            label: "density",
            min: 0.0,
            max: 1.0,
            default: 0.35,
        },
        super::params::ParamDef {
            label: "drift",
            min: 0.0,
            max: 1.0,
            default: 0.4,
        },
        super::params::ParamDef {
            label: "scale",
            min: 0.2,
            max: 1.0,
            default: 0.55,
        },
    ]
}

pub fn apply(rgba: &[u8], w: u32, h: u32, state: &VfxState, p: &AtmosphereParams) -> Vec<u8> {
    if p.smoke < 0.01 {
        return rgba.to_vec();
    }
    let ww = w as usize;
    let hh = h as usize;
    let mut out = rgba.to_vec();
    let phase = state.smoke_phase * (0.3 + p.drift);
    let scale = 0.004 + p.scale * 0.012;

    for y in 0..hh {
        for x in 0..ww {
            let n1 = hash2d(
                x as f32 * scale + phase,
                y as f32 * scale * 0.7 + phase * 0.6,
            );
            let n2 = hash2d(
                x as f32 * scale * 2.1 - phase * 0.4,
                y as f32 * scale * 1.6 + phase * 1.1,
            );
            let cloud = (n1 * 0.55 + n2 * 0.45).powf(1.4 + p.density);
            let amt = cloud * p.smoke * p.density;
            let i = (y * ww + x) * 4;
            let lum = 200u8;
            out[i] = lerp_u8(out[i], lum, amt * 0.85);
            out[i + 1] = lerp_u8(out[i + 1], lum, amt * 0.88);
            out[i + 2] = lerp_u8(out[i + 2], lum, amt * 0.92);
        }
    }
    out
}
