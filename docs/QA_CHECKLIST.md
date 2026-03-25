# QA Checklist

Structured procedure for verifying DJ Engine works end-to-end. Run this before any release, after major changes, or when onboarding a new contributor.

## Pre-Flight Checks

Run these first. If any fail, fix before proceeding.

```bash
make validate    # fmt + clippy + test + contracts + test count (490+ required)
```

Individual steps if validate fails:

```bash
cargo fmt --all --check                           # formatting
cargo clippy --workspace --all-targets -- -D warnings  # lint
cargo test --workspace                             # 497+ tests
make contracts                                     # API surface check
```

- [ ] `make validate` passes
- [ ] Zero clippy warnings
- [ ] Test count >= 490

## Automated Smoke Tests

```bash
make qa    # runs timeout-guarded launch of every game target
```

This verifies no game crashes on startup. It does NOT verify visual correctness.

## Manual Visual Test Cards

### Card 1: RPG Demo

**Command:** `make rpg-demo`

**Setup:** None required.

**Steps:**

1. [ ] Window opens (960x640) with title "DJ Engine -- RPG Demo"
2. [ ] Blue player square visible at center
3. [ ] Green NPC square visible to the right
4. [ ] Red/yellow enemy square visible to the left
5. [ ] HUD text visible at top: HP, Mana, Gold, Quest status
6. [ ] Press Space -- enemy HP decreases in HUD
7. [ ] Continue pressing Space -- enemy defeated, "QUEST COMPLETE" appears
8. [ ] HUD shows updated item counts (slime_gel, possibly health_potion from loot)

**Systems exercised:** Combat, Quest, Inventory, Loot, Input, HUD rendering

**Known issues:**
- Combat is instant (no attack cooldown) -- spam kills
- No movement in this demo (player is stationary)

---

### Card 2: DoomExe

**Command:** `make game`

**Setup:** None required. Assets (MIDI, Lua) are included.

**Steps:**

1. [ ] Window opens (800x600) with title screen
2. [ ] Title screen shows game options
3. [ ] Navigate to overworld (arrow keys or WASD)
4. [ ] Blue player square moves with collision (blocked by gray wall)
5. [ ] Walk to brown "Hamster Narrator" NPC, press E
6. [ ] Dialogue appears with story graph text
7. [ ] Advance dialogue (Space/Enter)
8. [ ] Walk to purple "Glitch Puddle" NPC, press E
9. [ ] Dialogue triggers battle transition
10. [ ] Battle screen shows HP display: "You: 80/80 | Glitch: 40/40"
11. [ ] Press Space to attack -- damage numbers update
12. [ ] Enemy defeated or player defeated -- returns to overworld

**Systems exercised:** Collision, Input, Story Graph, NPC Interaction, Combat (CombatEvent/DamageEvent), State Machine

**Known issues:**
- MIDI audio may be silent (audio backend dependent)
- Hamster narrator expressions change on battle result but no visual sprite

---

### Card 3: Helix RPG

**Command:** `make helix-rpg`

**Setup (minimal):** None -- uses fallback enemy.

**Setup (full):** `make helix-import-toml HELIX3D=~/dev/helix/helix_standardization/dist/helix3d/` first.

**Steps (minimal):**

1. [ ] Window opens (1024x768) with title "Helix RPG -- DJ Engine"
2. [ ] Blue player square at center
3. [ ] Green enemy square visible
4. [ ] HUD shows: HP, Mana, Gold, Enemies: 1, Quest: Accepted
5. [ ] WASD moves player
6. [ ] Space attacks nearest enemy
7. [ ] Enemy defeated -- quest auto-completes, victory text appears

**Steps (full, after import):**

8. [ ] Multiple enemies spawned from Database (up to 3)
9. [ ] NPCs spawned from Database (up to 2, green squares)
10. [ ] Console logs show entity names from Helix data

**Systems exercised:** Database consumption, Combat, Quest, Inventory, Movement, Interaction

---

### Card 4: Stratego

**Command:** `make stratego`

**Steps:**

1. [ ] Window opens with 10x10 board
2. [ ] Setup phase: pieces can be placed on bottom half
3. [ ] Auto-fill places remaining pieces
4. [ ] Game starts: click piece then click target square
5. [ ] AI opponent makes moves
6. [ ] Combat resolves (higher rank wins)
7. [ ] Game ends on flag capture

**Systems exercised:** Grid system, Input, AI, State machine, Rendering

---

### Card 5: Iso Sandbox

**Command:** `make iso`

**Steps:**

1. [ ] Window opens with isometric grid
2. [ ] Hover highlights tile under cursor
3. [ ] Click places tile
4. [ ] Entity palette visible for placement
5. [ ] Grid renders with correct isometric projection

**Systems exercised:** Rendering, Input, Grid, Entity placement

---

### Card 6: Editor

**Command:** `make dev`

**Steps:**

1. [ ] Window opens with egui editor interface
2. [ ] Menu bar visible at top
3. [ ] Panels render (scene, story graph, console, assets)
4. [ ] No crash on idle

**Systems exercised:** egui, Editor plugin, Project mounting

---

## System Verification Matrix

Which games exercise which engine systems:

| System | rpg_demo | doomexe | helix_rpg | stratego | iso |
|--------|----------|---------|-----------|----------|-----|
| Combat (CombatEvent) | X | X | X | | |
| Quest (QuestJournal) | X | | X | | |
| Inventory | X | | X | | |
| Loot (LootDropEvent) | X | | | | |
| NPC Interaction | X | X | X | | |
| Collision | | X | | | |
| Story Graph | | X | | | |
| Sprite Animation | | X | | | |
| Input (ActionState) | X | X | X | X | X |
| Save/Load | | X | | | |
| Movement (MovementIntent) | | X | X | | |
| Grid system | | | | X | X |
| Economy | | | | | |
| Character (Titles) | | | | | |
| Weapon Skills | | | | | |

**Gap:** Economy and Character systems are not exercised by any game yet. Consider adding vendor NPC to helix_rpg.

## Known Acceptable Warnings

These are expected and not bugs:

- `failed to parse serde attribute` (2x) -- ts-rs in helix-data crate, harmless
- `Dynamic collision; treating as kinematic` -- engine converts Dynamic to Kinematic
- MIDI "file not found" if no MIDI assets present in a game crate

## Post-Test Actions

After completing visual tests:

- [ ] Record pass/fail per card above
- [ ] File GitHub issues for any crashes or incorrect behavior
- [ ] Update `docs/CURRENT_GAPS.md` with visual test findings
- [ ] If all cards pass: tag `v0.2.0` release
