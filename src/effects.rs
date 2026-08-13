//! Compatibility re-exports — implementation lives in `vfx`.

pub use crate::vfx::{
    apply, atmo_param_defs, bg_param_defs, cycle_background, current_path, downscale_rgb,
    downscale_rgba, mirror_rgb, params_from_values as bg_params_from_values, set_atmosphere,
    set_background, set_params, standin_rgb, AtmosphereParams, BackgroundParams, Look,
    LookGroup, LookParams, ParamDef,
};
