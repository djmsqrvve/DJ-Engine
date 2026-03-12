# Getting Started with DJ Engine

This guide covers the current local and Codespaces workflow for the engine-first
repo layout.

## Toolchain

DJ Engine pins its Rust toolchain in [`../rust-toolchain.toml`](../rust-toolchain.toml).
Install `rustup`, clone the repo, and let `rustup` select the pinned toolchain
automatically when you enter the workspace.

## Platform Notes

| Platform | Status | Notes |
| --- | --- | --- |
| Linux | Recommended | Best fit for Bevy native dependencies and local runtime smokes |
| GitHub Codespaces | Supported | Best path for compile, test, and lint validation without local package setup |
| WSL2 | Supported | Use the Linux package instructions inside the distro |
| Windows | Partial | Expect more graphics/runtime variance |
| macOS | Untested | Compile may work, but not part of the validated path |

## Clone The Repository

```bash
git clone https://github.com/djmsqrvve/dj_engine.git
cd dj_engine
```

## GitHub Codespaces

Open the repository in GitHub Codespaces and wait for the devcontainer bootstrap
to finish. The configuration lives in
[`.devcontainer/devcontainer.json`](../.devcontainer/devcontainer.json) and
installs the native packages needed by Bevy, winit, audio backends, and the
forwarded desktop environment.

After the container is ready, validate the workspace with:

```bash
cargo fmt --all --check
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo check --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo test --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all
```

To view native Bevy windows remotely, open the forwarded `desktop` port on
`6080` in your browser and connect with password `vscode`. Then launch:

```bash
make dev
timeout 20s make game
```

If you want the heavier runtime warmup after the Codespace is ready, run:

```bash
bash .devcontainer/warm-runtime.sh
```

## Local Linux Setup

If you are building outside Codespaces on Debian or Ubuntu, install the same
native packages used by the devcontainer and CI first:

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
make dev                           # Launch the engine editor
make preview PROJECT=/path/to/project  # Launch runtime preview for a mounted project
make game                          # Run the sample DoomExe game
make doom                          # Alias for make game
make minimal                       # Run the minimal rendering binary
make test                          # Run workspace tests
make quality-check                 # fmt + clippy + test
make guardrail                     # Quick build/test/format safety sweep
make fmt                           # cargo fmt --all --check
make format-fix                    # cargo fmt --all
make lint                          # cargo clippy --workspace --all-targets -- -W clippy::all
make asset-gen                     # Run the asset generator
```

Inside the editor, `Run Project` auto-saves the mounted project and launches
the separate `runtime_preview` process. `Preview Graph` remains the editor-only
Story Graph tool.

## Mounted Project Shape

Mounted projects are rooted at `project.json` and can carry authored data beside
scenes and story graphs:

```text
project.json
scenes/
story_graphs/
assets/
data/
  registry.json
  <custom_kind>/
```

## Project Structure Overview

```text
dj_engine/
├── engine/                 # Core engine/editor/runtime crate
├── games/dev/doomexe/      # Optional sample game
├── tools/asset_generator/  # Asset processing tool
├── docs/                   # Documentation and handoff notes
└── Makefile                # Unified command surface
```

## Next References

- [Architecture Guide](ARCHITECTURE.md)
- [Testing Guide](TESTING.md)
- [Project Structure](PROJECT_STRUCTURE.md)
- [AI Handoff Suite](AI_HANDOFF_SUITE/README.md)
