//! Temporal state — frame counter, VHS ghost, smoke phase, morph ink history.

pub struct VfxState {
    pub frame: u64,
    pub smoke_phase: f32,
    prev_rgb: Option<Vec<u8>>,
    prev_morph: Option<Vec<u8>>,
    last_w: u32,
    last_h: u32,
    last_look: u8,
    // Gen II VFX states
    pub ring_buffer: Vec<Vec<u8>>,
    pub ring_head: usize,
    pub rd_u: Vec<f32>,
    pub rd_v: Vec<f32>,
    pub fluid_vx: Vec<f32>,
    pub fluid_vy: Vec<f32>,
    pub fluid_dye_r: Vec<f32>,
    pub fluid_dye_g: Vec<f32>,
    pub fluid_dye_b: Vec<f32>,
    pub strata_prev: Option<Vec<u8>>,
    pub mosh_rgb: Option<Vec<u8>>,
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
            ring_buffer: Vec::new(),
            ring_head: 0,
            rd_u: Vec::new(),
            rd_v: Vec::new(),
            fluid_vx: Vec::new(),
            fluid_vy: Vec::new(),
            fluid_dye_r: Vec::new(),
            fluid_dye_g: Vec::new(),
            fluid_dye_b: Vec::new(),
            strata_prev: None,
            mosh_rgb: None,
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
            self.ring_buffer.clear();
            self.ring_head = 0;
            self.rd_u.clear();
            self.rd_v.clear();
            self.fluid_vx.clear();
            self.fluid_vy.clear();
            self.fluid_dye_r.clear();
            self.fluid_dye_g.clear();
            self.fluid_dye_b.clear();
            self.strata_prev = None;
            self.mosh_rgb = None;
        }
        self.frame = self.frame.wrapping_add(1);
        self.smoke_phase += 0.016;
    }

    pub fn clear_temporal(&mut self) {
        self.frame = 0;
        self.prev_rgb = None;
        self.prev_morph = None;
        self.ring_buffer.clear();
        self.ring_head = 0;
        self.rd_u.clear();
        self.rd_v.clear();
        self.fluid_vx.clear();
        self.fluid_vy.clear();
        self.fluid_dye_r.clear();
        self.fluid_dye_g.clear();
        self.fluid_dye_b.clear();
        self.strata_prev = None;
        self.mosh_rgb = None;
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

    /// Push RGB frame into circular ring history (up to max_len).
    pub fn push_ring(&mut self, rgb: &[u8], max_len: usize) {
        if self.ring_buffer.len() < max_len {
            self.ring_buffer.push(rgb.to_vec());
        } else {
            let idx = self.ring_head;
            if self.ring_buffer[idx].len() != rgb.len() {
                self.ring_buffer[idx] = rgb.to_vec();
            } else {
                self.ring_buffer[idx].copy_from_slice(rgb);
            }
            self.ring_head = (self.ring_head + 1) % max_len;
        }
    }

    /// Retrieve frame from ring history `depth` steps in the past (0 = newest).
    pub fn get_ring(&self, depth: usize) -> Option<&[u8]> {
        if self.ring_buffer.is_empty() {
            return None;
        }
        let len = self.ring_buffer.len();
        let depth = depth.min(len - 1);
        let head = if len < self.ring_buffer.capacity().max(1) && self.ring_head == 0 {
            len
        } else {
            self.ring_head
        };
        let idx = (head + len - 1 - depth) % len;
        Some(&self.ring_buffer[idx])
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

