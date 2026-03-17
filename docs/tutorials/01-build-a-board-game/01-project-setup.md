# Chapter 1: Project Setup

Create a new game crate in the workspace and get a Bevy window on screen.

## What You'll Add

- A new crate at `games/dev/stratego/`
- Workspace member entry in the root `Cargo.toml`
- A Makefile target for quick launching
- A minimal `main.rs` that opens an 800x800 window

## Step 1: Scaffold the Crate

```sh
mkdir -p games/dev/stratego/src
```

## Step 2: Cargo.toml

> **File: `games/dev/stratego/Cargo.toml`**

```toml
[package]
name = "stratego"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true, features = ["dynamic_linking"] }
serde = { workspace = true }
serde_json = "1.0"
rand = { workspace = true }
dj_engine = { path = "../../../engine" }
```

- `dj_engine` is a path dependency so you can use engine utilities like `Grid<T>`
- `bevy`, `serde`, and `rand` come from workspace dependencies (version managed in root `Cargo.toml`)
- `features = ["dynamic_linking"]` speeds up compile times during development

## Step 3: Register in the Workspace

Add the crate to the root `Cargo.toml` members list:

```toml
[workspace]
members = [
    "engine",
    "games/dev/doomexe",
    "games/dev/stratego",   # <-- add this
    "plugins/helix_data",
    "tools/asset_generator"
]
```

## Step 4: Makefile Target

Add a target so you can run `make stratego`:

```makefile
stratego:
	cargo run -p stratego
```

## Step 5: main.rs

> **File: `games/dev/stratego/src/main.rs`**

```rust
//! Stratego-lite — a turn-based board game tutorial for DJ Engine.

use bevy::prelude::*;
use bevy::window::WindowResolution;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DJ Engine - Stratego".into(),
                        resolution: WindowResolution::new(800, 800)
                            .with_scale_factor_override(1.0),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.2)))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

Notes:

- `WindowResolution::new` takes `u32` values in Bevy 0.18
- `ImagePlugin::default_nearest()` gives crisp pixel rendering
- `Camera2d` is all you need for a 2D game

## Checkpoint

```sh
make stratego
```

You should see a dark blue-gray 800x800 window titled "DJ Engine - Stratego". Nothing renders yet.

```sh
make check
```

Should compile with no errors.

## Next

[Chapter 2: Board, Pieces, and Grid](02-board-and-grid.md) -- Define pieces and model the 10x10 board.
