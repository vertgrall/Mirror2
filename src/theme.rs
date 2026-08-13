//! Flat color. No paper, no tape, no stamp.

use freya::prelude::Color;

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
}

impl Palette {
    pub fn app() -> Self {
        Self {
            bg: Color::from_rgb(244, 244, 242),
            surface: Color::from_rgb(28, 28, 26),
            text: Color::from_rgb(22, 22, 20),
            muted: Color::from_rgb(110, 110, 106),
            accent: Color::from_rgb(22, 22, 20),
        }
    }
}
