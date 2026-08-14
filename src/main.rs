//! Mirror2 — outsider spiritual successor to Photo Booth.
//! No curtains. No booth. No fake materials. You, kept.

mod backgrounds;
mod camera;
mod effects;
mod keep;
#[cfg(target_os = "macos")]
mod macos_avf;
mod shutter;
mod stage;
mod stills;
mod theme;
mod vfx;

use std::time::{Duration, Instant};

use async_io::Timer;
use freya::prelude::*;

use camera::CameraStatus;
use effects::{
    atmo_param_defs, bg_param_defs, bg_params_from_values, cycle_background, current_path,
    set_atmosphere, set_background, set_params, AtmosphereParams, BackgroundParams, Look,
    LookParams, ParamDef,
};
use backgrounds::label as bg_label;
use keep::KeepShot;
use shutter::Shutter;
use stage::{draw_stage, draw_thumb, StageFrame};
use theme::Palette;

fn main() {
    let palette = Palette::app();
    launch(
        LaunchConfig::new()
            .with_future(|_proxy| async move {
                // AVFoundation needs the app run loop up before it will deliver frames.
                Timer::after(Duration::from_millis(200)).await;
                camera::start();
            })
            .with_window(
                WindowConfig::new(app)
                    .with_title("Mirror2")
                    .with_size(theme::WINDOW_W as f64, theme::WINDOW_H as f64)
                    .with_min_size(theme::WINDOW_W as f64, theme::WINDOW_H as f64)
                    .with_max_size(theme::WINDOW_W as f64, theme::WINDOW_H as f64)
                    .with_resizable(false)
                    .with_background(palette.bg),
            ),
    );
}

fn app() -> Element {
    let palette = Palette::app();
    let look = use_state(|| Look::None);
    let mut dock_page = use_state(|| 0usize);
    let params = use_state(|| LookParams::defaults(Look::None));
    let bg_enabled = use_state(|| false);
    let bg_values = use_state(|| {
        let d = BackgroundParams::default();
        [d.key_hue, d.key_width, d.feather, d.spill]
    });
    let atmo_values = use_state(|| {
        let d = AtmosphereParams::default();
        [d.smoke, d.density, d.drift, d.scale]
    });
    let shutter = use_state(|| Shutter::Idle);
    let beat = use_state(|| 0u32);
    let keeps = use_state(Vec::<KeepShot>::new);
    let keep_error = use_state(|| None::<String>);
    let next_id = use_state(|| 1u64);
    let frame_seq = use_state(|| 0u64);
    let controls_rev = use_state(|| 0u32);

    backgrounds::ensure_dir();

    use_future(move || {
        let mut shutter = shutter;
        let mut beat = beat;
        let mut keeps = keeps;
        let mut keep_error = keep_error;
        let mut next_id = next_id;
        let mut frame_seq = frame_seq;
        let controls_rev = controls_rev;
        async move {
            let redraw = Platform::get().sender.clone();
            let mut last_seen_seq = 0u64;
            let mut last_controls_rev = 0u32;
            loop {
                Timer::after(Duration::from_millis(33)).await;

                let mut needs_redraw = false;

                if let Some(frame) = camera::current_frame() {
                    if frame.seq != last_seen_seq {
                        last_seen_seq = frame.seq;
                        *frame_seq.write() = frame.seq;
                        needs_redraw = true;
                    }
                }

                let rev = *controls_rev.peek();
                if rev != last_controls_rev {
                    last_controls_rev = rev;
                    needs_redraw = true;
                }

                let now = Instant::now();
                let before = *shutter.peek();
                match before {
                    Shutter::Counting { started } if shutter::should_capture(started, now) => {
                        let shot_id = *next_id.peek();
                        if let Some(frame) = camera::snapshot_for_keep() {
                            match keep::save_keep(
                                shot_id,
                                frame.width,
                                frame.height,
                                frame.rgba.as_ref(),
                            ) {
                                Ok(shot) => {
                                    *next_id.write() = shot_id + 1;
                                    let mut list = keeps.peek().clone();
                                    list.insert(0, shot);
                                    list.truncate(10);
                                    *keeps.write() = list;
                                    *keep_error.write() = None;
                                }
                                Err(err) => *keep_error.write() = Some(err),
                            }
                        }
                        let next = Shutter::Flash {
                            started: Instant::now(),
                        };
                        shutter::publish(next);
                        *shutter.write() = next;
                    }
                    Shutter::Flash { started } if shutter::flash_done(started, now) => {
                        shutter::publish(Shutter::Idle);
                        *shutter.write() = Shutter::Idle;
                    }
                    _ => {}
                }

                let after = *shutter.peek();
                if after != before {
                    needs_redraw = true;
                }
                if matches!(
                    after,
                    Shutter::Counting { .. } | Shutter::Flash { .. }
                ) {
                    *beat.write() += 1;
                    needs_redraw = true;
                }

                if needs_redraw {
                    redraw(UserEvent::RequestRedraw);
                }
            }
        }
    });

    let _beat = beat();
    let now = Instant::now();
    let shutter_now = shutter();
    let overlay = shutter::overlay_at(shutter_now, now);
    let countdown = match overlay {
        shutter::Overlay::Digit(n) => Some(n),
        _ => None,
    };

    let status = camera::status();
    let seq = *frame_seq.peek();
    let current_look = look();
    let current_params = params();
    let page = dock_page();
    let _keeps_now = keeps.read().clone();
    let err = keep_error.read().clone();

    rect()
        .vertical()
        .width(Size::fill())
        .height(Size::fill())
        .background(palette.bg)
        .on_global_key_down(move |e: Event<KeyboardEventData>| {
            if shutter::is_arrow_left(&e.key, e.code) {
                e.stop_propagation();
                step_dock(&mut dock_page, -1);
                return;
            }
            if shutter::is_arrow_right(&e.key, e.code) {
                e.stop_propagation();
                step_dock(&mut dock_page, 1);
                return;
            }
            if shutter::is_space(&e.key, e.code) {
                e.stop_propagation();
                start_countdown(shutter);
            }
        })
        .child(
            rect()
                .vertical()
                .width(Size::px(theme::WINDOW_W))
                .height(Size::fill())
                .spacing(theme::GAP)
                .padding(Gaps::new(theme::GAP, 0., 0., 0.))
                .child(header(palette, &status))
                .child(stage_well(palette, current_look, seq))
                .child(shutter_button(palette, shutter, shutter_now, countdown))
                .child(look_fx_panel(
                    palette,
                    current_look,
                    current_params,
                    params,
                    controls_rev,
                    page,
                    err,
                ))
                .child(look_dock(
                    palette,
                    look,
                    params,
                    controls_rev,
                    dock_page,
                    current_look,
                    page,
                )),
        )
        .into()
}

fn dock_max() -> usize {
    Look::RAIL.len().saturating_sub(theme::DOCK_VISIBLE)
}

fn dock_start(page: usize) -> usize {
    page.min(dock_max())
}

fn step_dock(page: &mut State<usize>, delta: i32) {
    let start = dock_start(*page.peek());
    let next = (start as i32 + delta).clamp(0, dock_max() as i32) as usize;
    if next != start {
        *page.write() = next;
    }
}

fn start_countdown(mut shutter: State<Shutter>) {
    if matches!(*shutter.peek(), Shutter::Idle) {
        let next = Shutter::Counting {
            started: Instant::now(),
        };
        shutter::publish(next);
        *shutter.write() = next;
    }
}

/// Header status. Short enough to live in the right 200px of a 480 window.
fn status_copy(status: &CameraStatus) -> String {
    match status {
        CameraStatus::Starting => "asking camera".into(),
        CameraStatus::Live { name } => clip_status(name),
        CameraStatus::StandIn { reason } => clip_status(reason),
    }
}

fn clip_status(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.contains("FaceTime") {
        return "FaceTime".into();
    }
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= theme::STATUS_MAX_CHARS {
        return trimmed.to_string();
    }
    chars.into_iter().take(theme::STATUS_MAX_CHARS).collect()
}

fn gutter() -> Element {
    rect().width(Size::px(theme::GAP)).into()
}

fn px_spacer(w: f32) -> Element {
    rect().width(Size::px(w)).height(Size::px(1.)).into()
}

fn header(palette: Palette, status: &CameraStatus) -> Element {
    rect()
        .horizontal()
        .width(Size::px(theme::WINDOW_W))
        .height(Size::px(theme::HEADER_H))
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .child(gutter())
                .child(
                    label()
                        .text("MIRROR2")
                        .font_size(theme::FONT_SMALL)
                        .font_weight(FontWeight::BOLD)
                        .color(palette.text),
                ),
        )
        .child(
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(status_copy(status))
                        .font_size(theme::FONT_SMALL)
                        .color(palette.muted),
                )
                .child(gutter()),
        )
        .into()
}

fn look_fx_panel(
    palette: Palette,
    look: Look,
    values: LookParams,
    params: State<LookParams>,
    controls_rev: State<u32>,
    page: usize,
    err: Option<String>,
) -> Element {
    let start = dock_start(page);
    let end = (start + theme::DOCK_VISIBLE).min(Look::RAIL.len());
    let count = format!("{}–{} of {}", start + 1, end, Look::RAIL.len());
    let defs = look.param_defs();

    let mut col = rect()
        .vertical()
        .width(Size::px(theme::WINDOW_W))
        .height(Size::px(theme::FX_BAND_H))
        .spacing(4.)
        .padding(Gaps::new(theme::GAP, 0., 0., 0.))
        .child(
            rect()
                .horizontal()
                .width(Size::px(theme::WINDOW_W))
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .horizontal()
                        .cross_align(Alignment::Center)
                        .child(gutter())
                        .child(
                            label()
                                .text(format!("{}  ·  {}", look.label(), look.tile_line()))
                                .font_size(theme::FONT_SMALL)
                                .font_weight(FontWeight::BOLD)
                                .color(palette.text),
                        ),
                )
                .child(
                    rect()
                        .horizontal()
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(count)
                                .font_size(theme::FONT_SMALL)
                                .color(palette.muted),
                        )
                        .child(gutter()),
                ),
        );

    for (index, def) in defs.iter().enumerate() {
        col = col.child(kit_slider(palette, *def, values.values[index], {
            let mut param_state = params;
            let mut rev = controls_rev;
            move |pct| write_param_pct(&mut param_state, index, *def, pct, &mut rev)
        }));
    }
    col.child(footer(palette, err)).into()
}

fn look_dock(
    palette: Palette,
    look_state: State<Look>,
    params: State<LookParams>,
    controls_rev: State<u32>,
    mut page_state: State<usize>,
    current_look: Look,
    page: usize,
) -> Element {
    let start = dock_start(page);
    let shown = &Look::RAIL[start..start + theme::DOCK_VISIBLE.min(Look::RAIL.len() - start)];

    let mut cards = rect()
        .horizontal()
        .width(Size::px(theme::DOCK_CARDS_W))
        .height(Size::px(theme::CARD_H))
        .cross_align(Alignment::Center);
    for &look in shown {
        cards = cards.child(look_card(
            palette,
            look,
            look == current_look,
            look_state,
            params,
            controls_rev,
        ));
    }

    rect()
        .horizontal()
        .width(Size::px(theme::WINDOW_W))
        .height(Size::px(theme::DOCK_H))
        .background(palette.control)
        .border(theme::border_top(palette.stroke_soft))
        .cross_align(Alignment::Center)
        .on_wheel(move |e: Event<WheelEventData>| {
            if e.delta_y > 0.0 {
                step_dock(&mut page_state, 1);
            } else if e.delta_y < 0.0 {
                step_dock(&mut page_state, -1);
            }
        })
        .child(dock_chevron(palette, "<", move |_| {
            step_dock(&mut page_state, -1)
        }))
        .child(cards)
        .child(dock_chevron(palette, ">", move |_| {
            step_dock(&mut page_state, 1)
        }))
        .into()
}

fn dock_chevron(
    palette: Palette,
    mark: &'static str,
    on_press: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    let edge = if mark == "<" {
        theme::border_right(palette.stroke_soft)
    } else {
        theme::border_left(palette.stroke_soft)
    };
    rect()
        .width(Size::px(theme::CHEVRON_W))
        .height(Size::px(theme::DOCK_H))
        .background(palette.fill)
        .border(edge)
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .on_press(on_press)
        .child(
            label()
                .text(mark)
                .font_size(16.)
                .color(palette.text),
        )
        .into()
}

fn wear_handler(
    look: Look,
    look_state: State<Look>,
    params: State<LookParams>,
    controls_rev: State<u32>,
) -> impl FnMut(Event<PressEventData>) + 'static {
    let mut looks = look_state;
    let mut param_state = params;
    let mut rev = controls_rev;
    move |_| wear_look(look, &mut looks, &mut param_state, &mut rev)
}

fn look_card(
    palette: Palette,
    look: Look,
    selected: bool,
    look_state: State<Look>,
    params: State<LookParams>,
    controls_rev: State<u32>,
) -> Element {
    let text_w = theme::CARD_SLOT_W - theme::CARD_PAD * 2.;
    rect()
        .width(Size::px(theme::CARD_SLOT_W))
        .height(Size::px(theme::CARD_H))
        .corner_radius(theme::CARD_RADIUS)
        .background(palette.bg)
        .border(theme::border_all(if selected {
            palette.accent
        } else {
            palette.stroke
        }))
        .overflow(Overflow::Clip)
        .on_press(wear_handler(look, look_state, params, controls_rev))
        .child(
            canvas(RenderCallback::new({
                move |ctx| stills::draw_still(ctx, look)
            }))
            .width(Size::px(theme::CARD_SLOT_W))
            .height(Size::px(theme::CARD_H))
            .key(look.id()),
        )
        .child(
            rect()
                .position(Position::new_absolute().bottom(0.).left(0.))
                .width(Size::px(theme::CARD_SLOT_W))
                .height(Size::px(theme::CARD_CAPTION_H))
                .padding(Gaps::new(2., theme::CARD_PAD, 4., theme::CARD_PAD))
                .background(Color::from_argb(210, 24, 24, 24))
                .spacing(1.)
                .on_press(wear_handler(look, look_state, params, controls_rev))
                .child(
                    label()
                        .text(look.label())
                        .font_size(theme::FONT_SMALL)
                        .font_weight(FontWeight::BOLD)
                        .color(palette.text)
                        .width(Size::px(text_w))
                        .on_press(wear_handler(look, look_state, params, controls_rev)),
                )
                .child(
                    label()
                        .text(look.tile_line())
                        .font_size(theme::FONT_SMALL)
                        .color(palette.text_dim)
                        .width(Size::px(text_w))
                        .on_press(wear_handler(look, look_state, params, controls_rev)),
                ),
        )
        .into()
}

fn wear_look(
    look: Look,
    look_state: &mut State<Look>,
    params: &mut State<LookParams>,
    controls_rev: &mut State<u32>,
) {
    *look_state.write() = look;
    camera::set_look(look);
    let defaults = LookParams::defaults(look);
    params.set(defaults);
    apply_controls(defaults, controls_rev);
}

fn stage_well(palette: Palette, look: Look, seq: u64) -> Element {
    rect()
        .width(Size::px(theme::VIEWFINDER_W))
        .height(Size::px(theme::VIEWFINDER_H))
        .background(palette.surface)
        .child(
            canvas(RenderCallback::new({
                move |ctx| {
                    let live = camera::current_frame().map(StageFrame::from);
                    let status = camera::status();
                    let overlay = shutter::overlay_at(shutter::current(), Instant::now());
                    let (countdown, flash) = match overlay {
                        shutter::Overlay::Digit(n) => (Some(n), 0.0),
                        shutter::Overlay::Flash(a) => (None, a),
                        shutter::Overlay::None => (None, 0.0),
                    };
                    draw_stage(ctx, live, &status, look, countdown, flash);
                }
            }))
            .width(Size::fill())
            .height(Size::fill())
            .key(seq),
        )
        .into()
}

fn background_controls(
    palette: Palette,
    bg_enabled: State<bool>,
    bg_values: State<[f32; 4]>,
    controls_rev: State<u32>,
) -> Element {
    let enabled = bg_enabled();
    let values = bg_values();
    let plate_name = current_path()
        .map(|p| bg_label(&p))
        .unwrap_or_else(|| "VOID".into());

    let mut panel = rect()
        .vertical()
        .width(Size::fill())
        .spacing(6.)
        .child(
            label()
                .text("green screen + plate")
                .font_size(10.)
                .color(palette.muted),
        )
        .child({
            let mut en = bg_enabled;
            let vals = bg_values;
            let mut rev = controls_rev;
            rect()
                .horizontal()
                .spacing(4.)
                .cross_align(Alignment::Center)
                .child(bg_action_chip(
                    palette,
                    if enabled { "BG ON" } else { "BG OFF" },
                    enabled,
                    move |_| {
                        let next = !*en.peek();
                        *en.write() = next;
                        sync_background(next, *vals.peek());
                        touch_controls(&mut rev);
                    },
                ))
                .child({
                    let mut rev = controls_rev;
                    let vals = bg_values;
                    let en = bg_enabled;
                    bg_action_chip(palette, "NEXT", false, move |_| {
                        cycle_background();
                        sync_background(*en.peek(), *vals.peek());
                        touch_controls(&mut rev);
                    })
                })
                .child(bg_action_chip(palette, "FOLDER", false, |_| {
                    backgrounds::reveal_folder();
                }))
                .child(
                    label()
                        .text(plate_name)
                        .font_size(9.)
                        .color(palette.text),
                )
        });

    for (index, def) in bg_param_defs().iter().enumerate() {
        panel = panel.child(kit_slider(palette, *def, values[index], {
            let mut vals = bg_values;
            let en = bg_enabled;
            let mut rev = controls_rev;
            move |pct| write_bg_pct(&mut vals, en, index, *def, pct, &mut rev)
        }));
    }
    panel.into()
}

fn atmosphere_controls(
    palette: Palette,
    atmo_values: State<[f32; 4]>,
    controls_rev: State<u32>,
) -> Element {
    let values = atmo_values();
    let mut panel = rect()
        .vertical()
        .width(Size::fill())
        .spacing(6.)
        .child(
            label()
                .text("smoke haze")
                .font_size(10.)
                .color(palette.muted),
        );

    for (index, def) in atmo_param_defs().iter().enumerate() {
        panel = panel.child(kit_slider(palette, *def, values[index], {
            let mut vals = atmo_values;
            let mut rev = controls_rev;
            move |pct| write_atmo_pct(&mut vals, index, *def, pct, &mut rev)
        }));
    }
    panel.into()
}

fn kit_slider(
    palette: Palette,
    def: ParamDef,
    value: f32,
    on_moved: impl FnMut(f64) + 'static,
) -> Element {
    let span = def.max - def.min;
    let pct = def.to_pct(value);
    let value_label = if span >= 10.0 {
        format!("{:.0}", value)
    } else if span >= 1.0 {
        format!("{:.1}", value)
    } else {
        format!("{:.2}", value)
    };

    rect()
        .horizontal()
        .width(Size::px(theme::WINDOW_W))
        .height(Size::px(28.))
        .main_align(Alignment::Start)
        .cross_align(Alignment::Center)
        .spacing(8.)
        .child(
            label()
                .text(def.label)
                .font_size(10.)
                .color(palette.text)
                .width(Size::px(52.)),
        )
        .child(
            Slider::new(on_moved)
                .value(pct)
                .size(Size::px(theme::SLIDER_W))
                .direction(Direction::Horizontal),
        )
        .child(
            label()
                .text(value_label)
                .font_size(10.)
                .color(palette.muted)
                .width(Size::px(32.)),
        )
        .into()
}

fn bg_action_chip(
    palette: Palette,
    text: &'static str,
    selected: bool,
    on_press: impl FnMut(Event<MouseEventData>) + 'static,
) -> Element {
    rect()
        .padding(Gaps::new(4., 8., 4., 8.))
        .background(if selected {
            palette.fill_hi
        } else {
            palette.fill
        })
        .on_mouse_up(on_press)
        .child(
            label()
                .text(text)
                .font_size(theme::FONT_SMALL)
                .color(if selected {
                    palette.text
                } else {
                    palette.text_dim
                }),
        )
        .into()
}

fn sync_background(enabled: bool, values: [f32; 4]) {
    set_background(bg_params_from_values(values, enabled));
}

fn sync_atmosphere(values: [f32; 4]) {
    set_atmosphere(AtmosphereParams {
        smoke: values[0],
        density: values[1],
        drift: values[2],
        scale: values[3],
    });
}

fn touch_controls(controls_rev: &mut State<u32>) {
    camera::refresh_preview();
    *controls_rev.write() += 1;
    request_redraw();
}

fn request_redraw() {
    (Platform::get().sender)(UserEvent::RequestRedraw);
}

fn apply_controls(params: LookParams, controls_rev: &mut State<u32>) {
    set_params(params);
    camera::refresh_preview();
    *controls_rev.write() += 1;
    request_redraw();
}

fn write_param_pct(
    params: &mut State<LookParams>,
    index: usize,
    def: ParamDef,
    pct: f64,
    controls_rev: &mut State<u32>,
) {
    let mut p = *params.peek();
    if !p.apply_pct(index, def, pct) {
        return;
    }
    params.set(p);
    set_params(p);
    throttle_refresh_preview();
    *controls_rev.write() += 1;
    request_redraw();
}

fn write_bg_pct(
    bg_values: &mut State<[f32; 4]>,
    bg_enabled: State<bool>,
    index: usize,
    def: ParamDef,
    pct: f64,
    controls_rev: &mut State<u32>,
) {
    let v = def.from_pct(pct);
    let mut p = *bg_values.peek();
    if (p[index] - v).abs() < 0.0005 {
        return;
    }
    p[index] = v;
    bg_values.set(p);
    sync_background(*bg_enabled.peek(), p);
    throttle_refresh_preview();
    *controls_rev.write() += 1;
    request_redraw();
}

fn write_atmo_pct(
    atmo_values: &mut State<[f32; 4]>,
    index: usize,
    def: ParamDef,
    pct: f64,
    controls_rev: &mut State<u32>,
) {
    let v = def.from_pct(pct);
    let mut p = *atmo_values.peek();
    if (p[index] - v).abs() < 0.0005 {
        return;
    }
    p[index] = v;
    atmo_values.set(p);
    sync_atmosphere(p);
    throttle_refresh_preview();
    *controls_rev.write() += 1;
    request_redraw();
}

fn throttle_refresh_preview() {
    use std::sync::{Mutex, OnceLock};
    static LAST: OnceLock<Mutex<Instant>> = OnceLock::new();
    let Ok(mut last) = LAST.get_or_init(|| Mutex::new(Instant::now())).lock() else {
        return;
    };
    if last.elapsed() >= Duration::from_millis(33) {
        camera::refresh_preview();
        *last = Instant::now();
    }
}

fn shutter_button(
    palette: Palette,
    shutter: State<Shutter>,
    shutter_now: Shutter,
    countdown: Option<u32>,
) -> Element {
    let busy = !matches!(shutter_now, Shutter::Idle);
    let label_text = if let Some(n) = countdown {
        n.to_string()
    } else if matches!(shutter_now, Shutter::Flash { .. }) {
        "✓".into()
    } else {
        String::new()
    };

    rect()
        .horizontal()
        .width(Size::px(theme::WINDOW_W))
        .height(Size::px(theme::SHUTTER_D))
        .child(px_spacer(theme::SHUTTER_SIDE))
        .child(
            rect()
                .width(Size::px(theme::SHUTTER_D))
                .height(Size::px(theme::SHUTTER_D))
                .corner_radius(theme::SHUTTER_D / 2.)
                .background(if busy {
                    palette.shutter_pressed
                } else {
                    palette.shutter
                })
                .border(Border::new().fill(palette.bg).width(BorderWidth {
                    top: 3.,
                    right: 3.,
                    bottom: 3.,
                    left: 3.,
                }))
                .main_align(Alignment::Center)
                .cross_align(Alignment::Center)
                .a11y_focusable(true)
                .on_press(move |_| start_countdown(shutter))
                .child(shutter_face(label_text)),
        )
        .child(px_spacer(theme::SHUTTER_SIDE))
        .into()
}

fn shutter_face(label_text: String) -> Element {
    if label_text.is_empty() {
        rect()
            .width(Size::px(44.))
            .height(Size::px(44.))
            .corner_radius(22.)
            .background(Color::from_rgb(255, 255, 255))
            .into()
    } else {
        label()
            .text(label_text)
            .font_size(24.)
            .font_weight(FontWeight::BOLD)
            .color(Color::WHITE)
            .into()
    }
}

fn strip(palette: Palette, keeps: Vec<KeepShot>) -> Element {
    if keeps.is_empty() {
        return rect().width(Size::px(theme::WINDOW_W)).height(Size::px(0.)).into();
    }

    let mut row = rect()
        .horizontal()
        .width(Size::px(theme::WINDOW_W))
        .height(Size::px(48.))
        .spacing(6.)
        .overflow(Overflow::Clip);
    for shot in keeps {
        let path = shot.path.clone();
        row = row.child(
            rect()
                .width(Size::px(64.))
                .height(Size::px(48.))
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
    let text = err.unwrap_or_else(|| "open folder".into());
    rect()
        .horizontal()
        .width(Size::px(theme::WINDOW_W))
        .height(Size::px(16.))
        .main_align(Alignment::End)
        .cross_align(Alignment::Center)
        .child(
            rect().on_mouse_up(|_| keep::reveal_folder()).child(
                label()
                    .text(text)
                    .font_size(theme::FONT_SMALL)
                    .color(palette.muted),
            ),
        )
        .child(gutter())
        .into()
}

#[cfg(test)]
mod slider_ui_tests {
    use super::*;
    use freya_testing::prelude::*;
    use std::path::Path;

    fn slider_harness() -> impl IntoElement {
        let mut params = use_state(|| LookParams::defaults(Look::Morph));
        let def = Look::Morph.param_defs()[0];
        let value = params().values[0];
        kit_slider(Palette::app(), def, value, move |pct| {
            let mut p = *params.peek();
            if p.apply_pct(0, def, pct) {
                params.set(p);
            }
        })
    }

    fn app_shell_harness() -> impl IntoElement {
        let look = use_state(|| Look::None);
        let dock_page = use_state(|| 0usize);
        let params = use_state(|| LookParams::defaults(Look::None));
        let controls_rev = use_state(|| 0u32);
        let current = look();
        let page = dock_page();
        rect()
            .vertical()
            .width(Size::fill())
            .height(Size::fill())
            .overflow(Overflow::Clip)
            .child(
                rect()
                    .vertical()
                    .width(Size::fill())
                    .height(Size::func(|ctx| Some((ctx.parent - theme::DOCK_H).max(0.0))))
                    .child(
                        label()
                            .text("stage")
                            .font_size(theme::FONT_SMALL),
                    ),
            )
            .child(look_dock(
                Palette::app(),
                look,
                params,
                controls_rev,
                dock_page,
                current,
                page,
            ))
    }

    fn dock_harness() -> impl IntoElement {
        let look = use_state(|| Look::None);
        let mut dock_page = use_state(|| 0usize);
        let params = use_state(|| LookParams::defaults(Look::None));
        let controls_rev = use_state(|| 0u32);
        let current = look();
        let page = dock_page();
        rect()
            .vertical()
            .width(Size::fill())
            .height(Size::fill())
            .on_global_key_down(move |e: Event<KeyboardEventData>| {
                if shutter::is_arrow_left(&e.key, e.code) {
                    step_dock(&mut dock_page, -1);
                }
                if shutter::is_arrow_right(&e.key, e.code) {
                    step_dock(&mut dock_page, 1);
                }
            })
            .child(look_dock(
                Palette::app(),
                look,
                params,
                controls_rev,
                dock_page,
                current,
                page,
            ))
    }

    fn has_label(test: &TestingRunner, text: &str) -> bool {
        test.find(|_, element| {
            Label::try_downcast(element).filter(|label| label.text.as_ref() == text)
        })
        .is_some()
    }

    #[test]
    fn slider_changes_wet_value() {
        let mut test = launch_test(slider_harness);
        test.sync_and_update();
        assert!(has_label(&test, "1.0"), "Morph wet default is 1.0");

        let slider = test
            .find(|node, element| {
                Rect::try_downcast(element)
                    .filter(|_| (node.layout().area.size.width - theme::SLIDER_W).abs() < 1.0)
                    .map(|_| node)
            })
            .expect("slider track should be 220px wide");
        let area = slider.layout().area;
        // Click the left side of the track — Freya maps MouseDown to PointerDown.
        let x = area.min_x() as f64 + 12.0;
        let y = area.min_y() as f64 + (area.size.height as f64) * 0.5;
        test.click_cursor((x, y));

        assert!(
            ["0.0", "0.1", "0.2"].iter().any(|t| has_label(&test, t)),
            "clicking the left of the wet slider should drop the value from 1.0"
        );
        assert!(!has_label(&test, "1.0"));
    }

    fn click_label(test: &mut TestingRunner, text: &str) {
        let node = test
            .find(|node, element| {
                Label::try_downcast(element)
                    .filter(|label| label.text.as_ref() == text)
                    .map(|_| node)
            })
            .unwrap_or_else(|| panic!("missing label {text}"));
        let area = node.layout().area;
        test.click_cursor((
            area.min_x() as f64 + (area.size.width as f64) * 0.5,
            area.min_y() as f64 + (area.size.height as f64) * 0.5,
        ));
    }

    fn label_box(test: &TestingRunner, text: &str) -> Option<(f32, f32, f32, f32)> {
        test.find(|node, element| {
            Label::try_downcast(element)
                .filter(|label| label.text.as_ref() == text)
                .map(|_| node)
        })
        .map(|node| {
            let a = node.layout().area;
            (a.min_x(), a.min_y(), a.size.width, a.size.height)
        })
    }

    #[test]
    fn real_app_shows_dock_cards_in_window() {
        const W: f32 = theme::WINDOW_W;
        const H: f32 = theme::WINDOW_H;
        let mut test = TestingRunner::new(app, Size2D::new(W, H), |_| {}, 1.0).0;
        test.sync_and_update();

        let off = label_box(&test, "OFF").expect("OFF card name must exist");
        let vhs = label_box(&test, "VHS").expect("VHS card name must exist");
        let line = label_box(&test, "tracking · wear").expect("VHS line must exist");

        assert!(
            off.3 > 8.0 && off.2 > 8.0,
            "OFF label collapsed: {off:?}"
        );
        assert!(
            line.1 + line.3 <= H,
            "card line is clipped by the window: {line:?} window_h={H}"
        );
        assert!(
            off.1 >= H - theme::DOCK_H,
            "OFF should sit inside the bottom {h}px, was y={}",
            off.1,
            h = theme::DOCK_H
        );
        assert!(
            vhs.0 > off.0 + off.2,
            "cards should sit in a row, not stacked. OFF={off:?} VHS={vhs:?}"
        );
        let well = test
            .find(|node, element| {
                Rect::try_downcast(element)
                    .filter(|_| {
                        (node.layout().area.size.width - theme::VIEWFINDER_W).abs() < 1.0
                            && (node.layout().area.size.height - theme::VIEWFINDER_H).abs() < 1.0
                    })
                    .map(|_| node)
            })
            .expect("480×360 camera well");
        let well_area = well.layout().area;
        assert!(
            well_area.min_x().abs() < 1.0,
            "well must sit on the left edge, x={}",
            well_area.min_x()
        );
        let dock = test
            .find(|node, element| {
                Rect::try_downcast(element)
                    .filter(|_| {
                        (node.layout().area.size.height - theme::DOCK_H).abs() < 1.0
                            && node.layout().area.size.width > 400.0
                    })
                    .map(|_| node)
            })
            .expect("dock bar must span the window");
        let dock_area = dock.layout().area;
        assert!(
            (dock_area.size.width - W).abs() < 2.0,
            "dock width was {}",
            dock_area.size.width
        );
        assert!(
            dock_area.min_x().abs() < 1.0,
            "dock must share the well's left edge, x={}",
            dock_area.min_x()
        );
        assert!(
            (dock_area.min_y() - (H - theme::DOCK_H)).abs() < 2.0,
            "dock must sit on the window bottom, y={}..{}",
            dock_area.min_y(),
            dock_area.max_y()
        );
        assert!(
            dock_area.max_y() <= H + 1.0,
            "dock is clipped, max_y={}",
            dock_area.max_y()
        );
        assert!(
            (well_area.size.width - dock_area.size.width).abs() < 1.0,
            "well and dock must be the same width"
        );

        let out = std::env::temp_dir().join("mirror2-dock-verify.png");
        test.render_to_file(&out);
        assert!(out.exists(), "wrote {out:?}");
    }

    #[test]
    fn dock_stays_visible_under_a_flex_stage() {
        let mut test = launch_test(app_shell_harness);
        test.sync_and_update();
        assert!(has_label(&test, "OFF"), "cards must not be clipped off-screen");
        assert!(has_label(&test, "VHS"));
        let dock = test
            .find(|node, element| {
                Rect::try_downcast(element)
                    .filter(|_| (node.layout().area.size.height - theme::DOCK_H).abs() < 1.0)
                    .map(|_| node)
            })
            .expect("dock should keep its 88px height");
        assert!(
            dock.layout().area.size.width > 200.0,
            "dock should span the window, not collapse"
        );
    }

    #[test]
    fn dock_shows_first_three_cards() {
        let mut test = launch_test(dock_harness);
        test.sync_and_update();
        assert!(has_label(&test, "OFF"));
        assert!(has_label(&test, "clean camera"));
        assert!(has_label(&test, "MORPH"));
        assert!(has_label(&test, "ink drawing"));
        assert!(has_label(&test, "VHS"));
        assert!(has_label(&test, "tracking · wear"));
        assert!(!has_label(&test, "GX"));
        assert!(!has_label(&test, "looks"));
    }

    #[test]
    fn click_vhs_wears_on_camera() {
        camera::set_look(Look::None);
        let mut test = TestingRunner::new(
            app,
            Size2D::new(theme::WINDOW_W, theme::WINDOW_H),
            |_| {},
            1.0,
        )
        .0;
        test.sync_and_update();
        assert!(!has_label(&test, "wet"), "OFF has no sliders");
        click_label(&mut test, "tracking · wear");
        assert_eq!(camera::current_look(), Look::Vhs);
        assert!(has_label(&test, "VHS"), "dock stays up after a click");
        assert!(has_label(&test, "wet"), "wearing VHS shows wet");
        assert!(has_label(&test, "track"));
        assert!(has_label(&test, "chroma"));
        assert!(has_label(&test, "wear"));
        let out = std::env::temp_dir().join("mirror2-vhs-fx-verify.png");
        test.render_to_file(&out);
    }

    #[test]
    fn chevron_pages_the_catalog() {
        let mut test = launch_test(dock_harness);
        test.sync_and_update();
        click_label(&mut test, ">");
        assert!(has_label(&test, "GX"));
        assert!(has_label(&test, "Hi8 · 1994"));
        assert!(!has_label(&test, "OFF"));
    }

    #[test]
    fn arrow_right_pages_the_catalog() {
        let mut test = launch_test(dock_harness);
        test.sync_and_update();
        test.press_key(Key::Named(NamedKey::ArrowRight));
        assert!(has_label(&test, "GX"));
        assert!(!has_label(&test, "OFF"));
    }

    fn shutter_harness() -> impl IntoElement {
        let shutter = use_state(|| Shutter::Idle);
        let overlay = shutter::overlay_at(shutter(), Instant::now());
        let countdown = match overlay {
            shutter::Overlay::Digit(n) => Some(n),
            _ => None,
        };
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .on_global_key_down(move |e: Event<KeyboardEventData>| {
                if shutter::is_space(&e.key, e.code) {
                    start_countdown(shutter);
                }
            })
            .child(shutter_button(
                Palette::app(),
                shutter,
                shutter(),
                countdown,
            ))
    }

    fn click_shutter(test: &mut TestingRunner) {
        let btn = test
            .find(|node, element| {
                Rect::try_downcast(element)
                    .filter(|_| {
                        (node.layout().area.size.width - theme::SHUTTER_D).abs() < 1.0
                            && (node.layout().area.size.height - theme::SHUTTER_D).abs() < 1.0
                    })
                    .map(|_| node)
            })
            .expect("56px shutter");
        let area = btn.layout().area;
        test.click_cursor((
            area.min_x() as f64 + (area.size.width as f64) * 0.5,
            area.min_y() as f64 + (area.size.height as f64) * 0.5,
        ));
    }

    #[test]
    fn shutter_click_starts_countdown_at_three() {
        let mut test = launch_test(shutter_harness);
        test.sync_and_update();
        assert!(!has_label(&test, "3"));

        click_shutter(&mut test);

        assert!(has_label(&test, "3"), "first second of the count is 3");
    }

    #[test]
    fn space_starts_countdown() {
        let mut test = launch_test(shutter_harness);
        test.sync_and_update();
        test.press_key(Key::Character(" ".into()));
        assert!(has_label(&test, "3"), "spacebar starts the 3-2-1 count");
    }

    fn all_label_boxes(test: &TestingRunner) -> Vec<(String, f32, f32, f32, f32)> {
        test.find_many(|node, element| {
            Label::try_downcast(element).map(|label| {
                let a = node.layout().area;
                (
                    label.text.as_ref().to_string(),
                    a.min_x(),
                    a.min_y(),
                    a.max_x(),
                    a.max_y(),
                )
            })
        })
    }

    fn shutter_box(test: &TestingRunner) -> (f32, f32, f32, f32) {
        let btn = test
            .find(|node, element| {
                Rect::try_downcast(element)
                    .filter(|_| {
                        (node.layout().area.size.width - theme::SHUTTER_D).abs() < 1.0
                            && (node.layout().area.size.height - theme::SHUTTER_D).abs() < 1.0
                    })
                    .map(|_| node)
            })
            .expect("56px shutter");
        let a = btn.layout().area;
        (a.min_x(), a.min_y(), a.size.width, a.size.height)
    }

    #[test]
    fn clip_status_fits_the_header_slot() {
        assert_eq!(clip_status("FaceTime HD Camera"), "FaceTime");
        assert_eq!(
            clip_status("FaceTime HD Camera (Built-in)"),
            "FaceTime"
        );
        assert_eq!(clip_status("asking camera"), "asking camera");
        let long = clip_status("camera stalled: permission denied forever");
        assert!(
            long.chars().count() <= theme::STATUS_MAX_CHARS,
            "clipped status is still too long: {long}"
        );
        assert_eq!(
            status_copy(&CameraStatus::Live {
                name: "FaceTime HD Camera".into()
            }),
            "FaceTime"
        );
    }

    #[test]
    fn layout_stone_nothing_walks_off_the_glass() {
        camera::set_status_for_test(CameraStatus::Live {
            name: "FaceTime HD Camera".into(),
        });
        const W: f32 = theme::WINDOW_W;
        const H: f32 = theme::WINDOW_H;
        let mut test = TestingRunner::new(app, Size2D::new(W, H), |_| {}, 1.0).0;
        test.sync_and_update();

        assert!(has_label(&test, "FaceTime"), "long camera name must shorten");
        assert!(
            !has_label(&test, "FaceTime HD Camera"),
            "full camera name must not appear"
        );

        let shutter = shutter_box(&test);
        let shutter_cx = shutter.0 + shutter.2 / 2.0;
        assert!(
            (shutter_cx - W / 2.0).abs() < 2.0,
            "shutter center x={shutter_cx} must sit on {mid}",
            mid = W / 2.0
        );

        let left = label_box(&test, "<").expect("left chevron");
        let right = label_box(&test, ">").expect("right chevron");
        assert!(
            left.0 >= -1.0,
            "left chevron walks off the left: {left:?}"
        );
        assert!(
            right.0 + right.2 <= W + 1.0,
            "right chevron walks off the glass: {right:?} window_w={W}"
        );
        assert!(
            right.0 >= W - theme::CHEVRON_W - 1.0,
            "right chevron must sit in the last {w}px, was x={}",
            right.0,
            w = theme::CHEVRON_W
        );

        let mut overflowed = Vec::new();
        for (text, x0, y0, x1, y1) in all_label_boxes(&test) {
            if x0 < -1.0 || x1 > W + 1.0 || y0 < -1.0 || y1 > H + 1.0 {
                overflowed.push(format!("{text:?} ({x0:.1},{y0:.1})-({x1:.1},{y1:.1})"));
            }
        }
        assert!(
            overflowed.is_empty(),
            "copy walked off the {W}×{H} glass:\n{}",
            overflowed.join("\n")
        );

        let out = std::env::temp_dir().join("mirror2-layout-stone.png");
        test.render_to_file(&out);
        assert!(out.exists(), "wrote {out:?}");
    }

    fn seed_preview(look: Look) {
        let w = 960u32;
        let h = 720u32;
        let rgb = effects::standin_rgb(w, h, 0.0);
        let (pw, ph, small) = effects::downscale_rgb(&rgb, w, h, theme::PREVIEW_MAX_W);
        let mirrored = effects::mirror_rgb(&small, pw, ph);
        camera::set_look(look);
        let rgba = effects::apply(look, &mirrored, pw, ph);
        camera::set_preview_for_test(pw, ph, rgba);
    }

    fn page_dock_to(test: &mut TestingRunner, start: usize) {
        for _ in 0..start {
            click_label(test, ">");
        }
    }

    fn render_look_shot(shots: &Path, look: Look, filename: &str) {
        camera::set_status_for_test(CameraStatus::Live {
            name: "FaceTime HD Camera".into(),
        });
        seed_preview(look);

        let idx = Look::RAIL.iter().position(|&l| l == look).unwrap();
        let dock_start = idx.saturating_sub(1).min(dock_max());

        let mut test =
            TestingRunner::new(app, Size2D::new(theme::WINDOW_W, theme::WINDOW_H), |_| {}, 1.0).0;
        test.sync_and_update();
        page_dock_to(&mut test, dock_start);
        test.sync_and_update();
        click_label(&mut test, look.tile_line());
        test.sync_and_update();

        let path = shots.join(filename);
        test.render_to_file(&path);
        assert!(path.exists(), "wrote {path:?}");
    }

    fn render_dock_page(shots: &Path, start: usize, filename: &str) {
        camera::set_status_for_test(CameraStatus::Live {
            name: "FaceTime HD Camera".into(),
        });
        seed_preview(Look::None);

        let mut test =
            TestingRunner::new(app, Size2D::new(theme::WINDOW_W, theme::WINDOW_H), |_| {}, 1.0).0;
        test.sync_and_update();
        page_dock_to(&mut test, start);
        test.sync_and_update();

        let path = shots.join(filename);
        test.render_to_file(&path);
        assert!(path.exists(), "wrote {path:?}");
    }

    #[test]
    fn export_readme_screenshots() {
        let shots = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/screenshots");
        std::fs::create_dir_all(&shots).expect("docs/screenshots");

        render_look_shot(&shots, Look::None, "mirror2-off.png");
        // mirror2-vhs.png is a live camera capture — keep it out of the test harness.
        render_look_shot(&shots, Look::D8, "mirror2-d8.png");
        render_look_shot(&shots, Look::Sat, "mirror2-sat.png");
        render_look_shot(&shots, Look::Cctv, "mirror2-cctv.png");
        render_look_shot(&shots, Look::Ripple, "mirror2-ripple.png");
        render_dock_page(&shots, 4, "mirror2-dock-tape.png");
    }
}
