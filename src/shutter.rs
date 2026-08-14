//! 3-2-1 shutter. Timing is a pure function so tests can freeze the clock.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const COUNT_SECS: u32 = 3;
pub const FLASH_MS: u64 = 140;

#[derive(Clone, Copy, PartialEq)]
pub enum Shutter {
    Idle,
    Counting { started: Instant },
    Flash { started: Instant },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Overlay {
    None,
    Digit(u32),
    Flash(f32),
}

static LIVE: OnceLock<Mutex<Shutter>> = OnceLock::new();

fn slot() -> &'static Mutex<Shutter> {
    LIVE.get_or_init(|| Mutex::new(Shutter::Idle))
}

pub fn publish(s: Shutter) {
    if let Ok(mut g) = slot().lock() {
        *g = s;
    }
}

pub fn current() -> Shutter {
    slot().lock().map(|g| *g).unwrap_or(Shutter::Idle)
}

/// Digit on screen during the count: 3, then 2, then 1.
/// `None` once the three seconds have elapsed (time to capture).
pub fn remaining_digit(started: Instant, now: Instant) -> Option<u32> {
    let elapsed = now.saturating_duration_since(started);
    if elapsed >= Duration::from_secs(COUNT_SECS as u64) {
        return None;
    }
    Some(COUNT_SECS - elapsed.as_secs() as u32)
}

pub fn flash_alpha(started: Instant, now: Instant) -> f32 {
    let t = now.saturating_duration_since(started).as_secs_f32() / (FLASH_MS as f32 / 1000.0);
    (1.0 - t).clamp(0.0, 1.0)
}

pub fn overlay_at(shutter: Shutter, now: Instant) -> Overlay {
    match shutter {
        Shutter::Idle => Overlay::None,
        Shutter::Counting { started } => match remaining_digit(started, now) {
            Some(n) => Overlay::Digit(n),
            None => Overlay::None,
        },
        Shutter::Flash { started } => Overlay::Flash(flash_alpha(started, now)),
    }
}

pub fn should_capture(started: Instant, now: Instant) -> bool {
    now.saturating_duration_since(started) >= Duration::from_secs(COUNT_SECS as u64)
}

pub fn flash_done(started: Instant, now: Instant) -> bool {
    now.saturating_duration_since(started) >= Duration::from_millis(FLASH_MS)
}

pub fn is_space(key: &keyboard_types::Key, code: keyboard_types::Code) -> bool {
    code == keyboard_types::Code::Space || *key == keyboard_types::Key::Character(" ".into())
}

pub fn is_escape(key: &keyboard_types::Key, code: keyboard_types::Code) -> bool {
    code == keyboard_types::Code::Escape
        || *key == keyboard_types::Key::Named(keyboard_types::NamedKey::Escape)
}

pub fn is_arrow_left(key: &keyboard_types::Key, code: keyboard_types::Code) -> bool {
    code == keyboard_types::Code::ArrowLeft
        || *key == keyboard_types::Key::Named(keyboard_types::NamedKey::ArrowLeft)
}

pub fn is_arrow_right(key: &keyboard_types::Key, code: keyboard_types::Code) -> bool {
    code == keyboard_types::Code::ArrowRight
        || *key == keyboard_types::Key::Named(keyboard_types::NamedKey::ArrowRight)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t0() -> Instant {
        Instant::now()
    }

    #[test]
    fn digits_are_three_two_one() {
        let start = t0();
        assert_eq!(remaining_digit(start, start), Some(3));
        assert_eq!(
            remaining_digit(start, start + Duration::from_millis(999)),
            Some(3)
        );
        assert_eq!(
            remaining_digit(start, start + Duration::from_secs(1)),
            Some(2)
        );
        assert_eq!(
            remaining_digit(start, start + Duration::from_millis(1999)),
            Some(2)
        );
        assert_eq!(
            remaining_digit(start, start + Duration::from_secs(2)),
            Some(1)
        );
        assert_eq!(
            remaining_digit(start, start + Duration::from_millis(2999)),
            Some(1)
        );
        assert_eq!(remaining_digit(start, start + Duration::from_secs(3)), None);
    }

    #[test]
    fn capture_fires_at_three_seconds() {
        let start = t0();
        assert!(!should_capture(start, start + Duration::from_millis(2999)));
        assert!(should_capture(start, start + Duration::from_secs(3)));
    }

    #[test]
    fn flash_fades_out() {
        let start = t0();
        assert!((flash_alpha(start, start) - 1.0).abs() < 0.001);
        assert!(flash_alpha(start, start + Duration::from_millis(FLASH_MS)) < 0.01);
        assert!(flash_done(start, start + Duration::from_millis(FLASH_MS)));
        assert!(!flash_done(start, start + Duration::from_millis(FLASH_MS - 1)));
    }

    #[test]
    fn overlay_matches_phase() {
        let start = t0();
        assert_eq!(
            overlay_at(Shutter::Counting { started: start }, start),
            Overlay::Digit(3)
        );
        assert_eq!(
            overlay_at(
                Shutter::Counting { started: start },
                start + Duration::from_secs(2)
            ),
            Overlay::Digit(1)
        );
        assert_eq!(overlay_at(Shutter::Idle, start), Overlay::None);
        match overlay_at(Shutter::Flash { started: start }, start) {
            Overlay::Flash(a) => assert!(a > 0.9),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn space_is_space() {
        assert!(is_space(
            &keyboard_types::Key::Character(" ".into()),
            keyboard_types::Code::Space
        ));
        assert!(is_space(
            &keyboard_types::Key::Character("a".into()),
            keyboard_types::Code::Space
        ));
        assert!(!is_space(
            &keyboard_types::Key::Character("a".into()),
            keyboard_types::Code::KeyA
        ));
    }

    #[test]
    fn escape_is_escape() {
        assert!(is_escape(
            &keyboard_types::Key::Named(keyboard_types::NamedKey::Escape),
            keyboard_types::Code::Escape
        ));
        assert!(!is_escape(
            &keyboard_types::Key::Character(" ".into()),
            keyboard_types::Code::Space
        ));
    }

    #[test]
    fn arrows_are_arrows() {
        assert!(is_arrow_left(
            &keyboard_types::Key::Named(keyboard_types::NamedKey::ArrowLeft),
            keyboard_types::Code::ArrowLeft
        ));
        assert!(is_arrow_right(
            &keyboard_types::Key::Named(keyboard_types::NamedKey::ArrowRight),
            keyboard_types::Code::ArrowRight
        ));
        assert!(!is_arrow_left(
            &keyboard_types::Key::Named(keyboard_types::NamedKey::ArrowRight),
            keyboard_types::Code::ArrowRight
        ));
    }
}
