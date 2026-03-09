# Getting Started with DJ Engine

This guide covers the current setup path for local development and GitHub Codespaces.

## Toolchain

DJ Engine pins its Rust toolchain in [`../rust-toolchain.toml`](../rust-toolchain.toml). Install `rustup`, clone the repo, and let `rustup` select the pinned toolchain automatically when you enter the workspace.

## Platform Notes

| Platform | Status | Notes |
| --- | --- | --- |
| Linux | Recommended | Best fit for Bevy native dependencies and local runtime smoke tests |
| GitHub Codespaces | Supported | Best path for compile, test, and lint validation without local package setup |
| WSL2 | Supported | Use Linux package instructions inside the distro |
| Windows | Partial | Expect more graphics/runtime variance |
| macOS | Untested | Compile may work, but not part of the current validated path |

## Clone the Repository

```bash
git clone https://github.com/djmsqrvve/dj_engine.git
cd dj_engine
```

## GitHub Codespaces

Open the repository in GitHub Codespaces and wait for the devcontainer bootstrap to finish. The Codespaces configuration lives in [`.devcontainer/devcontainer.json`](../.devcontainer/devcontainer.json), installs the Linux build dependencies required by Bevy, winit, and audio backends, and warms build artifacts through `onCreateCommand` plus `updateContentCommand` so prebuilds can reuse real compiled binaries.

After the container is ready, validate the workspace with:

```bash
cargo fmt --all --check
RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo check --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo test --workspace --no-run
RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all
```

To view the editor or game remotely, open the forwarded `desktop` port on `6080` in your browser and connect with password `vscode`. Then launch:

```bash
./dj e --test-mode
timeout 20s ./dj d
```

Repository admins who want faster startup should also enable a Codespaces prebuild configuration in GitHub repository settings and select [`.devcontainer/devcontainer.json`](../.devcontainer/devcontainer.json).

## Local Linux Setup

If you are building outside Codespaces on Debian/Ubuntu, install the same native packages used by the devcontainer and CI first:

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  pkg-config \
  libasound2-dev \
  libudev-dev \
  libwayland-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  libxcursor-dev \
  libxi-dev \
  libxrandr-dev \
  libxinerama-dev \
  libx11-dev \
  libx11-xcb-dev \
  libgl1-mesa-dev \
  libvulkan-dev \
  libxcb1-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  clang \
  lld \
  cmake \
  git
```

Then run the same validation commands shown above.

## Common Commands

```bash
./dj e           # Launch the editor
./dj d           # Run DoomExe
./dj d --verbose # Run DoomExe with debug logging
./dj t           # Run workspace tests
./dj c           # cargo check --workspace
./dj fmt         # cargo fmt --all
./dj lint        # cargo clippy --workspace -- -W clippy::all
./dj g           # Run the asset generator
./dj b           # Build release binaries
```

## Project Structure Overview

```text
dj_engine/
├── engine/                 # Core engine library
├── games/dev/doomexe/      # Main game project
├── tools/asset_generator/  # Asset processing tool
├── docs/                   # Documentation
└── dj                      # Helper script
```

## Next References

- [Architecture Guide](ARCHITECTURE.md)
- [Testing Guide](TESTING.md)
- [Project Structure](PROJECT_STRUCTURE.md)
