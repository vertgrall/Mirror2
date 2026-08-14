//! Keep a still. Files land in ~/Pictures/Mirror2.

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::effects;

#[derive(Clone)]
pub struct KeepShot {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub path: PathBuf,
}

pub fn keep_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Pictures/Mirror2")
}

pub fn save_keep(id: u64, width: u32, height: u32, rgba: &[u8]) -> Result<KeepShot, String> {
    let dir = keep_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("could not make keep folder: {e}"))?;
    let stamp = chrono::Local::now().format("%Y-%m-%d-%H%M%S");
    let path = dir.join(format!("mirror2-{stamp}-{id}.png"));
    write_png(&path, width, height, rgba)?;
    let (tw, th, thumb) = effects::downscale_rgba(rgba, width, height, 140);
    Ok(KeepShot {
        id,
        width: tw,
        height: th,
        rgba: thumb,
        path,
    })
}

fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    let file =
        File::create(path).map_err(|e| format!("could not write {}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|e| format!("png header: {e}"))?;
    writer
        .write_image_data(rgba)
        .map_err(|e| format!("png data: {e}"))?;
    Ok(())
}

pub fn reveal(path: &Path) {
    let _ = Command::new("open").arg(path).spawn();
}

pub fn reveal_folder() {
    let _ = Command::new("open").arg(keep_dir()).spawn();
}
