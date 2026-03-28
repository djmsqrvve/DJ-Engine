use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::prelude::{
    CharacterPlugin, Inventory, PlayerTitle, QuestJournal, ScreenFlashEvent, StoryFlags,
    VendorBuyRequest,
};
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
    mut flags: ResMut<StoryFlags>,
    mut quest_journal: ResMut<QuestJournal>,
    mut inventory: ResMut<Inventory>,
    mut player_title: ResMut<PlayerTitle>,
    mut flash_events: Option<MessageWriter<ScreenFlashEvent>>,
) {
    for event in events.read() {
        match event.id.as_str() {
            "StartBattle" => {
                info!("Story Event: Start Battle — setting BattlePending flag");
                battle_pending.0 = true;
                next_state.set(GameState::Battle);
            }
            "QuestAccept_cellar" => {
                info!("Story Event: Quest accepted — clear_the_cellar");
                quest_journal.accept("clear_the_cellar");
                quest_journal.add_objective("clear_the_cellar", "kill_rats", 3);
                flags.0.insert("QuestAccepted_cellar".to_string(), true);
            }
            "QuestTurnIn_cellar" => {
                info!("Story Event: Quest turned in — clear_the_cellar");
                quest_journal.turn_in("clear_the_cellar");
                inventory.add_currency("gold", 50);
                player_title.earn("rat_slayer");
                player_title.equip("rat_slayer");
                flags.0.insert("QuestTurnedIn_cellar".to_string(), true);
                if let Some(ref mut flash) = flash_events {
                    flash.write(ScreenFlashEvent::gold());
                }
            }
            "VendorBuy_health_potion" => {
                let price = 10_u64;
                if inventory.currency_balance("gold") >= price {
                    inventory.spend_currency("gold", price);
                    inventory.add_item("health_potion", 1, 10);
                    info!("Vendor: bought health_potion for {} gold", price);
                } else {
                    info!(
                        "Vendor: not enough gold ({} < {})",
                        inventory.currency_balance("gold"),
                        price
                    );
                }
            }
            "EnterCellar" => {
                info!("Story Event: Entering cellar");
                battle_pending.0 = true; // Prevent dialogue from returning to overworld
                next_state.set(GameState::Cellar);
            }
            _ => {}
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

    fn setup_story_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.init_resource::<BattlePending>();
        app.init_resource::<StoryFlags>();
        app.insert_resource(Inventory::new(20));
        app.init_resource::<QuestJournal>();
        app.init_resource::<PlayerTitle>();
        app.add_message::<StoryEvent>();
        app.add_systems(Update, handle_story_events);
        app
    }

    #[test]
    fn test_handle_story_events_sets_battle() {
        let mut app = setup_story_test_app();

        app.world_mut()
            .resource_mut::<Messages<StoryEvent>>()
            .write(StoryEvent {
                id: "StartBattle".into(),
                payload: String::new(),
            });

        app.update();

        let bp = app.world().resource::<BattlePending>();
        assert!(bp.0, "BattlePending should be set after StartBattle event");

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            matches!(*next, NextState::Pending(GameState::Battle)),
            "NextState should be Pending(Battle)"
        );
    }

    #[test]
    fn test_non_battle_events_ignored() {
        let mut app = setup_story_test_app();

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

    #[test]
    fn test_quest_accept_event() {
        let mut app = setup_story_test_app();

        app.world_mut()
            .resource_mut::<Messages<StoryEvent>>()
            .write(StoryEvent {
                id: "QuestAccept_cellar".into(),
                payload: String::new(),
            });

        app.update();

        let journal = app.world().resource::<QuestJournal>();
        assert!(
            journal.status("clear_the_cellar").is_some(),
            "Quest should be accepted"
        );
        let flags = app.world().resource::<StoryFlags>();
        assert!(
            flags
                .0
                .get("QuestAccepted_cellar")
                .copied()
                .unwrap_or(false),
            "QuestAccepted flag should be set"
        );
    }

    #[test]
    fn test_vendor_buy_deducts_gold() {
        let mut app = setup_story_test_app();

        // Give player gold first
        app.world_mut()
            .resource_mut::<Inventory>()
            .add_currency("gold", 25);

        app.world_mut()
            .resource_mut::<Messages<StoryEvent>>()
            .write(StoryEvent {
                id: "VendorBuy_health_potion".into(),
                payload: String::new(),
            });

        app.update();

        let inv = app.world().resource::<Inventory>();
        assert_eq!(
            inv.currency_balance("gold"),
            15,
            "should have 25 - 10 = 15 gold"
        );
        assert!(
            inv.has_item("health_potion", 1),
            "should have health potion"
        );
    }

    #[test]
    fn test_vendor_buy_insufficient_gold() {
        let mut app = setup_story_test_app();
        // No gold given

        app.world_mut()
            .resource_mut::<Messages<StoryEvent>>()
            .write(StoryEvent {
                id: "VendorBuy_health_potion".into(),
                payload: String::new(),
            });

        app.update();

        let inv = app.world().resource::<Inventory>();
        assert_eq!(inv.currency_balance("gold"), 0);
        assert!(
            !inv.has_item("health_potion", 1),
            "should NOT have potion without gold"
        );
    }
}
