# DJ Engine Core

This crate is the reusable engine layer for DJ Engine projects.

## Current Focus

- Bevy-based runtime and editor foundations
- Data-driven scenes, story graphs, and project manifests
- Registry-driven custom documents under `data/registry.json`
- Manifest-driven runtime preview flow for mounted projects
- Palette-aware rendering and post-processing
- Lua scripting hooks for gameplay and tools
- 2D sprite-part animation helpers for character assembly

## Design Constraints

- Engine code stays project-agnostic and avoids sample-game assumptions.
- Prefer config-driven systems over hardcoded content-specific logic.
- Keep modules composable so games can opt into only the systems they need.

## Sample Integration

`games/dev/doomexe` is the current sample game that exercises the engine, but it is not the source of truth for engine APIs. Engine naming, data formats, and editor behavior should stay generic so future games can mount into the same workflow cleanly.

## Engine Entry Points

- `cargo run -p dj_engine --bin dj_engine` launches the editor shell.
- `cargo run -p dj_engine --bin runtime_preview -- --project <dir|project.json>` launches the engine-owned playable preview flow for a mounted project.
- `make dev` launches the editor, and `make preview PROJECT=<dir|project.json>` launches the runtime preview.
- Inside the editor, `Run Project` now auto-saves the mounted project and hands
  off to the separate `runtime_preview` process, while `Preview Graph` remains
  an editor-only Story Graph tool.

## Custom Documents

Mounted projects can now define generic custom documents in `data/registry.json`
without changing engine core. The engine owns discovery, path resolution,
validation payloads, editor routing, and runtime-preview loading for those
documents. Games are expected to register their own kinds and validators on top
of the shared registry surface.

The first built-in contract is `preview_profiles`, which lets runtime preview
request a scene, story graph, and custom document bundle together through the
same mounted-project flow.
