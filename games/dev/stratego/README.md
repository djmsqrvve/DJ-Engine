# Stratego

10x10 board game with 8 piece types and an AI opponent, built on DJ Engine. This is the tutorial game -- a complete 8-chapter walkthrough covers building it from scratch.

## Running

```bash
make stratego
```

## Features

- 10x10 grid board using the engine's reusable `Grid<T>` from `dj_engine::data::Grid`
- 8 piece types with rank-based combat resolution
- Setup phase for placing pieces on your half of the board
- AI opponent with strategic piece evaluation
- Turn-based state machine (Setup -> Play -> GameOver)
- Interactive tutorial overlay with step-by-step guidance

## Tutorial

The full 8-chapter walkthrough is at `docs/tutorials/01-build-a-board-game/`:

1. Project Setup
2. Board and Grid
3. Pieces and Rules
4. Rendering
5. Input and Setup
6. State Machine
7. AI Opponent
8. Polish
