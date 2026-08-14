//! Background plates — built-in presets, user folder, search cache.

use std::fs;
use std::path::{Path, PathBuf};

use image::ImageReader;

pub fn backgrounds_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Pictures/Mirror2/Backgrounds")
}

pub fn ensure_dir() {
    let _ = fs::create_dir_all(backgrounds_dir());
}

/// Curated outsider presets (procedural built-ins).
pub const PRESETS: &[(&str, &str)] = &[
    ("void", "VOID"),
    ("sky", "SKY"),
    ("concrete", "CONC"),
    ("mall", "MALL"),
    ("church", "CHURCH"),
    ("parking", "PARK"),
    ("bedroom", "90S"),
    ("corridor", "HALL"),
];

const BUILTIN: &[&str] = &[
    "__builtin_void",
    "__builtin_sky",
    "__builtin_concrete",
    "__builtin_mall",
    "__builtin_church",
    "__builtin_parking",
    "__builtin_bedroom",
    "__builtin_corridor",
];

pub fn builtin_path(name: &str) -> PathBuf {
    PathBuf::from(format!("__builtin_{name}"))
}

/// List PNG/JPEG paths: built-in names first, then user folder.
pub fn list_paths() -> Vec<PathBuf> {
    ensure_dir();
    let mut paths = Vec::new();
    for name in BUILTIN {
        paths.push(PathBuf::from(*name));
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

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg"))
        .unwrap_or(false)
}

pub fn label(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.starts_with("__builtin_") {
        let name = s.trim_start_matches("__builtin_");
        return PRESETS
            .iter()
            .find(|(id, _)| *id == name)
            .map(|(_, lbl)| (*lbl).to_string())
            .unwrap_or_else(|| name.to_uppercase());
    }
    if s.contains("pexels-") {
        return path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("stock")
            .to_uppercase();
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
    cover_rgb(&img.into_raw(), sw, sh, tw, th)
}

fn cover_rgb(raw: &[u8], sw: u32, sh: u32, tw: u32, th: u32) -> Option<Vec<u8>> {
    let sw = sw.max(1);
    let sh = sh.max(1);
    let mut out = vec![0u8; (tw as usize) * (th as usize) * 3];
    let scale = (tw as f32 / sw as f32).max(th as f32 / sh as f32);
    let dw = (sw as f32 * scale) as u32;
    let dh = (sh as f32 * scale) as u32;
    let ox = (dw.saturating_sub(tw)) / 2;
    let oy = (dh.saturating_sub(th)) / 2;
    for y in 0..th {
        for x in 0..tw {
            let sx = (x + ox).min(dw.saturating_sub(1)) * sw / dw.max(1);
            let sy = (y + oy).min(dh.saturating_sub(1)) * sh / dh.max(1);
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
            let u = x as f32 / tw as f32;
            let t = y as f32 / th as f32;
            let (r, g, b) = if name.contains("void") {
                (12, 12, 14)
            } else if name.contains("sky") {
                (
                    (40.0 + u * 40.0) as u8,
                    (90.0 + t * 80.0) as u8,
                    (160.0 + t * 60.0) as u8,
                )
            } else if name.contains("concrete") {
                let n = (hash(x, y) % 18) as u8;
                (
                    90u8.saturating_add(n),
                    88u8.saturating_add(n),
                    84u8.saturating_add(n),
                )
            } else if name.contains("mall") {
                // Liminal teal tile + sodium band
                let tile = ((x / 24) + (y / 24)) % 2;
                let band: u32 = if t > 0.12 && t < 0.18 { 40 } else { 0 };
                (
                    (48 + tile * 8 + band) as u8,
                    (118 + tile * 6 + band) as u8,
                    (108 + tile * 4 + band / 2) as u8,
                )
            } else if name.contains("church") {
                // Dark nave + amber window wash
                let beam = (0.55 - (u - 0.5).abs() * 1.4).max(0.0);
                (
                    (18.0 + beam * 90.0 + t * 12.0) as u8,
                    (14.0 + beam * 55.0 + t * 8.0) as u8,
                    (22.0 + beam * 20.0 + t * 6.0) as u8,
                )
            } else if name.contains("parking") {
                // Asphalt + sodium pools
                let pool = ((u * 3.0).sin() * (t * 2.0).cos() * 0.5 + 0.5).max(0.0);
                (
                    (22.0 + pool * 55.0) as u8,
                    (20.0 + pool * 28.0) as u8,
                    (18.0 + pool * 12.0) as u8,
                )
            } else if name.contains("bedroom") {
                // Warm peach + poster blocks
                let poster = u > 0.2 && u < 0.55 && t > 0.25 && t < 0.65;
                if poster {
                    (
                        (120.0 + (hash(x, y) % 40) as f32) as u8,
                        (70.0 + (hash(x.wrapping_add(3), y) % 30) as f32) as u8,
                        (90.0 + (hash(x, y.wrapping_add(2)) % 35) as f32) as u8,
                    )
                } else {
                    (228, 196, 168)
                }
            } else if name.contains("corridor") {
                // Vanishing hotel hall — doors rhythm
                let depth = 1.0 - t * 0.85;
                let door = (x as f32 / (tw as f32 / 5.0)).fract() < 0.12 && t > 0.2;
                let base = (34.0 * depth) as u8;
                if door {
                    (
                        base.saturating_add(18),
                        base.saturating_add(14),
                        base.saturating_add(10),
                    )
                } else {
                    (
                        base.saturating_add(8),
                        base.saturating_add(12),
                        base.saturating_add(10),
                    )
                }
            } else {
                let n = (hash(x, y) % 18) as u8;
                (
                    90u8.saturating_add(n),
                    88u8.saturating_add(n),
                    84u8.saturating_add(n),
                )
            };
            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
        }
    }
    out
}

fn hash(x: u32, y: u32) -> u32 {
    let mut n = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263));
    n ^= n >> 13;
    n.wrapping_mul(1274126177)
}

pub fn reveal_folder() {
    ensure_dir();
    let _ = std::process::Command::new("open")
        .arg(backgrounds_dir())
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_builtins() {
        assert_eq!(PRESETS.len(), BUILTIN.len());
    }

    #[test]
    fn builtin_mall_differs_from_void() {
        let mall = builtin_rgb(&builtin_path("mall"), 32, 24);
        let void = builtin_rgb(&builtin_path("void"), 32, 24);
        assert_ne!(mall, void);
    }

    #[test]
    fn builtin_at_preview_res_does_not_panic() {
        // Full-res plate load uses sensor/preview dimensions — hash must not overflow in debug.
        for name in ["mall", "church", "corridor", "bedroom", "parking"] {
            let rgb = builtin_rgb(&builtin_path(name), 640, 480);
            assert_eq!(rgb.len(), 640 * 480 * 3, "{name}");
        }
    }
}
