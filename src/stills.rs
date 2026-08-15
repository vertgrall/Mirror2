//! Photoreal stills for each look. The object, not a sign.

use std::sync::{Arc, OnceLock};

use freya::components::CanvasContext;
use freya::engine::prelude::{
    AlphaType, ColorType, Data, FilterMode, ImageInfo, MipmapMode, Paint, SamplingOptions,
};
use skia_safe::images;
use skia_safe::Rect as SkRect;

use crate::effects::Look;

pub struct Still {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}

fn decode(bytes: &[u8]) -> Still {
    let img = image::load_from_memory(bytes)
        .expect("still jpeg")
        .to_rgba8();
    let (width, height) = img.dimensions();
    Still {
        width,
        height,
        rgba: img.into_raw().into(),
    }
}

macro_rules! still_for {
    ($look:expr, $path:literal) => {{
        static SLOT: OnceLock<Still> = OnceLock::new();
        SLOT.get_or_init(|| decode(include_bytes!($path)))
    }};
}

fn base_off_rgb() -> (u32, u32, &'static [u8]) {
    static BASE: OnceLock<(u32, u32, Vec<u8>)> = OnceLock::new();
    let (w, h, rgb) = BASE.get_or_init(|| {
        let still = decode(include_bytes!("../assets/fx/off.jpg"));
        let mut rgb = Vec::with_capacity((still.width * still.height * 3) as usize);
        for chunk in still.rgba.chunks_exact(4) {
            rgb.push(chunk[0]);
            rgb.push(chunk[1]);
            rgb.push(chunk[2]);
        }
        (still.width, still.height, rgb)
    });
    (*w, *h, rgb.as_slice())
}

macro_rules! render_still_for {
    ($look:expr) => {{
        static SLOT: OnceLock<Still> = OnceLock::new();
        SLOT.get_or_init(|| {
            let (w, h, rgb) = base_off_rgb();
            let rgba = mirror_vfx::render_still_rgba($look, rgb, w, h);
            Still {
                width: w,
                height: h,
                rgba: rgba.into(),
            }
        })
    }};
}

pub fn for_look(look: Look) -> &'static Still {
    match look {
        Look::None => still_for!(Look::None, "../assets/fx/off.jpg"),
        Look::Morph => still_for!(Look::Morph, "../assets/fx/morph.jpg"),
        Look::Vhs => still_for!(Look::Vhs, "../assets/fx/vhs.jpg"),
        Look::Gx => still_for!(Look::Gx, "../assets/fx/gx.jpg"),
        Look::Uhf => still_for!(Look::Uhf, "../assets/fx/uhf.jpg"),
        Look::Beta => still_for!(Look::Beta, "../assets/fx/beta.jpg"),
        Look::D8 => still_for!(Look::D8, "../assets/fx/d8.jpg"),
        Look::Live => still_for!(Look::Live, "../assets/fx/live.jpg"),
        Look::Sat => still_for!(Look::Sat, "../assets/fx/sat.jpg"),
        Look::Cctv => still_for!(Look::Cctv, "../assets/fx/cctv.jpg"),
        Look::Ripple => still_for!(Look::Ripple, "../assets/fx/ripple.jpg"),
        Look::Smear => still_for!(Look::Smear, "../assets/fx/smear.jpg"),
        Look::Breathe => still_for!(Look::Breathe, "../assets/fx/breathe.jpg"),
        Look::Film => still_for!(Look::Film, "../assets/fx/film.jpg"),
        Look::Waves => still_for!(Look::Waves, "../assets/fx/waves.jpg"),

        Look::Thermal => render_still_for!(Look::Thermal),
        Look::Xray => render_still_for!(Look::Xray),
        Look::Cyber => render_still_for!(Look::Cyber),
        Look::Noir => render_still_for!(Look::Noir),
        Look::Glitch => render_still_for!(Look::Glitch),
        Look::Mosh => render_still_for!(Look::Mosh),
        Look::Holo => render_still_for!(Look::Holo),
        Look::Particles => render_still_for!(Look::Particles),
        Look::Stamp => render_still_for!(Look::Stamp),
        Look::Drift => render_still_for!(Look::Drift),
        Look::Echo => render_still_for!(Look::Echo),
        Look::Chrome => render_still_for!(Look::Chrome),
        Look::Bounce => render_still_for!(Look::Bounce),
        Look::Prism => render_still_for!(Look::Prism),
        Look::Slitscan => render_still_for!(Look::Slitscan),
        Look::Reaction => render_still_for!(Look::Reaction),
        Look::Fluid => render_still_for!(Look::Fluid),
        Look::Strata => render_still_for!(Look::Strata),
        Look::Datamosh => render_still_for!(Look::Datamosh),
        Look::Voronoi => render_still_for!(Look::Voronoi),
        Look::Topo => render_still_for!(Look::Topo),
        Look::Quantum => render_still_for!(Look::Quantum),
    }
}

/// Cover-crop the still into the canvas. Like a print filling a frame.
pub fn draw_still(ctx: &mut CanvasContext, look: Look) {
    let still = for_look(look);
    let dw = ctx.size.width.max(1.0);
    let dh = ctx.size.height.max(1.0);
    let info = ImageInfo::new(
        (still.width as i32, still.height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let Some(image) = images::raster_from_data(
        &info,
        Data::new_copy(still.rgba.as_ref()),
        still.width as usize * 4,
    ) else {
        return;
    };

    let sw = still.width as f32;
    let sh = still.height as f32;
    let scale = (dw / sw).max(dh / sh);
    let rw = sw * scale;
    let rh = sh * scale;
    let x = (dw - rw) * 0.5;
    let y = (dh - rh) * 0.5;

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::None);
    ctx.canvas.draw_image_rect_with_sampling_options(
        image,
        None,
        SkRect::from_xywh(x, y, rw, rh),
        sampling,
        &paint,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_look_has_a_still() {
        for look in Look::RAIL {
            let still = for_look(look);
            assert!(
                still.width >= 64 && still.height >= 64,
                "{look:?} still is {}×{}",
                still.width,
                still.height
            );
            assert_eq!(
                still.rgba.len(),
                still.width as usize * still.height as usize * 4
            );
        }
    }

    #[test]
    fn every_card_has_a_unique_graphic() {
        use std::collections::hash_map::DefaultHasher;
        use std::collections::HashSet;
        use std::hash::{Hash, Hasher};

        let mut signatures = HashSet::new();
        for look in Look::RAIL {
            let still = for_look(look);
            let mut hasher = DefaultHasher::new();
            still.rgba.hash(&mut hasher);
            let sig = hasher.finish();
            assert!(
                signatures.insert((sig, still.rgba.len())),
                "DUPLICATE CARD GRAPHIC DETECTED for look: {:?}",
                look
            );
        }
        assert_eq!(signatures.len(), Look::RAIL.len(), "All cards must have unique graphics");
    }
}
