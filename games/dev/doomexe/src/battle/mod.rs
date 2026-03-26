use crate::state::GameState;
use bevy::prelude::*;

pub mod systems;
pub mod ui;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Battle),
            (systems::setup_battle_entities, ui::setup_battle_ui),
        )
        .add_systems(
            Update,
            (
                systems::player_attack,
                systems::enemy_ai_attack,
                systems::handle_battle_damage,
                ui::update_battle_hud,
            )
                .run_if(in_state(GameState::Battle)),
        )
        .add_systems(
            OnExit(GameState::Battle),
            (systems::cleanup_battle_entities, ui::cleanup_battle_ui),
        );
    }
}
