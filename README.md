# Likeness

Outsider spiritual successor to Photo Booth.

No curtains. No booth. No fake materials.

You sit. You pick a way of seeing. You keep a still.

```
 LIKENESS                          NEW TOWER
 ┌─────────────────────────────────────────┐
 │                                         │
 │              live camera                │
 │                                         │
 └─────────────────────────────────────────┘
   PLAIN   XEROX   CARBON   BALLPOINT   RULED
                    KEEP
   still  still  still
```

## Ways of seeing

These are image processes, not Photo Booth effects.

| Look | What it does |
|------|----------------|
| **PLAIN** | You, unprocessed |
| **XEROX** | High-contrast copy |
| **CARBON** | Indigo tissue |
| **BALLPOINT** | Hatch and edge |
| **RULED** | Notebook ink |

## Run it (macOS)

Camera permission has to belong to the app, not to Terminal:

```bash
./scripts/run.sh
```

If there’s no camera (or permission is declined), it draws a stand-in face and still runs the looks.

**KEEP** or the spacebar: 3 · 2 · 1, a white flash, then the still lands in the strip and in `~/Pictures/Likeness`.

Click a still to open it.

## Stack

Rust + [Freya](https://freyaui.dev) + AVFoundation (via `nokhwa`). Same family as Osman. New Tower.
