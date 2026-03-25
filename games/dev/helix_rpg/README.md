# Helix RPG

MMORPG prototype consuming real game data from the Helix standardization pipeline (2,681 entities across 26 TOML files).

## Running

```bash
# With Helix data loaded via helix_data plugin:
make helix-import-toml HELIX3D=~/dev/helix/helix_standardization/dist/helix3d/
make helix-rpg

# Without Helix data (fallback enemies):
make helix-rpg
```

## What It Demonstrates

- **Database consumption** -- spawns enemies and NPCs from `Database` rows populated by the helix_data plugin bridge
- **Combat** -- player attacks nearest enemy via `CombatEvent`, real damage with stats from Helix enemy data
- **Quests** -- auto-quest "defeat all enemies" with objective tracking
- **Inventory** -- starting gold, loot drops on enemy defeat
- **NPC interaction** -- `InteractionEvent` for Helix NPCs with `dialogue_set_id`
- **HUD** -- real-time HP, mana, gold, enemy count, quest status

## Data Flow

```
helix_standardization dist/helix3d/*.toml
    |
    v
helix_data plugin -> HelixRegistries -> bridge -> Database (Bevy Resource)
    |
    v
helix_rpg game crate -> queries Database.enemies, Database.npcs at startup
    |
    v
Spawns entities with CombatStatsComponent, NpcComponent, InteractivityComponent
```

## Without Helix Data

If the Database resource is not populated (no helix-import step), the game spawns a single fallback enemy. This ensures the game always runs even without the full Helix pipeline.
