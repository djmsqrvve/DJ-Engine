# Branch Log

Last updated: 2026-03-11
Canonical repo: `/home/dj/dev/engines/DJ-Engine`

This file tracks the purpose, status, and next action for local working
branches so branch intent does not get lost between sessions.

## Current Branch Map

| Branch | Checkpoint | Purpose | Status | Next Action |
|--------|------------|---------|--------|-------------|
| `main` | `7d0f291` | Current integration branch | Active. Local `main` now contains the spawner runtime-state slice, manifest-driven editor/project mounting decoupling, engine-first launch defaults, generic sprite-part API cleanup, the mounted-project runtime preview flow, and the editor-to-runtime preview handoff. | Continue engine/editor work on top of the new handoff path: richer project boot flows, runtime preview expansion, and engine-only UX cleanup. |
| `checkpoint/phase3-phase4-save-crt` | `1d61b7e` | Save/load and CRT checkpoint branch | Parked checkpoint branch. | Keep as recovery/reference point unless resumed for save/load or CRT-specific work. |
| `feat/phase5-custom-aabb-collision` | `74505d0` | Collision prototype branch | Feature branch exists with custom AABB collision and trigger interaction work. | Revisit when Phase 5 becomes active or when collision decisions need comparison against `main`. |
| `refactor/phase6-data-api-cleanup` | `7c3602c` | Data/API cleanup refactor branch | Matches `main` tip right now. | Use when Phase 6 data cleanup starts; otherwise leave parked. |

## Planned Next Work

The immediate follow-up order after the 2026-03-11 engine preview checkpoint is:

1. Expand generic runtime preview behavior beyond `Title -> Dialogue -> Overworld` while keeping DoomExe-specific gameplay systems outside the engine crate.
2. Continue engine-only decoupling in older prose docs and onboarding material outside the already-updated engine-facing docs.
3. Improve editor/runtime workflow polish on top of the new handoff path: clearer mounted-project status, exit reporting, and future dirty-state UX.

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
