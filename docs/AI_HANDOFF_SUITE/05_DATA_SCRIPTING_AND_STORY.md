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

Primary loader functions reexported in the engine prelude:

- `load_project`
- `load_scene`
- `load_database`
- `load_story_graph`

Important data modules:

- `project.rs`
- `scene.rs`
- `database.rs`
- `story.rs`
- `spawner.rs`
- `loader.rs`

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
- `Start`, `Dialogue`, `Choice`, `Action`, and `End` variants are currently the
  clearest implemented path.
- Unhandled story variants presently collapse to `StoryNode::End`, so agent work
  in this area should be careful about assuming full graph coverage.

## Engine Scripting Layer

The shared engine scripting surface lives in `engine/src/scripting/`.

Current shape:

- `LuaContext` stores the shared Lua VM.
- `DJScriptingPlugin` creates the VM and registers core APIs.
- `ScriptCommand::Load { path }` reads a Lua file and executes it.
- The engine exposes helper registration functions for shared generic state.

This is a straightforward file-execution model. It is not a large asset-backed
script loader yet.

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

The editor plugin uses the data/story layer directly:

- it stores `ProjectMetadata`
- it stores `ActiveStoryGraph`
- it can switch between level and story-graph views
- on entering play mode it looks for `assets/scripts/hamster_test.lua` under the
  mounted project path and sends `ScriptCommand::Load`

That means editor "play" behavior and game startup scripting overlap around the
same `hamster_test.lua` entrypoint pattern.

## Asset Generator Role

The asset generator is intentionally small:

- writes a generated MIDI file to `games/dev/doomexe/assets/music`
- optionally repairs hamster sprite image files if those source files exist

It assumes a workspace-root working directory.

