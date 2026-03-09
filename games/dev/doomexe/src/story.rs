use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::story_graph::StoryEvent;

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
            .add_systems(Update, handle_story_events);
    }
}

fn handle_story_events(
    mut events: MessageReader<StoryEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        if event.id == "StartBattle" {
            info!("Story Event: Start Battle");
            next_state.set(GameState::Battle);
        }
    }
}
