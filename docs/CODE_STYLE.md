# DJ Engine Code Style Guide

This document defines coding standards for DJ Engine contributors.

## Formatting

### Automatic Formatting

Always run before committing:
```bash
cargo fmt
```

### Line Length
- **Soft limit**: 100 characters
- **Hard limit**: 120 characters

---

## Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Structs | PascalCase | `StoryGraph`, `PlayerData` |
| Traits | PascalCase | `Spawnable`, `Validatable` |
| Functions | snake_case | `load_scene`, `spawn_entity` |
| Variables | snake_case | `player_hp`, `node_count` |
| Constants | SCREAMING_SNAKE | `MAX_PLAYERS`, `DEFAULT_HP` |
| Modules | snake_case | `story_graph`, `data` |
| Files | snake_case | `story_graph.rs`, `mod.rs` |

---

## Code Organization

### File Structure
```rust
//! Module documentation
//!
//! Detailed description of what this module does.

// 1. Imports (grouped)
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::data::components::*;

// 2. Constants
const MAX_NODES: usize = 1000;

// 3. Types (structs, enums)
pub struct MyStruct { ... }

// 4. Implementations
impl MyStruct { ... }

// 5. Functions
pub fn helper_function() { ... }

// 6. Tests (at bottom)
#[cfg(test)]
mod tests { ... }
```

### Import Grouping

```rust
// Standard library
use std::collections::HashMap;

// External crates
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Crate modules
use crate::data::*;
use super::utils;
```

---

## Documentation

### Required Documentation
- Public APIs (structs, functions, traits)
- Module-level docs (`//!` at top of file)
- Non-obvious code sections

### Doc Comments
```rust
/// Creates a new story graph with the given ID.
///
/// # Arguments
/// * `id` - Unique identifier for the graph
/// * `name` - Human-readable name
///
/// # Example
/// ```
/// let graph = StoryGraphData::new("intro", "Introduction");
/// ```
pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self { ... }
```

---

## Error Handling

### Use Result for Fallible Operations
```rust
// ✅ Good
pub fn load_scene(path: &Path) -> Result<Scene, DataError> { ... }

// ❌ Bad
pub fn load_scene(path: &Path) -> Scene { panic!("...") }
```

### Custom Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Parse error: {0}")]
    ParseError(String),
}
```

---

## Bevy-Specific Guidelines

### Component Design
```rust
// Use derive macros
#[derive(Component, Debug, Clone)]
pub struct Health(pub i32);

// Small, focused components
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;
```

### System Naming
```rust
// Name describes what it does
fn update_health_system(query: Query<&mut Health>) { ... }
fn spawn_enemies_system(commands: Commands) { ... }
fn handle_input_system(input: Res<ButtonInput<KeyCode>>) { ... }
```

### Resource Access
```rust
// Prefer Res/ResMut over direct World access
fn my_system(config: Res<GameConfig>, mut state: ResMut<GameState>) { ... }
```

---

## Testing Guidelines

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() { ... }

    #[test]
    fn test_edge_case() { ... }
}
```

### Test Naming
```rust
#[test]
fn test_<function>_<scenario>_<expected>() { ... }

// Examples:
fn test_load_scene_valid_json_succeeds() { ... }
fn test_load_scene_missing_file_returns_error() { ... }
```

---

## Linting

Run before committing:
```bash
cargo clippy -- -W clippy::all
```

Address all warnings. Use `#[allow(...)]` sparingly and with justification.

### Accepted Workspace-Level Allows

These clippy lints are suppressed at the crate root (`#![allow(...)]`) across the workspace because they conflict with Bevy's ECS patterns:

- `clippy::too_many_arguments` -- Bevy systems receive many resource/query parameters by design
- `clippy::type_complexity` -- Bevy query types produce deeply nested generics
- `clippy::upper_case_acronyms` -- engine uses acronyms like `CRT`, `BGM`, `SFX`
- `clippy::module_inception` -- some modules share names with their parent (e.g., `hamster/tests`)
- `clippy::drop_non_drop` -- explicit `drop()` is used to end mutable borrows on Bevy `Mut<T>` before re-accessing the world
