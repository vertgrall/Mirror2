//! Background plate + chroma key params.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, Debug)]
pub struct BackgroundParams {
    pub enabled: bool,
    /// Sample frame corners each composite — works with walls, not just green screen.
    pub auto_key: bool,
    pub key_hue: f32,
    pub key_width: f32,
    pub feather: f32,
    pub spill: f32,
}

impl Default for BackgroundParams {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_key: true,
            key_hue: 0.33,
            key_width: 0.14,
            feather: 0.06,
            spill: 0.45,
        }
    }
}

struct BgSlot {
    params: BackgroundParams,
    path_index: usize,
    selected: Option<PathBuf>,
    plate: Option<Vec<u8>>,
    plate_w: u32,
    plate_h: u32,
}

impl Default for BgSlot {
    fn default() -> Self {
        Self {
            params: BackgroundParams::default(),
            path_index: 0,
            selected: None,
            plate: None,
            plate_w: 0,
            plate_h: 0,
        }
    }
}

static BG: OnceLock<Mutex<BgSlot>> = OnceLock::new();

fn slot() -> &'static Mutex<BgSlot> {
    BG.get_or_init(|| Mutex::new(BgSlot::default()))
}

pub fn set_params(p: BackgroundParams) {
    if let Ok(mut g) = slot().lock() {
        g.params = p;
    }
}

pub fn current_params() -> BackgroundParams {
    slot()
        .lock()
        .map(|g| g.params)
        .unwrap_or_default()
}

pub fn cycle_background() {
    let paths = crate::backgrounds::list_paths();
    if paths.is_empty() {
        return;
    }
    if let Ok(mut g) = slot().lock() {
        g.path_index = (g.path_index + 1) % paths.len();
        g.selected = None;
        g.plate = None;
    }
}

pub fn select_path(path: PathBuf) {
    if let Ok(mut g) = slot().lock() {
        g.selected = Some(path);
        g.plate = None;
        g.plate_w = 0;
        g.plate_h = 0;
    }
}

pub fn wear_plate(path: PathBuf) {
    select_path(path);
    let mut p = current_params();
    p.enabled = true;
    set_params(p);
}

pub fn select_preset(name: &str) {
    select_path(crate::backgrounds::builtin_path(name));
}

pub fn current_path() -> Option<PathBuf> {
    let paths = crate::backgrounds::list_paths();
    slot()
        .lock()
        .ok()
        .and_then(|g| {
            g.selected
                .clone()
                .or_else(|| paths.get(g.path_index).cloned())
        })
}

pub fn ensure_plate(w: u32, h: u32) {
    let paths = crate::backgrounds::list_paths();
    let Ok(mut g) = slot().lock() else {
        return;
    };
    if g.plate_w == w && g.plate_h == h && g.plate.is_some() {
        return;
    }
    if let Some(path) = active_path(&g, &paths) {
        g.plate = crate::backgrounds::load_rgb(&path, w, h);
        g.plate_w = w;
        g.plate_h = h;
    }
}

fn active_path(g: &BgSlot, paths: &[PathBuf]) -> Option<PathBuf> {
    g.selected
        .clone()
        .or_else(|| paths.get(g.path_index).cloned())
}

pub fn plate_for(w: u32, h: u32) -> Option<Vec<u8>> {
    ensure_plate(w, h);
    slot()
        .lock()
        .ok()
        .and_then(|g| g.plate.clone())
}

pub fn param_defs() -> &'static [super::params::ParamDef] {
    &[
        super::params::ParamDef {
            label: "key",
            min: 0.0,
            max: 1.0,
            default: 0.33,
        },
        super::params::ParamDef {
            label: "width",
            min: 0.02,
            max: 0.35,
            default: 0.14,
        },
        super::params::ParamDef {
            label: "feather",
            min: 0.01,
            max: 0.2,
            default: 0.06,
        },
        super::params::ParamDef {
            label: "spill",
            min: 0.0,
            max: 1.0,
            default: 0.45,
        },
    ]
}

pub fn params_from_values(v: [f32; 4], enabled: bool) -> BackgroundParams {
    BackgroundParams {
        enabled,
        auto_key: true,
        key_hue: v[0],
        key_width: v[1],
        feather: v[2],
        spill: v[3],
    }
}
