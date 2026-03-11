# DJ Engine

<p align="center">
  <strong>Custom Bevy 0.18 game framework for narrative-heavy JRPGs, procedural 2D animation, and palette-driven corruption effects.</strong>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.94.0-orange?style=flat-square" alt="Rust 1.94.0"></a>
  <a href="https://bevyengine.org/"><img src="https://img.shields.io/badge/Bevy-0.18-green?style=flat-square" alt="Bevy 0.18"></a>
  <a href="https://github.com/djmsqrvve/dj_engine/actions/workflows/ci.yml"><img src="https://github.com/djmsqrvve/dj_engine/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License"></a>
</p>

<p align="center">
  <img src="docs/images/editor_screenshot.png" alt="DJ Engine Editor" width="800">
</p>

## Features

| Feature | Description |
| --- | --- |
| Procedural 2D animation | Breathing, blinking, and expression-driven character motion |
| Palette-driven rendering | Real-time palette swaps and corruption/distortion effects |
| Story graph runtime | JSON-serializable dialogue, branching, and scripted actions |
| Lua scripting | Runtime game logic via `mlua` for content-heavy iteration |
| Modular Bevy plugins | Engine systems can be bundled or composed per game |

## Quick Start

```bash
git clone https://github.com/djmsqrvve/dj_engine.git
cd dj_engine

make dev           # Launch the engine editor
make game          # Optional: run the sample DoomExe game
make test
```

The workspace toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml), so `rustup` will automatically select the validated Rust version for local development and CI.

## Codespaces

GitHub Codespaces is supported through the checked-in devcontainer config at [`.devcontainer/devcontainer.json`](.devcontainer/devcontainer.json). Once the workspace is up, use the same engine-first commands as local development:

```bash
make dev              # Launch the engine editor
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
make game             # Run the sample DoomExe game
make doom             # Alias for make game
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
dj_engine/
├── engine/                 # Core engine library
├── games/dev/doomexe/      # Sample game project
├── tools/asset_generator/  # Asset processing utilities
├── docs/                   # Project and engine documentation
└── Makefile                # Unified command interface
```

## Documentation

| Document | Description |
| --- | --- |
| [Getting Started](docs/GETTING_STARTED.md) | Local setup, Codespaces notes, and validation commands |
| [Architecture](docs/ARCHITECTURE.md) | Engine system overview |
| [Testing Guide](docs/TESTING.md) | How tests are organized and run |
| [Project Structure](docs/PROJECT_STRUCTURE.md) | Detailed workspace layout |
| [Contributing](CONTRIBUTING.md) | Contribution workflow and expectations |

## Prerequisites

- `rustup` with the pinned toolchain from [`rust-toolchain.toml`](rust-toolchain.toml)
- `git`
- Linux, WSL2, or GitHub Codespaces for the smoothest build experience

For manual local Linux setup outside Codespaces, install the same native packages used by the devcontainer and CI before building Bevy-based crates.

## License

MIT License. See [LICENSE](LICENSE).
