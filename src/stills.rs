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
        Look::Thermal => still_for!(Look::Thermal, "../assets/fx/cctv.jpg"),
        Look::Xray => still_for!(Look::Xray, "../assets/fx/morph.jpg"),
        Look::Cyber => still_for!(Look::Cyber, "../assets/fx/waves.jpg"),
        Look::Noir => still_for!(Look::Noir, "../assets/fx/off.jpg"),
        Look::Glitch => still_for!(Look::Glitch, "../assets/fx/vhs.jpg"),
        Look::Mosh => still_for!(Look::Mosh, "../assets/fx/d8.jpg"),
        Look::Holo => still_for!(Look::Holo, "../assets/fx/live.jpg"),
        Look::Particles => still_for!(Look::Particles, "../assets/fx/smear.jpg"),
        Look::Stamp => still_for!(Look::Stamp, "../assets/fx/d8.jpg"),
        Look::Drift => still_for!(Look::Drift, "../assets/fx/ripple.jpg"),
        Look::Echo => still_for!(Look::Echo, "../assets/fx/breathe.jpg"),
        Look::Chrome => still_for!(Look::Chrome, "../assets/fx/sat.jpg"),
        Look::Bounce => still_for!(Look::Bounce, "../assets/fx/d8.jpg"),
        Look::Prism => still_for!(Look::Prism, "../assets/fx/waves.jpg"),
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
}
