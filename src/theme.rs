//! Cursor-dark tokens from the FX canvas mock. Flat. No paper.

use freya::prelude::{Border, BorderWidth, Color};

/// Camera well — 4:3. The window is this wide. Nothing is wider.
pub const VIEWFINDER_W: f32 = 480.;
pub const VIEWFINDER_H: f32 = 360.;
pub const WINDOW_W: f32 = VIEWFINDER_W;
/// header 20 + well 360 + shutter 56 + fx band 164 + dock 88 + 4×8 gaps + 8 top
pub const WINDOW_H: f32 = 728.;
pub const FX_BAND_H: f32 = 164.;

/// Vertical rhythm between the stacked blocks.
pub const GAP: f32 = 8.;

/// Effect slider track width — must be px, not fill, or Freya's slider gets 0 width.
/// 480 − 8 − 52 (label) − 8 − 32 (value) − 8 = 372, leave a hair.
pub const SLIDER_W: f32 = 360.;

/// Live VFX + preview run at this width (4:3). Full sensor res is capture-only.
pub const PREVIEW_MAX_W: u32 = 640;

/// Canvas `Text` size="small" — 12 / 16.
pub const FONT_SMALL: f32 = 12.;

/// Cards in the dock window. Radius 0 — square tiles.
pub const DOCK_VISIBLE: usize = 3;
pub const DOCK_H: f32 = 88.;
pub const CARD_H: f32 = 72.;
pub const CARD_PAD: f32 = 8.;
pub const CARD_RADIUS: f32 = 0.;
pub const CHEVRON_W: f32 = 32.;

fn edge(color: Color, top: f32, right: f32, bottom: f32, left: f32) -> Border {
    Border::new().fill(color).width(BorderWidth {
        top,
        right,
        bottom,
        left,
    })
}

pub fn border_all(color: Color) -> Border {
    edge(color, 1., 1., 1., 1.)
}

pub fn border_top(color: Color) -> Border {
    edge(color, 1., 0., 0., 0.)
}

pub fn border_right(color: Color) -> Border {
    edge(color, 0., 1., 0., 0.)
}

pub fn border_left(color: Color) -> Border {
    edge(color, 0., 0., 0., 1.)
}

/// Canvas dark palette (`canvasPaletteDark` / `useHostTheme`).
/// Alpha fills are composited on editor `#181818` so Freya gets the same
/// opaque RGB the mock shows.
#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub control: Color,
    pub fill: Color,
    pub fill_hi: Color,
    pub text: Color,
    pub text_dim: Color,
    pub muted: Color,
    pub stroke: Color,
    pub stroke_soft: Color,
    pub stroke_hair: Color,
    pub accent: Color,
    pub on_accent: Color,
    pub shutter: Color,
    pub shutter_pressed: Color,
}

impl Palette {
    pub fn app() -> Self {
        Self {
            // editor #181818
            bg: Color::from_rgb(24, 24, 24),
            surface: Color::from_rgb(24, 24, 24),
            // fill.tertiary #E4E4E411 on editor
            control: Color::from_rgb(37, 37, 37),
            // fill.secondary #E4E4E41E on editor — chevrons
            fill: Color::from_rgb(48, 48, 48),
            // fill.primary #E4E4E430 on editor — active card
            fill_hi: Color::from_rgb(63, 63, 63),
            // foreground #E4E4E4
            text: Color::from_rgb(228, 228, 228),
            // line at 70% of primary
            text_dim: Color::from_rgb(160, 160, 160),
            // foregroundTertiary #E4E4E45E
            muted: Color::from_rgb(94, 94, 94),
            // stroke.primary #E4E4E433 on editor
            stroke: Color::from_rgb(74, 74, 74),
            // stroke.secondary #E4E4E41F on editor
            stroke_soft: Color::from_rgb(52, 52, 52),
            // stroke.tertiary #E4E4E414 on editor
            stroke_hair: Color::from_rgb(40, 40, 40),
            // accent #599CE7
            accent: Color::from_rgb(89, 156, 231),
            // buttonForeground #191c22
            on_accent: Color::from_rgb(25, 28, 34),
            shutter: Color::from_rgb(220, 38, 38),
            shutter_pressed: Color::from_rgb(168, 28, 28),
        }
    }
}
