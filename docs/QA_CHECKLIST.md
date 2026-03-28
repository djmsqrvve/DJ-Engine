# QA Checklist

Structured procedure for verifying DJ Engine works end-to-end. Run this before any release, after major changes, or when onboarding a new contributor.

## Pre-Flight Checks

Run these first. If any fail, fix before proceeding.

```bash
make validate    # fmt + clippy + test + contracts + test count (510+ required)
```

Individual steps if validate fails:

```bash
cargo fmt --all --check                           # formatting
cargo clippy --workspace --all-targets -- -D warnings  # lint
cargo test --workspace                             # 539+ tests
make contracts                                     # API surface check
```

- [ ] `make validate` passes
- [ ] Zero clippy warnings
- [ ] Test count >= 510

## Quick Status Check

Run this for a fast health snapshot without the full QA walkthrough:

```bash
make status    # ~2 min: test count, clippy, contracts, doc staleness, smoke tests
```

**When to use:**
- After a multi-commit session, before pushing
- When resuming after a break
- Before starting a new QA session

| Check | When | Time |
|-------|------|------|
| `make status` | Daily / after sessions | ~2 min |
| `make validate` | Before push | ~5 min |
| `make qa` | Before release | ~7 min |
| Manual cards (below) | Monthly or after major feature | ~30 min |

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
11. [ ] Press Space to attack -- damage numbers update, attack cooldown gates spam
12. [ ] Enemy attacks back on 1.5s timer (AI-driven, not instant counterattack)
13. [ ] Enemy defeated -- returns to overworld
14. [ ] Player defeated (HP reaches 0) -- Game Over screen appears (red, "GAME OVER" text)
15. [ ] Press Space on Game Over screen -- returns to title

**Systems exercised:** Collision, Input, Story Graph, NPC Interaction, Combat (CombatEvent/DamageEvent), State Machine, Attack Cooldown, Game Over

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

**Dashboard check (after full setup):**

```bash
make helix-dashboard HELIX3D=~/dev/helix/helix_standardization/dist/helix3d/
```

- [ ] Boxed dashboard renders with check lines
- [ ] TOML Coverage shows [OK] 22/22
- [ ] API Health shows [OK] connected (if API on port 6800 is running) or [--] not running
- [ ] Data Freshness shows age in minutes
- [ ] Remote Validation shows sample result

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
| Attack Cooldown | X | X | X | | |
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
| Game Over | | X | | | |
| Debug Console (F1) | X | X | X | X | X |
| Objective Navigator | | X | | | |
| Grid system | | | | X | X |
| Economy | | X | | | |
| Character (Titles) | | X | | | |
| Weapon Skills | | | | | |
| Zone Transition | | X | | | |
| Particles | X | X | X | | |
| Screen FX | | X | X | | |

**Gap:** Weapon Skills not exercised yet. Economy and Character now exercised by DoomExe.

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

---

## Test Session Log

### Session: 2026-03-25

**Pre-flight:** 499 tests, 0 failures, 0 clippy warnings

| Card            | Result     | Notes                                                                                                                                                                                                                                                              |
| --------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1: RPG Demo     | PASS       | Window opens, HUD visible, Space attacks, HP drops, quest completes. User initially read own HP instead of enemy HP -- HUD working correctly.                                                                                                                      |
| 2: DoomExe      | PARTIAL    | Title screen loads. New Game -> hamster encounter works. Battle triggers but resolves instantly (one-hit kill). Objective HUD stuck on "Find the Narrator" after narrator conversation -- fixed via StoryFlags + StoryState bridge. Objective navigator added.        |
| 3: Helix RPG    | NOT TESTED | Blocked on Card 2 fixes consuming session time.                                                                                                                                                                                                                    |
| 4: Stratego     | NOT TESTED |                                                                                                                                                                                                                                                                    |
| 5: Iso Sandbox  | NOT TESTED |                                                                                                                                                                                                                                                                    |
| 6: Editor       | NOT TESTED |                                                                                                                                                                                                                                                                    |

**Fixes shipped during session:**

- `af91017` debug(doomexe): add dialogue input diagnostics for stuck state
- `146c7e1` feat: runtime debug console (F1) + DoomExe log filter
- `08fc8e9` feat: objective navigator -- [ and ] keys cycle through checkpoints
- `7c43e68` fix(doomexe): HUD objective reads engine StoryFlags + game StoryState
- `7b099da` fix(doomexe): 3 contract issues -- flags, database, input

**Issues found:**

- Economy and Character systems not exercised by any game yet
- Cards 3-6 have never been manually tested

### Session: 2026-03-27

**Pre-flight:** 539 tests, 0 failures, 0 clippy warnings

| Card | Result | Notes |
|------|--------|-------|
| 1: RPG Demo | | |
| 2: DoomExe | | Re-test: GameOver screen, enemy AI cooldowns, NPC highlights, battle pacing |
| 3: Helix RPG | | First manual test |
| 4: Stratego | | First manual test |
| 5: Iso Sandbox | | First manual test |
| 6: Editor | | First manual test |

**Fixes shipped during session:**

**Issues found:**
