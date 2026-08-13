//! Flat color. No paper, no tape, no stamp.

use freya::prelude::Color;

/// Viewfinder — 4:3, centered above controls.
pub const VIEWFINDER_W: f32 = 480.;
pub const VIEWFINDER_H: f32 = 360.;

/// Effect slider track width — must be px, not fill, or Freya's slider gets 0 width.
pub const SLIDER_W: f32 = 220.;

/// Live VFX + preview run at this width (4:3). Full sensor res is capture-only.
pub const PREVIEW_MAX_W: u32 = 640;

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub control: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub shutter: Color,
    pub shutter_pressed: Color,
}

impl Palette {
    pub fn app() -> Self {
        Self {
            bg: Color::from_rgb(244, 244, 242),
            surface: Color::from_rgb(28, 28, 26),
            control: Color::from_rgb(232, 232, 228),
            text: Color::from_rgb(22, 22, 20),
            muted: Color::from_rgb(110, 110, 106),
            accent: Color::from_rgb(22, 22, 20),
            shutter: Color::from_rgb(220, 38, 38),
            shutter_pressed: Color::from_rgb(168, 28, 28),
        }
    }
}
