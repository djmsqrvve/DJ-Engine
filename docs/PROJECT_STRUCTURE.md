# DJ Engine Project Structure

Current reference for the checked-in workspace layout.

## Root Directory

```text
DJ-Engine/
├── .devcontainer/        # Codespaces and remote-dev config
├── .github/              # CI and GitHub workflow config
├── archive/              # Historical docs and legacy files
├── docs/                 # Current documentation and tutorials
├── engine/               # Reusable engine crate and binaries
├── games/                # Game crates (sample and tutorial)
├── plugins/              # Data bridge plugins
├── tools/                # Workspace utilities
├── Cargo.toml            # Workspace manifest
├── Cargo.lock            # Dependency lock file
├── Makefile              # Unified command surface
├── rust-toolchain.toml   # Pinned Rust toolchain
├── README.md             # Repo overview
├── CONTRIBUTING.md       # Contribution workflow
├── CODE_OF_CONDUCT.md    # Contributor Covenant
├── SECURITY.md           # Vulnerability reporting
├── MAINTAINERS.md        # Project maintenance guide
└── LICENSE               # MIT license
```

## Workspace Crates

### `engine/`

The main engine crate. It is also the default workspace member.

Important files:

- `engine/src/lib.rs`
  - Library root and public reexports.
- `engine/src/main.rs`
  - Editor binary entrypoint (`dj_engine`).
- `engine/src/bin/minimal.rs`
  - Minimal rendering smoke binary.
- `engine/src/bin/runtime_preview.rs`
  - Engine-owned playable preview binary for mounted projects.
- `engine/src/project_mount.rs`
  - Shared mounted-project path normalization and manifest loading.

Important module directories:

```text
engine/src/
├── animation/
├── assets/
├── audio/
├── bin/
├── collision/
├── core/
├── data/
├── diagnostics/
├── editor/
├── input/
├── midi/
├── rendering/
├── runtime_preview/
├── scene/
├── scripting/
└── story_graph/
```

Notable data/editor/runtime files:

- `engine/src/data/project.rs`
  - Project manifest, startup defaults, and project-relative path settings.
- `engine/src/data/custom.rs`
  - Registry-driven custom documents under `data/registry.json`.
- `engine/src/data/grid.rs`
  - Generic `Grid<T>` with 8 tests.
- `engine/src/editor/`
  - Editor shell, document browser, graph preview, runtime handoff, tile editor, and dirty tracking.
- `engine/src/runtime_preview/mod.rs`
  - Title, dialogue, overworld preview, continue flow, and preview-profile loading.

### `games/dev/doomexe/`

Primary sample game that exercises the engine. Hamster narrator JRPG prototype.

```text
games/dev/doomexe/
├── Cargo.toml
├── assets/
│   ├── music/
│   ├── palettes/
│   └── scripts/
├── docs/
└── src/
    ├── assets/
    ├── battle/
    ├── dialogue/
    ├── hamster/
    ├── hud/
    ├── overworld/
    └── scripting/
```

### `games/dev/stratego/`

Tutorial game: 10x10 board game with 8 piece types and an AI opponent.
Full 8-chapter tutorial walkthrough at `docs/tutorials/01-build-a-board-game/`.

### `games/dev/iso_sandbox/`

Isometric sandbox: 16x16 tile grid with entity placement. 4 tests.

### `plugins/helix_data/`

Helix data bridge plugin. Consumes the `helix-data` Rust crate to import TOML
game data (abilities, items, mobs, zones, etc.) into engine-native types.

Key features:
- 22 typed `Registry<T>` collections via `HelixRegistries` Bevy Resource
- Bridge layer converting Helix types to engine `HelixDatabase`
- Balance overlays for per-engine tuning
- Contract validation dashboard
- Editor extension: toolbar action, preview presets

### `tools/asset_generator/`

Small workspace utility for MIDI generation and optional asset repair helpers.

## Docs Directory

`docs/` contains current, maintained documentation:

- `docs/MASTER-DOCUMENTATION-INDEX.md` — Navigation index
- `docs/GETTING_STARTED.md` — Setup and validation
- `docs/ARCHITECTURE.md` — Engine system overview
- `docs/TESTING.md` — Test organization
- `docs/PROJECT_STRUCTURE.md` — This file
- `docs/CODE_STYLE.md` — Coding standards
- `docs/ROADMAP.md` — Phased development plan
- `docs/CURRENT_GAPS.md` — Remaining gaps
- `docs/CURRENT_PRIORITIES.md` — Execution priorities
- `docs/LONG_TERM_GOALS.md` — Engine vision
- `docs/tutorials/` — 8-chapter Stratego tutorial

Historical and legacy docs live under `archive/`.

## Mounted Project Shape

Projects opened through the editor or runtime preview are rooted at
`project.json` and can carry custom documents beside scenes and story graphs:

```text
project.json
scenes/
story_graphs/
assets/
data/
  registry.json
  <custom_kind>/
```

## Key Commands

- `make dev`
  - Launch the editor.
- `make preview PROJECT=/path/to/project`
  - Launch runtime preview for a mounted project.
- `make game`
  - Launch the sample DoomExe game.
- `make stratego`
  - Launch the Stratego tutorial game.
- `make iso`
  - Launch the Iso Sandbox.
- `make test`
  - Run workspace tests.
