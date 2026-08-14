//! UI themes — dark editor, frosted glass, snow white.

use freya::prelude::{Border, BorderWidth, Color};

/// Camera well — 4:3. The window is this wide. Nothing is wider.
pub const VIEWFINDER_W: f32 = 480.;
pub const VIEWFINDER_H: f32 = 360.;
pub const WINDOW_W: f32 = VIEWFINDER_W;
/// header 20 + well 360 + shutter 56 + fx band 176 + dock 88 + 4×8 gaps + 8 top
pub const WINDOW_H: f32 = 740.;
pub const FX_BAND_H: f32 = 176.;
pub const HEADER_H: f32 = 20.;
pub const SHUTTER_D: f32 = 56.;
/// Equal walls so the button sits on x = 240. Never flex — Freya flex is 100% parent.
pub const SHUTTER_SIDE: f32 = (WINDOW_W - SHUTTER_D) / 2.;
/// Header status. Long camera names die here, not off the glass.
pub const STATUS_MAX_CHARS: usize = 14;

/// Vertical rhythm between the stacked blocks.
pub const GAP: f32 = 8.;

/// Effect slider track width — must be px, not fill, or Freya's slider gets 0 width.
pub const SLIDER_LABEL_W: f32 = 52.;
pub const SLIDER_VALUE_W: f32 = 30.;
/// Gap between track end and numeric readout.
pub const SLIDER_VALUE_GAP: f32 = 6.;
pub const SLIDER_W: f32 = WINDOW_W
    - GAP
    - SLIDER_LABEL_W
    - GAP
    - SLIDER_VALUE_GAP
    - SLIDER_VALUE_W
    - GAP;

/// Live VFX + preview run at this width (4:3). Full sensor res is capture-only.
pub const PREVIEW_MAX_W: u32 = 640;

/// Canvas `Text` size="small" — 12 / 16.
pub const FONT_SMALL: f32 = 12.;

/// Cards in the dock window. Radius 0 — square tiles.
pub const DOCK_VISIBLE: usize = 3;
pub const DOCK_H: f32 = 88.;
pub const CARD_H: f32 = 72.;
pub const CARD_PAD: f32 = 8.;
/// Name + line stamped on the boot of the still.
pub const CARD_CAPTION_H: f32 = 32.;
pub const CARD_RADIUS: f32 = 0.;
pub const CHEVRON_W: f32 = 32.;
/// Three equal slots between the walls. No percent. No leftover padding.
pub const CARD_SLOT_W: f32 = (WINDOW_W - CHEVRON_W * 2.) / DOCK_VISIBLE as f32;
pub const DOCK_CARDS_W: f32 = WINDOW_W - CHEVRON_W * 2.;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Glass,
    Snow,
}

impl Theme {
    pub const ALL: [Self; 3] = [Self::Dark, Self::Glass, Self::Snow];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "DARK",
            Self::Glass => "GLASS",
            Self::Snow => "SNOW",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Dark => Self::Glass,
            Self::Glass => Self::Snow,
            Self::Snow => Self::Dark,
        }
    }

    /// macOS window transparency — desktop shows through the glass theme.
    pub fn window_transparent(self) -> bool {
        matches!(self, Self::Glass)
    }

    pub fn palette(self) -> Palette {
        match self {
            Self::Dark => Palette::dark(),
            Self::Glass => Palette::glass(),
            Self::Snow => Palette::snow(),
        }
    }
}

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
        Theme::Dark.palette()
    }

    /// Cursor-dark — default editor mock.
    pub fn dark() -> Self {
        Self {
            bg: Color::from_rgb(24, 24, 24),
            surface: Color::from_rgb(24, 24, 24),
            control: Color::from_rgb(37, 37, 37),
            fill: Color::from_rgb(48, 48, 48),
            fill_hi: Color::from_rgb(63, 63, 63),
            text: Color::from_rgb(228, 228, 228),
            text_dim: Color::from_rgb(160, 160, 160),
            muted: Color::from_rgb(94, 94, 94),
            stroke: Color::from_rgb(74, 74, 74),
            stroke_soft: Color::from_rgb(52, 52, 52),
            stroke_hair: Color::from_rgb(40, 40, 40),
            accent: Color::from_rgb(89, 156, 231),
            on_accent: Color::from_rgb(25, 28, 34),
            shutter: Color::from_rgb(220, 38, 38),
            shutter_pressed: Color::from_rgb(168, 28, 28),
        }
    }

    /// Frosted glass — translucent layers, cool edge light, icy accent.
    pub fn glass() -> Self {
        Self {
            bg: Color::from_argb(150, 16, 20, 28),
            surface: Color::from_argb(120, 22, 28, 38),
            control: Color::from_argb(165, 32, 40, 52),
            fill: Color::from_argb(190, 48, 58, 72),
            fill_hi: Color::from_argb(215, 62, 74, 92),
            text: Color::from_argb(240, 236, 242, 252),
            text_dim: Color::from_argb(210, 190, 200, 220),
            muted: Color::from_argb(170, 140, 155, 175),
            stroke: Color::from_argb(200, 255, 255, 255),
            stroke_soft: Color::from_argb(130, 200, 210, 230),
            stroke_hair: Color::from_argb(90, 160, 175, 200),
            accent: Color::from_argb(255, 120, 190, 255),
            on_accent: Color::from_rgb(16, 22, 32),
            shutter: Color::from_rgb(220, 38, 38),
            shutter_pressed: Color::from_rgb(168, 28, 28),
        }
    }

    /// Snow white — paper-bright chrome, soft gray hairlines.
    pub fn snow() -> Self {
        Self {
            bg: Color::from_rgb(252, 252, 254),
            surface: Color::from_rgb(255, 255, 255),
            control: Color::from_rgb(245, 247, 250),
            fill: Color::from_rgb(236, 240, 245),
            fill_hi: Color::from_rgb(224, 230, 238),
            text: Color::from_rgb(28, 32, 38),
            text_dim: Color::from_rgb(72, 78, 88),
            muted: Color::from_rgb(140, 148, 158),
            stroke: Color::from_rgb(208, 214, 224),
            stroke_soft: Color::from_rgb(220, 226, 234),
            stroke_hair: Color::from_rgb(232, 236, 242),
            accent: Color::from_rgb(72, 132, 220),
            on_accent: Color::from_rgb(255, 255, 255),
            shutter: Color::from_rgb(220, 38, 38),
            shutter_pressed: Color::from_rgb(168, 28, 28),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themes_cycle() {
        assert_eq!(Theme::Dark.next(), Theme::Glass);
        assert_eq!(Theme::Glass.next(), Theme::Snow);
        assert_eq!(Theme::Snow.next(), Theme::Dark);
    }

    #[test]
    fn glass_uses_alpha() {
        assert!(Palette::glass().bg.a() < 255);
        assert!(Palette::glass().control.a() < 255);
    }

    #[test]
    fn snow_is_bright() {
        let p = Palette::snow();
        assert!(p.bg.r() > 250);
        assert!(p.text.r() < 40);
    }
}
