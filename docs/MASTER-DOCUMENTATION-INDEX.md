# DJ Engine: Master Documentation Index

> **Current index (2026-03-10).** The original January 2026 version of this file referenced a pre-migration doc set. This refreshed index points to the files that still exist and match the current repo.

## Start Here

1. `README.md` - current commands, Codespaces notes, and workspace overview
2. `docs/AI_HANDOFF_SUITE/README.md` - fastest current onboarding path
3. `docs/ROADMAP.md` - prioritized next implementation work
4. `docs/GETTING_STARTED.md` - setup and remote validation
5. `AGENTS.md` - repo-specific contributor and agent guidance

## Current Source-Of-Truth Docs

### Onboarding and workflow

- `README.md`
- `AGENTS.md`
- `docs/GETTING_STARTED.md`
- `docs/AI_HANDOFF_SUITE/01_CURRENT_STATE.md`
- `docs/AI_HANDOFF_SUITE/02_WORKSPACE_MAP.md`
- `docs/AI_HANDOFF_SUITE/03_ENGINE_GUIDE.md`
- `docs/AI_HANDOFF_SUITE/06_REMOTE_DEV_AND_CI.md`
- `docs/AI_HANDOFF_SUITE/07_AGENT_WORKFLOW.md`
- `docs/TESTING.md`

### Current planning and navigation

- `docs/ROADMAP.md`
- `docs/INDEX_Navigation_Guide.md`
- `docs/docs-summary-reference.md`
- `docs/ARCHITECTURE.md`

## Historical Docs Still In `docs/`

These files are useful for background, but they are not current implementation guides:

- `docs/complete-detailed-docs.md`
- `docs/DETAILED_TASK_DOCS.md`
- `docs/HANDOFF_BEVY18_MIGRATION.md`
- `docs/HANDOFF_CODESPACES_COMPILE_CLEANUP.md`
- `docs/HANDOFF_BROAD_AUDIT.md`
- `docs/AI_HANDOFF_SUITE/08_SESSION_HANDOFF_2026_03_09.md`
- `docs/AI_HANDOFF_SUITE/09_SESSION_HANDOFF_2026_03_10.md`
- `docs/AI_HANDOFF_SUITE/00_ACCURACY_AUDIT.md`
- `docs/AI_HANDOFF_SUITE/AUDIT_REQUEST.md`
- `docs/AI_HANDOFF_SUITE/PROMPT.md`

## Archived Legacy Specs

The pre-migration design/spec files now live under `archive/`:

- `archive/SPRITE_ARCHITECTURE.md`
- `archive/SPRITE_SYSTEM.md`
- `archive/ANIMATION_GUIDE.md`
- `archive/PROJECT_PLAN.md`
- `archive/LUA_FFI.md`
- `archive/ASSET_PIPELINE.md`
- `archive/SPRITE_QUICKSTART.md`
- `archive/ARCHITECTURE.md`
- `archive/README.md`

## How To Use This Index

- Need current commands: start with `README.md`, `AGENTS.md`, or `Makefile`
- Need current repo truth: start with the AI Handoff Suite
- Need what to build next: read `docs/ROADMAP.md`
- Need old design rationale: open the matching file under `archive/`
- Need to validate a doc claim: prefer `Makefile`, `rust-toolchain.toml`, `.github/workflows/ci.yml`, and current Rust source over prose

## Notes

- `DJ_ENGINE_PHASED_DEVELOPMENT_PLAN.md` was never created.
- The legacy `./dj` helper has been retired; current docs should use `make`.
- If two docs disagree, prefer the current source-of-truth docs listed above.
**Total Code Templates**: 2,300+ lines  
**Time to Read (Exec Summary)**: 1 hour  
**Time to Read (Complete)**: 8–10 hours  
**Time to Start Building**: 30 minutes after reading Phase 0  

**Status**: ✅ **READY FOR EXECUTION**
