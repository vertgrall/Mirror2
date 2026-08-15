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
    /// Normalized marquee selection (0–1). Used by FRACTURE / RUPTURE.
    pub sel_x0: f32,
    pub sel_y0: f32,
    pub sel_x1: f32,
    pub sel_y1: f32,
    pub sel_valid: bool,
    pub sel_dragging: bool,
    /// B-OMEN color select + omen clone placement.
    pub bomen_mask: Vec<f32>,
    pub bomen_src_rgb: Vec<u8>,
    pub bomen_box: [u32; 4],
    pub bomen_has_source: bool,
    pub bomen_tap_pending: bool,
    pub bomen_tap_x: f32,
    pub bomen_tap_y: f32,
    pub bomen_clone_x: f32,
    pub bomen_clone_y: f32,
    pub bomen_clone_placed: bool,
    pub bomen_placing: bool,
    pub bomen_down_active: bool,
    pub bomen_down_x: f32,
    pub bomen_down_y: f32,
    pub bomen_src_w: u32,
    pub bomen_src_h: u32,
    /// True while writing a shutter still — hide preview-only guides.
    pub exporting: bool,
    /// TUBER stretch strokes (up to 4).
    pub tuber_n: u8,
    pub tuber_drag: bool,
    pub tuber_ax: [f32; 4],
    pub tuber_ay: [f32; 4],
    pub tuber_bx: [f32; 4],
    pub tuber_by: [f32; 4],
    pub tuber_patch: Vec<u8>,
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
            sel_x0: 0.0,
            sel_y0: 0.0,
            sel_x1: 0.0,
            sel_y1: 0.0,
            sel_valid: false,
            sel_dragging: false,
            bomen_mask: Vec::new(),
            bomen_src_rgb: Vec::new(),
            bomen_box: [0, 0, 0, 0],
            bomen_has_source: false,
            bomen_tap_pending: false,
            bomen_tap_x: 0.5,
            bomen_tap_y: 0.5,
            bomen_clone_x: 0.5,
            bomen_clone_y: 0.5,
            bomen_clone_placed: false,
            bomen_placing: false,
            bomen_down_active: false,
            bomen_down_x: 0.5,
            bomen_down_y: 0.5,
            bomen_src_w: 0,
            bomen_src_h: 0,
            exporting: false,
            tuber_n: 0,
            tuber_drag: false,
            tuber_ax: [0.5; 4],
            tuber_ay: [0.5; 4],
            tuber_bx: [0.5; 4],
            tuber_by: [0.5; 4],
            tuber_patch: Vec::new(),
        }
    }
}

impl VfxState {
    /// Advance time for this frame. Does **not** overwrite history — read that first.
    pub fn advance(&mut self, w: u32, h: u32, look_id: u8) {
        if self.last_w != w || self.last_h != h || self.last_look != look_id {
            let look_changed = self.last_look != look_id;
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
            if look_changed {
                self.clear_bomen();
                self.clear_tuber();
            }
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
        self.sel_valid = false;
        self.sel_dragging = false;
        self.sel_x0 = 0.0;
        self.sel_y0 = 0.0;
        self.sel_x1 = 0.0;
        self.sel_y1 = 0.0;
        self.clear_bomen();
        self.clear_tuber();
    }

    pub fn clear_tuber(&mut self) {
        self.tuber_n = 0;
        self.tuber_drag = false;
    }

    pub fn ensure_tuber(&mut self) {
        let need = 4 * 48 * 48 * 3;
        if self.tuber_patch.len() != need {
            self.tuber_patch = vec![0u8; need];
        }
    }

    pub fn clear_bomen(&mut self) {
        self.bomen_mask.clear();
        self.bomen_src_rgb.clear();
        self.bomen_box = [0, 0, 0, 0];
        self.bomen_has_source = false;
        self.bomen_tap_pending = false;
        self.bomen_clone_placed = false;
        self.bomen_placing = false;
        self.bomen_down_active = false;
        self.bomen_src_w = 0;
        self.bomen_src_h = 0;
    }

    pub fn ensure_bomen(&mut self, len: usize) {
        if self.bomen_mask.len() != len {
            self.bomen_mask = vec![0.0; len];
            self.bomen_src_rgb = vec![0u8; len * 3];
        }
    }

    pub fn bomen_pointer_down(&mut self, nx: f32, ny: f32) {
        let nx = nx.clamp(0.0, 1.0);
        let ny = ny.clamp(0.0, 1.0);
        self.bomen_down_active = true;
        self.bomen_down_x = nx;
        self.bomen_down_y = ny;
        if self.bomen_has_source {
            self.bomen_placing = true;
            self.bomen_clone_x = nx;
            self.bomen_clone_y = ny;
        }
    }

    pub fn bomen_pointer_move(&mut self, nx: f32, ny: f32) {
        if self.bomen_placing {
            self.bomen_clone_x = nx.clamp(0.0, 1.0);
            self.bomen_clone_y = ny.clamp(0.0, 1.0);
        }
    }

    pub fn bomen_pointer_up(&mut self, nx: f32, ny: f32) {
        if !self.bomen_down_active {
            return;
        }
        self.bomen_down_active = false;
        let nx = nx.clamp(0.0, 1.0);
        let ny = ny.clamp(0.0, 1.0);
        if self.bomen_placing {
            self.bomen_placing = false;
            self.bomen_clone_placed = true;
            self.bomen_clone_x = nx;
            self.bomen_clone_y = ny;
            return;
        }
        let dx = (nx - self.bomen_down_x).abs();
        let dy = (ny - self.bomen_down_y).abs();
        if dx < 0.025 && dy < 0.025 {
            self.bomen_tap_pending = true;
            self.bomen_tap_x = nx;
            self.bomen_tap_y = ny;
        }
    }

    pub fn begin_selection(&mut self, nx: f32, ny: f32) {
        let nx = nx.clamp(0.0, 1.0);
        let ny = ny.clamp(0.0, 1.0);
        self.sel_x0 = nx;
        self.sel_y0 = ny;
        self.sel_x1 = nx;
        self.sel_y1 = ny;
        self.sel_valid = false;
        self.sel_dragging = true;
    }

    pub fn update_selection(&mut self, nx: f32, ny: f32) {
        if !self.sel_dragging {
            return;
        }
        self.sel_x1 = nx.clamp(0.0, 1.0);
        self.sel_y1 = ny.clamp(0.0, 1.0);
    }

    pub fn finish_selection(&mut self) {
        if !self.sel_dragging {
            return;
        }
        self.sel_dragging = false;
        let dx = (self.sel_x1 - self.sel_x0).abs();
        let dy = (self.sel_y1 - self.sel_y0).abs();
        self.sel_valid = dx > 0.015 || dy > 0.015;
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

