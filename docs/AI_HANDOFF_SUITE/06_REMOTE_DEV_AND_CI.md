# Remote Dev And CI

## Rust Toolchain

- Pinned in `rust-toolchain.toml`
- Channel: `1.93.1`
- Profile: `minimal`
- Components: `clippy`, `rustfmt`

## Devcontainer Shape

The devcontainer is defined by:

- `.devcontainer/devcontainer.json`
- `.devcontainer/Dockerfile`
- `.devcontainer/on-create.sh`
- `.devcontainer/update-content.sh`
- `.devcontainer/post-create.sh`
- `.devcontainer/post-attach.sh`
- `.devcontainer/warm-runtime.sh`

## What The Devcontainer Provides

- Base image: `mcr.microsoft.com/devcontainers/rust:1-1-bookworm`
- Linux packages for Bevy, winit, X11, Wayland, OpenGL, Vulkan, and ALSA builds
- VS Code extensions:
  - `rust-lang.rust-analyzer`
  - `tamasfe.even-better-toml`
  - `vadimcn.vscode-lldb`
- `desktop-lite` feature for browser-accessible GUI windows
- `sshd` feature so CLI-based remote access through `gh codespace ssh` works

Important container environment:

- `RUSTC_WRAPPER=""`
- `CARGO_TARGET_DIR=/home/vscode/.cache/dj-engine/cargo-target/dj_engine_bevy18`
- `LIBGL_ALWAYS_SOFTWARE=1`

## Codespaces Lifecycle

### `onCreateCommand`

Runs:

```bash
rustup show
cargo fetch --locked
```

### `updateContentCommand`

Runs a compile warmup intentionally limited to:

```bash
cargo fetch --locked
cargo check --workspace
```

This is deliberately lighter than a full build so the Codespace becomes
reachable sooner.

### `postCreateCommand`

Prints the GUI port and the recommended runtime smoke commands:

```bash
./dj e --test-mode
timeout 20s ./dj d
bash .devcontainer/warm-runtime.sh
```

### Optional runtime warmup

`bash .devcontainer/warm-runtime.sh` performs:

```bash
cargo build --workspace
```

This exists because full runtime warmup is useful, but running it during early
Codespace provisioning can delay reachability.

## GUI Runtime In Codespaces

- Port `6080` is the browser desktop entrypoint.
- Port `5901` is the VNC endpoint.
- The `desktop-lite` password is currently `vscode`.
- Both the editor and `doomexe` can be launched inside this forwarded desktop.

## CI Workflow

The current workflow file is `.github/workflows/ci.yml`.

It runs on `ubuntu-24.04` and performs:

1. apt install of the native Bevy/winit/audio build dependencies
2. install of Rust `1.93.1`
3. `cargo fetch --locked`
4. `cargo fmt --all --check`
5. `cargo check --workspace`
6. `cargo test --workspace --no-run`
7. `cargo clippy --workspace --all-targets -- -W clippy::all`

CI uses:

```bash
RUSTC_WRAPPER=""
CARGO_TARGET_DIR=/tmp/dj_engine_bevy18
```

## Recommended Remote Validation

Inside Codespaces or equivalent Linux remote environments:

```bash
cargo fmt --all --check
RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo check --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo test --workspace --no-run
RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all
./dj e --test-mode
timeout 20s ./dj d
```

