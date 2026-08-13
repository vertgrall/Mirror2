---
name: likeness-perf
description: >-
  Performance engineering for Likeness (Mirror2): camera pipeline, preview vs capture
  resolution, frame dropping, UI redraw budget, buffer pools, VFX CPU cost, memory
  churn, and Skia caching. Use when fixing stutter, choppy controls, memory growth,
  or optimizing the live camera path.
---

# Likeness performance

You are a **performance engineer** working on Likeness — a macOS Rust + Freya camera app with CPU VFX.

## Architecture (must internalize)

```
AVFoundation callback     camera thread              Freya main thread
─────────────────────     ──────────────             ─────────────────
BGRA → rgb Vec (6 MB)  →  downscale 640w           →  current_frame() try_lock
     try_send(1)            mirror + VFX               Skia blit (cached by seq)
                            publish preview Arc        sliders / look chips
                            store full-res rgb         RequestRedraw (conditional)
```

## Non-negotiable budgets

| Resource | Budget |
|----------|--------|
| Preview VFX input | **640px wide** (`theme::PREVIEW_MAX_W`) |
| Live rgba buffer | **~1.2 MB** (640×480×4) |
| Full-res RGB slot | **One frame** — capture path only |
| VFX frame time | **< 33 ms** at preview res |
| UI redraws (idle) | **On seq change only** — not 30 Hz unconditional |
| UI redraws (countdown/flash) | **30 Hz OK** |

## Root causes (check these first)

1. **VFX at 1080p** — preview must downscale *before* mirror/VFX
2. **Unconditional `RequestRedraw`** — floods main thread, breaks sliders
3. **Fresh `vec![]` every pass** — allocator churn looks like memory hoarding
4. **Skia `raster_from_data` every paint** — cache by `frame.seq`
5. **Camera loop with no frame skip** — drain channel to latest frame
6. **`current_frame()` blocking lock** — use `try_lock`, UI keeps last good frame

## File map

| File | Responsibility |
|------|----------------|
| `src/camera.rs` | Preview pipe, full-res slot, `snapshot_for_keep()`, stats |
| `src/macos_avf.rs` | AVFoundation; channel depth 1; `try_recv_frame()` |
| `src/theme.rs` | `PREVIEW_MAX_W` |
| `src/vfx/` | CPU operators — work at preview size only |
| `src/stage.rs` | Skia blit cache keyed by `seq` |
| `src/main.rs` | Conditional redraw; shutter uses `snapshot_for_keep()` |

## Phase checklist

### Done (Phase 0 + 1 baseline)
- [x] Preview downscale before VFX
- [x] Full-res stored for shutter only
- [x] Frame skip via `try_recv_frame` drain
- [x] Pipe stats logged every 3s (`likeness: preview pipe …`)
- [x] Conditional `RequestRedraw`
- [x] Skia image cache by seq
- [x] `current_frame()` non-blocking
- [x] **Controls wired**: `camera::refresh_preview()` on look/param change + `controls_rev` redraw + slider row hit targets

## Control wiring (must preserve)

When look chips or sliders change:

1. Update Freya `State` + `set_params()` / `camera::set_look()`
2. Call `camera::refresh_preview()` — reprocesses latest full-res RGB with current look/params
3. Bump `controls_rev` + `request_redraw()` — slider UI and viewfinder update without waiting for next camera frame

Sliders: mouse handlers live on the **full row** (28px tall), not the 18px track — `SLIDER_TRACK_INSET` maps x → value.

Do **not** rely on unconditional async-loop redraw for control feedback.

### Next (Phase 2 — assign when still slow)
- [ ] `FramePool` — reuse rgb/rgba/gray/mask buffers (zero steady-state alloc)
- [ ] Params via `Arc<LookParams>` or atomics — drop mutex in VFX hot path
- [ ] Slider debounce for VFX param writes (16–60 ms)
- [ ] AVFoundation: reuse rgb buffer in delegate instead of `vec!` per callback

### Later (Phase 3)
- [ ] Half-res warp pass
- [ ] Separable morphology
- [ ] GPU / Skia shader path for heavy looks
- [ ] Debug overlay: fps, vfx ms, dropped, live MB

## Adding features without regressing perf

1. **Never run VFX at sensor resolution for live preview**
2. **Capture/full-res only in `snapshot_for_keep()` or explicit save path**
3. **New temporal state must document memory** (`w×h×frames×3`)
4. **Profile before optimizing** — read `likeness: preview pipe` logs
5. **Test**: `cargo test` + manual slider drag + Activity Monitor flat memory over 60s

## Instrumentation

Camera thread logs every 3 seconds:
```
likeness: preview pipe  12.3 ms/frame avg  8 fps  2 dropped
```

If avg > 33 ms → VFX too heavy even at preview; optimize operator or reduce `PREVIEW_MAX_W` to 480.

## Anti-patterns (reject in review)

- `apply()` on 1920×1080 for live view
- `RequestRedraw` in a loop without gating
- Cloning full-res frame for UI display
- Storing frame history rings without a cap
- Blocking `Mutex::lock()` on UI thread for camera frame
