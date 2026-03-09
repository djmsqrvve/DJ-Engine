# Workspace Map

## Root Layout

```text
Cargo.toml
rust-toolchain.toml
dj
.devcontainer/
.github/workflows/
engine/
games/dev/doomexe/
tools/asset_generator/
docs/
```

## Top-Level Files And Folders

- `Cargo.toml`
  - Workspace manifest, member list, shared dependencies, and build profiles.
- `rust-toolchain.toml`
  - Pins Rust `1.93.1` with `clippy` and `rustfmt`.
- `dj`
  - Helper script for the common game, editor, test, lint, format, build, and
    asset generator commands.
- `.devcontainer/`
  - Codespaces and devcontainer definition, lifecycle scripts, and runtime
    warmup helper.
- `.github/workflows/ci.yml`
  - Ubuntu-based compile validation workflow.
- `docs/`
  - Mixed set of current docs, older design docs, and handoff notes.

## Crates

### `engine`

Purpose:

- Shared engine code used by both the editor and the game.
- Exposes the `DJEnginePlugin` bundle and the `prelude`.

Important entrypoints:

- `engine/src/lib.rs`
- `engine/src/core/mod.rs`
- `engine/src/main.rs`
- `engine/src/bin/minimal.rs`

### `games/dev/doomexe`

Purpose:

- Main playable game crate and current default workspace member.
- Uses the shared engine plus its own game-state, UI, Lua, and gameplay modules.

Important entrypoints:

- `games/dev/doomexe/src/main.rs`
- `games/dev/doomexe/src/state.rs`
- `games/dev/doomexe/src/story.rs`

### `tools/asset_generator`

Purpose:

- Workspace utility for generating MIDI assets and optionally fixing hamster
  sprite assets if those source files are present.

Important entrypoint:

- `tools/asset_generator/src/main.rs`

## Important Runtime And Tooling Files

- `engine/src/main.rs`
  - Editor binary entrypoint.
- `engine/src/bin/minimal.rs`
  - Minimal graphics sanity check binary.
- `games/dev/doomexe/src/main.rs`
  - Game binary entrypoint.
- `engine/src/editor/mod.rs`
  - Egui editor plugin and test-mode automation.
- `engine/src/data/`
  - Serializable project, scene, database, story, and prefab layer.
- `engine/src/story_graph/mod.rs`
  - Runtime story execution layer.
- `engine/src/scripting/`
  - Engine-level Lua runtime and API registration.

## Current Asset Layout

Committed game asset directories at the time of writing:

- `games/dev/doomexe/assets/music`
- `games/dev/doomexe/assets/palettes`
- `games/dev/doomexe/assets/scripts`

Important nuance:

- The asset generator expects `games/dev/doomexe/assets/sprites/hamster_parts`
  if hamster sprite source files exist, but that tree is not currently committed
  in the repo snapshot this suite describes.

