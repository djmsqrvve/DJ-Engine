# Chapter 4: Rendering and State

Draw the board and pieces using Bevy 2D sprites, and define the game phase state machine.

## What You'll Add

- `state.rs` -- Game phase enum, result resource (21 lines)
- `rendering.rs` -- Coordinate conversion, board spawning, piece sync, zone overlays (302 lines)

We introduce `state.rs` here because `rendering.rs` needs `GamePhase` for the setup zone overlay.

## Step 1: state.rs

> **File: `games/dev/stratego/src/state.rs`**

```rust
//! Game phase state machine.

use bevy::prelude::*;

use crate::pieces::Team;

/// The current phase of the game.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GamePhase {
    #[default]
    Setup,
    RedTurn,
    BlueTurn,
    GameOver,
}

/// Tracks the winner when GamePhase::GameOver is entered.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct GameResult {
    pub winner: Option<Team>,
}
```

`#[default]` on `Setup` means the game starts in the setup phase automatically when you call `init_state`.

## Step 2: rendering.rs

> **File: `games/dev/stratego/src/rendering.rs`**

Note: `sync_highlights_system` references `crate::input::PlayerSelection` which we haven't created yet. We'll add that function in Chapter 5. For now, create the file without it.

```rust
//! Board and piece rendering using Bevy 2D sprites.

use bevy::prelude::*;

use crate::board::{CellTerrain, StrategoBoard, BOARD_HEIGHT, BOARD_WIDTH};
use crate::pieces::Team;

pub const CELL_SIZE: f32 = 64.0;

use crate::state::GamePhase;

const COLOR_OPEN_LIGHT: Color = Color::srgb(0.87, 0.80, 0.68);
const COLOR_OPEN_DARK: Color = Color::srgb(0.82, 0.75, 0.62);
const COLOR_LAKE: Color = Color::srgb(0.3, 0.5, 0.8);
const COLOR_RED_PIECE: Color = Color::srgb(0.8, 0.2, 0.2);
const COLOR_BLUE_PIECE: Color = Color::srgb(0.2, 0.2, 0.8);
const COLOR_SELECTION: Color = Color::srgba(1.0, 1.0, 0.0, 0.4);
const COLOR_VALID_MOVE: Color = Color::srgba(0.2, 0.9, 0.2, 0.3);
const COLOR_RED_ZONE: Color = Color::srgba(0.9, 0.3, 0.3, 0.15);
const COLOR_BLUE_ZONE: Color = Color::srgba(0.3, 0.3, 0.9, 0.15);

/// Marker for a cell background sprite.
#[derive(Component)]
pub struct CellSprite {
    pub x: usize,
    pub y: usize,
}

/// Marker for a piece sprite entity.
#[derive(Component)]
pub struct PieceSprite;

/// Marker for the selection highlight overlay.
#[derive(Component)]
pub struct SelectionHighlight;

/// Marker for valid-move highlight overlays.
#[derive(Component)]
pub struct MoveHighlight;

/// Marker for status text at the top of the screen.
#[derive(Component)]
pub struct StatusText;

/// Marker for the tutorial overlay text below the board.
#[derive(Component)]
pub struct TutorialText;

/// Marker for the tutorial overlay background panel.
#[derive(Component)]
pub struct TutorialPanel;

/// Convert grid coordinates to world position (center of cell).
pub fn cell_to_world(x: usize, y: usize) -> Vec3 {
    let half_board = (BOARD_WIDTH as f32 * CELL_SIZE) / 2.0;
    let wx = (x as f32 + 0.5) * CELL_SIZE - half_board;
    let wy = (y as f32 + 0.5) * CELL_SIZE - half_board;
    Vec3::new(wx, wy, 0.0)
}

/// Convert world position to grid coordinates.
pub fn world_to_cell(pos: Vec2) -> Option<(usize, usize)> {
    let half_board = (BOARD_WIDTH as f32 * CELL_SIZE) / 2.0;
    let fx = (pos.x + half_board) / CELL_SIZE;
    let fy = (pos.y + half_board) / CELL_SIZE;
    if fx < 0.0 || fy < 0.0 {
        return None;
    }
    let x = fx as usize;
    let y = fy as usize;
    if x < BOARD_WIDTH && y < BOARD_HEIGHT {
        Some((x, y))
    } else {
        None
    }
}

/// Marker for setup zone overlay sprites.
#[derive(Component)]
pub struct SetupZoneOverlay;

/// Spawn the 10x10 cell background sprites with checkerboard pattern.
pub fn spawn_board_system(mut commands: Commands, board: Res<StrategoBoard>) {
    for (x, y, cell) in board.grid.iter() {
        let checkerboard = (x + y) % 2 == 0;
        let color = match cell.terrain {
            CellTerrain::Open => {
                if checkerboard {
                    COLOR_OPEN_LIGHT
                } else {
                    COLOR_OPEN_DARK
                }
            }
            CellTerrain::Lake => COLOR_LAKE,
        };

        commands.spawn((
            CellSprite { x, y },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(CELL_SIZE - 3.0)),
                ..default()
            },
            Transform::from_translation(cell_to_world(x, y)),
        ));
    }

    // Status text at top of screen.
    commands.spawn((
        StatusText,
        Text2d::new("Stratego — Place your pieces"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, (BOARD_HEIGHT as f32 * CELL_SIZE) / 2.0 + 30.0, 10.0),
    ));

    // Tutorial panel below the board.
    let tutorial_y = -(BOARD_HEIGHT as f32 * CELL_SIZE) / 2.0 - 40.0;
    commands
        .spawn((
            TutorialPanel,
            Sprite {
                color: Color::srgba(0.1, 0.1, 0.15, 0.85),
                custom_size: Some(Vec2::new(700.0, 44.0)),
                ..default()
            },
            Transform::from_xyz(0.0, tutorial_y, 9.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                TutorialText,
                Text2d::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.8)),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ));
        });
}

/// Sync piece sprites to match the current board state.
/// Only rebuilds when the board has actually changed (not every frame).
pub fn sync_pieces_system(
    mut commands: Commands,
    board: Res<StrategoBoard>,
    existing: Query<Entity, With<PieceSprite>>,
) {
    if !board.is_changed() {
        return;
    }

    // Despawn all existing piece sprites and rebuild.
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    for (x, y, cell) in board.grid.iter() {
        let Some(piece) = &cell.piece else {
            continue;
        };

        let bg_color = match piece.team {
            Team::Red => COLOR_RED_PIECE,
            Team::Blue => COLOR_BLUE_PIECE,
        };

        let label = if piece.team == Team::Red || piece.revealed {
            piece.rank.label()
        } else {
            "?"
        };

        let pos = cell_to_world(x, y);

        // Background square for the piece.
        commands
            .spawn((
                PieceSprite,
                Sprite {
                    color: bg_color,
                    custom_size: Some(Vec2::splat(CELL_SIZE - 8.0)),
                    ..default()
                },
                Transform::from_translation(pos + Vec3::Z * 1.0),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text2d::new(label),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 1.0),
                ));
            });
    }
}

/// Show/hide setup zone overlays based on game phase.
pub fn sync_setup_zone_system(
    mut commands: Commands,
    phase: Res<State<GamePhase>>,
    existing: Query<Entity, With<SetupZoneOverlay>>,
) {
    let in_setup = *phase.get() == GamePhase::Setup;

    if !in_setup {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Only spawn once.
    if !existing.is_empty() {
        return;
    }

    // Red zone (rows 0-3).
    for x in 0..BOARD_WIDTH {
        for y in 0..4 {
            let pos = cell_to_world(x, y);
            commands.spawn((
                SetupZoneOverlay,
                Sprite {
                    color: COLOR_RED_ZONE,
                    custom_size: Some(Vec2::splat(CELL_SIZE - 3.0)),
                    ..default()
                },
                Transform::from_translation(pos + Vec3::Z * 0.5),
            ));
        }
    }

    // Blue zone (rows 6-9).
    for x in 0..BOARD_WIDTH {
        for y in 6..BOARD_HEIGHT {
            let pos = cell_to_world(x, y);
            commands.spawn((
                SetupZoneOverlay,
                Sprite {
                    color: COLOR_BLUE_ZONE,
                    custom_size: Some(Vec2::splat(CELL_SIZE - 3.0)),
                    ..default()
                },
                Transform::from_translation(pos + Vec3::Z * 0.5),
            ));
        }
    }
}

/// Show selection and valid-move highlights.
/// NOTE: This function references crate::input::PlayerSelection.
/// We add it in Chapter 5 when input.rs exists.
pub fn sync_highlights_system(
    mut commands: Commands,
    existing_sel: Query<Entity, With<SelectionHighlight>>,
    existing_moves: Query<Entity, With<MoveHighlight>>,
    selection: Option<Res<crate::input::PlayerSelection>>,
) {
    // Clear old highlights.
    for entity in &existing_sel {
        commands.entity(entity).despawn();
    }
    for entity in &existing_moves {
        commands.entity(entity).despawn();
    }

    let Some(selection) = selection else {
        return;
    };

    if let Some((sx, sy)) = selection.selected {
        let pos = cell_to_world(sx, sy);
        commands.spawn((
            SelectionHighlight,
            Sprite {
                color: COLOR_SELECTION,
                custom_size: Some(Vec2::splat(CELL_SIZE - 2.0)),
                ..default()
            },
            Transform::from_translation(pos + Vec3::Z * 2.0),
        ));
    }

    for &(mx, my) in &selection.valid_moves {
        let pos = cell_to_world(mx, my);
        commands.spawn((
            MoveHighlight,
            Sprite {
                color: COLOR_VALID_MOVE,
                custom_size: Some(Vec2::splat(CELL_SIZE - 2.0)),
                ..default()
            },
            Transform::from_translation(pos + Vec3::Z * 2.0),
        ));
    }
}
```

Key rendering patterns:

- **Coordinate conversion**: Board is centered at origin. `cell_to_world(0,0)` maps to the bottom-left cell. Each cell is 64x64 pixels with a 3px gap.
- **Z-layering**: Cells Z=0, zone overlays Z=0.5, pieces Z=1, highlights Z=2, text Z=10.
- **Change detection**: `sync_pieces_system` checks `board.is_changed()` so it only rebuilds sprites when the board was actually modified. Without this, sprites would be destroyed and recreated every frame.
- **Parent-child text**: The rank label is a child entity of the piece sprite, so it moves with it.
- **Zone overlays**: `sync_setup_zone_system` only runs during Setup, spawns once (checks `!existing.is_empty()`), and despawns when leaving Setup.
- **`sync_highlights_system`** uses `Option<Res<...>>` for `PlayerSelection` -- this allows the function to exist even before the resource is registered (it just returns early if absent). However, because it references `crate::input::PlayerSelection`, the `input` module must exist for this to compile. We handle that in Chapter 5.

## Step 3: Update main.rs

Since `sync_highlights_system` references the `input` module (which doesn't exist yet), we can't register it in main.rs yet. We'll add it in Chapter 5. For now, main.rs registers everything else:

> **File: `games/dev/stratego/src/main.rs`**

```rust
//! Stratego-lite — a turn-based board game tutorial for DJ Engine.

use bevy::prelude::*;
use bevy::window::WindowResolution;

mod board;
mod pieces;
mod rendering;
mod rules;
mod state;

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
        .init_state::<state::GamePhase>()
        .init_resource::<board::StrategoBoard>()
        .init_resource::<state::GameResult>()
        // Startup
        .add_systems(Startup, (setup_camera, rendering::spawn_board_system))
        // Global rendering (runs in all states)
        .add_systems(Update, (
            rendering::sync_pieces_system,
            rendering::sync_setup_zone_system,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

Note that `sync_pieces_system` has no `.after()` constraints yet -- we add those in Chapter 5 when the input systems exist, and in Chapter 7 when the AI system exists.

**Important:** This main.rs won't compile yet because `rendering.rs` references `crate::input::PlayerSelection` in `sync_highlights_system`. To fix this temporarily, either:

1. Comment out `sync_highlights_system` in rendering.rs until Chapter 5, or
2. Skip ahead to create the `input` module stub (next chapter)

We recommend option 2 -- jump to Chapter 5 next and create the files together. The chapter ordering is for reading/understanding; in practice you'll create rendering.rs and input.rs in the same step.

## Checkpoint

After completing Chapter 5 (which creates input.rs), run:

```sh
make stratego
```

You should see a 10x10 checkerboard with blue lake cells in the center and red/blue zone overlays. Status text reads "Stratego -- Place your pieces" at the top.

## Next

[Chapter 5: Input and Setup](05-input-and-setup.md) -- Handle clicks for piece placement and movement.
