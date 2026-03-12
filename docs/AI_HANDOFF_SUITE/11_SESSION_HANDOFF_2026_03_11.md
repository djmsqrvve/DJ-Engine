# Session Handoff — 2026-03-11

This document records the 2026-03-11 integration checkpoints so the next agent
can resume without reconstructing the recent engine/editor cleanup.

---

## Primary Code Checkpoint

| Commit | Message |
|--------|---------|
| `e9a37a8` | feat: advance engine editor and decoupling work |
| `de5b8ea` | feat: add mounted project runtime preview |
| `7d0f291` | feat: hand off editor play to runtime preview |

These checkpoints landed on `main` as two intentionally-scoped slices: the
engine/editor decoupling foundation first, then the mounted-project runtime
preview path, then the editor-to-runtime handoff built on top of that preview
flow.

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

### 5. Editor-to-Runtime Preview Handoff

- Replaced the editor shell’s primary top-bar action with `Run Project`, which
  auto-saves the mounted project and launches the separate `runtime_preview`
  process.
- Renamed the in-editor preview state from `EditorState::Playing` to
  `EditorState::GraphPreview` so editor state no longer implies full project
  runtime.
- Added editor-side runtime preview lifecycle tracking for launch, running,
  stopping, failure, and exit status messaging.
- Kept the lightweight in-editor graph preview as a Story Graph-only tool under
  `Preview Graph` / `Stop Graph Preview`.
- Changed editor save helpers to return `Result<(), DataError>` so runtime
  launch can make a real go/no-go decision instead of only logging failures.
- Added command-resolution coverage for sibling `runtime_preview` binaries and
  dev-build `cargo run` fallback behavior.

---

## Validation Completed

The following commands were run successfully against the shared target dir
`/home/dj/.cargo-targets/dj_engine_bevy18`:

```bash
cargo fmt --all --check
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo check --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo test -p dj_engine
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo test -p doomexe
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 timeout 20s cargo run -p dj_engine --bin dj_engine -- --test-mode
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo run -p dj_engine --bin dj_engine -- --test-mode --project /tmp/dj_engine_editor_smoke_20260311
RUSTC_WRAPPER= CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo run -p dj_engine --bin runtime_preview -- --test-mode --project /tmp/dj_engine_runtime_preview_smoke_20260311
timeout 20s make dev
timeout 20s make preview PROJECT=/tmp/dj_engine_runtime_preview_smoke_20260311
timeout 20s make game
```

Runtime smoke notes:

- `make dev` launched the engine editor successfully and was stopped by timeout.
- `cargo run -p dj_engine --bin dj_engine -- --test-mode` launched the editor
  successfully after the handoff change and exited cleanly.
- `runtime_preview --test-mode` mounted the temp project, progressed through the
  preview loop, and exited successfully.
- `make preview` launched the runtime preview successfully and was stopped by timeout.
- `make game` launched the sample game successfully and was stopped by timeout.
- The Vulkan/portal warnings observed during launch were non-fatal environment
  warnings, not regressions from this session.

---

## Current State After This Checkpoint

- Branch: `main`
- Primary checkpoints: `e9a37a8`, `de5b8ea`, `7d0f291`
- Local `main` is ahead of `origin/main`.
- The engine/editor shell is now much less coupled to DoomExe.
- The engine now has a generic playable preview path for mounted projects that is
  separate from the editor shell and separate from DoomExe’s own crate-specific flow.
- The editor shell now launches that preview path intentionally instead of
  treating full project runtime as an in-editor state transition.
- `engine/src` and `engine/tests` no longer contain DoomExe/hamster sample naming
  in engine-generic code paths.

Intentional sample-specific references still remain in DoomExe and in older
historical docs. That is expected.

---

## Best Next Work

1. Expand generic runtime preview capabilities from the current
   `Title -> Dialogue -> Overworld` baseline while keeping DoomExe-specific battle,
   continue/save UX, and sample gameplay out of the engine crate.
2. Continue the decoupling pass through older docs in `docs/` that still describe
   the repo as DoomExe-first or still mention old hamster-era engine APIs.
3. Improve editor/runtime workflow polish on top of the new handoff path:
   mounted-project clarity, richer preview exit reporting, and future dirty-state UX.
