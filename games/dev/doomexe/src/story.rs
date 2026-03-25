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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_state_flags() {
        let mut state = StoryState::default();
        assert!(!state.has_flag("MetHamster"));

        state.add_flag("MetHamster");
        assert!(state.has_flag("MetHamster"));

        // Adding duplicate is a no-op
        state.add_flag("MetHamster");
        assert_eq!(state.flags.len(), 1);
    }

    #[test]
    fn test_battle_pending_default_false() {
        let bp = BattlePending::default();
        assert!(!bp.0);
    }

    #[test]
    fn test_handle_story_events_sets_battle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.init_resource::<BattlePending>();
        app.add_message::<StoryEvent>();
        app.add_systems(Update, handle_story_events);

        // Send StartBattle event
        app.world_mut()
            .resource_mut::<Messages<StoryEvent>>()
            .write(StoryEvent {
                id: "StartBattle".into(),
                payload: String::new(),
            });

        app.update();

        // BattlePending should be true
        let bp = app.world().resource::<BattlePending>();
        assert!(bp.0, "BattlePending should be set after StartBattle event");

        // NextState should be Battle
        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            matches!(*next, NextState::Pending(GameState::Battle)),
            "NextState should be Pending(Battle)"
        );
    }

    #[test]
    fn test_non_battle_events_ignored() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.init_resource::<BattlePending>();
        app.add_message::<StoryEvent>();
        app.add_systems(Update, handle_story_events);

        // Send a non-battle event
        app.world_mut()
            .resource_mut::<Messages<StoryEvent>>()
            .write(StoryEvent {
                id: "SomethingElse".into(),
                payload: String::new(),
            });

        app.update();

        let bp = app.world().resource::<BattlePending>();
        assert!(
            !bp.0,
            "BattlePending should NOT be set for non-battle events"
        );
    }
}
