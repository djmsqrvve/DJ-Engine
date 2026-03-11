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

make dev
make editor
make test
```

The workspace toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml), so `rustup` will automatically select the validated Rust version for local development, Codespaces, and CI.

## Codespaces

GitHub Codespaces is fully supported. The devcontainer includes a lightweight remote desktop (VNC), SSH server, and all Bevy/Linux build dependencies. Dependencies are pre-fetched and checked during provisioning via `make cache-warm-fast`.

To get started in a Codespace:

```bash
make codespace        # Verify environment is ready
make editor           # Launch the editor
make doom             # Run DoomExe
```

Open the forwarded `Desktop` port (`6080`) in your browser and connect with password `vscode` to view GUI windows.

For a full runtime warmup (pre-compiles all binaries):

```bash
make cache-warm
```

A prebuilt Docker image with pre-compiled dependencies is also available:

```bash
make codespace-image-build    # Build locally
make codespace-image-push     # Push to GHCR
```

For prebuilds, repository admins need to enable a Codespaces prebuild configuration in GitHub repository settings pointing at [`.devcontainer/devcontainer.json`](.devcontainer/devcontainer.json).

## Commands

All common tasks run through `make`:

```bash
# Development
make dev              # Launch the editor (default)
make dev-fast         # Fastest startup (skip checks)
make dev-release      # Optimized release build
make doom             # Run DoomExe test game
make minimal          # Run minimal rendering binary
make asset-gen        # Run the asset generator

# Codespaces
make codespace        # Verify Codespace environment
make cache-warm-fast  # Quick compile cache
make cache-warm       # Full build + test compile

# Quality
make test             # Run workspace tests
make quality-check    # Full pipeline (fmt + clippy + test)
make guardrail        # Quick safety checks
make guardrail-strict # Full safety checks
make lint             # Run clippy
make format           # Check formatting
make format-fix       # Fix formatting

# Build
make build            # Debug build
make build-release    # Release build (optimized)
make clean            # Remove build artifacts
```

## Project Structure

```text
dj_engine/
├── engine/                 # Core engine library
├── games/dev/doomexe/      # Primary game project
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
