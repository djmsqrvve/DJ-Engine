# DJ Engine Bevy 0.18 Migration Handoff

> **Historical.** This migration is complete. The engine now runs on Bevy 0.18 with Rust 1.94.0.

Date: 2026-03-08
Branch: `chore/cleanup-checkpoint-2026-03-08`

## Goal
Align DJ Engine with the Helix 3D renderer baseline by moving DJ Engine from Bevy 0.15-era APIs toward Bevy 0.18-compatible APIs and dependency versions.

## Current Status
- Workspace dependency versions were updated toward Bevy 0.18 parity.
- Large portions of the event API migration were completed (`Event*` -> `Message*`, `.add_event` -> `.add_message`) across multiple modules.
- Diagnostics and several warning fixes were applied.
- The repo currently captures a stable checkpoint of in-progress migration work.

## Remaining Compile Errors (at checkpoint time)
Run used:
```bash
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo check --workspace
```

Top remaining blocks:
- `engine/src/editor/mod.rs`
  - `EguiPlugin` initialization update required.
  - `EventWriter` -> `MessageWriter` conversions still pending in some systems.
  - Egui visuals API updates (`window_rounding` -> `window_corner_radius`).
  - `ctx_mut()` result handling updates.
  - `Timer::finished()` -> `Timer::is_finished()`.
  - `rect_stroke` now requires `StrokeKind`.
- `engine/src/data/spawner.rs`
  - `despawn_recursive()` no longer available in this context.
- `engine/src/story_graph/mod.rs`
  - One choice-index reference/value mismatch remains.
- `engine/src/midi/mod.rs`
  - A few `MessageReader` value/reference deref fixes still needed.

## Suggested Next Steps
1. Finish editor API migration first (`engine/src/editor/mod.rs`), then re-run `cargo check --workspace`.
2. Apply the small targeted fixes in `spawner`, `story_graph`, and `midi`.
3. Re-run:
   - `cargo fmt --all`
   - `RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo check --workspace`
4. Once green, run runtime workflows (`make dev-fast`, `make dev-web`, `make guardrail`) and commit in small logical batches.

## Files touched in this migration checkpoint
- `Cargo.toml`
- `Cargo.lock`
- `engine/Cargo.toml`
- `engine/src/audio/mod.rs`
- `engine/src/data/spawner.rs`
- `engine/src/diagnostics/console.rs`
- `engine/src/diagnostics/mod.rs`
- `engine/src/editor/mod.rs`
- `engine/src/midi/mod.rs`
- `engine/src/scene/mod.rs`
- `engine/src/scripting/mod.rs`
- `engine/src/story_graph/mod.rs`

