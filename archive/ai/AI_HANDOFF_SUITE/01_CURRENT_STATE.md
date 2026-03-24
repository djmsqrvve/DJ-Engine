# Current State

> **Historical snapshot (2026-03-09).** Repo state, commit IDs, and drift notes below describe the handoff moment, not the current worktree.

Date: 2026-03-09

## Git Snapshot

- Canonical repo: `/home/dj/dev/engines/DJ-Engine`
- Branch: `main`
- HEAD: `5f0107d`
- Remote refs: `origin/main` only
- Worktree: clean

## Snapshot

- This is a Cargo workspace with three members:
  - `engine`
  - `games/dev/doomexe`
  - `tools/asset_generator`
- The default workspace member is `games/dev/doomexe`.
- The actual workspace Bevy version is `0.18`.
- The pinned Rust toolchain is `1.94.0` with `clippy` and `rustfmt`.
- The main binaries in normal use are:
  - `doomexe` for the game
  - `dj_engine` for the editor
  - `minimal` for a stripped-down rendering smoke test
  - `asset_generator` for MIDI generation and optional sprite repair

## Confirmed Build And Validation Contract

These are the current high-signal validation commands used locally and in docs:

```bash
cargo fmt --all --check
CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo check --workspace
CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo test --workspace
CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all
```

Useful runtime smokes:

```bash
make editor
timeout 20s make doom
```

## Current Remote-Dev Contract

- Codespaces support is checked in under `.devcontainer/`.
- GitHub CI is checked in under `.github/workflows/ci.yml`.
- The Codespaces image is Debian Bookworm based and installs Bevy, winit, X11,
  Wayland, OpenGL, Vulkan, and audio build dependencies.
- Codespaces includes `desktop-lite` for browser-accessible GUI windows and
  `sshd` for remote CLI access through `gh codespace ssh`.

## Important Truths About The Current Code

- `doomexe` and the editor both create real Bevy windows. GUI runtime is a real
  part of this repo, not just compile-only scaffolding.
- Engine startup audio is muted by default because `AudioState::new()` sets
  `master_volume` to `0.0`.
- The rendering module includes a main camera, an offscreen 320×240 render
  target with upscaling, and a CRT post-processing pipeline (`crt_material.rs`,
  `crt.wgsl`) with configurable scanlines, barrel distortion, and color bleeding.
- The asset module provides asset definitions, a library resource, and
  `HamsterPartLoader` and `PaletteLoader` implemented as Bevy `AssetLoader`s.
- The story graph runtime bridge is partial: some data-layer story variants map
  cleanly into runtime nodes, while unimplemented variants currently fall back to
  `StoryNode::End`.
- `games/dev/doomexe/assets/` currently contains `music`, `palettes`, and
  `scripts`. There is no committed `sprites/` directory at the time of writing.

## Known Drift And Caveats

- Some older docs under `docs/` are roadmap/spec documents rather than accurate
  descriptions of the current working repo. Prefer the AI Handoff Suite,
  `README.md`, and `ROADMAP.md` for current onboarding.
- The legacy `./dj` helper has been retired in favor of the root `Makefile`.
- `games/dev/doomexe/src/assets/mod.rs` is currently just a placeholder plugin.
- The diagnostics inspector plugin is intentionally disabled due to window-kind
  issues in some Linux/WSL software-rendered environments.
- The branch-cleanup claims from the earlier conversation are reflected in the
  current refs, but new agents should still start with `git status` and
  `git branch -a` instead of assuming history from prose.

## Accepted Clippy Noise

The current known acceptable warning buckets are:

- `clippy::too_many_arguments`
- `clippy::type_complexity`
- `clippy::upper_case_acronyms`
- `clippy::module_inception`
