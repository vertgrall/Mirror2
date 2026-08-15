//! VFX pipeline — composite → look → atmosphere.

mod atmo;
mod beta;
mod bounce;
mod breathe;
mod cctv;
mod chrome;
mod cyber;
mod d8;
mod datamosh;
mod drift;
mod echo;
mod film;
mod fluid;
mod glitch;
mod gx;
mod holo;
mod live;
mod morph;
mod mosh;
mod noir;
mod ops;
mod osd;
mod params;
mod particles;
mod prism;
mod quantum;
mod reaction;
mod ripple;
mod sat;
mod slitscan;
mod smear;
mod stamp;
mod state;
mod strata;
mod thermal;
mod topo;
mod uhf;
mod vhs;
mod voronoi;
mod waves;
mod xray;


pub use atmo::{
    param_defs as atmo_param_defs, set_params as set_atmosphere, AtmosphereParams,
};
pub use ops::rgb_to_rgba;
pub use params::{current_params, set_params, LookParams, ParamDef};
pub use state::VfxState;

use std::sync::{Mutex, OnceLock};

static STATE: OnceLock<Mutex<VfxState>> = OnceLock::new();
pub static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn vfx_state() -> &'static Mutex<VfxState> {
    STATE.get_or_init(|| Mutex::new(VfxState::default()))
}

/// Drop frame history when the user picks a different look.
pub fn reset_temporal() {
    if let Ok(mut state) = vfx_state().lock() {
        state.clear_temporal();
    }
}

/// High-level VFX processing engine instance for standalone library consumers.
#[derive(Default)]
pub struct VfxEngine {
    pub state: VfxState,
}

impl VfxEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.state.clear_temporal();
    }

    pub fn apply(
        &mut self,
        look: Look,
        rgb: &[u8],
        w: u32,
        h: u32,
        look_params: &LookParams,
        atmo_params: &AtmosphereParams,
    ) -> Vec<u8> {
        let n = (w as usize) * (h as usize);
        assert_eq!(rgb.len(), n * 3);

        self.state.advance(w, h, look.id());

        let wet = look_params.wet();
        let rgba = if look.is_none() || (wet < 0.01 && !matches!(look, Look::Morph)) {
            ops::rgb_to_rgba(rgb, w, h)
        } else {
            let mut looked = apply_look(look, rgb, w, h, &self.state, look_params);
            if wet < 0.99 && !matches!(look, Look::Morph) {
                ops::mix_look_over_rgb(&mut looked, rgb, wet);
            }
            if matches!(look, Look::Morph) {
                self.state.commit_morph(&looked);
            }
            looked
        };
        self.state.commit_rgb(rgb);
        atmo::apply(&rgba, w, h, &self.state, atmo_params)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    None,
    Morph,
    Vhs,
    Gx,
    Uhf,
    Beta,
    D8,
    Live,
    Sat,
    Cctv,
    Ripple,
    Smear,
    Breathe,
    Film,
    Waves,
    Thermal,
    Xray,
    Cyber,
    Noir,
    Glitch,
    Mosh,
    Holo,
    Particles,
    Stamp,
    Drift,
    Echo,
    Chrome,
    Bounce,
    Prism,
    Slitscan,
    Reaction,
    Fluid,
    Strata,
    Datamosh,
    Voronoi,
    Topo,
    Quantum,
}

impl Look {
    pub const RAIL: [Self; 37] = [
        Self::None,
        Self::Morph,
        Self::Vhs,
        Self::Gx,
        Self::Uhf,
        Self::Beta,
        Self::D8,
        Self::Live,
        Self::Sat,
        Self::Cctv,
        Self::Ripple,
        Self::Smear,
        Self::Breathe,
        Self::Film,
        Self::Waves,
        Self::Thermal,
        Self::Xray,
        Self::Cyber,
        Self::Noir,
        Self::Glitch,
        Self::Mosh,
        Self::Holo,
        Self::Particles,
        Self::Stamp,
        Self::Drift,
        Self::Echo,
        Self::Chrome,
        Self::Bounce,
        Self::Prism,
        Self::Slitscan,
        Self::Reaction,
        Self::Fluid,
        Self::Strata,
        Self::Datamosh,
        Self::Voronoi,
        Self::Topo,
        Self::Quantum,
    ];

    pub fn id(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Morph => 1,
            Self::Vhs => 2,
            Self::Gx => 3,
            Self::Uhf => 4,
            Self::Beta => 5,
            Self::D8 => 6,
            Self::Live => 7,
            Self::Sat => 8,
            Self::Cctv => 9,
            Self::Ripple => 10,
            Self::Smear => 11,
            Self::Breathe => 12,
            Self::Film => 13,
            Self::Waves => 14,
            Self::Thermal => 15,
            Self::Xray => 16,
            Self::Cyber => 17,
            Self::Noir => 18,
            Self::Glitch => 19,
            Self::Mosh => 20,
            Self::Holo => 21,
            Self::Particles => 22,
            Self::Stamp => 23,
            Self::Drift => 24,
            Self::Echo => 25,
            Self::Chrome => 26,
            Self::Bounce => 27,
            Self::Prism => 28,
            Self::Slitscan => 29,
            Self::Reaction => 30,
            Self::Fluid => 31,
            Self::Strata => 32,
            Self::Datamosh => 33,
            Self::Voronoi => 34,
            Self::Topo => 35,
            Self::Quantum => 36,
        }
    }

    pub fn from_id(id: u8) -> Self {
        match id {
            1 => Self::Morph,
            2 => Self::Vhs,
            3 => Self::Gx,
            4 => Self::Uhf,
            5 => Self::Beta,
            6 => Self::D8,
            7 => Self::Live,
            8 => Self::Sat,
            9 => Self::Cctv,
            10 => Self::Ripple,
            11 => Self::Smear,
            12 => Self::Breathe,
            13 => Self::Film,
            14 => Self::Waves,
            15 => Self::Thermal,
            16 => Self::Xray,
            17 => Self::Cyber,
            18 => Self::Noir,
            19 => Self::Glitch,
            20 => Self::Mosh,
            21 => Self::Holo,
            22 => Self::Particles,
            23 => Self::Stamp,
            24 => Self::Drift,
            25 => Self::Echo,
            26 => Self::Chrome,
            27 => Self::Bounce,
            28 => Self::Prism,
            29 => Self::Slitscan,
            30 => Self::Reaction,
            31 => Self::Fluid,
            32 => Self::Strata,
            33 => Self::Datamosh,
            34 => Self::Voronoi,
            35 => Self::Topo,
            36 => Self::Quantum,
            _ => Self::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "OFF",
            Self::Morph => "MORPH",
            Self::Vhs => "VHS",
            Self::Gx => "GX",
            Self::Uhf => "UHF",
            Self::Beta => "BETA",
            Self::D8 => "D8",
            Self::Live => "LIVE",
            Self::Sat => "SAT",
            Self::Cctv => "CCTV",
            Self::Ripple => "RIPPLE",
            Self::Smear => "SMEAR",
            Self::Breathe => "BREATHE",
            Self::Film => "FILM",
            Self::Waves => "WAVES",
            Self::Thermal => "THERMAL",
            Self::Xray => "XRAY",
            Self::Cyber => "CYBER",
            Self::Noir => "NOIR",
            Self::Glitch => "GLITCH",
            Self::Mosh => "MOSH",
            Self::Holo => "HOLO",
            Self::Particles => "PARTICLES",
            Self::Stamp => "STAMP",
            Self::Drift => "DRIFT",
            Self::Echo => "ECHO",
            Self::Chrome => "CHROME",
            Self::Bounce => "BOUNCE",
            Self::Prism => "PRISM",
            Self::Slitscan => "SLITSCAN",
            Self::Reaction => "REACTION",
            Self::Fluid => "FLUID",
            Self::Strata => "STRATA",
            Self::Datamosh => "DATAMOSH",
            Self::Voronoi => "VORONOI",
            Self::Topo => "TOPO",
            Self::Quantum => "QUANTUM",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::None => "clean camera · pick a look",
            Self::Morph => "dark ink lines · color stays · lines + trail",
            Self::Vhs => "tracking · chroma bleed · tape wear",
            Self::Gx => "Hi8 warmth · comb · MAR 14 1994",
            Self::Uhf => "snow · vertical roll · dying UHF",
            Self::Beta => "luma dropout · sharper than VHS",
            Self::D8 => "8×8 blocks · MiniDV date burn-in",
            Self::Live => "tally · interlace · broadcast crush",
            Self::Sat => "rain fade · macro · 16:9 letterbox",
            Self::Cctv => "blocky green-grey · crushed",
            Self::Ripple => "water rings · VHS cam through a puddle",
            Self::Smear => "color drag · warm or cold trail · slow shutter",
            Self::Breathe => "inhale · exhale · whole frame scales",
            Self::Film => "35mm inset · sprockets · grain",
            Self::Waves => "sepia print · gate ripples · silver grain",
            Self::Thermal => "FLIR heat vision · ironbow & rainbow gradients",
            Self::Xray => "fluoroscopy radiograph · inverted contrast glow",
            Self::Cyber => "Trinitron CRT · shadow mask & phosphor bleed",
            Self::Noir => "40s Tri-X monochrome · crushed blacks & silver grain",
            Self::Glitch => "analog sync tear · horizontal line displacement",
            Self::Mosh => "datamosh compression · motion vector macroblocks",
            Self::Holo => "cyberpunk laser grid · holographic scanlines",
            Self::Particles => "pixel dust dispersal · floating particle swirl",
            Self::Stamp => "3-second snapshot PIPs smeared into viewport corners",
            Self::Drift => "slow undulating random LFO waveform distortion",
            Self::Echo => "temporal feedback infinity mirror tunnel loop",
            Self::Chrome => "super fast random coordinate metallic glinting",
            Self::Bounce => "bouncing frame fragments cut from live feed",
            Self::Prism => "triadic RGB spectral refraction & glass flare",
            Self::Slitscan => "space-time slice depth remapping history ring",
            Self::Reaction => "Turing Gray-Scott reaction-diffusion morphogenesis",
            Self::Fluid => "Eulerian optical flow vector grid & liquid dye solver",
            Self::Strata => "recursive homography feedback infinity mirror tunnel",
            Self::Datamosh => "macroblock motion vector smearing & P-frame bleed",
            Self::Voronoi => "dynamic Delaunay / Voronoi mosaic glass shatter",
            Self::Topo => "2.5D scanline heightmap topographic landscape",
            Self::Quantum => "2D spatial phase wave holographic light interference",
        }
    }

    /// One line on a sheet tile — what the look does, not a brand name.
    pub fn tile_line(self) -> &'static str {
        match self {
            Self::None => "clean camera",
            Self::Morph => "dark lines · color",
            Self::Vhs => "tracking · wear",
            Self::Gx => "Hi8 · 1994",
            Self::Uhf => "antenna · snow",
            Self::Beta => "luma · dropout",
            Self::D8 => "block · digital",
            Self::Live => "tally · interlace",
            Self::Sat => "rain · macro",
            Self::Cctv => "blocky · crushed",
            Self::Ripple => "water rings",
            Self::Smear => "drag · temperature",
            Self::Breathe => "inhale · exhale",
            Self::Film => "sprockets · rebate",
            Self::Waves => "sepia · film",
            Self::Thermal => "heat vision · FLIR",
            Self::Xray => "radiograph · bone",
            Self::Cyber => "Trinitron · CRT",
            Self::Noir => "crushed · silver",
            Self::Glitch => "sync tear · shear",
            Self::Mosh => "datamosh · vector",
            Self::Holo => "laser · hologram",
            Self::Particles => "pixel dust · swirl",
            Self::Stamp => "snapshot · corners",
            Self::Drift => "slow lfo · wave",
            Self::Echo => "feedback · tunnel",
            Self::Chrome => "rapid · chrome",
            Self::Bounce => "bouncing · clones",
            Self::Prism => "refract · prism",
            Self::Slitscan => "time slice · depth",
            Self::Reaction => "Turing · morphogenesis",
            Self::Fluid => "optical flow · liquid",
            Self::Strata => "homography · feedback",
            Self::Datamosh => "macroblock · mosh",
            Self::Voronoi => "Delaunay · mosaic",
            Self::Topo => "heightmap · contour",
            Self::Quantum => "holographic · phase",
        }
    }

    pub fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

/// Render a static preview frame RGBA for thumbnail generation using default parameters for `look`.
pub fn render_still_rgba(look: Look, rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let n = (w as usize) * (h as usize);
    assert_eq!(rgb.len(), n * 3);
    let mut state = VfxState::default();
    let params = LookParams::defaults(look);

    if look.is_none() {
        return ops::rgb_to_rgba(rgb, w, h);
    }

    // Warm up state history across multiple iterations
    for _ in 0..10 {
        state.advance(w, h, look.id());
        state.push_ring(rgb, 45);
        state.commit_rgb(rgb);
    }

    apply_look(look, rgb, w, h, &state, &params)
}

/// RGB (mirrored) → RGBA through full pipeline.
pub fn apply(look: Look, rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let n = (w as usize) * (h as usize);
    assert_eq!(rgb.len(), n * 3);

    let look_params = current_params();
    let atmo_params = atmo::current_params();

    let Ok(mut state) = vfx_state().lock() else {
        return ops::rgb_to_rgba(rgb, w, h);
    };
    state.advance(w, h, look.id());
    state.push_ring(rgb, 45);

    let wet = look_params.wet();
    let rgba = if look.is_none() || (wet < 0.01 && !matches!(look, Look::Morph)) {
        ops::rgb_to_rgba(rgb, w, h)
    } else {
        let mut looked = apply_look(look, rgb, w, h, &state, &look_params);
        // Morph keeps the photo visible — lines slider lives inside the look.
        if wet < 0.99 && !matches!(look, Look::Morph) {
            ops::mix_look_over_rgb(&mut looked, rgb, wet);
        }
        if matches!(look, Look::Morph) {
            state.commit_morph(&looked);
        }
        looked
    };
    state.commit_rgb(rgb);
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
        Look::Morph => morph::apply(rgb, w, h, state, params),
        Look::Vhs => vhs::apply(rgb, w, h, state, params),
        Look::Gx => gx::apply(rgb, w, h, state, params),
        Look::Uhf => uhf::apply(rgb, w, h, state, params),
        Look::Beta => beta::apply(rgb, w, h, state, params),
        Look::D8 => d8::apply(rgb, w, h, state, params),
        Look::Live => live::apply(rgb, w, h, state, params),
        Look::Sat => sat::apply(rgb, w, h, state, params),
        Look::Cctv => cctv::apply(rgb, w, h, state, params),
        Look::Ripple => ripple::apply(rgb, w, h, state, params),
        Look::Smear => smear::apply(rgb, w, h, state, params),
        Look::Breathe => breathe::apply(rgb, w, h, state, params),
        Look::Film => film::apply(rgb, w, h, state, params),
        Look::Waves => waves::apply(rgb, w, h, state, params),
        Look::Thermal => thermal::apply(rgb, w, h, state, params),
        Look::Xray => xray::apply(rgb, w, h, state, params),
        Look::Cyber => cyber::apply(rgb, w, h, state, params),
        Look::Noir => noir::apply(rgb, w, h, state, params),
        Look::Glitch => glitch::apply(rgb, w, h, state, params),
        Look::Mosh => mosh::apply(rgb, w, h, state, params),
        Look::Holo => holo::apply(rgb, w, h, state, params),
        Look::Particles => particles::apply(rgb, w, h, state, params),
        Look::Stamp => stamp::apply(rgb, w, h, state, params),
        Look::Drift => drift::apply(rgb, w, h, state, params),
        Look::Echo => echo::apply(rgb, w, h, state, params),
        Look::Chrome => chrome::apply(rgb, w, h, state, params),
        Look::Bounce => bounce::apply(rgb, w, h, state, params),
        Look::Prism => prism::apply(rgb, w, h, state, params),
        Look::Slitscan => slitscan::apply(rgb, w, h, state, params),
        Look::Reaction => reaction::apply(rgb, w, h, state, params),
        Look::Fluid => fluid::apply(rgb, w, h, state, params),
        Look::Strata => strata::apply(rgb, w, h, state, params),
        Look::Datamosh => datamosh::apply(rgb, w, h, state, params),
        Look::Voronoi => voronoi::apply(rgb, w, h, state, params),
        Look::Topo => topo::apply(rgb, w, h, state, params),
        Look::Quantum => quantum::apply(rgb, w, h, state, params),
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
pub mod tests {
    use super::*;
    use std::sync::Mutex;
    use atmo::set_params as set_atmo;

    pub static TEST_MUTEX: Mutex<()> = Mutex::new(());

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
        let _lock = TEST_MUTEX.lock().unwrap();
        set_atmo(AtmosphereParams::default());
        let rgb = sample_rgb(32, 24);
        for look in Look::RAIL {
            set_params(LookParams::defaults(look));
            let out = apply(look, &rgb, 32, 24);
            assert_eq!(out.len(), 32 * 24 * 4, "{look:?}");
            assert!(out.chunks(4).all(|p| p[3] == 255), "{look:?} alpha");
        }
    }

    #[test]
    fn morph_produces_edges() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_temporal();
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
    fn morph_trail_uses_previous_frame() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_temporal();
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        set_params(LookParams::defaults(Look::Morph));
        let mut a = vec![255u8; 32 * 24 * 3];
        let mut b = vec![255u8; 32 * 24 * 3];
        for y in 6..18 {
            for x in 4..20 {
                let i = (y * 32 + x) * 3;
                a[i] = 30;
                a[i + 1] = 30;
                a[i + 2] = 30;
            }
        }
        for y in 6..18 {
            for x in 10..26 {
                let i = (y * 32 + x) * 3;
                b[i] = 30;
                b[i + 1] = 30;
                b[i + 2] = 30;
            }
        }
        let first = apply(Look::Morph, &a, 32, 24);
        let second = apply(Look::Morph, &b, 32, 24);
        assert_ne!(first, second, "motion should stretch morph ink across frames");
    }

    #[test]
    fn vhs_chroma_survives_edge_jitter() {
        let _lock = TEST_MUTEX.lock().unwrap();
        set_params(LookParams::defaults(Look::Vhs));
        set_atmo(AtmosphereParams { smoke: 0.0, ..Default::default() });
        // Realistic preview size + default tracking/chroma — previously OOB on jittered sx.
        let rgb = vec![128u8; 640 * 480 * 3];
        let out = apply(Look::Vhs, &rgb, 640, 480);
        assert_eq!(out.len(), 640 * 480 * 4);
    }

    #[test]
    fn morph_lines_zero_is_clean_photo() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_temporal();
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
        let _lock = TEST_MUTEX.lock().unwrap();
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
    fn sat_applies_fullscreen() {
        let _lock = TEST_MUTEX.lock().unwrap();
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        set_params(LookParams::defaults(Look::Sat));
        let rgb = vec![200u8; 640 * 480 * 3];
        let out = apply(Look::Sat, &rgb, 640, 480);
        assert_eq!(out.len(), 640 * 480 * 4);
        assert!(out[0] > 100, "top edge should be styled, got {}", out[0]);
    }

    #[test]
    fn none_look_is_clean_passthrough() {
        let _lock = TEST_MUTEX.lock().unwrap();
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
        assert!(AtmosphereParams::default().smoke < 0.01);
        assert!(Look::None.param_defs().is_empty());
        assert_eq!(Look::RAIL.len(), 37);
    }

    #[test]
    fn film_applies_fullscreen() {
        let _lock = TEST_MUTEX.lock().unwrap();
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        set_params(LookParams::defaults(Look::Film));
        let rgb = vec![200u8; 640 * 480 * 3];
        let out = apply(Look::Film, &rgb, 640, 480);
        assert!(out[0] > 100, "outer shell should be styled, got {}", out[0]);
    }

    #[test]
    fn waves_applies_sepia_and_warp() {
        let _lock = TEST_MUTEX.lock().unwrap();
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        let mut p = LookParams::defaults(Look::Waves);
        p.values[1] = 1.0;
        p.values[2] = 0.8;
        p.values[3] = 0.5;
        set_params(p);
        let mut rgb = vec![80u8; 64 * 48 * 3];
        for i in 0..64 * 48 {
            rgb[i * 3] = 40;
            rgb[i * 3 + 1] = 160;
            rgb[i * 3 + 2] = 200;
        }
        let dry = ops::rgb_to_rgba(&rgb, 64, 48);
        let out = apply(Look::Waves, &rgb, 64, 48);
        assert_ne!(out, dry);
        let mid = out[(24 * 64 + 32) * 4];
        assert!(mid > dry[(24 * 64 + 32) * 4], "sepia should lift red channel");
    }

    #[test]
    fn smear_uses_previous_frame() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_temporal();
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        set_params(LookParams::defaults(Look::Smear));
        let mut a = vec![20u8; 32 * 24 * 3];
        let mut b = vec![220u8; 32 * 24 * 3];
        for i in 0..32 * 24 {
            a[i * 3 + 2] = 200;
            b[i * 3] = 240;
        }
        let first = apply(Look::Smear, &a, 32, 24);
        let second = apply(Look::Smear, &b, 32, 24);
        assert_ne!(
            second,
            ops::rgb_to_rgba(&b, 32, 24),
            "second smear frame must differ from dry camera"
        );
        assert_ne!(first, second);
    }

    #[test]
    fn smear_warm_and_cold_differ() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_temporal();
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        let mut rgb_a = vec![40u8; 32 * 24 * 3];
        let mut rgb_b = vec![200u8; 32 * 24 * 3];
        for i in 0..32 * 24 {
            rgb_a[i * 3 + 1] = 180;
            rgb_b[i * 3 + 2] = 220;
        }

        let mut warm = LookParams::defaults(Look::Smear);
        warm.values[1] = 1.0;
        warm.values[2] = 0.85;
        warm.values[3] = 0.6;
        set_params(warm);
        let warm_out = apply(Look::Smear, &rgb_a, 32, 24);
        let warm_out_b = apply(Look::Smear, &rgb_b, 32, 24);

        let mut cold = LookParams::defaults(Look::Smear);
        cold.values[1] = 0.0;
        cold.values[2] = 0.85;
        cold.values[3] = 0.6;
        set_params(cold);
        let cold_out = apply(Look::Smear, &rgb_a, 32, 24);

        assert_ne!(warm_out, cold_out, "warm vs cold tint must diverge");
        assert_ne!(warm_out, warm_out_b, "second frame should smear differently");
    }

    #[test]
    fn smear_stays_in_bounds() {
        let _lock = TEST_MUTEX.lock().unwrap();
        set_params(LookParams::defaults(Look::Smear));
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        let rgb = vec![90u8; 640 * 480 * 3];
        let out = apply(Look::Smear, &rgb, 640, 480);
        assert_eq!(out.len(), 640 * 480 * 4);
        assert!(out.chunks(4).all(|px| px[3] == 255));
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
    fn morph_lines_slider_changes_output() {
        let _lock = TEST_MUTEX.lock().unwrap();
        reset_temporal();
        set_atmo(AtmosphereParams {
            smoke: 0.0,
            ..Default::default()
        });
        let rgb = sample_rgb(32, 24);
        let def = Look::Morph.param_defs()[0];
        let mut faint = LookParams::defaults(Look::Morph);
        assert!(faint.apply_pct(0, def, 0.0));
        set_params(faint);
        let a = apply(Look::Morph, &rgb, 32, 24);

        let mut bold = LookParams::defaults(Look::Morph);
        assert!(bold.apply_pct(0, def, 100.0));
        set_params(bold);
        let b = apply(Look::Morph, &rgb, 32, 24);

        assert_ne!(a, b, "lines 0 vs 100 must change the image");
        assert!((faint.v(0) - 0.0).abs() < 0.001);
        assert!((bold.v(0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn gx_stamp_is_frozen_1994() {
        assert_eq!(osd::GX_DATE, "MAR 14 1994");
    }

    #[test]
    fn d8_stamp_is_frozen_2000() {
        assert_eq!(osd::D8_DATE, "JAN 01 2000");
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
