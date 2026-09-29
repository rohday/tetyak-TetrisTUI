# Clean Modern Terminal Tetris — Design Specification

**Date:** 2026-09-29  
**Target Repository:** `~/githubIGuess/tetris`  
**Binary Output:** `~/.local/bin/tetris`

---

## 1. Overview & Vision
A sleek, minimal, low-footprint terminal Tetris written in Rust with Ratatui and Crossterm. Inspired by the clean, typography-focused aesthetic of **`appleTUI`**:
- **Zero emojis**: Uses clean symbols (`>`, `*`, `[HOLD]`, `[NEXT]`), crisp typography, and refined muted palettes.
- **Spacious & Uncongested**: Large 10x20 well with double-width cells (`██`) and generous margin/padding cards.
- **Speed Capped**: Smooth acceleration that strictly stops at a comfortable, flow-state pace.
- **Dynamic Explosions**: Instant zero-pause clear for 1–2 lines; quick, impactful 120ms freeze-frame flash for 3+ lines (Triples and Tetrises).
- **Difficulty Selection with Custom Piece Distribution**: Pre-game choice that alters speeds, lock delays, and tetromino generation randomness.

---

## 2. Visual Aesthetic (AppleTUI Inspired)

### Color Palette (Apple Dark Default, Theme Support)
- **Background & Surrounds**: Deep dark terminal background with subtle muted borders (`rgb(60, 60, 70)` unfocused, `rgb(250, 45, 72)` or theme accent focused).
- **Block Colors**:
  - `I` (Cyan): `rgb(100, 210, 255)`
  - `O` (Yellow): `rgb(255, 214, 10)`
  - `T` (Purple): `rgb(175, 82, 222)`
  - `S` (Green): `rgb(48, 209, 88)`
  - `Z` (Red): `rgb(255, 69, 58)`
  - `J` (Blue): `rgb(10, 132, 255)`
  - `L` (Orange): `rgb(255, 159, 10)`
  - `Ghost`: Muted outline or dim block `rgb(70, 70, 80)`
- **UI Cards**: Ratatui `BorderType::Rounded` blocks.

### Layout Wireframe
```text
┌───────────────────────────────────────────────────────────────┐
│                      T E T R I S                              │
├──────────────┬───────────────────────────────┬────────────────┤
│ ╭── HOLD ──╮ │ ╭────────── FIELD ──────────╮ │ ╭── NEXT ────╮ │
│ │          │ │ │                           │ │ │            │ │
│ │    ██    │ │ │                           │ │ │     ██     │ │
│ │  ██████  │ │ │                           │ │ │   ██████   │ │
│ │          │ │ │                           │ │ │            │ │
│ ╰──────────╯ │ │             ██              │ ╰────────────╯ │
│              │ │             ██              │ ╭── STATS ───╮ │
│              │ │             ██              │ │ Mode: Normal │
│              │ │             ██              │ │ Score: 12400 │
│              │ │                             │ │ Lines: 28    │
│              │ │                             │ │ Pieces: 54   │
│              │ │                             │ │ Pace: 42 PPM │
│              │ │       ▒▒                    │ ├────────────┤ │
│              │ │       ▒▒                    │ │ BLASTS     │ │
│              │ │     ██████                  │ │ 1x: 8  2x: 4 │
│              │ │   ████████████              │ │ 3x: 2  4x: 1 │
│              │ │ ████████████████████        │ │              │
│              │ │ ████████████████████        │ │ > TETRIS <   │
│              │ ╰─────────────────────────────╯ ╰────────────╯ │
└──────────────┴───────────────────────────────┴────────────────┘
```

---

## 3. Difficulty System & Piece Distribution

Players can choose difficulty at startup via 1-touch key (`1`, `2`, `3`) or `Enter` for default:

| Setting | 1. Chill | 2. Normal (Default) | 3. Intense |
| :--- | :--- | :--- | :--- |
| **Initial Drop Interval** | 800ms | 600ms | 350ms |
| **Final Speed Cap** | 350ms (never faster) | 200ms (balanced flow) | 110ms (fast reflex) |
| **Acceleration Curve** | -15ms every 5 lines | -25ms every 4 lines | -25ms every 3 lines |
| **Lock Delay** | 500ms | 400ms | 250ms |
| **Piece Distribution** | **Friendly 7-Bag**: Early I/T guarantee, avoids consecutive S/Z | **Standard 7-Bag**: Standard guideline 7-piece bag (guaranteed cycle) | **NES Classic Memoryless**: Random roll with 1-history reroll (droughts & streaks possible) |

---

## 4. Gameplay Mechanics

1. **Grid**: Standard 10 columns × 20 rows.
2. **Double-width Blocks**: Rendered as `██` so the grid is visually proportionate and not vertically squashed.
3. **Ghost Piece**: Shows projected landing position in dimmed style (`░░` or faint `██`).
4. **Controls**:
   - `Left` / `A`: Move Left
   - `Right` / `D`: Move Right
   - `Up` / `W`: Rotate Clockwise
   - `Z`: Rotate Counter-Clockwise
   - `Down` / `S`: Soft Drop
   - `Space`: Hard Drop (instant lock)
   - `C` / `Shift`: Hold Piece (once per turn)
   - `P`: Pause / Resume
   - `Q` / `Esc`: Quit
5. **Explosion & Line Clear Animation**:
   - **1–2 Lines**: Cleared immediately in the same tick. No pause, maintaining smooth gameplay.
   - **3–4 Lines (Triple / Tetris)**: Triggers a **120ms freeze-frame flash** (`████` full brightness -> `▒▒▒▒` particle dissolution) before stack settles.
6. **Live Metrics**:
   - Total pieces locked.
   - Real-time drop pace (PPM = Pieces Per Minute).
   - Rows blown counter: Singles, Doubles, Triples, Tetrises.
   - Last blast banner (e.g. `> TETRIS (4x) <`).

---

## 5. Technical Footprint & Performance

- **Language**: Rust (Edition 2021)
- **TUI Stack**: `ratatui` + `crossterm`
- **Release Profile (`Cargo.toml`)**:
  ```toml
  [profile.release]
  opt-level = "z"
  lto = true
  codegen-units = 1
  panic = "abort"
  strip = true
  ```
- **Target Binary**: ~1.8 MB stripped executable installed to `~/.local/bin/tetris`.
- **Resource Footprint**: < 4 MB RAM, 0% CPU idle (uses event-driven loop with tick interval).
