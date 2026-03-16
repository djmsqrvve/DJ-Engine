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
