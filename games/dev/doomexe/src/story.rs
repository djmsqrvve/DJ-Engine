use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::story_graph::StoryEvent;

/// Set when a battle transition is triggered by a story event.
/// Checked by the dialogue system to avoid overriding with Overworld.
#[derive(Resource, Default)]
pub struct BattlePending(pub bool);

#[derive(Resource, Default, Debug)]
pub struct StoryState {
    pub _chapter: usize,
    pub flags: Vec<String>,
}

impl StoryState {
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(&flag.to_string())
    }

    pub fn add_flag(&mut self, flag: &str) {
        if !self.has_flag(flag) {
            self.flags.push(flag.to_string());
            info!("Story Flag Added: {}", flag);
        }
    }
}

pub struct StoryPlugin;

impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StoryState>()
            .init_resource::<BattlePending>()
            .add_systems(Update, handle_story_events);
    }
}

fn handle_story_events(
    mut events: MessageReader<StoryEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut battle_pending: ResMut<BattlePending>,
) {
    for event in events.read() {
        if event.id == "StartBattle" {
            info!("Story Event: Start Battle — setting BattlePending flag");
            battle_pending.0 = true;
            next_state.set(GameState::Battle);
        }
    }
}
