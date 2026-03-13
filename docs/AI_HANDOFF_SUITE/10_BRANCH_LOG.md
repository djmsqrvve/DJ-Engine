# Branch Log

Last updated: 2026-03-13
Canonical repo: `/home/dj/dev/engines/DJ-Engine`

This file tracks the purpose, status, and next action for local working
branches so branch intent does not get lost between sessions.

## Current Branch Map

| Branch | Checkpoint | Purpose | Status | Next Action |
|--------|------------|---------|--------|-------------|
| `main` | `ad8257e` | Current integration branch | Active. `main` contains inline table editing and safe mutation helpers. The current worktree adds table.rs extraction, nested property editing (inspector + data layer), nested round-trip integration tests, and editor extension wiring (toolbar actions via resource queue, preview preset selector, Helix registrations). | Wire toolbar action handlers, custom panel draw callbacks, preview preset launch integration, graph editing for relationship-heavy kinds, and runtime preview robustness. |
| `checkpoint/phase3-phase4-save-crt` | `1d61b7e` | Save/load and CRT checkpoint branch | Parked checkpoint branch. | Keep as recovery/reference point unless resumed for save/load or CRT-specific work. |
| `feat/phase5-custom-aabb-collision` | `74505d0` | Collision prototype branch | Feature branch exists with custom AABB collision and trigger interaction work. | Revisit when Phase 5 becomes active or when collision decisions need comparison against `main`. |
| `refactor/phase6-data-api-cleanup` | `7c3602c` | Data/API cleanup refactor branch | Matches `main` tip right now. | Use when Phase 6 data cleanup starts; otherwise leave parked. |

## Planned Next Work

The immediate follow-up order after the 2026-03-13 inline table-editing slice is:

1. Expand the generic authoring surface beyond top-level scalar table edits into nested/object/list editing, graph tooling, and broader reference-aware property inspection.
2. Turn the editor extension registry from metadata-only registration into real UI/runtime seams: toolbar actions, preview preset selection, and useful custom panels.
3. Continue generic runtime preview behavior beyond the current `Title -> Dialogue -> Overworld` baseline while keeping DoomExe-specific gameplay systems outside the engine crate.
4. Keep current docs aligned with the live engine-first workflow as new authoring/runtime features land.

## Logging Rules

- Update this file when a branch is created, renamed, repurposed, merged, or parked.
- Record a stable checkpoint commit, a one-line purpose, current status, and next action.
- Prefer keeping branch purpose notes here and in session handoff docs rather than relying on local-only git metadata.
- Keep branch work scoped: docs-only commits stay separate from runtime or infra changes.
- At the start of a session, check `git status --short --branch` and `git branch -vv`.
- At the end of a session, update this log if branch intent or status changed.

## Session Notes

- 2026-03-10: Active work is on `main`. Docs-only cleanup is staged separately from unrelated in-progress infra/code changes.
- 2026-03-11: Landed integration commit `e9a37a8` (`feat: advance engine editor and decoupling work`) covering spawner runtime initialization, project-agnostic editor loading/saving/play, engine-first launch defaults, and generic sprite-part API naming.
- 2026-03-11: Landed integration commit `de5b8ea` (`feat: add mounted project runtime preview`) covering the shared `MountedProject` path, the `runtime_preview` binary, `make preview`, generic `Title -> Dialogue -> Overworld` preview flow, and the story-graph advance fix required to complete that loop.
- 2026-03-11: Landed integration commit `7d0f291` (`feat: hand off editor play to runtime preview`) so the editor now auto-saves mounted projects, launches the separate `runtime_preview` process via `Run Project`, tracks preview lifecycle state, and keeps graph preview as a Story Graph-only editor tool.
- 2026-03-11: Landed integration commit `cb9e2be` (`feat: add preview continue flow and editor dirty tracking`) adding project-scoped runtime preview saves, `Continue`, and snapshot-based dirty tracking plus guarded reloads in the editor shell.
- 2026-03-11: Landed integration commit `7555544` (`feat: add custom document scaffolding`) adding `data/registry.json` discovery, generic custom document registration/validation, a document browser/editor surface in the engine editor, and preview-profile-driven custom-data loading in `runtime_preview`.
- 2026-03-11: Landed docs-only commit `da2c5c4` (`docs: refresh current onboarding guides`) aligning current onboarding/handoff docs with the engine-first workflow and adding a high-level current gap map.
- 2026-03-11: Landed integration commit `85f32fe` (`feat: add structured custom document editing`) adding structured custom-document metadata editing, editable reference-link pickers, and a typed `preview_profiles` editor on top of the raw JSON custom-document browser.
- 2026-03-13: Landed integration commit `c3a11a2` (`feat: add table editor and helix data plugin with import pipeline`) adding the generic table route/browser, the Helix data bridge/import loop, and table-route registry coverage.
- 2026-03-13: Landed integration commit `ad8257e` (`feat: inline table editing, safe mutation helpers, helix round-trip tests`) adding inline editing for table-route label/top-level scalar fields, field-targeted validation cues, and Helix save/reload/index round-trip coverage.
- 2026-03-13: Current worktree extends `ad8257e` with table.rs extraction, nested path mutation helper (`update_loaded_custom_document_nested_value`), recursive property inspector widget (`property_widgets.rs`), nested edit round-trip integration test, and editor extension wiring (ToolbarActionQueue, SelectedPreviewPreset, Tools menu, preset selector, Helix registrations).
