//! Compatibility re-exports — implementation lives in `vfx`.

pub use crate::vfx::{
    apply, apply_export, atmo_param_defs, begin_selection, downscale_rgb, downscale_rgba, finish_selection,
    mirror_rgb, pointer_down, reset_interactive, reset_temporal, set_atmosphere, set_params,
    set_pointer, standin_rgb, update_selection, bomen_pointer_active, bomen_pointer_down,
    bomen_pointer_move, bomen_pointer_up, bomen_placing, AtmosphereParams, Look, LookFamily, LookParams, ParamDef,
};
