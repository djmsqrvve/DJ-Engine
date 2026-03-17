# Tutorial 01: Build a Board Game

Build a complete turn-based board game from scratch using DJ Engine.

> **Try it first:** Run `make stratego` to play the finished game before reading the code.

## What You'll Build

- [x] 10x10 board with terrain (open cells + lakes)
- [x] 8 piece ranks per team (Flag through Marshal), 25 pieces each
- [x] Setup phase where you place your army
- [x] Turn-based play with click-to-select, click-to-move
- [x] Combat resolution (higher rank wins, equal ranks destroy each other)
- [x] Random AI opponent with a thinking delay
- [x] Feedback messages and a 7-step tutorial overlay

## Architecture

| File | Role | Lines |
| --- | --- | --- |
| `main.rs` | App setup, system registration, state wiring | 89 |
| `pieces.rs` | PieceRank, Team, PlacedPiece, army composition | 137 |
| `board.rs` | Grid\<Cell\> board, terrain, placement, auto-fill | 222 |
| `rules.rs` | Combat resolution, move validation, move execution | 346 |
| `state.rs` | GamePhase enum, GameResult | 21 |
| `rendering.rs` | Sprites, coordinate conversion, highlights, tutorial panel | 302 |
| `input.rs` | Click handlers, setup queue, feedback, restart | 333 |
| `ai.rs` | Random AI with timer delay | 116 |
| `tutorial_steps.rs` | 7-step tutorial overlay | 128 |

## Chapters

| # | Title | Files Created | Tests | Concepts |
| --- | --- | --- | --- | --- |
| 01 | [Project Setup](01-project-setup.md) | `Cargo.toml`, `main.rs` | 0 | Crate scaffolding, Bevy window |
| 02 | [Board, Pieces, and Grid](02-board-and-grid.md) | `pieces.rs`, `board.rs` | 9 | Grid\<T\>, cross-module imports, serde |
| 03 | [Rules and Combat](03-pieces-and-rules.md) | `rules.rs` | 11 | Combat logic, movement validation, ray-casting |
| 04 | [Rendering and State](04-rendering.md) | `state.rs`, `rendering.rs` | 0 | Sprites, Z-layering, change detection, Bevy States |
| 05 | [Input and Setup](05-input-and-setup.md) | `input.rs`, `tutorial_steps.rs` (stub) | 0 | Click handling, viewport conversion, state transitions |
| 06 | [State Machine Deep Dive](06-state-machine.md) | *(conceptual)* | 0 | Bevy States patterns, system scheduling |
| 07 | [AI Opponent](07-ai-opponent.md) | `ai.rs` | 0 | Timer delay, random selection, cross-module resources |
| 08 | [Tutorial Overlay](08-polish.md) | `tutorial_steps.rs` (full) | 0 | Event-driven advancement, panel visibility |

## Design Decisions

1. **Grid\<T\> from the engine, not custom**: Reuses `dj_engine::data::Grid<T>` so the spatial data structure is tested and shared across games.
2. **State machine over booleans**: Bevy's `States` API enforces exactly one phase at a time and gates system execution cleanly.
3. **Change detection for rendering**: `sync_pieces_system` only rebuilds sprites when `board.is_changed()`, avoiding per-frame entity churn.
4. **Shared `FeedbackMessage` resource**: Used by input, AI, and tutorial systems -- single point of truth for status display.
5. **Tutorial stub pattern**: `tutorial_steps.rs` is introduced as a minimal stub in Ch 05 so `input.rs` compiles, then replaced with the full implementation in Ch 08.

## Related Systems

- **Engine Grid\<T\>**: `engine/src/data/grid.rs` -- generic 2D grid with neighbors, bounds, serde
- **Editor tutorial overlay**: `engine/src/editor/tutorial.rs` -- the in-editor "Make Your First Game" uses a similar step-based pattern
- **Editor entity placement**: `engine/src/editor/views.rs` -- grid-snapped click-to-place pattern reused in Stratego's `input.rs`

## Source Reference

The complete source lives at `games/dev/stratego/` (~1,700 lines across 9 modules, 20 tests).
