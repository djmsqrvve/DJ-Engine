//! Simple random AI for the Blue team.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::board::StrategoBoard;
use crate::pieces::Team;
use crate::rules::{self, CombatResult};
use crate::state::{GamePhase, GameResult};

/// AI takes a random valid move for Blue, then switches to RedTurn.
pub fn ai_turn_system(
    mut board: ResMut<StrategoBoard>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut game_result: ResMut<GameResult>,
) {
    let mut candidates: Vec<(usize, usize, Vec<(usize, usize)>)> = Vec::new();

    for (x, y, cell) in board.grid.iter() {
        if let Some(piece) = &cell.piece {
            if piece.team == Team::Blue && piece.rank.can_move() {
                let moves = rules::valid_moves(&board, Team::Blue, x, y);
                if !moves.is_empty() {
                    candidates.push((x, y, moves));
                }
            }
        }
    }

    if candidates.is_empty() {
        // No valid moves — Red wins by default.
        game_result.winner = Some(Team::Red);
        next_state.set(GamePhase::GameOver);
        return;
    }

    let mut rng = rand::thread_rng();
    let (fx, fy, moves) = candidates.choose(&mut rng).unwrap();
    let &(tx, ty) = moves.choose(&mut rng).unwrap();

    let combat = rules::execute_move(&mut board, *fx, *fy, tx, ty);

    if let Some(CombatResult::FlagCaptured(loser)) = combat {
        game_result.winner = Some(loser.opponent());
        next_state.set(GamePhase::GameOver);
    } else {
        next_state.set(GamePhase::RedTurn);
    }
}
