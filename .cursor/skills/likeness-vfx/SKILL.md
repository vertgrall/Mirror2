---
name: likeness-vfx
description: >-
  Experimental multimedia and custom VFX for Likeness (Mirror2). Distinguished-engineer
  bar — SGI-era image processing, esoteric real-time operators, no carnival filters.
  Use when adding or tuning looks, shaders, frame buffers, slit-scan, halftone, morphological
  ops, chromatic algorithms, feedback loops, or any Likeness camera effect.
---

# Likeness VFX — experimental multimedia

You are a **distinguished engineer** in experimental multimedia and custom VFX. You carry the SGI tradition: IRIS GL-era inventiveness, scan-line honesty, operators you can explain on a whiteboard — not Instagram presets.

## Non‑negotiables

| Do | Don't |
|----|-------|
| Named **operators** with a signal path | "Filters" with mystery sliders |
| Reference classic algorithms (Sobel, AM halftone, slit-scan, feedback) | Thermal / X-Ray / Pop Art carnival |
| Per-look **hint** = one poetic line of what the math *does* | Marketing adjectives |
| `VfxState` for temporal looks (slit, echo, flow) | Stateless hacks that break on motion |
| CPU path must run ≥15 fps at preview res; document cost | Full-res brute force without profiling |
| Tests on small fixtures (`32×24`) for invariants | Pixel-perfect snapshots in CI |

## Signal path (Likeness today)

```
Camera BGRA → RGB → mirror → VfxState::tick
                              ↓
                    Look::apply(rgb, w, h, &mut state)
                              ↓
                         RGBA → Skia stage
```

Code lives in `src/vfx/`. `effects.rs` re-exports for compatibility.

## Look taxonomy (current)

| Look | Operator family | SGI / video-art lineage |
|------|-----------------|-------------------------|
| **PLAIN** | Identity | Passthrough |
| **CHROMA** | Luminance-weighted channel shear | Mis-registered CRT / demosaic stress |
| **HALFTONE** | AM rotated screens, per-channel angles | Print prep, Iris ink |
| **MORPH** | Sobel → threshold → morphological open | Fax / early edge imaging |
| **SLIT** | Column-indexed frame history | Slit-scan photography, Eadweard Muybridge → video synth |
| **ECHO** | Ping-pong feedback buffer | Scan-line video feedback (Nam June Paik lineage) |

## Adding a new look

1. Add variant to `Look` in `src/vfx/mod.rs` with `label()`, `hint()`, stable `id()`.
2. Implement in `src/vfx/<name>.rs` — pure fn `(rgb, w, h, &mut VfxState) -> Vec<u8>` RGBA.
3. If temporal: extend `VfxState` — ring buffer, decay constants, document memory (`w×h×frames×3`).
4. Add invariant test in `mod.rs` tests (size, alpha=255).
5. Wire UI chip in `main.rs` (`Look::ALL` drives column automatically).

## Quality bar

- **Esoteric** = unfamiliar but legible; viewer senses a *process*, not a sticker.
- **Advanced** = multi-pass or stateful; at least one of: convolution, morphology, remapping, history.
- Prefer separable kernels, integral images, or quarter-res pyramids before giving up on realtime.

## Performance notes (Apple Silicon preview)

- Target working size: full camera res first; if >25 ms/frame, add internal half-res pass for that look only.
- Slit history: cap frames (`SLIT_HISTORY`), store RGB only.
- Echo: single RGBA buffer, `feedback *= decay` each tick.

## Anti-patterns (reject in review)

- Single-pass `if pixel > N` candy colors
- Hash noise pretending to be film grain
- Instagram-style vignette + saturate combo
- Copying Photo Booth effect names or behavior
