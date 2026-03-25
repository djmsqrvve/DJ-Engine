# DoomExe State Machine

Valid state transitions for the DoomExe game loop.

## States

| State | Description |
| --- | --- |
| TitleScreen | NEW GAME / CONTINUE / QUIT menu |
| Overworld | Player movement, NPC interaction, exploration |
| NarratorDialogue | Story graph executor running (dialogue, branches, events) |
| Battle | Turn-based combat against Glitch |

## Transitions

```
TitleScreen
  |-- [NEW GAME]     --> Overworld        (title.rs: MenuAction::NewGame)
  |-- [CONTINUE]     --> Overworld/Battle  (title.rs: MenuAction::Continue, depends on save)

Overworld
  |-- [Press E + NPC] --> NarratorDialogue (interaction.rs: handle_interactions)

NarratorDialogue
  |-- [GraphComplete] --> Overworld        (dialogue/ui.rs: GraphComplete handler)
  |-- [StartBattle]   --> Battle           (story.rs: handle_story_events, via BattlePending flag)

Battle
  |-- [Enemy defeated] --> Overworld       (battle/systems.rs: handle_battle_damage, victory)
  |-- [Player defeated] --> Overworld      (battle/systems.rs: handle_battle_damage, defeat)
```

## Race Condition Guard

The NarratorDialogue -> Battle transition races with GraphComplete -> Overworld
because both fire in the same executor tick. This is guarded by the `BattlePending`
resource flag:

1. `handle_story_events` sets `BattlePending(true)` + `NextState::Battle`
2. `update_dialogue_ui` checks `BattlePending` before setting Overworld
3. `setup_battle_entities` clears `BattlePending(false)` on battle entry

## Auto-Save Points

- `OnEnter(Overworld)` triggers `auto_save` (overworld/mod.rs)

## Logging

Every state transition is logged with source file context via `info!()`.
Search logs for "Story Event:", "Battle:", or "Starting New Game" to trace transitions.
