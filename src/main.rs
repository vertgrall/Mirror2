//! Likeness — outsider spiritual successor to Photo Booth.
//! No curtains. No booth. No fake materials. You, kept.

mod camera;
mod effects;
mod keep;
mod stage;
mod theme;

use std::time::{Duration, Instant};

use async_io::Timer;
use freya::prelude::*;
use keyboard_types::{Code, Key};

use camera::CameraStatus;
use effects::Look;
use keep::KeepShot;
use stage::{draw_stage, draw_thumb, StageFrame};
use theme::Palette;

fn main() {
    camera::start();
    let palette = Palette::app();
    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(app)
                .with_title("Likeness")
                .with_size(1040., 820.)
                .with_min_size(820., 640.)
                .with_background(palette.bg),
        ),
    );
}

#[derive(Clone, Copy)]
enum Shutter {
    Idle,
    Counting { started: Instant },
    Flash { started: Instant },
}

fn app() -> Element {
    let palette = Palette::app();
    let look = use_state(|| Look::Plain);
    let shutter = use_state(|| Shutter::Idle);
    let keeps = use_state(Vec::<KeepShot>::new);
    let keep_error = use_state(|| None::<String>);
    let next_id = use_state(|| 1u64);
    let frame_seq = use_state(|| 0u64);

    camera::set_look(*look.peek());

    use_future(move || {
        let mut shutter = shutter;
        let mut keeps = keeps;
        let mut keep_error = keep_error;
        let mut next_id = next_id;
        let mut frame_seq = frame_seq;
        async move {
            let redraw = Platform::get().sender.clone();
            loop {
                Timer::after(Duration::from_millis(33)).await;
                if let Some(frame) = camera::current_frame() {
                    *frame_seq.write() = frame.seq;
                }
                let now = Instant::now();
                let current = *shutter.peek();
                match current {
                    Shutter::Counting { started }
                        if now.duration_since(started) >= Duration::from_secs(3) =>
                    {
                        if let Some(frame) = camera::current_frame() {
                            match keep::save_keep(
                                *next_id.peek(),
                                frame.width,
                                frame.height,
                                frame.rgba.as_ref(),
                            ) {
                                Ok(shot) => {
                                    *next_id.write() += 1;
                                    let mut list = keeps.peek().clone();
                                    list.insert(0, shot);
                                    list.truncate(10);
                                    *keeps.write() = list;
                                    *keep_error.write() = None;
                                }
                                Err(err) => *keep_error.write() = Some(err),
                            }
                        }
                        *shutter.write() = Shutter::Flash {
                            started: Instant::now(),
                        };
                    }
                    Shutter::Flash { started }
                        if now.duration_since(started) >= Duration::from_millis(140) =>
                    {
                        *shutter.write() = Shutter::Idle;
                    }
                    _ => {}
                }
                redraw(UserEvent::RequestRedraw);
            }
        }
    });

    let countdown = match *shutter.peek() {
        Shutter::Counting { started } => {
            let elapsed = started.elapsed().as_secs_f32();
            Some((3 - elapsed.floor() as u32).clamp(1, 3))
        }
        _ => None,
    };
    let flash = match *shutter.peek() {
        Shutter::Flash { started } => {
            let t = started.elapsed().as_secs_f32() / 0.14;
            (1.0 - t).clamp(0.0, 1.0)
        }
        _ => 0.0,
    };

    let status = camera::status();
    let live = camera::current_frame().map(StageFrame::from);
    let seq = *frame_seq.peek();
    let current_look = *look.peek();
    let keeps_now = keeps.peek().clone();
    let err = keep_error.peek().clone();
    let shutter_now = *shutter.peek();

    rect()
        .vertical()
        .width(Size::fill())
        .height(Size::fill())
        .background(palette.bg)
        .padding(Gaps::new_all(16.))
        .spacing(12.)
        .on_key_down(move |e: Event<KeyboardEventData>| {
            if e.key == Key::Character(" ".into()) || e.code == Code::Space {
                e.stop_propagation();
                start_countdown(shutter);
            }
        })
        .child(header(palette, &status))
        .child(stage_well(
            palette,
            live,
            status,
            current_look,
            countdown,
            flash,
            seq,
        ))
        .child(looks_row(palette, look, current_look))
        .child(keep_row(palette, shutter, shutter_now, countdown))
        .child(strip(palette, keeps_now))
        .child(footer(palette, err))
        .into()
}

fn start_countdown(mut shutter: State<Shutter>) {
    if matches!(*shutter.peek(), Shutter::Idle) {
        *shutter.write() = Shutter::Counting {
            started: Instant::now(),
        };
    }
}

fn header(palette: Palette, status: &CameraStatus) -> Element {
    let status_line = match status {
        CameraStatus::Starting => "asking the camera".to_string(),
        CameraStatus::Live { name } => name.clone(),
        CameraStatus::StandIn { reason } => reason.clone(),
    };
    rect()
        .horizontal()
        .width(Size::fill())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            label()
                .text("LIKENESS")
                .font_size(18.)
                .font_weight(FontWeight::BOLD)
                .color(palette.text),
        )
        .child(
            label()
                .text(format!("NEW TOWER  ·  {status_line}"))
                .font_size(11.)
                .color(palette.muted),
        )
        .into()
}

fn stage_well(
    palette: Palette,
    live: Option<StageFrame>,
    status: CameraStatus,
    look: Look,
    countdown: Option<u32>,
    flash: f32,
    seq: u64,
) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::flex(1.))
        .background(palette.surface)
        .child(
            canvas(RenderCallback::new({
                move |ctx| {
                    draw_stage(ctx, live.clone(), &status, look, countdown, flash);
                }
            }))
            .width(Size::fill())
            .height(Size::fill())
            .key(seq),
        )
        .into()
}

fn looks_row(palette: Palette, look_state: State<Look>, current: Look) -> Element {
    let mut row = rect()
        .horizontal()
        .width(Size::fill())
        .main_align(Alignment::Center)
        .spacing(4.);
    for look in Look::ALL {
        let selected = look == current;
        let mut chip = look_state;
        row = row.child(
            rect()
                .padding(Gaps::new(8., 14., 8., 14.))
                .background(if selected {
                    palette.accent
                } else {
                    Color::TRANSPARENT
                })
                .on_mouse_up(move |_| {
                    *chip.write() = look;
                    camera::set_look(look);
                })
                .child(
                    label()
                        .text(look.label())
                        .font_size(12.)
                        .font_weight(if selected {
                            FontWeight::BOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .color(if selected { palette.bg } else { palette.text }),
                ),
        );
    }
    row.into()
}

fn keep_row(
    palette: Palette,
    shutter: State<Shutter>,
    shutter_now: Shutter,
    countdown: Option<u32>,
) -> Element {
    let busy = !matches!(shutter_now, Shutter::Idle);
    let caption = if let Some(n) = countdown {
        n.to_string()
    } else if matches!(shutter_now, Shutter::Flash { .. }) {
        "KEPT".into()
    } else {
        "KEEP".into()
    };
    rect()
        .horizontal()
        .width(Size::fill())
        .main_align(Alignment::Center)
        .child(
            rect()
                .padding(Gaps::new(10., 28., 10., 28.))
                .background(if busy { palette.muted } else { palette.accent })
                .on_mouse_up(move |_| start_countdown(shutter))
                .child(
                    label()
                        .text(caption)
                        .font_size(14.)
                        .font_weight(FontWeight::BOLD)
                        .color(palette.bg),
                ),
        )
        .into()
}

fn strip(palette: Palette, keeps: Vec<KeepShot>) -> Element {
    if keeps.is_empty() {
        return rect()
            .width(Size::fill())
            .height(Size::px(72.))
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .child(
                label()
                    .text("nothing kept yet — KEEP or spacebar")
                    .font_size(12.)
                    .color(palette.muted),
            )
            .into();
    }

    let mut row = rect()
        .horizontal()
        .width(Size::fill())
        .height(Size::px(72.))
        .spacing(6.)
        .overflow(Overflow::Clip);
    for shot in keeps {
        let path = shot.path.clone();
        row = row.child(
            rect()
                .width(Size::px(96.))
                .height(Size::px(72.))
                .background(palette.surface)
                .on_mouse_up(move |_| keep::reveal(&path))
                .child(
                    canvas(RenderCallback::new({
                        let rgba = shot.rgba.clone();
                        let w = shot.width;
                        let h = shot.height;
                        move |ctx| draw_thumb(ctx, w, h, &rgba)
                    }))
                    .width(Size::fill())
                    .height(Size::fill())
                    .key(shot.id),
                ),
        );
    }
    row.into()
}

fn footer(palette: Palette, err: Option<String>) -> Element {
    let text = err.unwrap_or_else(|| {
        format!(
            "{}  ·  click a still to open it",
            keep::keep_dir().display()
        )
    });
    rect()
        .horizontal()
        .width(Size::fill())
        .main_align(Alignment::SpaceBetween)
        .child(label().text(text).font_size(11.).color(palette.muted))
        .child(
            rect().on_mouse_up(|_| keep::reveal_folder()).child(
                label()
                    .text("open folder")
                    .font_size(11.)
                    .color(palette.text),
            ),
        )
        .into()
}
