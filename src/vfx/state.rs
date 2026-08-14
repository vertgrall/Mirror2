//! Temporal state — frame counter, VHS ghost, smoke phase.

pub struct VfxState {
    pub frame: u64,
    pub smoke_phase: f32,
    prev_rgb: Option<Vec<u8>>,
    last_w: u32,
    last_h: u32,
}

impl Default for VfxState {
    fn default() -> Self {
        Self {
            frame: 0,
            smoke_phase: 0.0,
            prev_rgb: None,
            last_w: 0,
            last_h: 0,
        }
    }
}

impl VfxState {
    /// Advance time for this frame. Does **not** overwrite `prev_rgb` — read that first.
    pub fn advance(&mut self, w: u32, h: u32) {
        if self.last_w != w || self.last_h != h {
            self.frame = 0;
            self.prev_rgb = None;
            self.last_w = w;
            self.last_h = h;
        }
        self.frame = self.frame.wrapping_add(1);
        self.smoke_phase += 0.016;
    }

    /// Store composited RGB for temporal looks on the **next** frame.
    pub fn commit_rgb(&mut self, rgb: &[u8]) {
        if self.prev_rgb.as_ref().map(|v| v.len()) != Some(rgb.len()) {
            self.prev_rgb = Some(vec![0u8; rgb.len()]);
        }
        if let Some(prev) = self.prev_rgb.as_mut() {
            prev.copy_from_slice(rgb);
        }
    }

    pub fn prev_rgb(&self) -> Option<&[u8]> {
        self.prev_rgb.as_deref()
    }
}
