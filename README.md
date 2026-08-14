# Mirror2

Outsider spiritual successor to Photo Booth.

No curtains. No booth. No fake materials.

You sit. You pick a look. The camera well is always the preview. You keep a still.

```
 MIRROR2                           NEW TOWER
 ┌─────────────────────────────────────────┐
 │              live camera                │
 │         (480 × 360, locked)             │
 └─────────────────────────────────────────┘
              ◉ shutter
         wet · sliders · open folder
   <  OFF   VHS   GX   >        chevron dock
```

## Looks (v0.1)

| Look | Line | Family |
|------|------|--------|
| **OFF** | clean camera | — |
| **MORPH** | ink drawing | paper |
| **VHS** | tracking · wear | tape |
| **GX** | Hi8 · 1994 | tape |
| **UHF** | antenna · snow | tape |
| **BETA** | luma · dropout | tape |
| **D8** | block · digital | tape |
| **LIVE** | tally · interlace | tape |
| **SAT** | rain · macro | tape (16:9 letterbox) |
| **CCTV** | blocky · crushed | eye |
| **RIPPLE** | water rings | water |

Dock cards show photoreal still-life specimens. Chevrons page the catalog.

## Run it (macOS)

Camera permission must belong to the app bundle, not Terminal:

```bash
./scripts/run.sh
```

Quit any old window before relaunching — stale binaries are the usual reason changes do not show up.

**Space** or the shutter: 3 · 2 · 1, flash, then the still saves to `~/Pictures/Mirror2`.

## Stack

Rust + [Freya](https://freyaui.dev) + AVFoundation (32BGRA). New Tower.

## Layout

Fixed **480×728** window. Shutter centered on the spine. Layout regression tests in `main.rs` (`layout_stone_nothing_walks_off_the_glass`).
