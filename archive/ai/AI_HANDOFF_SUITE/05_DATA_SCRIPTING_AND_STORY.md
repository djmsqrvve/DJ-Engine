# Data, Scripting, And Story

## Data Layer

The `engine/src/data/` tree is the repo's serializable interchange layer. It is
separate from live ECS components so the editor and runtime can share the same
JSON-backed structures.

Key responsibilities:

- project configuration
- scene composition
- asset indexing and prefabs
- gameplay database rows
- story graph serialization
- scene spawning support
- registry-driven custom document loading and validation

Important loader and path helpers include:

- `load_project`
- `load_scene`
- `load_database`
- `load_story_graph`
- mounted-project helpers in `engine/src/project_mount.rs`
- custom-document loading and validation in `engine/src/data/custom.rs`

Important data modules:

- `project.rs`
- `scene.rs`
- `database.rs`
- `story.rs`
- `spawner.rs`
- `loader.rs`
- `custom.rs`

Key current project facts:

- `ProjectPaths` now includes a `data` root, defaulting to `data`.
- mounted projects can define startup defaults for scene, story graph, and
  optional entry script
- custom authored documents are discovered from `data/registry.json`

## Story Graph Split

There are two distinct story layers:

### Serialized story data

- lives under `engine/src/data/story.rs`
- stores editor-friendly graph data such as node ids, localized text, node
  variants, conditions, and effects

### Runtime story execution

- lives under `engine/src/story_graph/mod.rs`
- uses `StoryGraph`, `StoryNode`, `GraphExecutor`, and message types for runtime
  flow

Important current behavior:

- `GraphExecutor::load_from_data()` bridges editor data into runtime nodes.
- The clearest implemented runtime path remains dialogue/choice/action-oriented
  flows that can drive the engine preview loop.
- Runtime preview now uses these graphs directly as part of its mounted-project
  startup flow.

## Engine Scripting Layer

The shared engine scripting surface lives in `engine/src/scripting/`.

Current shape:

- `LuaContext` stores the shared Lua VM.
- `DJScriptingPlugin` creates the VM and registers core APIs.
- `ScriptCommand::Load { path }` reads a Lua file and executes it.
- The engine exposes helper registration functions for shared generic state.

This is still a straightforward file-execution model rather than a large
asset-backed script pipeline.

Mounted projects can now also set `project.settings.startup.entry_script`, and
runtime preview will attempt to resolve and load that script relative to the
project root.

## DoomExe Scripting Layer

`games/dev/doomexe/src/scripting/mod.rs` extends the shared Lua environment with
hamster-specific globals.

Current hamster Lua globals:

- `set_corruption`
- `get_corruption`
- `set_expression`
- `get_expression`

The game also:

- creates a shared Rust buffer for Lua-to-ECS state exchange
- syncs that buffer into the `HamsterNarrator` ECS state each frame
- runs a startup script if one of these files exists:
  - `games/dev/doomexe/assets/scripts/hamster_test.lua`
  - `assets/scripts/hamster_test.lua`

## Editor Interaction With Data And Scripts

The editor plugin now uses the mounted-project and data layers directly:

- it stores `MountedProject`
- it stores `ActiveStoryGraph`
- it stores `LoadedCustomDocuments`
- it can switch between level and story-graph views
- it includes a `Docs` browser for custom documents discovered from
  `data/registry.json`
- `Run Project` auto-saves and launches the separate `runtime_preview` process
- `Preview Graph` remains an editor-only Story Graph tool

That means the editor no longer tries to behave like full project runtime. The
runtime preview binary is now responsible for mounted-project startup scripts.

## Asset Generator Role

The asset generator is intentionally small:

- writes a generated MIDI file to `games/dev/doomexe/assets/music`
- optionally repairs hamster sprite image files if those source files exist

It assumes a workspace-root working directory.
