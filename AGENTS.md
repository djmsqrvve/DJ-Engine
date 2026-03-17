# DJ Engine - AI Agent Configuration Guide

This file provides essential information for AI coding agents working with the DJ Engine project. Read this before making any modifications.

## Project Overview

**DJ Engine** is a custom game framework built on Rust and Bevy 0.18, designed for procedural 2D character animation and narrative-heavy JRPGs with "cursed" visual effects. The engine emphasizes palette-driven corruption effects and Lua scripting for flexible game logic.

### Key Features
- **Procedural 2D Animation**: Breathing, blinking, expression-driven character animations
- **Palette-Driven Effects**: Real-time palette swapping for corruption/distortion visuals  
- **Story Graph System**: Node-based narrative branching with JSON serialization
- **Lua Scripting**: Runtime game logic via mlua, enabling rapid iteration without recompiling Rust
- **Universal Actor System**: Standardized ECS components for both JRPG and RTS gameplay

### Technology Stack
- **Engine**: Bevy 0.18 (ECS, Rendering, Audio, Input)
- **Language**: Rust (core systems), Lua (game logic & content)
- **Data Formats**: JSON (assets, story graphs), TOML (config), Lua (scripts)
- **UI**: bevy_egui (tools/debug), Bevy UI (in-game interfaces)
- **Audio**: rodio + midly (MIDI support)
- **Development Tools**: Custom asset generator, integrated editor

## Project Structure

This is a **Cargo workspace** with six crates:

```
dj_engine/
├── engine/                      # Core engine library (dj_engine crate)
│   └── src/
│       ├── animation/          # Procedural animation systems
│       ├── assets/             # Asset loading and definitions
│       ├── audio/              # Audio systems (SFX, BGM, MIDI)
│       ├── core/               # Master plugin bundle (DJEnginePlugin)
│       ├── data/               # Serialization types, Grid<T>, loading
│       ├── diagnostics/        # Debug overlay and inspector
│       ├── editor/             # In-engine editing tools
│       ├── input/              # Input handling and action mapping
│       ├── midi/               # MIDI music integration
│       ├── rendering/          # Custom rendering & palette effects
│       ├── scene/              # Scene management
│       ├── scripting/          # Lua integration & FFI
│       ├── story_graph/        # Narrative node system
│       ├── types.rs            # Shared engine types
│       └── lib.rs              # Public API & prelude
├── games/dev/doomexe/           # Primary game — hamster narrator JRPG
├── games/dev/stratego/          # Tutorial game — 10x10 board, 8 piece types, AI
├── games/dev/iso_sandbox/       # Isometric sandbox — 16x16 tile grid, entity placement
├── plugins/helix_data/          # Helix data bridge plugin and TOML import tooling
├── tools/asset_generator/       # Build-time asset processing
├── docs/                        # Documentation + tutorials
│   └── tutorials/              # 8-chapter Stratego walkthrough with full code
└── Cargo.toml                   # Workspace manifest
```

## Build & Development Commands

### Primary Commands (via Makefile)
```bash
# Development
make dev               # Launch the editor (default)
make editor            # Launch the editor
make engine            # Alias for make editor
make minimal           # Run minimal rendering binary
make preview PROJECT=<dir>  # Launch runtime preview
make new-game NAME=<name>   # Scaffold a new game crate

# Games
make game              # Run DoomExe (hamster narrator JRPG)
make doom              # Alias for make game
make stratego          # Run Stratego (10x10 board game tutorial)
make iso               # Run Iso Sandbox (isometric tile grid)

# Helix Data
make helix-import HELIX_DIST=<dir> PROJECT=<dir>  # Import Helix dist
make helix-import-toml HELIX3D=<dir>               # Load typed TOML registries
make helix-dashboard HELIX3D=<dir>                  # Data contract validation
make helix-editor PROJECT=<dir>                     # Helix editor wrapper
make helix-preview PROJECT=<dir>                    # Helix runtime preview

# Testing & Quality
make test              # Run all workspace tests (302+ passing)
make quality-check     # Full pipeline (fmt + clippy + test)
make guardrail         # Quick safety checks
make lint              # Run clippy
make fmt               # Check formatting
make format-fix        # Fix formatting

# Build
make build             # Debug build
make clean             # Remove build artifacts
```

### Direct Cargo Commands
```bash
# Build entire workspace
cargo build
cargo build --release

# Run specific crate
cargo run -p doomexe
cargo run -p dj_engine --bin dj_engine  # Editor binary
cargo run -p dj_engine --bin minimal

# Run tests
cargo test --workspace
cargo test -p dj_engine
cargo test -p doomexe

# Check code
cargo check --workspace
cargo clippy --workspace

# Format code
cargo fmt --all

# Generate documentation
cargo doc --workspace --open
```

### Development Profiles
- **Development**: Optimized dependencies (`opt-level = 3` for deps, `opt-level = 1` for local)
- **Release**: Full optimizations for distribution

## Architecture & Core Concepts

### Plugin-Based Design
The engine uses Bevy's plugin architecture extensively. The master `DJEnginePlugin` bundles all subsystems:

```rust
// engine/src/core/mod.rs
app.add_plugins(RenderingPlugin);
app.add_plugins(DJAnimationPlugin);
app.add_plugins(DJAssetPlugin);
app.add_plugins(DJAudioPlugin);
app.add_plugins(DJInputPlugin);
app.add_plugins(DJScenePlugin);
app.add_plugins(StoryGraphPlugin);
app.add_plugins(DJScriptingPlugin);
app.add_plugins(MidiPlugin);
app.add_plugins(DataPlugin);
```

Games can enable individual plugins or use the full bundle:
```rust
// games/dev/doomexe/src/main.rs
.add_plugins(DJEnginePlugin::default())  // Full engine
.add_plugins(scripting::GameScriptingPlugin)  // Game-specific extensions
```

### Key Architectural Patterns

1. **Resource-Driven Configuration**: `EngineConfig` and `DiagnosticConfig` as Bevy resources
2. **Component Dependencies**: Uses Bevy 0.18 `#[require(Component)]` for enforcing relationships
3. **Data-Driven Design**: JSON for story graphs, scenes, and databases loaded at runtime
4. **FFI Boundaries**: Clear separation between Rust systems and Lua scripting
5. **Event-Driven Communication**: Bevy events for cross-system communication

### Core Systems

**Animation**: Procedural animation using Bevy ECS
- Components: `Animator`, `ExpressionState`, palette swap materials
- Systems: Breathing/blinking timers, expression transitions

**Scripting**: Lua integration via mlua
- Sandboxed execution environment
- FFI bridge for Rust function exposure
- Hot-reload capable for development

**Story Graph**: Node-based narrative system
- JSON-serializable node types: Dialogue, Choice, Action, Branch
- Runtime execution via `StoryDirector`
- Integration with Lua for custom actions

**Data Management**: Unified loading system
- `Project`, `Scene`, `Database` types with Serde serialization
- Asset index for tracking dependencies
- Prefab system for entity spawning

## Code Style Guidelines

### Rust Conventions
- **Naming**: Snake_case for modules/functions, PascalCase for types/traits
- **Error Handling**: Use `DJResult<T>` alias and `thiserror` for custom errors
- **Bevy Systems**: Prefer query filters over manual iteration
- **Resources**: Use `#[derive(Resource, Debug, Clone, Reflect)]` for config types
- **Components**: Always derive `Component`, often with `#[require(...)]`

### Organization Principles
```rust
// In engine modules
pub mod components;    // Data components
pub mod systems;       // Bevy systems  
pub mod resources;     // Bevy resources
pub mod events;        // Bevy events
pub mod plugin;        // Plugin implementation

// Public API in lib.rs prelude
pub mod prelude {
    pub use crate::module::{PublicType, public_function};
}
```

### Lua Scripting Conventions
- Place game scripts in `games/dev/doomexe/assets/scripts/`
- Use `.lua` extension
- Expose functions via FFI in `engine/src/scripting/ffi.rs`
- Keep scripts focused: one script per character or gameplay system

### Asset Organization
```
games/dev/doomexe/assets/
├── music/          # MIDI files (.mid)
├── palettes/       # Palette definitions (.json)
├── scripts/        # Lua scripts (.lua)
├── shaders/        # WGSL shaders (.wgsl)
└── sprites/        # Sprite sheets (.png)
```

## Testing Strategy

### Test Structure
- **Unit Tests**: Inline in source files (`#[cfg(test)] mod tests`)
- **Integration Tests**: In `tests/` directories (currently minimal)
- **Runtime Tests**: Use the `minimal` binary for engine feature testing

### Running Tests
```bash
# All workspace tests
cargo test --workspace

# Specific crate tests  
cargo test -p dj_engine
cargo test -p doomexe

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name -- --exact
```

### What to Test
- **Engine Modules**: Data serialization/deserialization, component behavior
- **Game Logic**: Battle calculations, state transitions, UI interactions
- **Lua Integration**: FFI function correctness, error handling
- **Asset Loading**: JSON schema validation, file I/O

## Development Workflow

### Adding New Features

1. **Engine Features**: Place in appropriate `engine/src/module/` directory
   - Add components, systems, and resources
   - Create plugin in `core/` or as standalone plugin
   - Export from `lib.rs` prelude
   - Test with `minimal` binary

2. **Game Features**: Place in `games/dev/doomexe/src/`
   - Create new module or add to existing module
   - Add plugin in `main.rs`
   - Use Lua scripting for content-heavy logic
   - Test by running `make doom`

3. **Assets**: 
   - Place game assets in `games/dev/doomexe/assets/`
   - For engine assets, consider `engine/assets/` (currently minimal)
   - Run `make asset-gen` to process assets if needed

### Making Changes

**Small Changes** (bug fixes, tweaks):
```bash
# Make changes
cargo check --workspace
cargo test --workspace  # If tests exist
make doom  # Test the game
```

**Large Changes** (new systems, refactoring):
```bash
# Create feature branch (if using git)
git checkout -b feature-name

# Implement changes incrementally
cargo check  # Frequent validation
cargo fmt --all  # Keep formatting consistent

# Test thoroughly
make test  # Run all tests
make doom  # Manual game testing

# Commit with clear message
git commit -m "feat: add new animation system"
```

## Debugging & Diagnostics

### Built-in Diagnostics
Enable with `DJEnginePlugin::default()` (enabled by default):
- FPS counter
- System timing
- Entity counts
- Resource usage

### Verbose Logging
```bash
# Via helper script
RUST_LOG=debug make doom

# Direct environment variable
RUST_LOG=debug cargo run -p doomexe

# Filter specific modules
RUST_LOG=dj_engine::scripting=debug,bevy_render=warn cargo run -p doomexe
```

### Common Debug Targets
- `dj_engine::scripting`: Lua execution and FFI calls
- `dj_engine::data`: Asset loading and serialization
- `dj_engine::story_graph`: Narrative system execution
- `bevy_asset`: Asset pipeline issues

## Important Files & References

### Critical Files
- **`engine/src/lib.rs`**: Public API and prelude - start here
- **`engine/src/core/mod.rs`**: Master plugin bundle configuration
- **`engine/src/types.rs`**: Shared engine types and configuration
- **`games/dev/doomexe/src/main.rs`**: Game entry point showing plugin usage
- **`docs/Implementation_Summary.md`**: Complete roadmap and architecture guide
- **`docs/Game_Engine_Technical_Roadmap.md`**: 20-week implementation plan

### Documentation Index
- **`docs/INDEX_Navigation_Guide.md`**: Where to find specific information
- **`docs/IDE_Configuration_Guide.md`**: VS Code + Rust setup
- **`docs/EDITOR_Specification_Complete.md`**: Editor system details
- **`docs/AI_Coding_Assistant_Config.md`**: Guidelines for AI assistance

### Configuration Files
- **`Cargo.toml`**: Workspace dependencies and profiles
- **`engine/Cargo.toml`**: Engine crate dependencies
- **`games/dev/doomexe/Cargo.toml`**: Game-specific dependencies
- **`.gitignore`**: Standard Rust + game assets exclusions

## Security Considerations

### Lua Scripting
- Scripts are sandboxed but can execute arbitrary code
- Use FFI boundary carefully - only expose necessary functions
- Validate all script inputs at the Rust boundary

### Asset Loading
- JSON deserialization uses Serde with explicit types
- Avoid untyped deserialization from user-provided content
- Validate asset paths to prevent directory traversal

### Dependencies
- Workspace uses workspace-level dependency specification
- Check `cargo audit` before releases
- Pin versions in `Cargo.lock` for reproducible builds

## Deployment Notes

### Release Builds
```bash
cargo build --release -p doomexe
# Binary at: target/release/doomexe
```

### Asset Distribution
Include with binary:
- `games/dev/doomexe/assets/` directory
- Config files if externalized
- README with runtime requirements

### Platform Support
Currently targeting desktop platforms:
- Linux (primary development)
- Windows (planned)
- macOS (potential future)

## Getting Help

1. **Check existing documentation** in `docs/` directory
2. **Run examples**: `make minimal` for engine features
3. **Check run.log**: Recent build/run output  
4. **Review architecture**: `engine/src/core/mod.rs` for system overview
5. **Search codebase**: Use grep for specific component/function names

## Quick Reference: Most Common Commands

```bash
# Daily development
make doom       # Run DoomExe game
make stratego   # Run Stratego tutorial game
make iso        # Run Iso Sandbox
make test       # Run all tests (302+)
cargo check     # Validate code

# Debugging
RUST_LOG=debug make doom    # Verbose logging
RUST_LOG=debug cargo run -p doomexe  # Fine-grained control

# Release
cargo build --release -p doomexe
```
