# DJ Engine Broad Audit Handoff

Date: 2026-03-08  
Branch: `chore/cleanup-checkpoint-2026-03-08`  
Commit: `96ab402`

## Goal
Run a broad, repo-wide technical audit to identify and fix compile errors, API regressions, warnings, test failures, and high-risk code issues after the Bevy 0.18 migration pass.

## Current Baseline
- `cargo check --workspace` passes.
- Known warnings (6): deprecated egui API usage in `engine/src/editor/mod.rs` (`Frame::none`, `Ui::close_menu`).
- `cargo test --workspace --no-run` currently fails with:
  - `engine/tests/editor_integrity.rs:14` -> `HierarchyPlugin` unresolved (`E0425`).
- Tracked TODO/FIXME/HACK/XXX markers in core code paths: 13 (`engine/src`, `games/dev/doomexe/src`, `tools/asset_generator/src`).
- This repo does not use a root `Makefile`; use `./dj` helper and direct cargo commands.

## Verified Commands
Use this target dir consistently for speed and cache isolation:
```bash
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18
```

Baseline checks:
```bash
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo check --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo test --workspace --no-run
cargo fmt --all
```

Runtime entry points:
```bash
./dj minimal
./dj doomexe --verbose
./dj editor
```

## Priority Audit Plan
1. **Stabilize test compilation first**
   - Fix `engine/tests/editor_integrity.rs` unresolved `HierarchyPlugin`.
   - Re-run `cargo test --workspace --no-run` until compile-clean.

2. **Deprecation cleanup in editor**
   - File: `engine/src/editor/mod.rs`.
   - Replace:
     - `egui::Frame::none()` -> `egui::Frame::NONE` or `egui::Frame::new()`.
     - `ui.close_menu()` -> `ui.close()` / `ui.close_kind(...)`.
   - Re-run `cargo check --workspace`.

3. **Broad compile/lint sweep**
   - Run `cargo check --workspace` and fix newly surfaced warnings/errors.
   - Then run `cargo clippy --workspace --all-targets -- -W clippy::all` and triage:
     - fix clear correctness/perf issues now,
     - defer low-value style nits only if justified.

4. **Runtime smoke validation**
   - Verify minimal/editor/game startup (`./dj minimal`, `./dj editor`, `./dj doomexe --verbose`).
   - Capture any panic/backtrace and patch obvious startup regressions.

5. **Code health backlog pass**
   - Review and triage existing TODOs in:
     - `engine/src/data/spawner.rs`
     - `engine/src/rendering/mod.rs`
     - `engine/src/assets/mod.rs`
     - `engine/src/data/story.rs`
     - `games/dev/doomexe/src/title.rs`
   - Convert high-risk TODOs into fixes or tracked issues.

## Known Hotspots
- `engine/src/editor/mod.rs` (current deprecations + large UI surface).
- `engine/tests/editor_integrity.rs` (currently breaks `cargo test --no-run`).
- `games/dev/doomexe/src/dialogue/ui.rs` (complex UI flow, recent migration churn).
- `games/dev/doomexe/src/overworld/mod.rs` and `hud/minimap.rs` (recent Bevy API updates).

## Deliverable Format for Audit Agent
For each batch, report:
1. What command was run.
2. Exact errors/warnings found.
3. Fixes applied (files + intent).
4. Re-run result.
5. Remaining blockers.

Keep commits small and scoped by concern (`test-fix`, `editor-deprecation`, `runtime-smoke-fix`, etc.).

## Exit Criteria
- `cargo fmt --all` clean.
- `cargo check --workspace` clean (or only explicitly accepted warnings).
- `cargo test --workspace --no-run` clean.
- At least one successful runtime launch path validated.
- Final summary includes remaining risk items and deferred work.

