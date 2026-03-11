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
- Small and direct; these systems are actively used by the hamster prototype.

### `assets`

- Defines palette and hamster-part data structures.
- Initializes `HamsterPartLibrary`.
- Custom asset loaders implemented: `HamsterPartLoader` and `PaletteLoader`.

### `audio`

- Message-driven BGM and SFX playback through Bevy audio.
- Exposes `AudioCommand`, `AudioState`, `BgmSource`, and `SfxSource`.
- Startup audio is muted until something raises the master volume.
- The command surface mentions crossfade and fade-out values, but the current
  implementation does not yet perform a full crossfade system.

### `core`

- Home of `DJEnginePlugin`.
- Inserts `EngineConfig` and wires the subsystem plugin graph.

### `data`

- Serializable editor/runtime model for projects, scenes, prefabs, databases,
  assets, and story graphs.
- Registers a large reflected type surface for Bevy editor and inspector use.

### `diagnostics`

- FPS, frame time, entity count, story status, and window size overlay.
- Console logging plugin is active.
- Inspector plugin is currently disabled due to Linux software-rendering
  compatibility issues.

### `editor`

- Egui-based editor plugin.
- Supports `--project`, `--view`, and `--test-mode`.
- `--test-mode` drives an automated UI sequence and exits the app cleanly.

### `input`

- Action-based keyboard input abstraction.
- Current actions: confirm, cancel, menu, and the four directions.

### `midi`

- Generates waveform assets in-memory and loads a MIDI file from the doomexe
  assets folder.
- Provides a small sequencer that emits note on/off messages during playback.

### `rendering`

- Currently sets up the main camera and exposes `MainCamera`, `GAME_WIDTH`, and
  `GAME_HEIGHT`.
- Offscreen rendering, upscaling, and CRT post-processing remain TODOs.

### `scene`

- Supports background image swapping and fade transitions through
  `ChangeSceneEvent`.

### `scripting`

- Owns the shared `LuaContext`.
- Registers core Lua APIs and handles `ScriptCommand::Load`.

### `story_graph`

- Runtime node executor for dialogue, choices, waits, events, scenes, audio, and
  flags.
- Bridges from serialized story data into runtime graph execution.

### `types`

- Defines shared engine error types, result alias, engine config, and diagnostic
  config.

## What To Read First In The Engine

For a fast code walk:

1. `engine/src/lib.rs`
2. `engine/src/core/mod.rs`
3. `engine/src/types.rs`
4. `engine/src/editor/mod.rs`
5. `engine/src/data/mod.rs`
6. `engine/src/story_graph/mod.rs`
7. `engine/src/scripting/mod.rs`

