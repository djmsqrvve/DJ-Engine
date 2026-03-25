# DJ Engine: Documentation Index

> Current index (2026-03-24). Points to the active documentation that matches the current repo.

## Start Here

1. `README.md` -- project overview, commands, Codespaces notes
2. `docs/GETTING_STARTED.md` -- local setup, remote validation
3. `docs/ROADMAP.md` -- prioritized next implementation work
4. `CONTRIBUTING.md` -- contribution workflow and expectations

## Current Documentation

### Setup and Workflow

- `README.md` -- quick start, commands, project structure
- `docs/GETTING_STARTED.md` -- local setup and validation
- `CONTRIBUTING.md` -- contribution guidelines
- `SECURITY.md` -- vulnerability reporting
- `MAINTAINERS.md` -- project maintenance guide

### Architecture and Design

- `docs/ARCHITECTURE.md` -- engine system overview
- `docs/PROJECT_STRUCTURE.md` -- detailed workspace layout
- `docs/CODE_STYLE.md` -- Rust coding conventions

### Planning

- `docs/ROADMAP.md` -- phased development plan (Phases 1-4 complete, 411 tests)
- `docs/CURRENT_GAPS.md` -- short snapshot of remaining gaps
- `docs/CURRENT_PRIORITIES.md` -- near-term and mid-term execution priorities
- `docs/LONG_TERM_GOALS.md` -- engine vision and strategic proof points

### Testing

- `docs/TESTING.md` -- test organization and how to run tests

### Plugins

- `plugins/helix_data/README.md` -- Helix MMORPG data bridge: TOML import, typed registries, contract validation

### Tutorials

- `docs/tutorials/` -- 8-chapter Stratego board game walkthrough with full code

## Archived Documentation

Historical design specs, migration notes, and legacy planning docs live under `archive/`. These are useful for background but are not current implementation guides:

- `archive/` -- pre-migration specs, sprite system docs, early roadmaps
- `archive/ai/` -- AI agent configuration and session handoff files

## How To Use This Index

- Need current commands: start with `README.md` or `Makefile`
- Need architecture understanding: read `docs/ARCHITECTURE.md`
- Need what to build next: read `docs/ROADMAP.md`
- Need old design rationale: open the matching file under `archive/`
- Need to validate a doc claim: prefer `Makefile`, `rust-toolchain.toml`, `.github/workflows/ci.yml`, and current Rust source over prose

## Notes

- The legacy `./dj` helper has been retired; use `make` targets instead.
- If two docs disagree, prefer current source-of-truth docs listed above.
