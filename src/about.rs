//! About panel — small dedicated window with Tower Village splash + Mirror2 copy.

use freya::prelude::*;

use crate::about_art::{draw_about_brand_mark, draw_about_splash_card};
use crate::about_assets::preload;
use crate::theme::{self, Palette, Theme};

const APP_NAME: &str = "Mirror2";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_BUILD: &str = "1";

pub const SPLASH_W: f32 = 300.;
pub const SPLASH_H: f32 = 323.;

pub const ABOUT_WINDOW_W: f32 = 440.;
pub const ABOUT_WINDOW_H: f32 = 640.;

pub fn about_content(palette: Palette) -> Element {
    rect()
        .vertical()
        .width(Size::px(420.))
        .main_align(Alignment::Start)
        .cross_align(Alignment::Center)
        .spacing(16.)
        .child(about_splash_header(palette))
        .child(about_title_block(palette))
        .child(about_bullets(palette))
        .child(about_brand_mark())
        .child(about_meta_rows(palette))
        .child(
            label()
                .text("Designed by Jon McMillion 2026")
                .font_size(11.)
                .color(palette.muted)
                .text_align(TextAlign::Center)
                .width(Size::px(380.)),
        )
        .into()
}

pub fn about_window() -> Element {
    preload();
    let palette = Theme::Dark.palette();

    rect()
        .expanded()
        .background(palette.bg)
        .padding(Gaps::new_all(16.))
        .child(ScrollView::new().expanded().child(about_content(palette)))
        .into()
}

fn about_splash_header(palette: Palette) -> Element {
    rect()
        .width(Size::px(SPLASH_W))
        .height(Size::px(SPLASH_H))
        .corner_radius(10.)
        .border(theme::border_all(palette.stroke_soft))
        .overflow(Overflow::Clip)
        .child(
            canvas(RenderCallback::new(|ctx| draw_about_splash_card(ctx)))
                .width(Size::px(SPLASH_W))
                .height(Size::px(SPLASH_H)),
        )
        .child(splash_lockup_label())
        .into()
}

fn splash_lockup_label() -> Element {
    rect()
        .position(Position::new_absolute().bottom(22.).right(24.))
        .padding(Gaps::new(0., 0., 0., 42.))
        .child(
            label()
                .text("By New Tower")
                .font_size(14.)
                .font_family("Times New Roman")
                .color(Color::from_rgb(255, 255, 255)),
        )
        .into()
}

fn about_brand_mark() -> Element {
    canvas(RenderCallback::new(|ctx| draw_about_brand_mark(ctx)))
        .width(Size::px(88.))
        .height(Size::px(64.))
        .into()
}

fn about_title_block(palette: Palette) -> Element {
    rect()
        .vertical()
        .spacing(8.)
        .width(Size::px(400.))
        .cross_align(Alignment::Center)
        .child(
            label()
                .text(APP_NAME)
                .font_size(22.)
                .font_weight(FontWeight::BOLD)
                .color(palette.text)
                .text_align(TextAlign::Center),
        )
        .child(
            label()
                .text("You, kept.")
                .font_size(15.)
                .font_weight(FontWeight::BOLD)
                .color(palette.text)
                .text_align(TextAlign::Center),
        )
        .child(
            label()
                .text(
                    "Outsider spiritual successor to Photo Booth — no curtains, no booth, \
                     no fake materials. Pick a look, live in the well, keep a still.",
                )
                .font_size(13.)
                .color(palette.muted)
                .text_align(TextAlign::Center)
                .width(Size::px(400.)),
        )
        .into()
}

fn about_bullets(palette: Palette) -> Element {
    rect()
        .vertical()
        .spacing(6.)
        .width(Size::px(400.))
        .padding(Gaps::new(0., 4., 0., 4.))
        .children([
            about_bullet(
                "480×360 camera well — always the preview, never a fake booth",
                palette,
            ),
            about_bullet(
                "Fifteen looks with wet + three sliders · photoreal dock stills",
                palette,
            ),
            about_bullet("Countdown shutter · themes · canyon icon", palette),
        ])
        .into()
}

fn about_bullet(text: &'static str, palette: Palette) -> Element {
    rect()
        .horizontal()
        .spacing(8.)
        .width(Size::fill())
        .child(
            label()
                .text("·")
                .font_size(13.)
                .font_weight(FontWeight::BOLD)
                .color(palette.muted),
        )
        .child(
            label()
                .text(text)
                .font_size(13.)
                .color(palette.muted)
                .width(Size::px(380.)),
        )
        .into()
}

fn about_meta_rows(palette: Palette) -> Element {
    rect()
        .vertical()
        .spacing(8.)
        .width(Size::px(400.))
        .children([
            meta_row(
                "Version",
                format!("{APP_VERSION} ({APP_BUILD})"),
                palette,
                false,
            ),
            meta_row("Platforms", "macOS 13+".to_string(), palette, false),
        ])
        .into()
}

fn meta_row(label_text: &'static str, value: String, palette: Palette, mono: bool) -> Element {
    let mut value_label = label()
        .text(value)
        .font_size(if mono { 11. } else { 13. })
        .color(palette.muted);

    if mono {
        value_label = value_label.font_family("Menlo");
    }

    rect()
        .horizontal()
        .width(Size::fill())
        .child(
            label()
                .text(label_text)
                .font_size(13.)
                .color(palette.text),
        )
        .child(rect().width(Size::fill()).child(value_label))
        .into()
}

#[cfg(test)]
mod tests {
    use freya::prelude::*;
    use freya_testing::prelude::*;

    use super::about_content;
    use crate::theme::Theme;

    fn collect_label_texts(test: &TestingRunner) -> Vec<String> {
        test.find_many(|_, element| {
            Label::try_downcast(element).map(|label| label.text.to_string())
        })
    }

    #[test]
    fn about_content_renders_mirror2_copy() {
        let palette = Theme::Dark.palette();
        let mut test = launch_test(move || {
            rect()
                .width(Size::px(460.))
                .height(Size::px(920.))
                .padding(Gaps::new_all(16.))
                .child(about_content(palette))
        });
        test.sync_and_update();

        let labels = collect_label_texts(&test);
        for needle in [
            "Mirror2",
            "You, kept.",
            "By New Tower",
            "Designed by Jon McMillion 2026",
            "Fifteen looks with wet + three sliders",
        ] {
            assert!(
                labels.iter().any(|text| text.contains(needle)),
                "missing label containing {needle:?}; got: {labels:?}"
            );
        }
    }
}
