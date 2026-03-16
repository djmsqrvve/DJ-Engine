# Tutorial System — Project Tracker

Tracking document for the DJ-Engine interactive tutorial overlay system.

## Overview

A data-driven tutorial overlay that guides new users through making their first game. Each step highlights a target panel with a dim cutout effect and a floating instruction window. Steps can auto-advance when the user completes an action (e.g. places an entity, switches views).

## Architecture

| File | Role |
|------|------|
| `engine/src/editor/tutorial.rs` | Types, overlay rendering, completion detection |
| `engine/template/tutorials/first_game.json` | 8-step "Make Your First Game" tutorial definition |
| `engine/src/editor/mod.rs` | Captures panel rects, calls `draw_tutorial_overlay` |
| `engine/src/editor/panels.rs` | "Tutorial" button in top menu bar |
| `engine/src/editor/plugin.rs` | Registers `TutorialState` resource |

## Current Tutorial: "Make Your First Game" (8 steps)

| # | Title | Target | Completion | Status |
|---|-------|--------|------------|--------|
| 1 | Welcome to DJ Engine | FullScreen | Manual | Done |
| 2 | The Menu Bar | TopPanel | Manual | Done |
| 3 | The Browser Panel | LeftPanel | Manual | Done |
| 4 | Place an Entity | CentralPanel | EntityPlaced | Done |
| 5 | Switch to Story Graph | TopPanel | ViewChanged | Done |
| 6 | Select a Story Node | CentralPanel | NodeSelected | Done |
| 7 | The Inspector | RightPanel | Manual | Done |
| 8 | Run Your Game | FullScreen | Manual | Done |

## Completion Conditions

| Condition | Trigger | Implemented |
|-----------|---------|-------------|
| `Manual` | User clicks Next/Done | Yes |
| `ViewChanged(String)` | `EditorUiState.current_view` matches | Yes |
| `TabChanged(String)` | `EditorUiState.browser_tab` matches | Yes |
| `EntityPlaced` | `selected_entities` is non-empty | Yes |
| `NodeSelected` | `selected_node_id` is Some | Yes |

## Implemented Features

- [x] JSON-driven tutorial definitions (`include_str!` embedded)
- [x] Dim overlay with bright cutout around target panel
- [x] Floating instruction window positioned opposite to highlighted panel
- [x] Back / Next / Skip Tutorial buttons
- [x] Auto-advance on completion conditions
- [x] Progress indicator ("Step N of M")
- [x] Tutorial button in top menu bar
- [x] Animated transitions between steps (lerp cutout to new panel)
- [x] Pulsing border animation on highlighted panel
- [x] Arrow/pointer from instruction window to target panel
- [x] Completion persistence (save completed tutorial IDs to disk)
- [x] 6 unit tests (deserialization, panel rects, window positioning, persistence)

## Planned Features

- [ ] Multiple tutorial definitions (load by name, not just "first game")
- [ ] Tutorial selection menu / launcher dialog
- [ ] Rich text formatting in step body (bold, inline code, links)
- [ ] Image/screenshot support in tutorial steps
- [ ] "Don't show again" / auto-show on first launch
- [ ] Custom tutorials loaded from project files (not just embedded)
- [ ] Conditional steps (skip step if user already did the action)
- [ ] Tooltip-style micro-tutorials for individual UI elements
- [ ] Localization support (tutorial text in multiple languages)
- [ ] Tutorial recording/authoring mode (click panels to define steps)
- [ ] Sound effects / audio cues on step advance

## Related Systems

- **Panel Export** (`panel_export.rs`): Structured data export for all panels, committed alongside the tutorial system.
- **Editor Prefs** (`prefs.rs`): Persistent storage for user settings and tutorial completion.
- **EditorUiState**: Provides the state fields that completion conditions check against.
- **EditorView / BrowserTab**: Enums used for `ViewChanged` and `TabChanged` conditions.

## Design Decisions

1. **Overlay approach over visibility toggling**: Rather than hiding/showing panels, we dim the full screen and cut out the target panel. This keeps the full editor functional during the tutorial.
2. **Snapshot pattern for borrow safety**: `draw_tutorial_overlay` clones needed data into an enum snapshot before releasing the immutable `World` borrow, allowing subsequent mutation.
3. **JSON over Rust for content**: Tutorial steps are defined in JSON so non-programmers can author tutorials without recompiling.
4. **Embedded via `include_str!`**: The built-in tutorial ships with the binary. Custom tutorials will load from project files.
