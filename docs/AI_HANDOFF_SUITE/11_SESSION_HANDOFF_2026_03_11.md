# Session Handoff — 2026-03-11

This document records the 2026-03-11 integration checkpoints so the next agent
can resume without reconstructing the recent engine/editor cleanup.

---

## Primary Code Checkpoint

| Commit | Message |
|--------|---------|
| `e9a37a8` | feat: advance engine editor and decoupling work |
| `de5b8ea` | feat: add mounted project runtime preview |

These checkpoints landed on `main` as two intentionally-scoped slices: the
engine/editor decoupling foundation first, then the mounted-project runtime
preview path built on top of it.

---

## What Landed

### 1. Spawner Runtime Initialization

- Added runtime-only spawner state initialization in `engine/src/data/spawner.rs`.
- Spawner entities now receive deterministic runtime state during scene spawn.
- Added unit and integration coverage for seeded wave state, empty-wave behavior,
  and scene-spawn insertion.

### 2. Manifest-Driven, Project-Agnostic Editor

- Added project startup defaults to the project schema in `engine/src/data/project.rs`.
- Replaced editor-only project ownership with manifest-backed shared mounting.
- Added shared `MountedProject` helpers so the editor and runtime preview use the
  same project path normalization and manifest loading rules.
- Editor load/save/play now resolve from `project.json` rather than DoomExe paths
  or sample scripts.
- Asset browser now lists the mounted project’s actual assets root instead of fake
  sample labels.

### 3. Engine-First Defaults and Generic Sprite API

- Workspace default member is now `engine`; `engine/Cargo.toml` sets
  `default-run = "dj_engine"`.
- The engine asset API now uses `SpritePartDefinition`,
  `SpritePartsManifest`, `SpritePartLibrary`, and `SpritePartLoader`.
- The custom asset extension changed from `hamsterpart.json` to
  `spritepart.json`.
- Engine animation helpers now use `character_default()` instead of
  `hamster_default()`.
- Engine-generic tests/fixtures were scrubbed of sample-game vocabulary where the
  behavior is meant to be reusable.
- Engine-facing docs (`README.md`, `engine/README.md`) now describe DoomExe as a
  sample game rather than the project identity.

### 4. Engine-Owned Runtime Preview for Mounted Projects

- Added `engine/src/runtime_preview/mod.rs` and the new binary
  `engine/src/bin/runtime_preview.rs`.
- Added `make preview PROJECT=<dir|project.json>` as the engine-first command for
  playable manifest-driven project preview.
- Runtime preview now mounts a project via `MountedProject`, opens in a generic
  title screen, loads the configured startup story graph and/or startup scene,
  and supports a basic `Title -> Dialogue -> Overworld` loop without using
  DoomExe state/plugins.
- Added a simple preview player, movement intent wiring, and camera follow for
  authored scenes.
- Added `--test-mode` for automated runtime smoke coverage.
- Fixed story-graph advance behavior in `engine/src/story_graph/executor.rs` so
  `StoryInputEvent::Advance` progresses past dialogue nodes instead of stalling
  in `WaitingForInput`.

---

## Validation Completed

The following commands were run successfully against the shared target dir
`/home/dj/.cargo-targets/dj_engine_bevy18`:

```bash
cargo fmt --all --check
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo check --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo test -p dj_engine
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo test -p doomexe
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo run -p dj_engine --bin dj_engine -- --test-mode --project /tmp/dj_engine_editor_smoke_20260311
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo run -p dj_engine --bin runtime_preview -- --test-mode --project /tmp/dj_engine_runtime_preview_smoke_20260311
timeout 20s make dev
timeout 20s make preview PROJECT=/tmp/dj_engine_runtime_preview_smoke_20260311
timeout 20s make game
```

Runtime smoke notes:

- `make dev` launched the engine editor successfully and was stopped by timeout.
- `runtime_preview --test-mode` mounted the temp project, progressed through the
  preview loop, and exited successfully.
- `make preview` launched the runtime preview successfully and was stopped by timeout.
- `make game` launched the sample game successfully and was stopped by timeout.
- The Vulkan/portal warnings observed during launch were non-fatal environment
  warnings, not regressions from this session.

---

## Current State After This Checkpoint

- Branch: `main`
- Primary checkpoints: `e9a37a8`, `de5b8ea`
- Local `main` is ahead of `origin/main`.
- The engine/editor shell is now much less coupled to DoomExe.
- The engine now has a generic playable preview path for mounted projects that is
  separate from the editor shell and separate from DoomExe’s own crate-specific flow.
- `engine/src` and `engine/tests` no longer contain DoomExe/hamster sample naming
  in engine-generic code paths.

Intentional sample-specific references still remain in DoomExe and in older
historical docs. That is expected.

---

## Best Next Work

1. Decide how the editor should hand off into the new runtime preview path
   without collapsing editor mode and runtime mode into one state machine.
2. Expand generic runtime preview capabilities from the current
   `Title -> Dialogue -> Overworld` baseline while keeping DoomExe-specific battle,
   continue/save UX, and sample gameplay out of the engine crate.
3. Continue the decoupling pass through older docs in `docs/` that still describe
   the repo as DoomExe-first or still mention old hamster-era engine APIs.
