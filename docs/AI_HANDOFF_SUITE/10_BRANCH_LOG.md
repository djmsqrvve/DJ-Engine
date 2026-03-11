# Branch Log

Last updated: 2026-03-11
Canonical repo: `/home/dj/dev/engines/DJ-Engine`

This file tracks the purpose, status, and next action for local working
branches so branch intent does not get lost between sessions.

## Current Branch Map

| Branch | HEAD | Purpose | Status | Next Action |
|--------|------|---------|--------|-------------|
| `main` | `e9a37a8` | Current integration branch | Active. Local `main` now contains the spawner runtime-state slice, manifest-driven editor/project mounting decoupling, engine-first launch defaults, and generic sprite-part API cleanup. | Continue engine/sample separation outside the engine-facing surface: historical docs cleanup, sample-game boundary cleanup, and the next editor/runtime improvements. |
| `checkpoint/phase3-phase4-save-crt` | `1d61b7e` | Save/load and CRT checkpoint branch | Parked checkpoint branch. | Keep as recovery/reference point unless resumed for save/load or CRT-specific work. |
| `feat/phase5-custom-aabb-collision` | `74505d0` | Collision prototype branch | Feature branch exists with custom AABB collision and trigger interaction work. | Revisit when Phase 5 becomes active or when collision decisions need comparison against `main`. |
| `refactor/phase6-data-api-cleanup` | `7c3602c` | Data/API cleanup refactor branch | Matches `main` tip right now. | Use when Phase 6 data cleanup starts; otherwise leave parked. |

## Planned Next Work

The immediate follow-up order after the 2026-03-11 integration checkpoint is:

1. Continue engine-only decoupling in older prose docs and onboarding material outside the already-updated engine-facing docs.
2. Tighten sample-game boundaries where DoomExe-specific concepts still leak into optional workflows or shared assumptions.
3. Resume the next engine/editor improvements from the roadmap on top of the now-cleaner manifest-driven editor shell.

## Logging Rules

- Update this file when a branch is created, renamed, repurposed, merged, or parked.
- Record the branch head, a one-line purpose, current status, and next action.
- Prefer keeping branch purpose notes here and in session handoff docs rather than relying on local-only git metadata.
- Keep branch work scoped: docs-only commits stay separate from runtime or infra changes.
- At the start of a session, check `git status --short --branch` and `git branch -vv`.
- At the end of a session, update this log if branch intent or status changed.

## Session Notes

- 2026-03-10: Active work is on `main`. Docs-only cleanup is staged separately from unrelated in-progress infra/code changes.
- 2026-03-11: Landed integration commit `e9a37a8` (`feat: advance engine editor and decoupling work`) covering spawner runtime initialization, project-agnostic editor loading/saving/play, engine-first launch defaults, and generic sprite-part API naming.
