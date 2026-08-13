//! Likeness — outsider spiritual successor to Photo Booth.
//! No curtains. No booth. No fake materials. You, kept.

mod backgrounds;
mod camera;
mod effects;
mod keep;
#[cfg(target_os = "macos")]
mod macos_avf;
mod stage;
mod theme;
mod vfx;

use std::time::{Duration, Instant};

use async_io::Timer;
use freya::prelude::*;
use keyboard_types::{Code, Key};

use camera::CameraStatus;
use effects::{
    atmo_param_defs, bg_param_defs, bg_params_from_values, cycle_background, current_path,
    set_atmosphere, set_background, set_params, AtmosphereParams, BackgroundParams, Look,
    LookGroup, LookParams, ParamDef,
};
use backgrounds::label as bg_label;
use keep::KeepShot;
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
                    .with_title("Likeness")
                    .with_size(560., 860.)
                    .with_min_size(520., 820.)
                    .with_background(palette.bg),
            ),
    );
}

#[derive(Clone, Copy, PartialEq)]
enum KitTab {
    Look,
    Bg,
    Haze,
}

#[derive(Clone, Copy, PartialEq)]
enum Shutter {
    Idle,
    Counting { started: Instant },
    Flash { started: Instant },
}

fn app() -> Element {
    let palette = Palette::app();
    let look = use_state(|| Look::None);
    let kit_open = use_state(|| false);
    let kit_tab = use_state(|| KitTab::Look);
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
    let keeps = use_state(Vec::<KeepShot>::new);
    let keep_error = use_state(|| None::<String>);
    let next_id = use_state(|| 1u64);
    let frame_seq = use_state(|| 0u64);
    let controls_rev = use_state(|| 0u32);

    backgrounds::ensure_dir();

    use_future(move || {
        let mut shutter = shutter;
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
                    Shutter::Counting { started }
                        if now.duration_since(started) >= Duration::from_secs(3) =>
                    {
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

                let after = *shutter.peek();
                if after != before {
                    needs_redraw = true;
                }
                if matches!(
                    after,
                    Shutter::Counting { .. } | Shutter::Flash { .. }
                ) {
                    needs_redraw = true;
                }

                if needs_redraw {
                    redraw(UserEvent::RequestRedraw);
                }
            }
        }
    });

    let countdown = match shutter() {
        Shutter::Counting { started } => {
            let elapsed = started.elapsed().as_secs_f32();
            Some((3 - elapsed.floor() as u32).clamp(1, 3))
        }
        _ => None,
    };
    let flash = match shutter() {
        Shutter::Flash { started } => {
            let t = started.elapsed().as_secs_f32() / 0.14;
            (1.0 - t).clamp(0.0, 1.0)
        }
        _ => 0.0,
    };

    let status = camera::status();
    let seq = *frame_seq.peek();
    let current_look = look();
    let current_tab = kit_tab();
    let kit_is_open = kit_open();
    let keeps_now = keeps.read().clone();
    let err = keep_error.read().clone();
    let shutter_now = shutter();
    let shutter_now = *shutter.peek();

    rect()
        .vertical()
        .width(Size::fill())
        .height(Size::fill())
        .overflow(Overflow::Clip)
        .background(palette.bg)
        .padding(Gaps::new_all(16.))
        .spacing(10.)
        .on_key_down(move |e: Event<KeyboardEventData>| {
            if e.key == Key::Character(" ".into()) || e.code == Code::Space {
                e.stop_propagation();
                start_countdown(shutter);
            }
        })
        .child(header(palette, &status))
        .child(stage_area(
            palette,
            look,
            kit_open,
            kit_tab,
            params,
            bg_enabled,
            bg_values,
            atmo_values,
            controls_rev,
            kit_is_open,
            current_tab,
            current_look,
            shutter,
            shutter_now,
            countdown,
            flash,
            seq,
        ))
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

fn stage_area(
    palette: Palette,
    look_state: State<Look>,
    kit_open: State<bool>,
    kit_tab: State<KitTab>,
    params: State<LookParams>,
    bg_enabled: State<bool>,
    bg_values: State<[f32; 4]>,
    atmo_values: State<[f32; 4]>,
    controls_rev: State<u32>,
    kit_is_open: bool,
    current_tab: KitTab,
    current_look: Look,
    shutter: State<Shutter>,
    shutter_now: Shutter,
    countdown: Option<u32>,
    flash: f32,
    seq: u64,
) -> Element {
    let mut col = rect()
        .vertical()
        .width(Size::fill())
        .cross_align(Alignment::Center)
        .spacing(10.)
        .child(stage_well(palette, current_look, countdown, flash, seq))
        .child(fx_toggle(palette, kit_open, kit_is_open, current_look));
    if kit_is_open {
        col = col.child(kit_tray(
            palette,
            look_state,
            kit_tab,
            params,
            bg_enabled,
            bg_values,
            atmo_values,
            controls_rev,
            current_tab,
            current_look,
        ));
    }
    col.child(shutter_button(palette, shutter, shutter_now, countdown))
        .into()
}

fn fx_toggle(
    palette: Palette,
    mut kit_open: State<bool>,
    open: bool,
    look: Look,
) -> Element {
    let label_text = if open {
        "HIDE FX"
    } else if look.is_none() {
        "FX"
    } else {
        look.label()
    };
    Chip::new()
        .selected(open)
        .on_press(move |_| {
            let next = !*kit_open.peek();
            *kit_open.write() = next;
        })
        .child(
            label()
                .text(label_text)
                .font_size(10.)
                .color(if open { palette.bg } else { palette.text }),
        )
        .into()
}

fn kit_tray(
    palette: Palette,
    look_state: State<Look>,
    kit_tab: State<KitTab>,
    params: State<LookParams>,
    bg_enabled: State<bool>,
    bg_values: State<[f32; 4]>,
    atmo_values: State<[f32; 4]>,
    controls_rev: State<u32>,
    current_tab: KitTab,
    current_look: Look,
) -> Element {
    let body = match current_tab {
        KitTab::Look => look_pane(palette, look_state, params, controls_rev, current_look),
        KitTab::Bg => background_controls(palette, bg_enabled, bg_values, controls_rev),
        KitTab::Haze => atmosphere_controls(palette, atmo_values, controls_rev),
    };
    rect()
        .vertical()
        .width(Size::fill())
        .overflow(Overflow::Clip)
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(palette.control)
        .spacing(8.)
        .child(kit_tabs(palette, kit_tab, current_tab))
        .child(body)
        .into()
}

fn kit_tabs(palette: Palette, kit_tab: State<KitTab>, current: KitTab) -> Element {
    let mut row = rect()
        .horizontal()
        .width(Size::fill())
        .spacing(6.);
    for (tab, name) in [
        (KitTab::Look, "LOOK"),
        (KitTab::Bg, "BG"),
        (KitTab::Haze, "HAZE"),
    ] {
        let selected = tab == current;
        let mut kit = kit_tab;
        row = row.child(
            Chip::new()
                .selected(selected)
                .on_press(move |_| {
                    *kit.write() = tab;
                })
                .child(
                    label()
                        .text(name)
                        .font_size(10.)
                        .color(if selected { palette.bg } else { palette.text }),
                ),
        );
    }
    row.into()
}

fn look_pane(
    palette: Palette,
    look_state: State<Look>,
    params: State<LookParams>,
    controls_rev: State<u32>,
    current: Look,
) -> Element {
    let mut menu = Select::new().selected_item(
        label()
            .text(current.label())
            .font_size(12.)
            .color(palette.text),
    );
    {
        let mut chip = look_state;
        let mut param_state = params;
        let mut rev = controls_rev;
        menu = menu.child(
            MenuItem::new()
                .selected(current.is_none())
                .on_press(move |_| {
                    *chip.write() = Look::None;
                    camera::set_look(Look::None);
                    let defaults = LookParams::defaults(Look::None);
                    param_state.set(defaults);
                    apply_controls(defaults, &mut rev);
                })
                .child(
                    label()
                        .text(Look::None.label())
                        .font_size(12.)
                        .color(palette.text),
                ),
        );
    }
    for group in [LookGroup::Process, LookGroup::Tape] {
        menu = menu.child(
            label()
                .text(group.label())
                .font_size(9.)
                .color(palette.muted),
        );
        for &look in Look::for_group(group) {
            let mut chip = look_state;
            let mut param_state = params;
            let mut rev = controls_rev;
            menu = menu.child(
                MenuItem::new()
                    .selected(look == current)
                    .on_press(move |_| {
                        *chip.write() = look;
                        camera::set_look(look);
                        let defaults = LookParams::defaults(look);
                        param_state.set(defaults);
                        apply_controls(defaults, &mut rev);
                    })
                    .child(
                        label()
                            .text(look.label())
                            .font_size(12.)
                            .color(palette.text),
                    ),
            );
        }
    }

    let mut pane = rect()
        .vertical()
        .width(Size::fill())
        .spacing(6.)
        .child(menu)
        .child(
            label()
                .text(current.hint())
                .font_size(10.)
                .color(palette.muted),
        );

    let values = params().values;
    for (index, def) in current.param_defs().iter().enumerate() {
        pane = pane.child(kit_slider(
            palette,
            *def,
            values[index],
            {
                let mut params = params;
                let mut rev = controls_rev;
                move |pct| write_param_pct(&mut params, index, *def, pct, &mut rev)
            },
        ));
    }
    pane.into()
}

fn stage_well(
    palette: Palette,
    look: Look,
    countdown: Option<u32>,
    flash: f32,
    seq: u64,
) -> Element {
    rect()
        .width(Size::px(theme::VIEWFINDER_W))
        .height(Size::px(theme::VIEWFINDER_H))
        .background(palette.surface)
        .child(
            canvas(RenderCallback::new({
                move |ctx| {
                    let live = camera::current_frame().map(StageFrame::from);
                    let status = camera::status();
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
    let pct = if span > 0.0 {
        ((value - def.min) / span * 100.0).clamp(0.0, 100.0) as f64
    } else {
        0.0
    };
    let value_label = if span >= 10.0 {
        format!("{:.0}", value)
    } else if span >= 1.0 {
        format!("{:.1}", value)
    } else {
        format!("{:.2}", value)
    };

    rect()
        .horizontal()
        .width(Size::fill())
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
            palette.accent
        } else {
            Color::from_rgb(220, 220, 216)
        })
        .on_mouse_up(on_press)
        .child(
            label()
                .text(text)
                .font_size(9.)
                .color(if selected {
                    palette.bg
                } else {
                    palette.text
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

fn pct_to_value(def: ParamDef, pct: f64) -> f32 {
    def.min + (pct as f32 / 100.0).clamp(0.0, 1.0) * (def.max - def.min)
}

fn write_param_pct(
    params: &mut State<LookParams>,
    index: usize,
    def: ParamDef,
    pct: f64,
    controls_rev: &mut State<u32>,
) {
    let v = def.from_pct(pct);
    let mut p = params();
    if !p.apply_pct(index, def, pct) {
        return;
    }
    let _ = v;
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
    let v = pct_to_value(def, pct);
    let mut p = *bg_values.peek();
    if (p[index] - v).abs() < 0.0005 {
        return;
    }
    p[index] = v;
    *bg_values.write() = p;
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
    let v = pct_to_value(def, pct);
    let mut p = *atmo_values.peek();
    if (p[index] - v).abs() < 0.0005 {
        return;
    }
    p[index] = v;
    *atmo_values.write() = p;
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
        .vertical()
        .cross_align(Alignment::Center)
        .spacing(8.)
        .child(
            rect()
                .width(Size::px(72.))
                .height(Size::px(72.))
                .corner_radius(36.)
                .background(if busy {
                    palette.shutter_pressed
                } else {
                    palette.shutter
                })
                .border(Border::new().fill(palette.bg).width(BorderWidth {
                    top: 4.,
                    right: 4.,
                    bottom: 4.,
                    left: 4.,
                }))
                .main_align(Alignment::Center)
                .cross_align(Alignment::Center)
                .on_mouse_up(move |_| start_countdown(shutter))
                .child(shutter_face(label_text)),
        )
        .child(
            label()
                .text(if busy { "…" } else { "take picture" })
                .font_size(11.)
                .color(palette.muted),
        )
        .into()
}

fn shutter_face(label_text: String) -> Element {
    if label_text.is_empty() {
        rect()
            .width(Size::px(56.))
            .height(Size::px(56.))
            .corner_radius(28.)
            .background(Color::from_rgb(255, 255, 255))
            .into()
    } else {
        label()
            .text(label_text)
            .font_size(32.)
            .font_weight(FontWeight::BOLD)
            .color(Color::WHITE)
            .into()
    }
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
                    .text("no pictures yet — tap the button or press space")
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
