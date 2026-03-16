//! In-game tutorial that guides the player through their first game.

use bevy::prelude::*;

use crate::board::StrategoBoard;
use crate::input::{PlayerSelection, SetupQueue};
use crate::pieces::{PieceRank, Team};
use crate::rendering::StatusText;
use crate::rules::CombatResult;
use crate::state::GamePhase;

/// Current tutorial step.
#[derive(Resource, Debug)]
pub struct TutorialState {
    pub step: usize,
    pub timer: f32,
    pub first_move_done: bool,
    pub first_combat_done: bool,
    pub enabled: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            step: 0,
            timer: 0.0,
            first_move_done: false,
            first_combat_done: false,
            enabled: true,
        }
    }
}

const TUTORIAL_MESSAGES: &[&str] = &[
    "Welcome to Stratego! This is a 10x10 board with two lakes in the center.",
    "Place your Flag first — it can't move, so hide it in the back row!",
    "Now place your Marshal (10) — your strongest piece. Higher rank wins in combat.",
    "Place the rest of your army. The board will fill automatically when you're done.",
    "Your turn! Click one of your pieces to select it.",
    "Move into an enemy piece to attack! Higher rank captures lower rank.",
    "Capture the enemy Flag to win. Equal ranks destroy each other. Good luck!",
];

fn tutorial_message(step: usize) -> &'static str {
    TUTORIAL_MESSAGES
        .get(step)
        .copied()
        .unwrap_or("Play on!")
}

/// Advance the tutorial based on game events.
pub fn tutorial_system(
    time: Res<Time>,
    phase: Res<State<GamePhase>>,
    board: Res<StrategoBoard>,
    queue: Res<SetupQueue>,
    selection: Res<PlayerSelection>,
    mut tutorial: ResMut<TutorialState>,
    mut text_q: Query<&mut Text2d, With<StatusText>>,
) {
    if !tutorial.enabled {
        return;
    }

    tutorial.timer += time.delta_secs();

    let should_advance = match tutorial.step {
        // Step 0: Welcome — auto-advance after 3 seconds.
        0 => tutorial.timer > 3.0,

        // Step 1: Place Flag — advance when flag is on the board.
        1 => {
            board.grid.iter().any(|(_, _, cell)| {
                cell.piece
                    .as_ref()
                    .map(|p| p.team == Team::Red && p.rank == PieceRank::Flag)
                    .unwrap_or(false)
            })
        }

        // Step 2: Place Marshal — advance when marshal is on the board.
        2 => {
            board.grid.iter().any(|(_, _, cell)| {
                cell.piece
                    .as_ref()
                    .map(|p| p.team == Team::Red && p.rank == PieceRank::Marshal)
                    .unwrap_or(false)
            })
        }

        // Step 3: Fill army — advance when setup is complete.
        3 => queue.remaining.is_empty(),

        // Step 4: First move — advance when player has made a move (entered BlueTurn).
        4 => *phase.get() == GamePhase::BlueTurn || tutorial.first_move_done,

        // Step 5: First combat — advance when a piece has been revealed.
        5 => {
            tutorial.first_combat_done
                || board.grid.iter().any(|(_, _, cell)| {
                    cell.piece
                        .as_ref()
                        .map(|p| p.revealed)
                        .unwrap_or(false)
                })
        }

        // Step 6: Final message — stays until game ends.
        6 => *phase.get() == GamePhase::GameOver,

        _ => false,
    };

    if should_advance {
        tutorial.step += 1;
        tutorial.timer = 0.0;
        if tutorial.step == 4 {
            tutorial.first_move_done = false;
        }
    }

    // Override status text with tutorial message when in tutorial steps.
    if tutorial.step < TUTORIAL_MESSAGES.len() {
        if let Ok(mut text) = text_q.single_mut() {
            **text = tutorial_message(tutorial.step).to_string();
        }
    }
}

/// Mark that the player made their first move (called from input systems).
pub fn mark_first_move(mut tutorial: ResMut<TutorialState>) {
    if tutorial.step == 4 {
        tutorial.first_move_done = true;
    }
}

/// Mark that the player's first combat happened.
pub fn mark_first_combat(mut tutorial: ResMut<TutorialState>) {
    if tutorial.step == 5 {
        tutorial.first_combat_done = true;
    }
}
