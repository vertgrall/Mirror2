//! Temporal state — frame counter, VHS ghost, smoke phase, morph ink history.

pub struct VfxState {
    pub frame: u64,
    pub smoke_phase: f32,
    prev_rgb: Option<Vec<u8>>,
    prev_morph: Option<Vec<u8>>,
    last_w: u32,
    last_h: u32,
    last_look: u8,
}

impl Default for VfxState {
    fn default() -> Self {
        Self {
            frame: 0,
            smoke_phase: 0.0,
            prev_rgb: None,
            prev_morph: None,
            last_w: 0,
            last_h: 0,
            last_look: 255,
        }
    }
}

impl VfxState {
    /// Advance time for this frame. Does **not** overwrite history — read that first.
    pub fn advance(&mut self, w: u32, h: u32, look_id: u8) {
        if self.last_w != w || self.last_h != h || self.last_look != look_id {
            self.frame = 0;
            self.prev_rgb = None;
            self.prev_morph = None;
            self.last_w = w;
            self.last_h = h;
            self.last_look = look_id;
        }
        self.frame = self.frame.wrapping_add(1);
        self.smoke_phase += 0.016;
    }

    pub fn clear_temporal(&mut self) {
        self.frame = 0;
        self.prev_rgb = None;
        self.prev_morph = None;
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

    /// Store last morph ink RGBA for motion stretch on the **next** frame.
    pub fn commit_morph(&mut self, rgba: &[u8]) {
        if self.prev_morph.as_ref().map(|v| v.len()) != Some(rgba.len()) {
            self.prev_morph = Some(vec![0u8; rgba.len()]);
        }
        if let Some(prev) = self.prev_morph.as_mut() {
            prev.copy_from_slice(rgba);
        }
    }

    pub fn prev_rgb(&self) -> Option<&[u8]> {
        self.prev_rgb.as_deref()
    }

    pub fn prev_morph(&self) -> Option<&[u8]> {
        self.prev_morph.as_deref()
    }
}
