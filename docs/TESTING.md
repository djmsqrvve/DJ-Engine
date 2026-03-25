# DJ Engine Testing Guide

This document explains the current validation flow and where tests live in the
workspace.

## Fast Validation Commands

Use the same target-dir convention as the repo docs and handoff suite:

```bash
cargo fmt --all --check
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo check --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo test --workspace
RUSTC_WRAPPER= CARGO_TARGET_DIR=~/.cargo-targets/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all
```

Equivalent `make` entrypoints:

```bash
make fmt
make test
make quality-check
make guardrail
```

## Package-Level Commands

```bash
cargo test -p dj_engine
cargo test -p doomexe
cargo test test_name -- --exact
cargo test -- --nocapture
```

## Runtime Smoke Commands

These are useful when a change touches native window boot, editor flow, runtime
preview, or the sample game:

```bash
timeout 20s make dev
timeout 20s make preview PROJECT=/path/to/project
timeout 20s make game
```

Inside the editor, `Run Project` hands off to the separate `runtime_preview`
process. That means editor and runtime-preview changes often need both unit
coverage and a native smoke boot.

## Test Layout

### Unit tests

Most engine modules keep focused unit tests inline with the source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        let result = my_function(42);
        assert_eq!(result, 42);
    }
}
```

### Integration tests

Cross-system coverage typically lives under `engine/tests/`, especially for:

- editor shell behavior
- mounted project loading
- runtime preview boot paths
- scene/data serialization boundaries

### Temp-project tests

When testing mounted-project flows, prefer `tempfile`-backed project roots with
real `project.json`, `scenes/`, `story_graphs/`, and `data/registry.json`
content instead of hardcoding repo-local fixture assumptions.

## Writing Good Tests

### Test one behavior at a time

```rust
#[test]
fn test_add_node_increases_count() {
    let mut graph = StoryGraphData::new("test", "Test");
    assert_eq!(graph.nodes.len(), 0);

    graph.add_node(StoryNodeData::dialogue("n1", "NPC", "Hello"));
    assert_eq!(graph.nodes.len(), 1);
}
```

### Prefer stable assertions

- Assert on resources, state transitions, and loaded data.
- Avoid brittle UI text assertions unless the exact copy matters.
- For editor/runtime tests, prefer helper-level command resolution and lifecycle
  assertions over trying to automate a full native click path.

### Cover edge cases

- missing files
- broken refs
- duplicate IDs
- malformed custom document manifests
- save/load mismatch conditions
- runtime preview fallback behavior

## Helix Data Plugin

`plugins/helix_data/tests/` contains integration tests:

- `import_integration.rs` -- import fixtures, build runtime index, edit/save/reload round trips
- `cli_smoke.rs` -- smoke tests for helix_dashboard, helix_import, contracts binaries

Run specifically:

```bash
cargo test -p dj_engine_helix
```

## Current Hot Spots Worth Testing

When touching these areas, add or extend tests nearby:

- `engine/src/data/custom.rs`
  - custom document registration, manifest loading, validation, preview profiles
- `engine/src/project_mount.rs`
  - project path normalization and mounted-project resolution
- `engine/src/editor/`
  - dirty tracking, runtime handoff, document browser, load/save behavior
- `engine/src/runtime_preview/mod.rs`
  - title flow, continue flow, dialogue transitions, preview-profile loading
- `games/dev/doomexe/`
  - sample-game compatibility with engine changes

## CI Expectation

Workspace compile and test validation runs in CI. Before landing non-trivial
changes, match that locally with the `fmt`, `check`, `test`, and `clippy`
commands above.
