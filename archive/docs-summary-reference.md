# DJ Engine: Documentation Summary & Quick Reference

> **Current summary (2026-03-10).** The original January 2026 version of this file assumed a pre-Bevy 0.18 doc set. This refreshed summary points to the docs that still exist and match the current repo.

## Start Here

1. `README.md` - current commands, Codespaces notes, and workspace overview
2. `docs/AI_HANDOFF_SUITE/README.md` - fastest current onboarding path
3. `docs/ROADMAP.md` - prioritized next implementation work
4. `docs/GETTING_STARTED.md` - setup and remote validation

## Current Source-Of-Truth Docs

### Commands, setup, and validation

- `README.md`
- `AGENTS.md`
- `docs/GETTING_STARTED.md`
- `docs/AI_HANDOFF_SUITE/06_REMOTE_DEV_AND_CI.md`
- `docs/TESTING.md`

### Repo orientation

- `docs/AI_HANDOFF_SUITE/01_CURRENT_STATE.md`
- `docs/AI_HANDOFF_SUITE/02_WORKSPACE_MAP.md`
- `docs/AI_HANDOFF_SUITE/03_ENGINE_GUIDE.md`
- `docs/AI_HANDOFF_SUITE/07_AGENT_WORKFLOW.md`
- `docs/ARCHITECTURE.md`

### Planning and current priorities

- `docs/ROADMAP.md`
- `docs/INDEX_Navigation_Guide.md`
- `docs/MASTER-DOCUMENTATION-INDEX.md`

## Historical Planning Docs

These are still useful for background, but they are not current implementation guides:

- `docs/complete-detailed-docs.md`
- `docs/DETAILED_TASK_DOCS.md`
- `docs/HANDOFF_BEVY18_MIGRATION.md`
- `docs/HANDOFF_CODESPACES_COMPILE_CLEANUP.md`
- `docs/HANDOFF_BROAD_AUDIT.md`

## Archived Legacy Specs

The pre-migration design/spec files now live under `archive/`:

- `archive/SPRITE_ARCHITECTURE.md`
- `archive/SPRITE_SYSTEM.md`
- `archive/ANIMATION_GUIDE.md`
- `archive/PROJECT_PLAN.md`
- `archive/LUA_FFI.md`
- `archive/ASSET_PIPELINE.md`
- `archive/SPRITE_QUICKSTART.md`
- `archive/README.md`

## Quick Answers

- Need the right commands: check `README.md`, `AGENTS.md`, or `Makefile`
- Need current repo truth: start in `docs/AI_HANDOFF_SUITE/`
- Need what to build next: read `docs/ROADMAP.md`
- Need older design rationale: read the matching file under `archive/`

## Notes

- `DJ_ENGINE_PHASED_DEVELOPMENT_PLAN.md` was never created.
- The legacy `./dj` helper has been retired; current docs should use `make`.
- If docs disagree, prefer `Makefile`, `rust-toolchain.toml`, `.github/workflows/ci.yml`, and the current Rust source.
