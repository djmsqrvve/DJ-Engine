# DJ-Engine

> Helix integration context: see `~/dev/helix/docs/HELIX_AGENT_BRIEFING.md`

## Overview

Custom 2D game engine built with Rust and Bevy 0.18. Plugin-based architecture. 302+ tests. Tile editor complete. Consumes Helix MMORPG data through `plugins/helix_data/`.

## Stack

- Rust, Bevy 0.18 (custom framework)

## Key Commands

```bash
make test                            # 302+ workspace tests
make lint                            # clippy
make fmt                             # format check
make quality-check                   # fmt + clippy + test
make dev                             # launch engine editor

# Helix integration
make helix-import-toml HELIX3D=~/dev/helix/helix_standardization/dist/helix3d/
make helix-dashboard HELIX3D=~/dev/helix/helix_standardization/dist/helix3d/
make helix-editor PROJECT=/tmp/helix_project
make helix-preview PROJECT=/tmp/helix_project
```

## Architecture

- **Plugin-based**: core engine in `engine/src/`, game crates in `games/dev/`
- **Helix data plugin** (`plugins/helix_data/`):
  - `HelixRegistries` Bevy Resource: 22 typed `Registry<T>` collections (915 entities when loaded)
  - Bridge layer (`bridge.rs`): converts Helix types to engine-native `HelixDatabase` types
  - Balance overlays: per-engine tuning applied during bridge conversion
  - Dashboard (`dashboard.rs`): cross-ref validation (mobs->abilities, quests->prereqs, etc.)
  - Editor seam: "Re-import Helix Data" toolbar, "Helix Default" preview preset
- **Server-client boundary**: all gameplay logic through a server trait/service

## Key Paths

```
engine/src/                          Core engine
plugins/helix_data/                  Helix data bridge plugin
plugins/helix_data/src/bridge.rs     Helix -> engine type conversion
plugins/helix_data/src/dashboard.rs  Contract validation
games/dev/                           Game crates (doomexe, stratego, iso_sandbox)
```

## Integration Points

- Path dependency on `helix-data` crate from `~/dev/helix/helix_3d_render_prototype/crates/helix-data/`
- Consumes `dist/helix3d/` TOML from helix_standardization
- 8/22 entity kinds have typed validators, 14 remaining

## Conventions

- Dashboard must be green before commit
- Server-client boundary enforced -- no direct game logic in UI
- snake_case everywhere (matches Helix ecosystem conventions)
