//! Embedded About PNGs decoded once at first use.

use std::sync::LazyLock;

use bytes::Bytes;
use freya::elements::image::ImageHandle;
use freya::engine::prelude::{SkData, SkImage};

pub const SPLASH_BYTES: &[u8] = include_bytes!("../resources/brand/SplashTowerVillage.png");
pub const BRAND_BYTES: &[u8] = include_bytes!("../resources/brand/NewTowerBrandMark.png");

pub static SPLASH: LazyLock<ImageHandle> =
    LazyLock::new(|| decode_png("SplashTowerVillage", SPLASH_BYTES));
pub static BRAND: LazyLock<ImageHandle> =
    LazyLock::new(|| decode_png("NewTowerBrandMark", BRAND_BYTES));

pub fn preload() {
    let _ = (&*SPLASH, &*BRAND);
}

fn decode_png(label: &str, bytes: &'static [u8]) -> ImageHandle {
    let image = SkImage::from_encoded(unsafe { SkData::new_bytes(bytes) })
        .and_then(|img| img.make_raster_image(None, None))
        .unwrap_or_else(|| {
            panic!(
                "failed to decode {label} ({bytes_len} bytes)",
                bytes_len = bytes.len()
            )
        });
    ImageHandle::new(image, Bytes::from_static(bytes))
}

#[cfg(test)]
mod tests {
    use super::{BRAND_BYTES, SPLASH_BYTES};

    #[test]
    fn embedded_bytes_are_valid_pngs() {
        for (name, bytes) in [("splash", SPLASH_BYTES), ("brand", BRAND_BYTES)] {
            assert!(bytes.len() > 8, "{name}: asset too small");
            assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "{name}: not a PNG");
        }
    }
}
