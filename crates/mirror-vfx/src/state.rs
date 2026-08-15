//! Temporal state — frame counter, VHS ghost, smoke phase, morph ink history.

use std::sync::atomic::{AtomicU32, Ordering};

static BREATHE_SEED: AtomicU32 = AtomicU32::new(0xA5A5_5A5A);

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
    /// Normalized pointer in frame space (0–1). Used by SMUDGE / POSSESS.
    pub pointer_x: f32,
    pub pointer_y: f32,
    pub pointer_prev_x: f32,
    pub pointer_prev_y: f32,
    pub pointer_down: bool,
    /// Wet paint layer for SMUDGE.
    pub paint_r: Vec<f32>,
    pub paint_g: Vec<f32>,
    pub paint_b: Vec<f32>,
    pub paint_a: Vec<f32>,
    /// Afterimage burn-in buffer for POSSESS.
    pub burn_r: Vec<f32>,
    pub burn_g: Vec<f32>,
    pub burn_b: Vec<f32>,
    /// Random layout seed for BREATHE particle clusters (re-rolled on look entry).
    pub breathe_seed: u32,
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
            pointer_x: 0.5,
            pointer_y: 0.5,
            pointer_prev_x: 0.5,
            pointer_prev_y: 0.5,
            pointer_down: false,
            paint_r: Vec::new(),
            paint_g: Vec::new(),
            paint_b: Vec::new(),
            paint_a: Vec::new(),
            burn_r: Vec::new(),
            burn_g: Vec::new(),
            burn_b: Vec::new(),
            breathe_seed: 1,
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
            self.burn_r.clear();
            self.burn_g.clear();
            self.burn_b.clear();
            self.paint_r.clear();
            self.paint_g.clear();
            self.paint_b.clear();
            self.paint_a.clear();
            self.breathe_seed = BREATHE_SEED.fetch_add(0x9E37_79B9, Ordering::Relaxed);
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
        self.burn_r.clear();
        self.burn_g.clear();
        self.burn_b.clear();
        self.paint_r.clear();
        self.paint_g.clear();
        self.paint_b.clear();
        self.paint_a.clear();
    }

    pub fn set_pointer(&mut self, nx: f32, ny: f32, down: bool) {
        let nx = nx.clamp(0.0, 1.0);
        let ny = ny.clamp(0.0, 1.0);
        if down {
            if self.pointer_down {
                self.pointer_prev_x = self.pointer_x;
                self.pointer_prev_y = self.pointer_y;
            } else {
                self.pointer_prev_x = nx;
                self.pointer_prev_y = ny;
            }
        }
        self.pointer_x = nx;
        self.pointer_y = ny;
        self.pointer_down = down;
    }

    pub fn ensure_paint(&mut self, len: usize) {
        if self.paint_r.len() != len {
            self.paint_r = vec![0.0; len];
            self.paint_g = vec![0.0; len];
            self.paint_b = vec![0.0; len];
            self.paint_a = vec![0.0; len];
        }
    }

    pub fn ensure_burn(&mut self, len: usize) {
        if self.burn_r.len() != len {
            self.burn_r = vec![0.0; len];
            self.burn_g = vec![0.0; len];
            self.burn_b = vec![0.0; len];
        }
    }

    /// Gray-Scott reaction-diffusion grid (u,v in 0..1).
    pub fn ensure_rd(&mut self, len: usize) {
        if self.rd_u.len() != len {
            self.rd_u = vec![1.0; len];
            self.rd_v = vec![0.0; len];
        }
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

