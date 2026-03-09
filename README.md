# DJ Engine

<p align="center">
  <strong>Custom Bevy 0.18 game framework for narrative-heavy JRPGs, procedural 2D animation, and palette-driven corruption effects.</strong>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.93.1-orange?style=flat-square" alt="Rust 1.93.1"></a>
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

./dj d
./dj e
./dj t
```

The workspace toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml), so `rustup` will automatically select the validated Rust version for local development, Codespaces, and CI.

## Codespaces

GitHub Codespaces is supported through [`.devcontainer/devcontainer.json`](.devcontainer/devcontainer.json). The devcontainer now includes a lightweight remote desktop for GUI apps, an SSH server so `gh codespace ssh` can work against ready environments, and the Bevy/Linux native build plus Mesa runtime dependencies during image build. Provisioning warms the compile-validation layer through `onCreateCommand` and `updateContentCommand`, while a separate script handles the heavier runtime binary warmup.

To run the editor or game inside Codespaces:

```bash
./dj e --test-mode
timeout 20s ./dj d
```

Open the forwarded `desktop` port on `6080` in your browser and connect with password `vscode` to view GUI windows.

If you want to prebuild runtime binaries after the Codespace is reachable, run:

```bash
bash .devcontainer/warm-runtime.sh
```

Codespaces support is still compile-first, with GUI runtime intended for smoke runs and manual checks. The primary validation flow is:

```bash
cargo fmt --all --check
RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo check --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo test --workspace --no-run
RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all
```

For prebuilds, repository admins still need to enable a Codespaces prebuild configuration in GitHub repository settings and point it at [`.devcontainer/devcontainer.json`](.devcontainer/devcontainer.json).

## Helper Commands

All common tasks run through the `./dj` helper script:

```bash
# Development
./dj e                # Launch the editor
./dj d                # Run DoomExe
./dj d --verbose      # Run DoomExe with debug logging
./dj m                # Run the minimal engine test binary

# Validation
./dj t                # Run workspace tests
./dj c                # cargo check --workspace
./dj fmt              # cargo fmt --all
./dj lint             # cargo clippy --workspace -- -W clippy::all

# Build and tools
./dj g                # Run the asset generator
./dj b                # Build release binaries
./dj doc              # Generate workspace docs
./dj clean            # Remove build artifacts
```

## Project Structure

```text
dj_engine/
├── engine/                 # Core engine library
├── games/dev/doomexe/      # Primary game project
├── tools/asset_generator/  # Asset processing utilities
├── docs/                   # Project and engine documentation
└── dj                      # Workspace helper script
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
