//! Board and piece rendering using Bevy 2D sprites.

use bevy::prelude::*;

use crate::board::{CellTerrain, StrategoBoard, BOARD_HEIGHT, BOARD_WIDTH};
use crate::pieces::Team;

pub const CELL_SIZE: f32 = 64.0;

const COLOR_OPEN: Color = Color::srgb(0.85, 0.78, 0.65);
const COLOR_LAKE: Color = Color::srgb(0.3, 0.5, 0.8);
const COLOR_RED_PIECE: Color = Color::srgb(0.8, 0.2, 0.2);
const COLOR_BLUE_PIECE: Color = Color::srgb(0.2, 0.2, 0.8);
const COLOR_SELECTION: Color = Color::srgba(1.0, 1.0, 0.0, 0.4);
const COLOR_VALID_MOVE: Color = Color::srgba(0.2, 0.9, 0.2, 0.3);

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

/// Spawn the 10x10 cell background sprites.
pub fn spawn_board_system(mut commands: Commands, board: Res<StrategoBoard>) {
    for (x, y, cell) in board.grid.iter() {
        let color = match cell.terrain {
            CellTerrain::Open => COLOR_OPEN,
            CellTerrain::Lake => COLOR_LAKE,
        };

        commands.spawn((
            CellSprite { x, y },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(CELL_SIZE - 2.0)),
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
}

/// Sync piece sprites to match the current board state.
pub fn sync_pieces_system(
    mut commands: Commands,
    board: Res<StrategoBoard>,
    existing: Query<Entity, With<PieceSprite>>,
) {
    // Despawn all existing piece sprites and rebuild.
    // Simple approach — fine for 10x10 board.
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
