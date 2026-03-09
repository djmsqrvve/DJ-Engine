use super::BattleResultEvent;
use crate::hamster::components::{CharacterRoot, Expression};
use crate::state::GameState;
use crate::story::StoryState;
use bevy::prelude::*;

pub fn handle_battle_result(
    mut events: MessageReader<BattleResultEvent>,
    mut hamster_query: Query<&mut CharacterRoot>,
    mut story: ResMut<StoryState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        for mut hamster in &mut hamster_query {
            match event {
                BattleResultEvent::Win => {
                    info!("Battle WON! Hamster is pleased.");
                    hamster.expression = Expression::Happy;
                    // Winning reduces corruption slightly (healing)
                    hamster.corruption = (hamster.corruption - 10.0).max(0.0);

                    story.add_flag("DefeatedGlitch");
                }
                BattleResultEvent::Lose => {
                    info!("Battle LOST! Hamster is angry.");
                    hamster.expression = Expression::Angry;
                    // Losing increases corruption (damage/stress)
                    hamster.corruption = (hamster.corruption + 15.0).min(100.0);
                }
            }
            // Return to Overworld after battle
            next_state.set(GameState::Overworld);
        }
    }
}
