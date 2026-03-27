# DJ-Engine

> Helix integration context: see `~/dev/helix/docs/HELIX_AGENT_BRIEFING.md`

## Overview

Custom 2D game engine built with Rust and Bevy 0.18. Plugin-based architecture with 25 registered plugins and typed contracts. 539 tests, zero clippy warnings. 18 runtime systems. 5 game crates. Tile editor complete. Consumes Helix MMORPG data through `plugins/helix_data/`.

## Stack

- Rust 1.94.0 (pinned), Bevy 0.18 (custom framework), mlua 0.10 (Lua 5.4)

## Key Commands

```bash
make test                            # 539 workspace tests
make lint                            # clippy (zero warnings enforced)
make fmt                             # format check
make quality-check                   # fmt + clippy + test
make validate                        # quality-check + contracts + test count (>=510)
make qa                              # validate + smoke-test all 5 games
make contracts                       # print plugin contract dashboard
make dev                             # launch engine editor
make game                            # launch RPG Demo
make doomexe                         # launch DoomExe (hamster narrator JRPG)
make stratego                        # launch Stratego (10x10 board game)
make iso                             # launch Iso Sandbox
make helix-rpg                       # launch Helix RPG (MMORPG data consumer)

# Helix integration
make helix-import-toml HELIX3D=~/dev/helix/helix_standardization/dist/helix3d/
make helix-dashboard HELIX3D=~/dev/helix/helix_standardization/dist/helix3d/
make helix-editor PROJECT=/tmp/helix_project
make helix-preview PROJECT=/tmp/helix_project
```

## Architecture

- **25 plugins** registered via `DJEnginePlugin`, each with a typed `PluginContract`
- **18 runtime systems**: combat, combat FX, quests, inventory, interaction, animation, spawner, status effects, ability cooldowns, loot, economy, character, debug console, objective navigator, zone, collision, audio, save/load
- **Lua API**: 8 tables (ecs, quest, combat, inventory, economy, character) + global log/warn/error
- **5 game crates**: doomexe (JRPG), stratego (board game), iso_sandbox (isometric), rpg_demo (SDK reference), helix_rpg (MMORPG data consumer)
- **Helix data plugin** (`plugins/helix_data/`):
  - `HelixRegistries` Bevy Resource: 22 typed `Registry<T>` collections (2,868 entities when loaded)
  - Bridge layer (`bridge.rs`): converts Helix types to engine-native `HelixDatabase` types
  - Balance overlays: per-engine tuning applied during bridge conversion
  - Dashboard (`dashboard.rs`): cross-ref validation (mobs->abilities, quests->prereqs, etc.)
  - API health (`api_health.rs`): opt-in remote checks against standardization API (port 6800, 2s timeout)
  - Editor seam: "Re-import Helix Data" toolbar, "Helix Default" preview preset
- **Server-client boundary**: all gameplay logic through a server trait/service

## Key Paths

```text
engine/src/                          Core engine (25 plugins)
plugins/helix_data/                  Helix data bridge plugin
plugins/helix_data/src/bridge.rs     Helix -> engine type conversion
plugins/helix_data/src/dashboard.rs  Contract validation + boxed dashboard
plugins/helix_data/src/api_health.rs API health, freshness, remote validation
games/dev/doomexe/                   Primary game (hamster narrator JRPG)
games/dev/stratego/                  Tutorial game (10x10 board, AI)
games/dev/iso_sandbox/               Isometric sandbox
games/dev/rpg_demo/                  SDK reference game
games/dev/helix_rpg/                 Helix MMORPG data consumer
docs/GAME_DEVELOPER_GUIDE.md         SDK guide for building games
docs/QA_CHECKLIST.md                 Visual test cards + session log
```

## Integration Points

- Path dependency on `helix-data` crate from `~/dev/helix/helix_3d_render_prototype/crates/helix-data/`
- Consumes `dist/helix3d/` TOML from helix_standardization
- All 7 Helix Row types consumed: ConsumableRow, CurrencyRow, EquipmentRow, InventoryRow, TitleRow, TradeGoodRow, WeaponSkillRow

## Conventions

- Dashboard must be green before commit
- Server-client boundary enforced -- no direct game logic in UI
- snake_case everywhere (matches Helix ecosystem conventions)
- `make validate` before every push (fmt + clippy + test + contracts + count)
