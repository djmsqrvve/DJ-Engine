# DoomExe

Dark-fantasy horror JRPG prototype built on DJ Engine. Features a corrupted hamster narrator that reacts to player choices, remembers previous runs, and visually degrades as the story darkens.

DoomExe is primarily a sandbox for developing DJ Engine. It exercises the engine's core systems and will eventually become a full game.

## Running

```bash
make game    # or: make doom
```

## What It Demonstrates

- **Procedural animation** -- breathing, blinking, expression-driven hamster assembly
- **Palette corruption** -- real-time palette swaps driven by a corruption float (0-100)
- **Story graph** -- JSON-serializable dialogue, branching, and scripted actions
- **Lua scripting** -- runtime game logic via mlua for content-heavy iteration
- **Overworld** -- player movement, camera, NPC interaction
- **Battle system** -- turn-based combat prototype
- **HUD** -- minimap, tracker, in-game UI

## Milestone 1: Hamster Narrator

A single animated scene with a procedural hamster: a large, candle-lit, slightly corrupted hamster portrait delivering text.

Requirements:
- Procedural hamster assembly from sprite parts (body, head, ears, eyes, mouth, paws)
- Breathing (squash/stretch), blinking (timer), idle sway (noise)
- Expression system (neutral, amused, angry, corrupted)
- Corruption influences palette shift, screen jitter, scanline intensity
- Internal resolution 320x240, upscaled with CRT post-processing
- Lua scripting for dialogue branching and corruption changes
- Hot-reload for scripts and palette config

## Project Layout

```text
games/dev/doomexe/
├── Cargo.toml
├── assets/
│   ├── music/          # MIDI files
│   ├── palettes/       # Palette definitions (JSON)
│   └── scripts/        # Lua scripts
├── docs/
│   └── hamster_milestone.md
└── src/
    ├── main.rs
    ├── assets/         # Asset loading
    ├── battle/         # Turn-based combat
    ├── dialogue/       # Dialogue UI
    ├── hamster/        # Narrator assembly, animation, corruption
    ├── hud/            # Minimap, tracker
    ├── overworld/      # Player, camera, NPC interaction
    ├── scripting/      # Lua integration
    ├── story.rs        # Story graph integration
    ├── state.rs        # Game state management
    ├── title.rs        # Title screen (NEW GAME / CONTINUE / QUIT)
    └── types.rs        # Game-specific types
```
