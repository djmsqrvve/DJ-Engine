# RPG Demo

Minimal reference game demonstrating all DJ Engine runtime systems working together in a single gameplay loop.

## Running

```bash
make rpg-demo
```

## What It Demonstrates

This ~300 line game wires every major engine system into a playable flow:

- **Combat** -- player attacks enemy via `CombatEvent`, damage resolved with crit/defense/variance
- **Quests** -- quest accepted at startup, objective progressed on enemy kill, auto-completes
- **Inventory** -- starting gold, loot items added on enemy defeat, HUD shows counts
- **NPC Interaction** -- NPC with `InteractivityComponent` + `NpcComponent`, reacts to `InteractionEvent`
- **Loot** -- enemy has `loot_table_id`, defeat fires `LootDropEvent`, items added to inventory
- **Status Effects** -- `StatusEffectsComponent` and `AbilityCooldownsComponent` on player
- **Input** -- `ActionState` reads Confirm (Space) for attacks
- **Collision** -- `MovementIntent` + `InteractionSource` on player entity
- **HUD** -- real-time display of HP, mana, gold, items, quest status

## Game Flow

1. Player spawns with stats, cooldowns, and 50 gold
2. NPC "Village Elder" stands nearby with quest "Slay Slimes"
3. Quest auto-accepted at startup with objective: kill 1 slime
4. Press Space to attack the green slime
5. On defeat: loot drops (slime gel + maybe health potion), quest progresses
6. Quest completes, victory screen shown

## For SDK Developers

This game is the **reference implementation** for DJ Engine's gameplay systems. Every system usage follows the recommended pattern from the Game Developer's Guide. Use it as a starting point for your own game.

Key patterns shown:
- Explicit imports (no wildcard `prelude::*`) to avoid name conflicts
- `MessageWriter<T>` for sending events, `MessageReader<T>` for receiving
- Bevy `States` for game phase transitions
- Engine resources (`QuestJournal`, `Inventory`, `ActionState`) as system parameters
