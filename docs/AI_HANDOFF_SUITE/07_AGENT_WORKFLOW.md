# Agent Workflow

## Read Order For A New Agent

Start with:

1. `docs/AI_HANDOFF_SUITE/00_ACCURACY_AUDIT.md`
2. `docs/AI_HANDOFF_SUITE/README.md`
3. `docs/AI_HANDOFF_SUITE/01_CURRENT_STATE.md`
4. `Cargo.toml`
5. `engine/src/lib.rs`
6. `engine/src/core/mod.rs`
7. `games/dev/doomexe/src/main.rs`
8. the specific module you are changing

## Fast Orientation Commands

Use these first:

```bash
git status --short --branch
rg --files engine/src games/dev/doomexe/src tools/asset_generator/src .devcontainer .github/workflows docs/AI_HANDOFF_SUITE
sed -n '1,220p' Cargo.toml
sed -n '1,220p' rust-toolchain.toml
```

## Validation Ladder

Use the lightest command that still proves the change:

1. `cargo fmt --all --check`
2. `CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo check --workspace`
3. `CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo test --workspace`
4. `CARGO_TARGET_DIR=/home/dj/.cargo-targets/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all`
5. `./dj e --test-mode`
6. `timeout 20s ./dj d`

Do not skip straight to speculative refactors when a smaller compile or runtime
smoke will answer the question faster.

## High-Value Truths To Remember

- Bevy is `0.18`, even if older docs say `0.15`.
- The repo is a mix of current code and older aspirational design docs.
- The editor and game are real windowed apps, so GUI assumptions matter.
- Some subsystems are deliberately partial or TODO-backed; avoid documenting them
  as complete if you have not read the code.

## Files That Deserve Extra Skepticism

- `AGENTS.md`
  - useful for workflow constraints, but its project overview is partly stale
- older design-roadmap docs under `docs/`
  - often valuable context, but not always current repo truth
- `games/dev/doomexe/README.md`
  - treat as background material, not canonical current-state documentation

## Safe Default Places To Inspect Before Changing Behavior

After module decomposition (March 2026), the `mod.rs` files for editor,
story_graph, and midi are thin orchestrators. Inspect the focused submodules:

- `engine/src/lib.rs`
- `engine/src/core/mod.rs`
- `engine/src/types.rs`
- `engine/src/editor/panels.rs` — UI panel drawing
- `engine/src/editor/views.rs` — grid and story graph visualization
- `engine/src/editor/scene_io.rs` — save/load I/O, world-scene conversion
- `engine/src/editor/plugin.rs` — EditorPlugin lifecycle
- `engine/src/story_graph/types.rs` — types, resources, events, flags
- `engine/src/story_graph/executor.rs` — graph execution, node processing
- `engine/src/midi/wav.rs` — WAV synthesis, MIDI file loading
- `engine/src/midi/sequencer.rs` — sequencer, command handling
- `engine/src/assets/loaders.rs` — HamsterPartLoader, PaletteLoader
- `engine/src/scripting/mod.rs`
- `games/dev/doomexe/src/main.rs`
- `games/dev/doomexe/src/state.rs`

## Common Pitfalls

- Assuming the asset tree is richer than what is actually committed.
- Forgetting that startup audio is muted on purpose.
- Treating story graph serialization coverage as if every variant is already
  wired into runtime execution.
- Removing diagnostics or editor behavior just to simplify remote builds.
- Confusing Codespaces prebuild warmup with runtime smoke validation.

## Git Hygiene

- Fetch before making assumptions about remote state.
- Check `git status` before and after edits.
- Keep commits focused and descriptive.
- Do not overwrite user-authored handoff notes unless explicitly asked.
- If older prose conflicts with current refs or source, trust current refs and
  code.
