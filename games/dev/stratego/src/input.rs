//! Player input handling: setup placement, piece selection, movement.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::board::StrategoBoard;
use crate::pieces::{army_composition, PieceRank, PlacedPiece, Team};
use crate::rendering::{cell_to_world, world_to_cell, StatusText};
use crate::rules::{self, CombatResult};
use crate::state::{GamePhase, GameResult};

/// Tracks the player's current selection during play.
#[derive(Resource, Default, Debug)]
pub struct PlayerSelection {
    pub selected: Option<(usize, usize)>,
    pub valid_moves: Vec<(usize, usize)>,
}

/// Queue of pieces the player still needs to place during setup.
#[derive(Resource, Default, Debug)]
pub struct SetupQueue {
    pub remaining: Vec<PieceRank>,
}

/// Get the cursor position in world coordinates.
fn cursor_world_pos(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = camera_q.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

/// Initialize setup queue with all army pieces.
pub fn init_setup_system(mut queue: ResMut<SetupQueue>) {
    queue.remaining.clear();
    for (rank, count) in army_composition() {
        for _ in 0..count {
            queue.remaining.push(rank);
        }
    }
    // Place flag and marshal first for tutorial flow.
    queue.remaining.sort_by(|a, b| {
        let priority = |r: &PieceRank| match r {
            PieceRank::Flag => 0,
            PieceRank::Marshal => 1,
            _ => 2 + r.strength() as i32,
        };
        priority(a).cmp(&priority(b))
    });
}

/// Handle clicks during setup phase — place pieces one by one.
pub fn setup_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut board: ResMut<StrategoBoard>,
    mut queue: ResMut<SetupQueue>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_pos) = cursor_world_pos(&windows, &camera_q) else {
        return;
    };
    let Some((x, y)) = world_to_cell(world_pos) else {
        return;
    };

    let Some(&rank) = queue.remaining.first() else {
        return;
    };

    let piece = PlacedPiece {
        rank,
        team: Team::Red,
        revealed: false,
    };

    if board.place_piece(x, y, piece) {
        queue.remaining.remove(0);

        if queue.remaining.is_empty() {
            // Auto-fill Blue team and start the game.
            board.auto_fill_army(Team::Blue);
            next_state.set(GamePhase::RedTurn);
        }
    }
}

/// Update status text during setup.
pub fn setup_status_system(
    queue: Res<SetupQueue>,
    mut text_q: Query<&mut Text2d, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };

    if let Some(rank) = queue.remaining.first() {
        **text = format!(
            "Place your {} ({} pieces remaining)",
            rank.name(),
            queue.remaining.len()
        );
    } else {
        **text = "Setup complete!".to_string();
    }
}

/// Handle clicks during Red's turn — select piece, then move.
pub fn player_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut board: ResMut<StrategoBoard>,
    mut selection: ResMut<PlayerSelection>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut game_result: ResMut<GameResult>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_pos) = cursor_world_pos(&windows, &camera_q) else {
        return;
    };
    let Some((x, y)) = world_to_cell(world_pos) else {
        return;
    };

    // If we have a selection and clicked a valid move target — execute the move.
    if let Some((sx, sy)) = selection.selected {
        if selection.valid_moves.contains(&(x, y)) {
            let combat = rules::execute_move(&mut board, sx, sy, x, y);

            selection.selected = None;
            selection.valid_moves.clear();

            if let Some(CombatResult::FlagCaptured(loser)) = combat {
                game_result.winner = Some(loser.opponent());
                next_state.set(GamePhase::GameOver);
            } else {
                next_state.set(GamePhase::BlueTurn);
            }
            return;
        }
    }

    // Otherwise, try to select a Red piece.
    if let Some(cell) = board.get(x, y) {
        if let Some(piece) = &cell.piece {
            if piece.team == Team::Red && piece.rank.can_move() {
                let moves = rules::valid_moves(&board, Team::Red, x, y);
                selection.selected = Some((x, y));
                selection.valid_moves = moves;
                return;
            }
        }
    }

    // Clicked empty/invalid — deselect.
    selection.selected = None;
    selection.valid_moves.clear();
}

/// Update status text during play.
pub fn play_status_system(
    phase: Res<State<GamePhase>>,
    selection: Res<PlayerSelection>,
    mut text_q: Query<&mut Text2d, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };

    match phase.get() {
        GamePhase::RedTurn => {
            if selection.selected.is_some() {
                **text = "Click a highlighted cell to move".to_string();
            } else {
                **text = "Your turn — click a piece to select it".to_string();
            }
        }
        GamePhase::BlueTurn => {
            **text = "Opponent is thinking...".to_string();
        }
        _ => {}
    }
}

/// Display game over screen.
pub fn game_over_system(
    game_result: Res<GameResult>,
    mut text_q: Query<&mut Text2d, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    let winner_name = match game_result.winner {
        Some(Team::Red) => "You win!",
        Some(Team::Blue) => "You lose!",
        None => "Draw!",
    };
    **text = format!("{winner_name} Press R to restart.");
}

/// Restart the game on R key.
pub fn restart_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut board: ResMut<StrategoBoard>,
    mut selection: ResMut<PlayerSelection>,
    mut game_result: ResMut<GameResult>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        *board = StrategoBoard::new();
        *selection = PlayerSelection::default();
        *game_result = GameResult::default();
        next_state.set(GamePhase::Setup);
    }
}
