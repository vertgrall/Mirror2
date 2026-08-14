//! Launch the small About window from the App menu or header.

use std::cell::Cell;
use std::sync::Mutex;

use freya::prelude::*;
use freya::winit::window::WindowId;

use crate::about::{about_window, ABOUT_WINDOW_H, ABOUT_WINDOW_W};
use crate::theme::Theme;

type RendererDispatch = Box<dyn Fn(Box<dyn FnOnce(&mut RendererContext)>) + Send + Sync>;

static RENDERER_DISPATCH: Mutex<Option<RendererDispatch>> = Mutex::new(None);

thread_local! {
    static ABOUT_WINDOW_ID: Cell<Option<WindowId>> = const { Cell::new(None) };
}

pub fn set_renderer_dispatch(dispatch: RendererDispatch) {
    let mut slot = RENDERER_DISPATCH.lock().expect("renderer dispatch lock");
    if slot.is_none() {
        *slot = Some(dispatch);
    }
}

fn post_to_renderer(f: impl FnOnce(&mut RendererContext) + 'static) {
    if let Ok(guard) = RENDERER_DISPATCH.lock() {
        if let Some(dispatch) = guard.as_ref() {
            dispatch(Box::new(f));
        }
    }
}

/// Open (or focus) the About window — safe from App menu or header click.
pub fn request_about_window() {
    post_to_renderer(launch_about_window);
}

fn launch_about_window(ctx: &mut RendererContext) {
    crate::about_assets::preload();

    if let Some(id) = ABOUT_WINDOW_ID.get() {
        if let Some(app) = ctx.windows.get_mut(&id) {
            app.window().set_visible(true);
            app.window().focus_window();
            return;
        }
        ABOUT_WINDOW_ID.set(None);
    }

    let palette = Theme::Dark.palette();
    let icon = LaunchConfig::window_icon(include_bytes!("../resources/icon-window.png"));
    let id = ctx.launch_window(
        WindowConfig::new(about_window)
            .with_title("About Mirror2")
            .with_size(ABOUT_WINDOW_W as f64, ABOUT_WINDOW_H as f64)
            .with_max_size(ABOUT_WINDOW_W as f64, ABOUT_WINDOW_H as f64)
            .with_resizable(false)
            .with_background(palette.bg)
            .with_icon(icon)
            .with_on_close(|_ctx, closed_id| {
                ABOUT_WINDOW_ID.with(|slot| {
                    if slot.get() == Some(closed_id) {
                        slot.set(None);
                    }
                });
                CloseDecision::Close
            }),
    );
    ABOUT_WINDOW_ID.set(Some(id));

    if let Some(app) = ctx.windows.get_mut(&id) {
        app.window().set_visible(true);
        app.window().focus_window();
    }
}

#[cfg(test)]
pub fn set_renderer_dispatch_for_test(dispatch: RendererDispatch) {
    *RENDERER_DISPATCH.lock().expect("renderer dispatch lock") = Some(dispatch);
}
