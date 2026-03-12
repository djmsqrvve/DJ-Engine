# Engine Guide

## What The Engine Crate Provides

The `engine` crate is the reusable runtime and editor layer. Its public API is
centered around `DJEnginePlugin` plus the `prelude` reexports in
`engine/src/lib.rs`.

The engine plugin bundle currently adds:

- `RenderingPlugin`
- `DJAnimationPlugin`
- `DJAssetPlugin`
- `DJAudioPlugin`
- `DJInputPlugin`
- `DJScenePlugin`
- `StoryGraphPlugin`
- `DJScriptingPlugin`
- `MidiPlugin`
- `DataPlugin`
- `DiagnosticsPlugin` when diagnostics are enabled

## Engine Binaries

### Editor binary

- File: `engine/src/main.rs`
- Behavior:
  - creates a normal Bevy window
  - adds `DefaultPlugins`
  - adds `DJEnginePlugin::default()`
  - adds `EditorPlugin`
  - optionally mounts a project path passed on the command line
  - keeps runtime preview as a separate process launched from `Run Project`

### Runtime preview binary

- File: `engine/src/bin/runtime_preview.rs`
- Purpose:
  - mount a project from `project.json`
  - load startup scene/story data
  - load `data/registry.json` custom documents and preview profiles
  - run the engine-owned `Title -> Dialogue -> Overworld` preview loop
  - support project-scoped continue checkpoints

### Minimal binary

- File: `engine/src/bin/minimal.rs`
- Purpose:
  - bare rendering sanity check
  - no Egui
  - no engine plugin bundle
  - no game logic

## Engine Module Status

### `animation`

- Procedural breathing, blinking, and idle motion systems.
- Includes generic character-default helpers used by sample-game consumers.

### `assets`

- Defines palette and sprite-part data structures.
- Initializes `SpritePartLibrary`.
- Custom asset loaders implemented: `SpritePartLoader` and `PaletteLoader`.

### `audio`

- Message-driven BGM and SFX playback through Bevy audio.
- Exposes `AudioCommand`, `AudioState`, `BgmSource`, and `SfxSource`.
- Startup audio is muted until something raises the master volume.
- Supports fade and crossfade-oriented command inputs used by runtime and tests.

### `core`

- Home of `DJEnginePlugin`.
- Inserts `EngineConfig` and wires the subsystem plugin graph.

### `data`

- Serializable editor/runtime model for projects, scenes, prefabs, databases,
- assets, story graphs, and custom documents.
- Registers a large reflected type surface for Bevy editor and inspector use.
- Includes the registry-driven custom-document platform under
  `engine/src/data/custom.rs`.

### `diagnostics`

- FPS, frame time, entity count, story status, and window size overlay.
- Console logging plugin is active.
- Inspector plugin is currently disabled due to Linux software-rendering
  compatibility issues.

### `editor`

- Egui-based editor plugin.
- Supports `--project`, `--view`, and `--test-mode`.
- `--test-mode` drives an automated UI sequence and exits the app cleanly.
- Tracks mounted-project dirty state across scene, story, project, and custom
  document edits.
- Launches the separate `runtime_preview` process via `Run Project`.
- Includes a `Docs` browser for registry-driven custom documents.

### `input`

- Action-based keyboard input abstraction.
- Current actions: confirm, cancel, menu, and the four directions.

### `midi`

- Generates waveform assets in-memory and provides MIDI/WAV helper utilities
  used by runtime code and tests.
- Provides a small sequencer that emits note on/off messages during playback.

### `rendering`

- Owns the main camera plus engine rendering helpers used by both editor and
  runtime.
- Includes the offscreen/canvas path the sample content builds on.

### `scene`

- Supports background image swapping and fade transitions through
  `ChangeSceneEvent`.

### `scripting`

- Owns the shared `LuaContext`.
- Registers core Lua APIs and handles `ScriptCommand::Load`.
- Runtime preview can resolve an optional project startup `entry_script` from
  the mounted manifest.

### `story_graph`

- Runtime node executor for dialogue, choices, waits, events, scenes, audio, and
  flags.
- Bridges from serialized story data into runtime graph execution.

### `types`

- Defines shared engine error types, result alias, engine config, and diagnostic
  config.

### `runtime_preview`

- Engine-owned playable preview loop separate from the editor shell.
- Uses mounted project startup defaults, preview profiles, and project-scoped
  saves rather than DoomExe state/plugins.

## External Data Integration

DJ-Engine is the primary Rust/Bevy consumer in a DJ multiverse that includes
multiple Helix game variants (Helix2000, potential Helix MMORPG, etc.).

`helix_standardization` (`~/dev/helix/helix_standardization`) provides the shared
data standard across all Helix variants: ~7,800 entities, 22 JSON schemas, 284
categories covering abilities, items, mobs, quests, zones, equipment, mounts,
currencies, and more. It includes a visualizer, CLI, TypeScript/Python SDKs, and
an ecosystem audit for DJ-Engine at `audits/AUDIT_DJ_ENGINE.md`.

The integration point is the custom document registry (`data/registry.json`), not
engine core. A game plugin maps `helix_standardization` categories to DJ-Engine
document kinds registered via `CustomDocumentRegistration<T>`. The engine never
depends on `helix_standardization` at compile time or runtime.

In production, each game is self-contained. `helix_standardization` is a
dev/testing data bridge for keeping data organized and shareable across Helix game
variants regardless of language or engine.

See `docs/LONG_TERM_GOALS.md` for the full integration path and guardrails, and
`docs/ARCHITECTURE.md` for the data flow diagram.

## What To Read First In The Engine

For a fast code walk:

1. `engine/src/lib.rs`
2. `engine/src/core/mod.rs`
3. `engine/src/types.rs`
4. `engine/src/project_mount.rs`
5. `engine/src/data/mod.rs`
6. `engine/src/data/custom.rs`
7. `engine/src/editor/mod.rs`
8. `engine/src/runtime_preview/mod.rs`
9. `engine/src/story_graph/mod.rs`
10. `engine/src/scripting/mod.rs`
