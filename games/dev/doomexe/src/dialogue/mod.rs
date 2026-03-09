use crate::state::GameState;
use bevy::prelude::*;

mod ui;

#[derive(Resource, Default)]
pub struct DialogueUiState {
    pub visible: bool,
    pub speaker: String,
    pub text: String,
    pub _portrait: Option<String>,
    pub choices: Vec<String>,
    pub prompt: String,
    pub selected_index: usize,
    pub is_choice_mode: bool,
}

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogueUiState>()
            .add_systems(OnEnter(GameState::NarratorDialogue), ui::setup_dialogue_ui)
            .add_systems(
                Update,
                (
                    ui::dialogue_input,
                    ui::update_dialogue_ui,
                    ui::typewriter_system,
                )
                    .run_if(in_state(GameState::NarratorDialogue)),
            )
            .add_systems(
                OnExit(GameState::NarratorDialogue),
                ui::teardown_dialogue_ui,
            );
    }
}
