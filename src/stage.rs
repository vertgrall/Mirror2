//! Live likeness, countdown, flash. Flat — no tape, no frame, no grain.

use std::sync::Arc;

use freya::components::CanvasContext;
use freya::engine::prelude::{
    AlphaType, ClipOp, Color4f, ColorType, Data, FilterMode, Font, ImageInfo, MipmapMode, Paint,
    Point, Rect as SkRect, SamplingOptions,
};
use skia_safe::images;
use skia_safe::utils::text_utils::Align as TextAlign;

use crate::camera::{CameraStatus, Frame};
use crate::effects::Look;

#[derive(Clone)]
pub struct StageFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}

impl From<Frame> for StageFrame {
    fn from(frame: Frame) -> Self {
        Self {
            width: frame.width,
            height: frame.height,
            rgba: frame.rgba,
        }
    }
}

pub fn draw_stage(
    ctx: &mut CanvasContext,
    frame: Option<StageFrame>,
    status: &CameraStatus,
    look: Look,
    countdown: Option<u32>,
    flash: f32,
) {
    let w = ctx.size.width.max(1.0);
    let h = ctx.size.height.max(1.0);
    let well = SkRect::from_xywh(0.0, 0.0, w, h);

    fill(ctx, w, h, Color4f::new(0.11, 0.11, 0.10, 1.0));

    ctx.canvas.save();
    ctx.canvas.clip_rect(well, ClipOp::Intersect, true);
    if let Some(frame) = frame.as_ref() {
        blit_cover(ctx, frame, well);
    } else {
        draw_waiting(ctx, well, status);
    }
    ctx.canvas.restore();

    if let Some(n) = countdown {
        draw_countdown(ctx, w, h, n);
    }
    if flash > 0.01 {
        fill(ctx, w, h, Color4f::new(1.0, 1.0, 1.0, flash));
    }

    draw_look_caption(ctx, w, h, look, status);
}

fn fill(ctx: &mut CanvasContext, w: f32, h: f32, color: Color4f) {
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    paint.set_color4f(color, None);
    ctx.canvas
        .draw_rect(SkRect::from_xywh(0.0, 0.0, w, h), &paint);
}

fn blit_cover(ctx: &mut CanvasContext, frame: &StageFrame, well: SkRect) {
    let info = ImageInfo::new(
        (frame.width as i32, frame.height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let Some(image) = images::raster_from_data(
        &info,
        Data::new_copy(frame.rgba.as_ref()),
        frame.width as usize * 4,
    ) else {
        return;
    };
    let iw = frame.width as f32;
    let ih = frame.height as f32;
    let scale = (well.width() / iw).max(well.height() / ih);
    let dw = iw * scale;
    let dh = ih * scale;
    let dx = well.left() + (well.width() - dw) * 0.5;
    let dy = well.top() + (well.height() - dh) * 0.5;
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear);
    ctx.canvas.draw_image_rect_with_sampling_options(
        image,
        None,
        SkRect::from_xywh(dx, dy, dw, dh),
        sampling,
        &paint,
    );
}

fn draw_waiting(ctx: &mut CanvasContext, well: SkRect, status: &CameraStatus) {
    let msg = match status {
        CameraStatus::Starting => "asking the camera…",
        CameraStatus::Live { .. } => "waiting for a frame…",
        CameraStatus::StandIn { reason } => reason.as_str(),
    };
    let mut font = Font::default();
    font.set_size(15.0);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(Color4f::new(0.78, 0.78, 0.76, 1.0), None);
    ctx.canvas.draw_str_align(
        msg,
        Point::new(well.center_x(), well.center_y()),
        &font,
        &paint,
        TextAlign::Center,
    );
}

fn draw_countdown(ctx: &mut CanvasContext, w: f32, h: f32, n: u32) {
    let mut font = Font::default();
    font.set_size(160.0);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
    ctx.canvas.draw_str_align(
        n.to_string(),
        Point::new(w * 0.5, h * 0.55),
        &font,
        &paint,
        TextAlign::Center,
    );
}

fn draw_look_caption(ctx: &mut CanvasContext, w: f32, h: f32, look: Look, status: &CameraStatus) {
    let mut font = Font::default();
    font.set_size(11.0);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(Color4f::new(1.0, 1.0, 1.0, 0.85), None);
    let label = match status {
        CameraStatus::Live { name } => format!("{}  ·  {}", look.hint(), name),
        CameraStatus::StandIn { .. } => format!("{}  ·  stand-in", look.hint()),
        CameraStatus::Starting => "asking the camera…".into(),
    };
    ctx.canvas.draw_str_align(
        label,
        Point::new(w * 0.5, h - 16.0),
        &font,
        &paint,
        TextAlign::Center,
    );
}

pub fn draw_thumb(ctx: &mut CanvasContext, width: u32, height: u32, rgba: &[u8]) {
    let w = ctx.size.width.max(1.0);
    let h = ctx.size.height.max(1.0);
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let Some(image) = images::raster_from_data(&info, Data::new_copy(rgba), width as usize * 4)
    else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::None);
    ctx.canvas.draw_image_rect_with_sampling_options(
        image,
        None,
        SkRect::from_xywh(0.0, 0.0, w, h),
        sampling,
        &paint,
    );
}
