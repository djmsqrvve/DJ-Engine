//! Simple random AI for the Blue team with a brief delay.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::board::StrategoBoard;
use crate::input::FeedbackMessage;
use crate::pieces::Team;
use crate::rules::{self, CombatResult};
use crate::state::{GamePhase, GameResult};

/// Timer for AI thinking delay.
#[derive(Resource, Debug)]
pub struct AiTimer {
    pub timer: Timer,
    pub ready: bool,
}

impl Default for AiTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.8, TimerMode::Once),
            ready: false,
        }
    }
}

/// AI waits briefly, then takes a random valid move for Blue.
pub fn ai_turn_system(
    time: Res<Time>,
    mut ai_timer: ResMut<AiTimer>,
    mut board: ResMut<StrategoBoard>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut game_result: ResMut<GameResult>,
    mut feedback: ResMut<FeedbackMessage>,
) {
    ai_timer.timer.tick(time.delta());

    if !ai_timer.timer.is_finished() {
        return;
    }

    if ai_timer.ready {
        return; // Already moved this turn.
    }
    ai_timer.ready = true;

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
        game_result.winner = Some(Team::Red);
        next_state.set(GamePhase::GameOver);
        ai_timer.timer.reset();
        ai_timer.ready = false;
        return;
    }

    let mut rng = rand::thread_rng();
    let (fx, fy, moves) = candidates.choose(&mut rng).unwrap();
    let &(tx, ty) = moves.choose(&mut rng).unwrap();

    // Snapshot ranks for feedback.
    let attacker_rank = board
        .get(*fx, *fy)
        .and_then(|c| c.piece.as_ref())
        .map(|p| p.rank);
    let defender_rank = board
        .get(tx, ty)
        .and_then(|c| c.piece.as_ref())
        .map(|p| p.rank);

    let combat = rules::execute_move(&mut board, *fx, *fy, tx, ty);

    // Combat feedback.
    if let (Some(combat), Some(atk), Some(def)) = (combat, attacker_rank, defender_rank) {
        match combat {
            CombatResult::AttackerWins => {
                feedback.set(format!("AI's {} defeated your {}!", atk.name(), def.name()));
            }
            CombatResult::DefenderWins => {
                feedback.set(format!("Your {} held against AI's {}!", def.name(), atk.name()));
            }
            CombatResult::BothDie => {
                feedback.set(format!("AI's {} and your {} destroyed each other!", atk.name(), def.name()));
            }
            CombatResult::FlagCaptured(loser) => {
                game_result.winner = Some(loser.opponent());
                next_state.set(GamePhase::GameOver);
                ai_timer.timer.reset();
                ai_timer.ready = false;
                return;
            }
        }
    }

    if let Some(CombatResult::FlagCaptured(loser)) = combat {
        game_result.winner = Some(loser.opponent());
        next_state.set(GamePhase::GameOver);
    } else {
        next_state.set(GamePhase::RedTurn);
    }

    ai_timer.timer.reset();
    ai_timer.ready = false;
}
