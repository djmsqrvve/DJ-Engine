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
pub struct CellSprite;

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
            CellSprite,
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
