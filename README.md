# Mirror2

Outsider spiritual successor to Photo Booth.

No curtains. No booth. No fake materials.

You sit. You pick a look. The camera well is always the preview. You keep a still.

## Screenshots

| Clean camera | VHS · tracking · wear | D8 · block · digital |
|:---:|:---:|:---:|
| ![OFF](docs/screenshots/mirror2-off.png) | ![VHS](docs/screenshots/mirror2-vhs.png) | ![D8](docs/screenshots/mirror2-d8.png) |

| SAT · rain · macro | CCTV · blocky · crushed | RIPPLE · water rings |
|:---:|:---:|:---:|
| ![SAT](docs/screenshots/mirror2-sat.png) | ![CCTV](docs/screenshots/mirror2-cctv.png) | ![RIPPLE](docs/screenshots/mirror2-ripple.png) |

| Tape catalog dock (UHF · BETA · D8) |
|:---:|
| ![dock](docs/screenshots/mirror2-dock-tape.png) |

Dock cards are photoreal still-life specimens. Chevrons page through the catalog. Family chips jump rails: **tape · eye · water · weird**.

```
 MIRROR2                           NEW TOWER
 ┌─────────────────────────────────────────┐
 │              live camera                │
 │         (480 × 360, locked)             │
 └─────────────────────────────────────────┘
              ◉ shutter
    LOOK / ATMO · wet · sliders · open folder
   <  OFF   VHS   GX   >        chevron dock
        tape · eye · water · weird
```

## Looks (56 in catalog)

| Family | Examples |
|--------|----------|
| **tape** | OFF, VHS, GX, UHF, BETA, D8, LIVE, SAT, MOSH, GLITCH, DATAMOSH, JAM, SUPER8 |
| **eye** | CCTV, THERMAL, XRAY, NOIR, CYBER, SLITSCAN, GILT |
| **water** | RIPPLE, SMEAR, BREATHE, FILM, WAVES, FLUID, DRIFT, REACTION, POLAR, KODAK, PHASE |
| **weird** | MORPH, HAUNT, B-OMEN, COMMUNION, TUBER, STICK, FRACTURE, … |

Hero interactive looks — drag on the camera well:

| Look | Line |
|------|------|
| **B-OMEN** | wand · omen |
| **COMMUNION** | wand · copies |
| **TUBER** | tap · stretch |
| **HAUNT** | smear · ghosts · burn |

**GILT** (eye): kintsugi — live cracks fill with gold leaf.

## Run it (macOS)

Camera permission must belong to the app bundle, not Terminal:

```bash
./scripts/run.sh
```

Quit any old window before relaunching — stale binaries are the usual reason changes do not show up.

**Space** or the shutter: 3 · 2 · 1, flash, then the still saves to `~/Pictures/Mirror2`.

Toggle **bypass 3s** to fire instantly. **ATMO** tab: global smoke haze over any look.

## Stack

Rust + [Freya](https://freyaui.dev) + AVFoundation (32BGRA). New Tower.

## Layout

Fixed **480×834** window. Shutter centered on the spine. Layout regression tests in `main.rs` (`layout_stone_nothing_walks_off_the_glass`).

Regenerate README screenshots:

```bash
cargo test export_readme_screenshots
```

Build release DMG:

```bash
cargo build --release && ./build_dmg.sh
```
