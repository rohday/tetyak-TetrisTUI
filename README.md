# tetyak

A taste-first, minimal, low-footprint terminal Tetris built with Rust and Ratatui.

Inspired by the clean, typography-first aesthetic of [appleTUI](../appleTUI).

## Features

- **Zero Emojis**: Clean typography, crisp Ratatui rounded borders, and subtle Unicode blocks (`██`, `░░`, `▒▒`).
- **Spacious & Uncongested**: Large 10×20 field with double-width cells (`██`) and generous margins.
- **Speed Capped**: Smooth drop acceleration that stops at a comfortable, flow-state pace (never runs away infinitely).
- **Conditional Explosion**: Instant line clear for 1–2 lines; quick ~120ms freeze-frame particle flash for 3+ lines (Triples & Tetrises).
- **Live Metrics**: Real-time pace (Pieces Per Minute), pieces locked, and multi-line blast tracker.
- **Difficulty Selection with Unique RNG**:
  - `[1] Chill`: 800ms → 350ms cap, generous lock delay, friendly 14-piece anti-clustering bag.
  - `[2] Normal` *(Default)*: 600ms → 200ms cap, standard lock delay, authentic classic NES 1-history reroll (no rigid 7-bag cycle).
  - `[3] Intense`: 350ms → 110ms cap, snappy lock delay, pure uniform memoryless randomizer.
- **Theme Cycling**: Press `T` anytime to cycle through **Apple Dark**, **Catppuccin Mocha**, and **Tokyo Night**.
- **Tiny Footprint**: ~560 KB standalone binary, ~2.5 MB runtime RAM, 0% idle CPU.

## Controls

| Key | Action |
| :--- | :--- |
| `A` / `←` | Move Left |
| `D` / `→` | Move Right |
| `W` / `↑` | Rotate Clockwise |
| `Z` | Rotate Counter-Clockwise |
| `S` / `↓` | Soft Drop |
| `Space` | Hard Drop (Instant lock) |
| `C` / `Shift` | Hold Piece (once per turn) |
| `P` | Pause / Resume |
| `T` | Cycle Theme (Apple Dark / Catppuccin / Tokyo Night) |
| `R` | Restart (on Game Over) |
| `Q` / `Esc` | Quit |

## Installation & Running

Both commands are installed to `~/.local/bin`:

```bash
tetyak
# or
tetris
```
