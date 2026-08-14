//! Compatibility re-exports — implementation lives in `vfx`.

pub use crate::vfx::{
    apply, atmo_param_defs, downscale_rgb, downscale_rgba, mirror_rgb, reset_temporal,
    set_atmosphere, set_params, standin_rgb, AtmosphereParams, Look, LookParams, ParamDef,
};
