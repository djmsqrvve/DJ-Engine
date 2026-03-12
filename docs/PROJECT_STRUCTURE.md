# DJ Engine Project Structure

Current reference for the checked-in workspace layout.

## Root Directory

```text
dj_engine/
├── .devcontainer/        # Codespaces and remote-dev config
├── .github/              # CI and GitHub workflow config
├── docs/                 # Current docs, handoff notes, and historical specs
├── engine/               # Reusable engine crate and binaries
├── games/                # Sample and future game crates
├── tools/                # Workspace utilities
├── Cargo.toml            # Workspace manifest
├── Cargo.lock            # Dependency lock file
├── Makefile              # Unified command surface
├── rust-toolchain.toml   # Pinned Rust toolchain
├── README.md             # Repo overview
└── LICENSE               # MIT license
```

## Workspace Crates

### `engine/`

The main engine crate. It is also the default workspace member.

Important files:

- `engine/src/lib.rs`
  - Library root and public reexports.
- `engine/src/main.rs`
  - Editor binary entrypoint (`dj_engine`).
- `engine/src/bin/minimal.rs`
  - Minimal rendering smoke binary.
- `engine/src/bin/runtime_preview.rs`
  - Engine-owned playable preview binary for mounted projects.
- `engine/src/project_mount.rs`
  - Shared mounted-project path normalization and manifest loading.

Important module directories:

```text
engine/src/
├── animation/
├── assets/
├── audio/
├── bin/
├── collision/
├── core/
├── data/
├── diagnostics/
├── editor/
├── input/
├── midi/
├── rendering/
├── runtime_preview/
├── scene/
├── scripting/
└── story_graph/
```

Notable data/editor/runtime files:

- `engine/src/data/project.rs`
  - Project manifest, startup defaults, and project-relative path settings.
- `engine/src/data/custom.rs`
  - Registry-driven custom documents under `data/registry.json`.
- `engine/src/editor/`
  - Editor shell, document browser, graph preview, runtime handoff, and dirty tracking.
- `engine/src/runtime_preview/mod.rs`
  - Title, dialogue, overworld preview, continue flow, and preview-profile loading.

### `games/dev/doomexe/`

Sample game crate that exercises the engine. It is not the engine source of
truth, but it remains a useful consumer and regression target.

Current high-level layout:

```text
games/dev/doomexe/
├── Cargo.toml
├── assets/
│   ├── music/
│   ├── palettes/
│   └── scripts/
├── docs/
└── src/
    ├── assets/
    ├── battle/
    ├── dialogue/
    ├── hamster/
    ├── hud/
    ├── overworld/
    └── scripting/
```

### `tools/asset_generator/`

Small workspace utility for MIDI generation and optional asset repair helpers.

## Docs Directory

`docs/` contains both current guides and older planning/spec files.

Use these as current docs first:

- `docs/README.md`
- `docs/GETTING_STARTED.md`
- `docs/ARCHITECTURE.md`
- `docs/TESTING.md`
- `docs/AI_HANDOFF_SUITE/`

Treat these as historical/planning context rather than current repo truth:

- `docs/ROADMAP.md`
- `docs/Game_Engine_Technical_Roadmap.md`
- `docs/EDITOR_Specification_Complete.md`
- `docs/complete-detailed-docs.md`
- `docs/DETAILED_TASK_DOCS.md`
- `docs/Architecture_Specification.json`

## Mounted Project Shape

Projects opened through the editor or runtime preview are rooted at
`project.json` and can carry custom documents beside scenes and story graphs:

```text
project.json
scenes/
story_graphs/
assets/
data/
  registry.json
  <custom_kind>/
```

## Key Commands

- `make dev`
  - Launch the editor.
- `make preview PROJECT=/path/to/project`
  - Launch runtime preview for a mounted project.
- `make game`
  - Launch the sample DoomExe game.
- `make test`
  - Run workspace tests.
