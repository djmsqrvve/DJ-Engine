# DJ Engine

<p align="center">
  <strong>Custom Bevy 0.18 game framework for narrative-heavy JRPGs, procedural 2D animation, and palette-driven corruption effects.</strong>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.94.0-orange?style=flat-square" alt="Rust 1.94.0"></a>
  <a href="https://bevyengine.org/"><img src="https://img.shields.io/badge/Bevy-0.18-green?style=flat-square" alt="Bevy 0.18"></a>
  <a href="https://github.com/djmsqrvve/DJ-Engine/actions/workflows/ci.yml"><img src="https://github.com/djmsqrvve/DJ-Engine/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/tests-501-brightgreen?style=flat-square" alt="501 tests">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License"></a>
</p>

<p align="center">
  <img src="docs/images/editor_screenshot.png" alt="DJ Engine Editor" width="800">
</p>

## Why DJ Engine?

Most Bevy frameworks focus on general-purpose 2D/3D rendering. DJ Engine is purpose-built for **data-authored narrative games** -- JRPGs, visual novels, and story-driven prototypes where the content pipeline matters as much as the renderer.

- **Narrative-first architecture** -- story graphs, dialogue branching, and Lua scripting are engine primitives, not afterthoughts
- **Palette corruption pipeline** -- real-time palette swaps and visual degradation driven by game state, not just shader tricks
- **Custom document platform** -- register your own game data types (abilities, enemies, quests) and get editor browsing, validation, and runtime loading for free
- **Multi-game workspace** -- one engine, multiple game crates. DoomExe, Stratego, and Iso Sandbox ship as proof that the framework generalizes
- **Helix data integration** -- consumes 2,681 MMORPG entities from the Helix standardization pipeline via typed TOML registries
- **501 tests, zero warnings** -- CI-enforced quality across the full workspace

## Features

| Feature | Description |
| --- | --- |
| Procedural 2D animation | Breathing, blinking, and expression-driven character motion |
| Palette-driven rendering | Real-time palette swaps and corruption/distortion effects |
| Story graph runtime | JSON-serializable dialogue, branching, and scripted actions |
| Lua scripting | Runtime game logic via `mlua` for content-heavy iteration |
| Modular Bevy plugins | Engine systems can be bundled or composed per game |
| Custom document platform | Registry-driven game data under `data/registry.json` for reusable non-scene content |

## Quick Start

```bash
git clone https://github.com/djmsqrvve/DJ-Engine.git
cd DJ-Engine

make dev           # Launch the engine editor
make preview PROJECT=/path/to/project   # Launch manifest-driven runtime preview
make helix-import HELIX_DIST=/path/to/helix/dist PROJECT=/tmp/helix_project
make helix-preview PROJECT=/tmp/helix_project
make game          # Optional: run the sample DoomExe game
make test
```

Inside the editor, the primary top-bar action is now `Run Project`, which saves
the mounted project and launches the separate `runtime_preview` process. The
old in-editor graph preview remains available as `Preview Graph` inside the
Story Graph view.

Mounted projects can now also carry custom authored data beside scenes and
story graphs. DJ Engine looks for `data/registry.json` under the mounted
project root and uses that registry to discover reusable custom document kinds
such as abilities, enemy archetypes, waves, evolution graphs, and preview
profiles.

The workspace toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml), so `rustup` will automatically select the validated Rust version for local development and CI.

## Codespaces

GitHub Codespaces is supported through the checked-in devcontainer config at [`.devcontainer/devcontainer.json`](.devcontainer/devcontainer.json). Once the workspace is up, use the same engine-first commands as local development:

```bash
make dev              # Launch the engine editor
make preview PROJECT=/path/to/project   # Launch runtime preview for a mounted project
make game             # Optional: run the sample DoomExe game
```

If you need a GUI desktop for Bevy windows, forward the desktop port configured by the devcontainer and open it in your browser.

## Commands

All common tasks run through `make`:

```bash
# Development
make dev              # Launch the engine editor
make engine           # Alias for make editor
make editor           # Launch the engine editor
make preview PROJECT=/path/to/project  # Launch runtime preview for a project manifest
make helix-import HELIX_DIST=/path/to/helix/dist PROJECT=/tmp/helix_project
make helix-editor PROJECT=/tmp/helix_project
make helix-preview PROJECT=/tmp/helix_project
make game             # Run the sample DoomExe game
make doom             # Alias for make game
make stratego         # Run Stratego (10x10 board game tutorial)
make iso              # Run Iso Sandbox (isometric tile grid)
make minimal          # Run minimal rendering binary

# Quality
make test             # Run workspace tests
make quality-check    # Full pipeline (fmt + clippy + test)
make guardrail        # Quick safety checks
make lint             # Run clippy
make fmt              # Check formatting
make format-fix       # Fix formatting

# Build
make build            # Debug build
make clean            # Remove build artifacts
```

## Project Structure

```text
DJ-Engine/
├── engine/                 # Core engine library
├── games/dev/doomexe/      # Primary game — hamster narrator JRPG
├── games/dev/stratego/     # Tutorial game — 10x10 board, AI opponent
├── games/dev/iso_sandbox/  # Isometric sandbox — 16x16 tile grid
├── games/dev/rpg_demo/     # SDK reference game — all systems wired
├── plugins/helix_data/     # Helix data bridge plugin and import tooling
├── tools/asset_generator/  # Asset processing utilities
├── docs/                   # Documentation + 8-chapter Stratego tutorial
└── Makefile                # Unified command interface
```

Mounted project content now follows this shape:

```text
project.json
scenes/
story_graphs/
assets/
data/
  registry.json
  <custom_kind>/
```

## Documentation

| Document | Description |
| --- | --- |
| [Getting Started](docs/GETTING_STARTED.md) | Local setup, Codespaces notes, and validation commands |
| [Architecture](docs/ARCHITECTURE.md) | Engine system overview |
| [Testing Guide](docs/TESTING.md) | How tests are organized and run |
| [Project Structure](docs/PROJECT_STRUCTURE.md) | Detailed workspace layout |
| [Game Developer's Guide](docs/GAME_DEVELOPER_GUIDE.md) | How to use combat, quests, inventory, interaction, abilities, loot, Lua |
| [Contributing](CONTRIBUTING.md) | Contribution workflow and expectations |

## Prerequisites

- `rustup` with the pinned toolchain from [`rust-toolchain.toml`](rust-toolchain.toml)
- `git`
- Linux, WSL2, or GitHub Codespaces for the smoothest build experience

For manual local Linux setup outside Codespaces, install the same native packages used by the devcontainer and CI before building Bevy-based crates.

## License

MIT License. See [LICENSE](LICENSE).
