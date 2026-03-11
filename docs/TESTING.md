# DJ Engine Testing Guide

This document explains how to write and run tests for DJ Engine.

## Running Tests

### All Tests
```bash
make test
# or
cargo test --workspace
```

### Specific Package
```bash
cargo test -p dj_engine
cargo test -p doomexe
```

### Specific Test
```bash
cargo test test_story_graph_serialization
```

### With Output
```bash
cargo test -- --nocapture
```

---

## Test Structure

### Unit Tests (in same file)
```rust
// At bottom of any .rs file
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

### Integration Tests (separate files)
Location: `engine/tests/*.rs`

```rust
// engine/tests/integration_tests.rs
use dj_engine::prelude::*;

#[test]
fn test_engine_initialization() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DJEnginePlugin);
    app.update();
}
```

---

## Current Test Coverage

| Module | Tests | Description |
|--------|-------|-------------|
| `data::loader` | 3 | Load/save operations |
| `data::scene` | 3 | Scene serialization |
| `data::story` | 3 | Story graph validation |
| `data::database` | 2 | Database operations |
| `data::components` | 2 | Component serialization |
| `data::assets` | 1 | Asset indexing |
| `data::project` | 2 | Project structure |
| `data::spawner` | 1 | Entity spawning |
| `editor_integrity` | 2 | Editor plugin tests |
| `integration` | 2 | Full engine tests |
| `doomexe::hamster` | 4 | Game-specific tests |

**Total: 26 tests**

---

## Writing Good Tests

### Test One Thing
```rust
// ✅ Good - tests one behavior
#[test]
fn test_add_node_increases_count() {
    let mut graph = StoryGraphData::new("test", "Test");
    assert_eq!(graph.nodes.len(), 0);
    
    graph.add_node(StoryNodeData::dialogue("n1", "NPC", "Hello"));
    assert_eq!(graph.nodes.len(), 1);
}

// ❌ Bad - tests multiple unrelated things
#[test]
fn test_everything() {
    // Tests add, remove, find, validate all in one
}
```

### Test Edge Cases
```rust
#[test]
fn test_empty_graph_validation() {
    let graph = StoryGraphData::new("test", "Test");
    let errors = graph.validate();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::MissingRootNode(_))));
}

#[test]
fn test_broken_reference_detection() {
    let mut graph = StoryGraphData::new("test", "Test");
    // Node points to non-existent target
    let mut node = StoryNodeData::dialogue("start", "NPC", "Hi");
    node.data = StoryNodeVariant::Dialogue(DialogueNodeData {
        next_node_id: Some("nonexistent".to_string()),
        ..Default::default()
    });
    graph.add_node(node);
    graph.root_node_id = "start".to_string();
    
    let errors = graph.validate();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::BrokenReference { .. })));
}
```

### Use Descriptive Names
```rust
// ✅ Good
#[test]
fn test_load_scene_with_missing_file_returns_file_not_found_error() { ... }

// ❌ Bad
#[test]
fn test1() { ... }
```

---

## Testing Bevy Systems

### Using App for Integration Tests
```rust
#[test]
fn test_editor_initialization() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(EditorPlugin);
    
    // Run one frame
    app.update();
    
    // Check resources exist
    assert!(app.world().contains_resource::<EditorUiState>());
}
```

### Testing with World
```rust
#[test]
fn test_entity_spawning() {
    let mut world = World::new();
    
    // Spawn entity
    let entity = world.spawn((Name::new("Test"), Transform::default())).id();
    
    // Verify
    assert!(world.get::<Name>(entity).is_some());
}
```

---

## Test Data

### Example JSON Files
Located in `engine/examples/`:
- `jrpg_scene.json`
- `td_scene.json`
- `story_graph.json`
- `database.json`

Use these for testing serialization/deserialization.

---

## Continuous Integration

Tests run automatically on:
- Push to `main`
- Pull requests

Required to pass before merge.
