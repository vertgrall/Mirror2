//! Live camera on a background thread. Falls back to a drawn witness.
//!
//! Preview path: sensor → downscale → mirror → VFX → UI (~640px wide).
//! Capture path: latest full-res RGB kept for shutter saves only.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::effects::{self, Look};
use crate::theme::PREVIEW_MAX_W;

#[derive(Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
    pub seq: u64,
}

#[derive(Clone, Debug)]
pub enum CameraStatus {
    Starting,
    Live { name: String },
    StandIn { reason: String },
}

static LOOK: AtomicU8 = AtomicU8::new(0);
static FRAME_SEQ: AtomicU64 = AtomicU64::new(0);
static RUNNING: AtomicBool = AtomicBool::new(false);
static FRAME: OnceLock<Mutex<Option<Frame>>> = OnceLock::new();
static STATUS: OnceLock<Mutex<CameraStatus>> = OnceLock::new();
static FULL_RGB: OnceLock<Mutex<Option<(u32, u32, Vec<u8>)>>> = OnceLock::new();

fn frame_lock() -> &'static Mutex<Option<Frame>> {
    FRAME.get_or_init(|| Mutex::new(None))
}

fn status_lock() -> &'static Mutex<CameraStatus> {
    STATUS.get_or_init(|| Mutex::new(CameraStatus::Starting))
}

fn full_lock() -> &'static Mutex<Option<(u32, u32, Vec<u8>)>> {
    FULL_RGB.get_or_init(|| Mutex::new(None))
}

pub fn set_look(look: Look) {
    crate::debug_log!("camera: look set to {:?}", look);
    LOOK.store(look.id(), Ordering::Relaxed);
    effects::reset_temporal();
}

pub fn current_look() -> Look {
    Look::from_id(LOOK.load(Ordering::Relaxed))
}

/// Re-run VFX on the latest camera frame with current look + params.
/// Call from UI when controls change so preview updates immediately.
pub fn refresh_preview() {
    let Ok(guard) = full_lock().lock() else {
        return;
    };
    let Some((w, h, rgb)) = guard.as_ref() else {
        return;
    };
    let (pw, ph, small) = effects::downscale_rgb(rgb, *w, *h, PREVIEW_MAX_W);
    let mirrored = effects::mirror_rgb(&small, pw, ph);
    let look = Look::from_id(LOOK.load(Ordering::Relaxed));
    let rgba = effects::apply(look, &mirrored, pw, ph);
    publish(pw, ph, rgba);
}

/// Latest preview frame for live view. Non-blocking — returns None if camera holds the lock.
pub fn current_frame() -> Option<Frame> {
    frame_lock().try_lock().ok()?.clone()
}

/// Full-res frame with current look applied — for shutter saves only.
pub fn snapshot_for_keep() -> Option<Frame> {
    let (w, h, mirrored) = {
        let guard = full_lock().try_lock().ok()?;
        let (w, h, rgb) = guard.as_ref()?;
        (*w, *h, effects::mirror_rgb(rgb, *w, *h))
    };
    let look = Look::from_id(LOOK.load(Ordering::Relaxed));
    let rgba = effects::apply(look, &mirrored, w, h);
    let seq = frame_lock()
        .try_lock()
        .ok()
        .and_then(|g| g.as_ref().map(|f| f.seq))
        .unwrap_or(0);
    Some(Frame {
        width: w,
        height: h,
        rgba: rgba.into(),
        seq,
    })
}

pub fn status() -> CameraStatus {
    status_lock()
        .lock()
        .map(|g| g.clone())
        .unwrap_or(CameraStatus::Starting)
}

#[cfg(test)]
pub fn set_status_for_test(next: CameraStatus) {
    set_status(next);
}

#[cfg(test)]
pub fn set_preview_for_test(width: u32, height: u32, rgba: Vec<u8>) {
    publish(width, height, rgba);
}

pub fn start() {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    thread::Builder::new()
        .name("mirror2-camera".into())
        .spawn(camera_loop)
        .expect("spawn camera thread");
}

fn store_full(w: u32, h: u32, rgb: Vec<u8>) {
    if let Ok(mut slot) = full_lock().lock() {
        *slot = Some((w, h, rgb));
    }
}

fn publish(width: u32, height: u32, rgba: Vec<u8>) {
    let seq = FRAME_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    if let Ok(mut slot) = frame_lock().lock() {
        *slot = Some(Frame {
            width,
            height,
            rgba: rgba.into(),
            seq,
        });
    }
}

fn set_status(next: CameraStatus) {
    if let Ok(mut slot) = status_lock().lock() {
        *slot = next;
    }
}

struct PipeStats {
    frames: u64,
    dropped: u64,
    total_ms: f64,
    last_log: Instant,
}

impl PipeStats {
    fn record(&mut self, dropped: u64, ms: f64) {
        self.frames += 1;
        self.dropped += dropped;
        self.total_ms += ms;
        if self.last_log.elapsed() >= Duration::from_secs(3) {
            let avg = if self.frames > 0 {
                self.total_ms / self.frames as f64
            } else {
                0.0
            };
            eprintln!(
                "mirror2: preview pipe  {:.1} ms/frame avg  {} fps  {} dropped",
                avg,
                self.frames / 3,
                self.dropped,
            );
            self.frames = 0;
            self.dropped = 0;
            self.total_ms = 0.0;
            self.last_log = Instant::now();
        }
    }
}

fn process_preview(w: u32, h: u32, rgb: Vec<u8>, stats: &mut PipeStats, dropped: u64) {
    let t0 = Instant::now();
    let (pw, ph, small) = effects::downscale_rgb(&rgb, w, h, PREVIEW_MAX_W);
    store_full(w, h, rgb);
    let mirrored = effects::mirror_rgb(&small, pw, ph);
    let look = Look::from_id(LOOK.load(Ordering::Relaxed));
    let rgba = effects::apply(look, &mirrored, pw, ph);
    publish(pw, ph, rgba);
    stats.record(dropped, t0.elapsed().as_secs_f64() * 1000.0);
}

fn camera_loop() {
    #[cfg(target_os = "macos")]
    {
        match crate::macos_avf::open() {
            Ok(camera) => {
                let name = camera.name.clone();
                let mut live = false;
                let mut stats = PipeStats {
                    frames: 0,
                    dropped: 0,
                    total_ms: 0.0,
                    last_log: Instant::now(),
                };
                loop {
                    match camera.recv_frame(Duration::from_secs(2)) {
                        Ok((mut w, mut h, mut rgb)) => {
                            let mut dropped = 0u64;
                            while let Some((w2, h2, rgb2)) = camera.try_recv_frame() {
                                w = w2;
                                h = h2;
                                rgb = rgb2;
                                dropped += 1;
                            }
                            process_preview(w, h, rgb, &mut stats, dropped);
                            if !live {
                                live = true;
                                set_status(CameraStatus::Live { name: name.clone() });
                                eprintln!("mirror2: first frame {w}×{h} → preview {PREVIEW_MAX_W}px wide");
                            }
                        }
                        Err(err) if err == "timeout" => {
                            if !live {
                                set_status(CameraStatus::Starting);
                            }
                        }
                        Err(err) => {
                            set_status(CameraStatus::StandIn {
                                reason: format!("camera stalled: {err}"),
                            });
                            standin_loop();
                            return;
                        }
                    }
                }
            }
            Err(reason) => {
                eprintln!("mirror2: {reason}");
                set_status(CameraStatus::StandIn { reason });
                standin_loop();
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        set_status(CameraStatus::StandIn {
            reason: "camera is macOS-only for now".into(),
        });
        standin_loop();
    }
}

fn standin_loop() {
    let w = 960u32;
    let h = 720u32;
    let start = Instant::now();
    let mut stats = PipeStats {
        frames: 0,
        dropped: 0,
        total_ms: 0.0,
        last_log: Instant::now(),
    };
    loop {
        let t = start.elapsed().as_secs_f32();
        let rgb = effects::standin_rgb(w, h, t);
        process_preview(w, h, rgb, &mut stats, 0);
        thread::sleep(Duration::from_millis(33));
    }
}
