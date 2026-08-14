//! Per-look slider definitions and runtime values.
//!
//! Slot 0 is always **wet** (0 = dry camera, 1 = full look).
//! Slots 1–3 are named for what that look actually does.

use std::sync::{Mutex, OnceLock};

use super::Look;

#[derive(Clone, Copy, Debug)]
pub struct ParamDef {
    pub label: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LookParams {
    pub values: [f32; 4],
}

static PARAMS: OnceLock<Mutex<LookParams>> = OnceLock::new();

fn lock() -> &'static Mutex<LookParams> {
    PARAMS.get_or_init(|| Mutex::new(LookParams::defaults(Look::None)))
}

pub fn set_params(p: LookParams) {
    if let Ok(mut g) = lock().lock() {
        *g = p;
    }
}

pub fn current_params() -> LookParams {
    lock()
        .lock()
        .map(|g| *g)
        .unwrap_or_else(|_| LookParams::defaults(Look::None))
}

impl LookParams {
    pub fn defaults(look: Look) -> Self {
        let mut values = [0.0; 4];
        for (i, def) in look.param_defs().iter().enumerate() {
            values[i] = def.default;
        }
        Self { values }
    }

    pub fn v(&self, i: usize) -> f32 {
        self.values[i]
    }

    pub fn wet(self) -> f32 {
        self.values[0].clamp(0.0, 1.0)
    }

    /// Apply a 0–100 slider percentage to slot `index`. Returns true if the value changed.
    pub fn apply_pct(&mut self, index: usize, def: ParamDef, pct: f64) -> bool {
        let v = def.from_pct(pct);
        if (self.values[index] - v).abs() < 0.0005 {
            return false;
        }
        self.values[index] = v;
        true
    }
}

impl ParamDef {
    pub fn to_pct(self, value: f32) -> f64 {
        let span = self.max - self.min;
        if span <= 0.0 {
            return 0.0;
        }
        (((value - self.min) / span) * 100.0).clamp(0.0, 100.0) as f64
    }

    pub fn from_pct(self, pct: f64) -> f32 {
        let t = (pct as f32 / 100.0).clamp(0.0, 1.0);
        self.min + t * (self.max - self.min)
    }
}

const WET: ParamDef = ParamDef {
    label: "wet",
    min: 0.0,
    max: 1.0,
    default: 1.0,
};

impl Look {
    pub fn param_defs(self) -> &'static [ParamDef] {
        match self {
            Look::None => &[],
            Look::Morph => &[
                WET,
                ParamDef {
                    label: "edge",
                    min: 0.02,
                    max: 0.25,
                    default: 0.08,
                },
                ParamDef {
                    label: "ink",
                    min: 0.3,
                    max: 1.0,
                    default: 0.92,
                },
                ParamDef {
                    label: "fill",
                    min: 0.0,
                    max: 0.4,
                    default: 0.18,
                },
            ],
            Look::Vhs => &[
                WET,
                ParamDef {
                    label: "track",
                    min: 0.0,
                    max: 1.0,
                    default: 0.45,
                },
                ParamDef {
                    label: "chroma",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "wear",
                    min: 0.0,
                    max: 1.0,
                    default: 0.35,
                },
            ],
            Look::Gx => &[
                WET,
                ParamDef {
                    label: "warmth",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "comb",
                    min: 0.0,
                    max: 1.0,
                    default: 0.65,
                },
                ParamDef {
                    label: "stamp",
                    min: 0.0,
                    max: 1.0,
                    default: 0.85,
                },
            ],
            Look::Cctv => &[
                WET,
                ParamDef {
                    label: "block",
                    min: 2.0,
                    max: 8.0,
                    default: 4.0,
                },
                ParamDef {
                    label: "green",
                    min: 0.0,
                    max: 1.0,
                    default: 0.7,
                },
                ParamDef {
                    label: "crush",
                    min: 0.0,
                    max: 1.0,
                    default: 0.75,
                },
            ],
            Look::Ripple => &[
                WET,
                ParamDef {
                    label: "swell",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "rings",
                    min: 0.0,
                    max: 1.0,
                    default: 0.45,
                },
                ParamDef {
                    label: "tape",
                    min: 0.0,
                    max: 1.0,
                    default: 0.4,
                },
            ],
            Look::Uhf => &[
                WET,
                ParamDef {
                    label: "snow",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "roll",
                    min: 0.0,
                    max: 1.0,
                    default: 0.45,
                },
                ParamDef {
                    label: "tint",
                    min: 0.0,
                    max: 1.0,
                    default: 0.35,
                },
            ],
            Look::Beta => &[
                WET,
                ParamDef {
                    label: "drop",
                    min: 0.0,
                    max: 1.0,
                    default: 0.4,
                },
                ParamDef {
                    label: "luma",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "edge",
                    min: 0.0,
                    max: 1.0,
                    default: 0.35,
                },
            ],
            Look::D8 => &[
                WET,
                ParamDef {
                    label: "block",
                    min: 0.0,
                    max: 1.0,
                    default: 0.65,
                },
                ParamDef {
                    label: "drop",
                    min: 0.0,
                    max: 1.0,
                    default: 0.35,
                },
                ParamDef {
                    label: "date",
                    min: 0.0,
                    max: 1.0,
                    default: 0.85,
                },
            ],
            Look::Live => &[
                WET,
                ParamDef {
                    label: "tally",
                    min: 0.0,
                    max: 1.0,
                    default: 0.75,
                },
                ParamDef {
                    label: "comb",
                    min: 0.0,
                    max: 1.0,
                    default: 0.65,
                },
                ParamDef {
                    label: "crush",
                    min: 0.0,
                    max: 1.0,
                    default: 0.6,
                },
            ],
            Look::Sat => &[
                WET,
                ParamDef {
                    label: "rain",
                    min: 0.0,
                    max: 1.0,
                    default: 0.5,
                },
                ParamDef {
                    label: "block",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "sat",
                    min: 0.0,
                    max: 1.0,
                    default: 0.45,
                },
            ],
            Look::Smear => &[
                WET,
                ParamDef {
                    label: "warm",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "lag",
                    min: 0.0,
                    max: 1.0,
                    default: 0.82,
                },
                ParamDef {
                    label: "spread",
                    min: 0.0,
                    max: 1.0,
                    default: 0.72,
                },
            ],
            Look::Breathe => &[
                WET,
                ParamDef {
                    label: "depth",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "pace",
                    min: 0.0,
                    max: 1.0,
                    default: 0.45,
                },
                ParamDef {
                    label: "hold",
                    min: 0.0,
                    max: 1.0,
                    default: 0.35,
                },
            ],
            Look::Film => &[
                WET,
                ParamDef {
                    label: "grain",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "warm",
                    min: 0.0,
                    max: 1.0,
                    default: 0.62,
                },
                ParamDef {
                    label: "frame",
                    min: 0.0,
                    max: 1.0,
                    default: 0.72,
                },
            ],
            Look::Waves => &[
                WET,
                ParamDef {
                    label: "sepia",
                    min: 0.0,
                    max: 1.0,
                    default: 0.78,
                },
                ParamDef {
                    label: "wave",
                    min: 0.0,
                    max: 1.0,
                    default: 0.55,
                },
                ParamDef {
                    label: "grain",
                    min: 0.0,
                    max: 1.0,
                    default: 0.48,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pct_roundtrip_wet() {
        let def = Look::Morph.param_defs()[0];
        assert_eq!(def.label, "wet");
        assert!((def.from_pct(0.0) - 0.0).abs() < 0.0001);
        assert!((def.from_pct(100.0) - 1.0).abs() < 0.0001);
        assert!((def.from_pct(50.0) - 0.5).abs() < 0.0001);
        assert!((def.to_pct(0.5) - 50.0).abs() < 0.01);
    }

    #[test]
    fn apply_pct_writes_slot() {
        let def = Look::Morph.param_defs()[1];
        let mut p = LookParams::defaults(Look::Morph);
        assert!(p.apply_pct(1, def, 100.0));
        assert!((p.v(1) - def.max).abs() < 0.0001);
        assert!(!p.apply_pct(1, def, 100.0), "same value is a no-op");
    }

    #[test]
    fn none_has_no_sliders() {
        assert!(Look::None.param_defs().is_empty());
        assert_eq!(Look::from_id(0), Look::None);
        assert_eq!(Look::None.id(), 0);
    }

    #[test]
    fn tile_line_names_the_effect() {
        assert_eq!(Look::Vhs.tile_line(), "tracking · wear");
        assert_eq!(Look::Morph.tile_line(), "ink drawing");
        assert_eq!(Look::None.tile_line(), "clean camera");
    }
}
