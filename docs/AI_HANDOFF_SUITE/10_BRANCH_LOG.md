# Branch Log

Last updated: 2026-03-10
Canonical repo: `/home/dj/dev/engines/DJ-Engine`

This file tracks the purpose, status, and next action for local working
branches so branch intent does not get lost between sessions.

## Current Branch Map

| Branch | HEAD | Purpose | Status | Next Action |
|--------|------|---------|--------|-------------|
| `main` | `7c3602c` | Current integration branch | Active. Local `main` is ahead of `origin/main` by 13 commits. A docs-only cleanup batch is staged; unrelated devcontainer, workflow, Makefile, Cargo, and helper-script changes remain unstaged in the worktree. | Commit the staged docs cleanup separately, then continue Phase 1 quick wins from `docs/ROADMAP.md`. |
| `checkpoint/phase3-phase4-save-crt` | `1d61b7e` | Save/load and CRT checkpoint branch | Parked checkpoint branch. | Keep as recovery/reference point unless resumed for save/load or CRT-specific work. |
| `feat/phase5-custom-aabb-collision` | `74505d0` | Collision prototype branch | Feature branch exists with custom AABB collision and trigger interaction work. | Revisit when Phase 5 becomes active or when collision decisions need comparison against `main`. |
| `refactor/phase6-data-api-cleanup` | `7c3602c` | Data/API cleanup refactor branch | Matches `main` tip right now. | Use when Phase 6 data cleanup starts; otherwise leave parked. |

## Planned Next Work

The current planned order from `docs/ROADMAP.md` is:

1. Commit the staged docs-only cleanup on `main`.
2. Phase 1 quick wins:
   - `1b` Story graph validation
   - `1c` `set_next` helper
   - `1d` editor MIDI path warning cleanup
3. Phase 2 audio crossfade.
4. Phase 3 save/load.
5. Phase 4 CRT post-processing.
6. Phase 5 physics/collision follow-up.

## Logging Rules

- Update this file when a branch is created, renamed, repurposed, merged, or parked.
- Record the branch head, a one-line purpose, current status, and next action.
- Prefer keeping branch purpose notes here and in session handoff docs rather than relying on local-only git metadata.
- Keep branch work scoped: docs-only commits stay separate from runtime or infra changes.
- At the start of a session, check `git status --short --branch` and `git branch -vv`.
- At the end of a session, update this log if branch intent or status changed.

## Session Notes

- 2026-03-10: Active work is on `main`. Docs-only cleanup is staged separately from unrelated in-progress infra/code changes.
