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
