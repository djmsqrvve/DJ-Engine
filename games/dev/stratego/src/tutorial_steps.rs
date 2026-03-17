//! In-game tutorial that guides the player through their first game.

use bevy::prelude::*;

use crate::board::StrategoBoard;
use crate::input::{FeedbackMessage, SetupQueue};
use crate::pieces::{PieceRank, Team};
use crate::rendering::{TutorialPanel, TutorialText};
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
    "Place your Flag first — it can't move, so hide it in the red zone!",
    "Now place your Marshal (10) — your strongest piece. Higher rank wins!",
    "Place the rest of your army. It auto-fills when you're done.",
    "Your turn! Click one of your red pieces to select it.",
    "Move into an enemy piece to attack! Higher rank wins.",
    "Capture the enemy Flag to win. Equal ranks destroy each other. Good luck!",
];

fn tutorial_message(step: usize) -> &'static str {
    TUTORIAL_MESSAGES.get(step).copied().unwrap_or("Play on!")
}

/// Advance the tutorial based on game events. Runs every frame in all phases.
pub fn tutorial_system(
    time: Res<Time>,
    phase: Res<State<GamePhase>>,
    board: Res<StrategoBoard>,
    queue: Res<SetupQueue>,
    feedback: Res<FeedbackMessage>,
    mut tutorial: ResMut<TutorialState>,
    mut text_q: Query<&mut Text2d, With<TutorialText>>,
    mut panel_q: Query<&mut Visibility, With<TutorialPanel>>,
) {
    if !tutorial.enabled {
        // Hide panel when tutorial is off.
        if let Ok(mut vis) = panel_q.single_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    // Don't override feedback messages.
    if feedback.is_active() {
        return;
    }

    tutorial.timer += time.delta_secs();

    let should_advance = match tutorial.step {
        0 => tutorial.timer > 3.0,

        1 => board.grid.iter().any(|(_, _, cell)| {
            cell.piece
                .as_ref()
                .map(|p| p.team == Team::Red && p.rank == PieceRank::Flag)
                .unwrap_or(false)
        }),

        2 => board.grid.iter().any(|(_, _, cell)| {
            cell.piece
                .as_ref()
                .map(|p| p.team == Team::Red && p.rank == PieceRank::Marshal)
                .unwrap_or(false)
        }),

        3 => queue.remaining.is_empty(),

        4 => tutorial.first_move_done,

        5 => {
            tutorial.first_combat_done
                || board
                    .grid
                    .iter()
                    .any(|(_, _, cell)| cell.piece.as_ref().map(|p| p.revealed).unwrap_or(false))
        }

        6 => *phase.get() == GamePhase::GameOver,

        _ => false,
    };

    if should_advance {
        tutorial.step += 1;
        tutorial.timer = 0.0;
    }

    // Show tutorial text in the separate panel below the board.
    if tutorial.step < TUTORIAL_MESSAGES.len() {
        if let Ok(mut text) = text_q.single_mut() {
            **text = tutorial_message(tutorial.step).to_string();
        }
        if let Ok(mut vis) = panel_q.single_mut() {
            *vis = Visibility::Visible;
        }
    } else {
        // Tutorial complete — hide the panel.
        if let Ok(mut vis) = panel_q.single_mut() {
            *vis = Visibility::Hidden;
        }
    }
}
