//! Live camera on a background thread. Falls back to a drawn witness.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::effects::{self, Look};

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
static RUNNING: AtomicBool = AtomicBool::new(false);
static FRAME: OnceLock<Mutex<Option<Frame>>> = OnceLock::new();
static STATUS: OnceLock<Mutex<CameraStatus>> = OnceLock::new();

fn frame_lock() -> &'static Mutex<Option<Frame>> {
    FRAME.get_or_init(|| Mutex::new(None))
}

fn status_lock() -> &'static Mutex<CameraStatus> {
    STATUS.get_or_init(|| Mutex::new(CameraStatus::Starting))
}

pub fn set_look(look: Look) {
    LOOK.store(look.id(), Ordering::Relaxed);
}

pub fn current_frame() -> Option<Frame> {
    frame_lock().lock().ok()?.clone()
}

pub fn status() -> CameraStatus {
    status_lock()
        .lock()
        .map(|g| g.clone())
        .unwrap_or(CameraStatus::Starting)
}

pub fn start() {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    thread::Builder::new()
        .name("likeness-camera".into())
        .spawn(camera_loop)
        .expect("spawn camera thread");
}

fn publish(width: u32, height: u32, rgba: Vec<u8>, seq: &mut u64) {
    *seq += 1;
    if let Ok(mut slot) = frame_lock().lock() {
        *slot = Some(Frame {
            width,
            height,
            rgba: rgba.into(),
            seq: *seq,
        });
    }
}

fn set_status(next: CameraStatus) {
    if let Ok(mut slot) = status_lock().lock() {
        *slot = next;
    }
}

fn camera_loop() {
    match open_live_camera() {
        Ok((mut camera, name)) => {
            set_status(CameraStatus::Live { name });
            let mut seq = 0u64;
            loop {
                match camera.frame() {
                    Ok(buffer) => {
                        if let Ok(decoded) =
                            buffer.decode_image::<nokhwa::pixel_format::RgbFormat>()
                        {
                            let w = decoded.width();
                            let h = decoded.height();
                            let mirrored = effects::mirror_rgb(decoded.as_raw(), w, h);
                            let look = Look::from_id(LOOK.load(Ordering::Relaxed));
                            let rgba = effects::apply(look, &mirrored, w, h);
                            publish(w, h, rgba, &mut seq);
                        }
                    }
                    Err(err) => {
                        set_status(CameraStatus::StandIn {
                            reason: format!("camera stalled: {err}"),
                        });
                        standin_loop(&mut seq);
                        return;
                    }
                }
            }
        }
        Err(reason) => {
            set_status(CameraStatus::StandIn { reason });
            let mut seq = 0u64;
            standin_loop(&mut seq);
        }
    }
}

fn standin_loop(seq: &mut u64) {
    let w = 960u32;
    let h = 720u32;
    let start = Instant::now();
    loop {
        let t = start.elapsed().as_secs_f32();
        let rgb = effects::standin_rgb(w, h, t);
        let look = Look::from_id(LOOK.load(Ordering::Relaxed));
        let rgba = effects::apply(look, &rgb, w, h);
        publish(w, h, rgba, seq);
        thread::sleep(Duration::from_millis(33));
    }
}

fn open_live_camera() -> Result<(nokhwa::Camera, String), String> {
    #[cfg(target_os = "macos")]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        nokhwa::nokhwa_initialize(move |granted| {
            let _ = tx.send(granted);
        });
        match rx.recv_timeout(Duration::from_secs(90)) {
            Ok(true) => {}
            Ok(false) => {
                return Err("camera permission was declined".into());
            }
            Err(_) => {
                return Err("camera permission timed out".into());
            }
        }
    }

    let devices = nokhwa::query(nokhwa::utils::ApiBackend::Auto)
        .map_err(|e| format!("could not list cameras: {e}"))?;
    if devices.is_empty() {
        return Err("no camera found".into());
    }

    let preferred = devices
        .iter()
        .find(|d| {
            let n = d.human_name().to_lowercase();
            n.contains("facetime") || n.contains("built-in") || n.contains("continuity")
        })
        .unwrap_or(&devices[0]);

    let index = preferred.index().clone();
    let name = preferred.human_name().to_string();

    let requested = nokhwa::utils::RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(
        nokhwa::utils::RequestedFormatType::Closest(nokhwa::utils::CameraFormat::new(
            nokhwa::utils::Resolution::new(1280, 720),
            nokhwa::utils::FrameFormat::MJPEG,
            30,
        )),
    );

    let mut camera = nokhwa::Camera::new(index.clone(), requested)
        .or_else(|_| {
            let fallback = nokhwa::utils::RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(
                nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate,
            );
            nokhwa::Camera::new(index, fallback)
        })
        .map_err(|e| format!("could not open camera: {e}"))?;

    camera
        .open_stream()
        .map_err(|e| format!("could not start camera: {e}"))?;
    Ok((camera, name))
}
