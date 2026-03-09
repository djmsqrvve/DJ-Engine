use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    TitleScreen,
    Overworld,

    // Narrative states
    NarratorDialogue,
    Battle,
}
