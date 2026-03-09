use crate::state::GameState;
use bevy::prelude::*;

mod systems;
mod ui;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BattleResultEvent>()
            .add_systems(OnEnter(GameState::Battle), ui::setup_battle_ui)
            .add_systems(
                Update,
                (ui::battle_ui_interaction, systems::handle_battle_result)
                    .run_if(in_state(GameState::Battle)),
            )
            .add_systems(OnExit(GameState::Battle), ui::cleanup_battle_ui);
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[allow(dead_code)]
pub enum BattleState {
    #[default]
    Idle, // Waiting for battle to start
    InBattle,
    Victory,
    Defeat,
}

#[derive(Message, Debug, Clone, Copy)]
pub enum BattleResultEvent {
    Win,
    Lose,
}
