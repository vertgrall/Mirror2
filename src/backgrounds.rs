//! Background plates — built-in + user folder.

use std::fs;
use std::path::{Path, PathBuf};

use image::ImageReader;

pub fn backgrounds_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Pictures/Likeness/Backgrounds")
}

pub fn ensure_dir() {
    let _ = fs::create_dir_all(backgrounds_dir());
}

/// List PNG/JPEG paths: built-in names first, then user folder.
pub fn list_paths() -> Vec<PathBuf> {
    ensure_dir();
    let mut paths = Vec::new();
    for name in BUILTIN {
        paths.push(PathBuf::from(name));
    }
    if let Ok(read) = fs::read_dir(backgrounds_dir()) {
        let mut user: Vec<_> = read
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| is_image(p))
            .collect();
        user.sort();
        paths.extend(user);
    }
    paths
}

const BUILTIN: &[&str] = &["__builtin_void", "__builtin_sky", "__builtin_concrete"];

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg"))
        .unwrap_or(false)
}

pub fn label(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.starts_with("__builtin_") {
        return s.trim_start_matches("__builtin_").to_uppercase();
    }
    path.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("background")
        .to_string()
}

/// Load RGB (3 bytes/px) cover-scaled to target size.
pub fn load_rgb(path: &Path, tw: u32, th: u32) -> Option<Vec<u8>> {
    if path.to_string_lossy().contains("__builtin_") {
        return Some(builtin_rgb(path, tw, th));
    }
    let img = ImageReader::open(path).ok()?.decode().ok()?.to_rgb8();
    let sw = img.width().max(1);
    let sh = img.height().max(1);
    let mut out = vec![0u8; (tw as usize) * (th as usize) * 3];
    let scale = (tw as f32 / sw as f32).max(th as f32 / sh as f32);
    let dw = (sw as f32 * scale) as u32;
    let dh = (sh as f32 * scale) as u32;
    let ox = (dw.saturating_sub(tw)) / 2;
    let oy = (dh.saturating_sub(th)) / 2;
    let raw = img.into_raw();
    for y in 0..th {
        for x in 0..tw {
            let sx = (x + ox).min(dw - 1) * sw / dw.max(1);
            let sy = (y + oy).min(dh - 1) * sh / dh.max(1);
            let si = ((sy * sw + sx) as usize) * 3;
            let di = ((y * tw + x) as usize) * 3;
            out[di..di + 3].copy_from_slice(&raw[si..si + 3]);
        }
    }
    Some(out)
}

fn builtin_rgb(path: &Path, tw: u32, th: u32) -> Vec<u8> {
    let name = path.to_string_lossy();
    let mut out = vec![0u8; (tw as usize) * (th as usize) * 3];
    for y in 0..th {
        for x in 0..tw {
            let i = ((y * tw + x) as usize) * 3;
            let t = y as f32 / th as f32;
            let (r, g, b) = if name.contains("void") {
                (12, 12, 14)
            } else if name.contains("sky") {
                let u = x as f32 / tw as f32;
                (
                    (40.0 + u * 40.0) as u8,
                    (90.0 + t * 80.0) as u8,
                    (160.0 + t * 60.0) as u8,
                )
            } else {
                let n = (hash(x, y) % 18) as u8;
                (90 + n, 88 + n, 84 + n)
            };
            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
        }
    }
    out
}

fn hash(x: u32, y: u32) -> u32 {
    let mut n = x.wrapping_mul(374761393) + y.wrapping_mul(668265263);
    n ^= n >> 13;
    n.wrapping_mul(1274126177)
}

pub fn reveal_folder() {
    ensure_dir();
    let _ = std::process::Command::new("open")
        .arg(backgrounds_dir())
        .spawn();
}
