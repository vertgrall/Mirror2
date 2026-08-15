//! Compatibility re-exports — implementation lives in `vfx`.

pub use crate::vfx::{
    apply, atmo_param_defs, downscale_rgb, downscale_rgba, mirror_rgb, pointer_down, reset_temporal,
    set_atmosphere, set_params, set_pointer, standin_rgb, AtmosphereParams, Look, LookParams,
    ParamDef,
};
