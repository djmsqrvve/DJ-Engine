# Workspace Map

## Root Layout

```text
Cargo.toml
rust-toolchain.toml
Makefile
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
  - Pins Rust `1.94.0` with `clippy` and `rustfmt`.
- `Makefile`
  - Current command interface for common game, editor, test, lint, format,
    build, and asset generator workflows.
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

- Shared engine code used by the editor, runtime preview, and sample games.
- Exposes the `DJEnginePlugin` bundle and the `prelude`.
- Also owns the mounted-project path, custom-document registry, and runtime
  preview flow.

Important entrypoints:

- `engine/src/lib.rs`
- `engine/src/core/mod.rs`
- `engine/src/main.rs`
- `engine/src/bin/minimal.rs`
- `engine/src/bin/runtime_preview.rs`
- `engine/src/project_mount.rs`
- `engine/src/data/custom.rs`

### `games/dev/doomexe`

Purpose:

- Current sample game crate.
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
- `engine/src/bin/runtime_preview.rs`
  - Engine-owned playable preview for mounted projects.
- `games/dev/doomexe/src/main.rs`
  - Game binary entrypoint.
- `engine/src/editor/` — decomposed into submodules (March 2026):
  - `mod.rs` — thin orchestrator (62 lines), re-exports submodules
  - `panels.rs` — 5 UI panel draw functions (390 lines)
  - `views.rs` — `draw_grid`, `draw_story_graph` (304 lines)
  - `scene_io.rs` — save/load I/O, `world_to_scene`, `load_scene_into_editor` + tests
  - `types.rs` — `EditorState`, `EditorView`, `BrowserTab`, resources, color constants
  - `plugin.rs` — `EditorPlugin` + lifecycle systems
  - `validation.rs` — `ValidationState`, `draw_validation_panel`
- `engine/src/data/`
  - Serializable project, scene, database, story, prefab, and custom-document
    layer.
- `engine/src/story_graph/` — decomposed into submodules (March 2026):
  - `mod.rs` — `StoryGraphPlugin` + re-exports (33 lines)
  - `types.rs` — all types/resources/events/impls + unit tests (300 lines)
  - `executor.rs` — `execute_graph` system + helpers (223 lines)
- `engine/src/scripting/`
  - Engine-level Lua runtime and API registration.
- `engine/src/runtime_preview/`
  - Generic `Title -> Dialogue -> Overworld` preview loop plus continue support.

## Current Asset Layout

Committed game asset directories at the time of writing:

- `games/dev/doomexe/assets/music`
- `games/dev/doomexe/assets/palettes`
- `games/dev/doomexe/assets/scripts`

Important nuance:

- The asset generator expects `games/dev/doomexe/assets/sprites/hamster_parts`
  if hamster sprite source files exist, but that tree is not currently committed
  in the repo snapshot this suite describes.
