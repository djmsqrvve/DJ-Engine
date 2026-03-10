# dj_engine Technical Specification

## 🚀 Project Status: Milestone 1 (Hamster Narrator)
**Current Phase:** Phase 1 of 5 (Story Graph & Foundation)
**Core Tech:** Rust, Bevy 0.18, Lua 5.4 (mlua), Egui

## 🗺️ Documentation Map
*   **`docs/MASTER-DOCUMENTATION-INDEX.md`**: The entry point.
*   **`docs/Architecture_Specification.json`**: The canonical high-level spec.
*   **`docs/complete-detailed-docs.md`**: Implementation guides & code templates.
*   **`docs/Game_Engine_Technical_Roadmap.md`**: The 20-week execution plan.

## 🏛️ Core Architecture Pillars

### 1. Universal Unit (`Actor`)
Standardized ECS entity structure for characters across genres (JRPG/RTS).
*   **Required Components:** `Actor` (ID/Name), `Stats` (HP/Mana), `Transform`, `Visibility`.
*   **Optional Layers:**
    *   `DirectInput` + `PartyLeader` (JRPG)
    *   `RTSUnit` + `PathfindingAgent` (RTS)
*   **Philosophy:** Use `#[require(Component)]` to enforce dependencies.

### 2. Story Graph System (`engine/src/story_graph`)
A directed graph of nodes driving narrative flow.
*   **Resource:** `StoryGraph` (holds `HashMap<NodeId, StoryNode>`).
*   **Executor:** `GraphExecutor` (manages state: `Running`, `WaitingForInput`, etc.).
*   **Events:** `StoryFlowEvent` (Executor -> UI), `StoryInputEvent` (UI -> Executor).
*   **Node Types:**
    *   `Dialogue`: `{ speaker, text, portrait, next }`
    *   `Choice`: `{ prompt, options: Vec<{text, next}> }`
    *   `Branch`: `{ flag, if_true, if_false }`
    *   `Event`: `{ event_id, payload }` (Triggers Lua or custom logic)
    *   `Scene`, `Audio`, `Wait`, `SetFlag`

### 3. Scripting Layer (`engine/src/scripting`)
Lua integration for game logic and cutscenes.
*   **Engine:** `mlua` 0.9.
*   **Resource:** `LuaContext` (thread-safe `Mutex<Lua>`).
*   **Pattern:** Game plugins register their own APIs (e.g., `unit:move_to`).
*   **FFI:** Rust exposes functions; Lua scripts are assets.

### 4. Data Serialization (`engine/src/data`)
**Strict Separation:** Runtime ECS components != Serialized Data.
*   **Runtime:** `bevy::prelude::Component`
*   **Serialized:** `serde::Serialize` structs (e.g., `StoryGraphData`, `SceneData`).
*   **Loader:** `GraphExecutor::load_from_data()` bridges the gap.

## 🛠️ Developer Cheatsheet

### Common Commands
```bash
# Run the main game (DoomExe)
cargo run -p doomexe

# Run all tests
cargo test --workspace

# Check for compilation errors
cargo check --workspace
```

### Key File Locations
*   **Engine Lib:** `engine/src/lib.rs`
*   **Story Graph Logic:** `engine/src/story_graph/mod.rs`
*   **Hamster Component:** `games/dev/doomexe/src/hamster/components.rs`
*   **Game Entry:** `games/dev/doomexe/src/main.rs`

### "Start to Finish" Roadmap Summary
1.  **Phase 1 (Now):** Story Graph foundation, basic UI, "Hamster" narrator.
2.  **Phase 2:** Director system (Camera, TimeControl, Event Sequencing).
3.  **Phase 3:** Universal Unit (JRPG + RTS shared components).
4.  **Phase 4:** Standardized Lua API (write once, run anywhere).
5.  **Phase 5:** Visual Editors (Story Graph Editor in Egui).

## ⚠️ Critical Constraints
1.  **Do not** mix logic into `data` structs. They are for storage only.
2.  **Do not** call Bevy queries from Lua. Use events or deferred commands.
3.  **Always** check `docs/Architecture_Specification.json` before creating new subsystems.
