# DoomExe Game Guide

## What `doomexe` Is

`games/dev/doomexe` is the current playable sample game crate. It is a
prototype game built on top of `DJEnginePlugin` with its own
state machine, UI, Lua extensions, and gameplay modules.

## App Composition

`games/dev/doomexe/src/main.rs` does the following:

- creates a normal Bevy window
- uses nearest-neighbor image scaling
- adds `DJEnginePlugin::default()`
- adds `GameScriptingPlugin`
- initializes `GameState`
- adds the game plugins in this order:
  - `TitlePlugin`
  - `StoryPlugin`
  - `HamsterPlugin`
  - `OverworldPlugin`
  - `HudPlugin`
  - `DialoguePlugin`
  - `BattlePlugin`
  - `GameAssetsPlugin`

## Game State Model

Current states:

- `TitleScreen`
- `Overworld`
- `NarratorDialogue`
- `Battle`

Current state transitions that matter:

- Title `NEW GAME` moves to `NarratorDialogue`.
- Title `CONTINUE` resumes the game's own save flow.
- `StoryEvent` with id `StartBattle` moves the game into `Battle`.

## Game Modules

### `title`

- Builds the title-screen UI.
- Uses engine input actions rather than raw key handling.
- Can start a new game, jump to overworld, or quit.

### `story`

- Holds `StoryState` and handles runtime `StoryEvent`s.
- The current important event bridge is `StartBattle`.

### `hamster`

- Spawns the hamster narrator entity and drives procedural behavior.
- Includes breathing, blinking, idle motion, expression changes, corruption, and
  debug input helpers.
- Debug controls are documented in `games/dev/doomexe/src/hamster/mod.rs`.

### `overworld`

- Spawns a simple prototype overworld.
- Reuses the engine main camera and adds camera follow.
- Spawns a player, two NPCs, and a flat floor sprite.

### `dialogue`

- Owns narrator dialogue UI state and interaction systems.
- Includes typewriter behavior and choice-mode support.

### `battle`

- Creates and cleans up battle UI.
- Uses `BattleResultEvent` for result handling.

### `hud`

- Provides HUD-related code such as tracker/minimap pieces.

### `scripting`

- Extends the engine Lua runtime with hamster-specific globals.
- Runs startup scripts at app startup if a known script path exists.

### `assets`

- Placeholder plugin at the moment.
- Present so asset-specific initialization can grow later.

### `types`

- Holds game-specific data types such as hamster expression/state structures used
  by the scripting and gameplay layers.

## Current Asset Reality

The committed `doomexe` asset tree is currently small:

- `assets/music`
- `assets/palettes`
- `assets/scripts`

This matters because older docs may imply a much richer checked-in asset tree
than the repo currently contains.
