# DJ Engine Tutorials

Learn DJ Engine by building complete games from scratch.

> **Quick start:** Run `make stratego` to play the finished Tutorial 01 before reading the code.

## Tutorials

| # | Title | What You'll Build | Modules | Tests |
| --- | --- | --- | --- | --- |
| 01 | [Build a Board Game](01-build-a-board-game/) | Stratego-lite (10x10, 8 piece types, AI) | 9 | 20 |
| 02 | Isometric Sandbox *(coming soon)* | Iso tile grid with entity placement | -- | -- |

## What You'll Learn

- [x] Scaffold a new game crate in the DJ Engine workspace
- [x] Use the engine's `Grid<T>` for 2D spatial data
- [x] Render sprites with Bevy 2D (coordinates, Z-layering, change detection)
- [x] Handle mouse input (viewport-to-world conversion, click detection)
- [x] Build a state machine with Bevy States
- [x] Write a simple AI opponent
- [x] Create an in-game tutorial overlay

## Prerequisites

- Rust toolchain installed (see `rust-toolchain.toml` for pinned version)
- `make dev` launches the editor successfully
- Basic familiarity with Rust and ECS concepts

## Conventions

- Every code block is **complete and copy-pasteable** -- no `// ...` ellipsis
- `main.rs` is shown in full at the end of every chapter where it changes
- Each chapter ends with a checkpoint: what you should see + tests to run
- Code is extracted from the actual source files in `games/dev/`

## Source Coverage

| Source File | Lines | Tutorial |
| --- | --- | --- |
| `games/dev/stratego/src/main.rs` | 89 | 01 (evolves each chapter) |
| `games/dev/stratego/src/pieces.rs` | 137 | 01, Ch 02 |
| `games/dev/stratego/src/board.rs` | 222 | 01, Ch 02 |
| `games/dev/stratego/src/rules.rs` | 346 | 01, Ch 03 |
| `games/dev/stratego/src/state.rs` | 21 | 01, Ch 04 |
| `games/dev/stratego/src/rendering.rs` | 302 | 01, Ch 04 |
| `games/dev/stratego/src/input.rs` | 333 | 01, Ch 05 |
| `games/dev/stratego/src/ai.rs` | 116 | 01, Ch 07 |
| `games/dev/stratego/src/tutorial_steps.rs` | 128 | 01, Ch 08 |
