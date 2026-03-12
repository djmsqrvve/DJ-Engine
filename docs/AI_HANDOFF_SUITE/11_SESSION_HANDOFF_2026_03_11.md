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
| `cb9e2be` | feat: add preview continue flow and editor dirty tracking |
| `7555544` | feat: add custom document scaffolding |
| `da2c5c4` | docs: refresh current onboarding guides |

These checkpoints landed on `main` as two intentionally-scoped slices: the
engine/editor decoupling foundation first, then the mounted-project runtime
preview path, then the editor-to-runtime handoff built on top of that preview
flow, then project-scoped continue/save plus snapshot-based dirty tracking.

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

### 6. Preview Continue Flow and Editor Dirty Tracking

- Added project-scoped save plumbing for runtime preview so mounted projects can
  use `New Game / Continue / Quit` without colliding with global sample-game
  save slots.
- Runtime preview now writes overworld checkpoints per `Project.id` and can
  resume flags, variables, scene context, and story-graph context safely.
- The editor now tracks snapshot-based dirty state across scene, story graph,
  and mounted-project resource changes, and protects reload/discard actions with
  explicit confirmation.
- The toolbar now reports mounted-project context, dirty state, preview status,
  and last preview exit more clearly.

### 7. Custom Document Scaffold

- Added a generic custom-document system under `engine/src/data/custom.rs`
  centered around `data/registry.json`.
- Mounted projects can now resolve a custom data root from `ProjectPaths.data`
  and load custom documents beside scenes and story graphs.
- Added generic registration and validation surfaces:
  `CustomDocumentRegistration<T>`, `LoadedCustomDocuments`,
  `ValidationIssue`, and `EditorDocumentRoute`.
- Added engine editor scaffolding for custom documents:
  a new `Docs` browser tab, custom-document selection state, raw JSON editing,
  issue display, and extension registries for custom panels/actions/views.
- Added runtime-preview support for registry-loaded custom data and
  `preview_profiles`, so preview startup can request a scene, story graph, and
  custom document bundle together.
- Added thin-slice tests covering registry loading, broken refs, editor state,
  and preview-profile-driven runtime startup with Helix-shaped sample kinds.

### 8. Current Docs And Onboarding Sweep

- Refreshed the current onboarding guides under `docs/` so the active entry path
  matches the engine-first repo shape rather than older DoomExe-first docs.
- Updated the AI Handoff Suite index, workspace map, engine guide, game guide,
  data/scripting guide, and branch log to reflect the mounted-project runtime
  preview path, custom-document scaffold, and current launch surface.
- Added `docs/CURRENT_GAPS.md` as a high-level near-, mid-, and long-term gap
  map for the current engine state.
- Reclassified older long-form roadmap/spec docs as historical context rather
  than treating them as the source of truth for the checked-in repo.

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
- Primary checkpoints: `e9a37a8`, `de5b8ea`, `7d0f291`, `cb9e2be`, `7555544`, `da2c5c4`
- Local `main` is ahead of `origin/main`.
- The engine/editor shell is now much less coupled to DoomExe.
- The engine now has a generic playable preview path for mounted projects that is
  separate from the editor shell and separate from DoomExe’s own crate-specific flow.
- The editor shell now launches that preview path intentionally instead of
  treating full project runtime as an in-editor state transition.
- The engine now has the first scaffolding layer for registry-driven custom game
  data that can sit beside `project.json`, scenes, story graphs, and the legacy
  built-in database.
- `engine/src` and `engine/tests` no longer contain DoomExe/hamster sample naming
  in engine-generic code paths.
- The current onboarding path now points contributors toward the engine-first
  docs and explicitly marks older roadmap/spec material as historical.

Intentional sample-specific references still remain in DoomExe and in older
historical docs. That is expected.

---

## Best Next Work

1. Expand the custom-document platform from scaffold-level support into richer
   authoring and extension workflows: typed editors, reference pickers,
   validators, and better preview-profile UX.
2. Expand generic runtime preview capabilities beyond the current
   `Title -> Dialogue -> Overworld` baseline while keeping DoomExe-specific battle,
   continue/save UX, and sample gameplay out of the engine crate.
3. Improve editor/runtime extension seams so future games can mount custom
   panels, preview presets, and runtime data consumers without forking the shell.
4. Use `docs/CURRENT_GAPS.md` as the short current planning map while older
   long-form roadmap/spec docs remain historical context.
