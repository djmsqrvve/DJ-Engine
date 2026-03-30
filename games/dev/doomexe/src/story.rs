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
                quest_journal.add_objective("clear_the_cellar", "kill_rats", 5);
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
            "VendorSell_rat_tail" => {
                let sell_value = 3_u64;
                if inventory.has_item("rat_tail", 1) {
                    inventory.remove_item("rat_tail", 1);
                    inventory.add_currency("gold", sell_value);
                    info!("Vendor: sold 1 rat_tail for {} gold", sell_value);
                } else {
                    info!("Vendor: no rat_tail to sell");
                }
            }
            "VendorSellAll_rat_tail" => {
                let sell_value = 3_u64;
                let count = inventory.count_item("rat_tail");
                if count > 0 {
                    inventory.remove_item("rat_tail", count);
                    let total = sell_value * count as u64;
                    inventory.add_currency("gold", total);
                    info!("Vendor: sold {} rat_tails for {} gold", count, total);
                } else {
                    info!("Vendor: no rat_tails to sell");
                }
            }
            "EnterCellar" => {
                info!("Story Event: Entering cellar");
                battle_pending.0 = true;
                next_state.set(GameState::Cellar);
            }
            "QuestAccept_grove" => {
                info!("Story Event: Quest accepted — purify_grove");
                quest_journal.accept("purify_grove");
                quest_journal.add_objective("purify_grove", "defeat_corruption", 4);
                flags.0.insert("QuestAccepted_grove".to_string(), true);
            }
            "EnterCorruptedGrove" => {
                info!("Story Event: Entering corrupted grove");
                next_state.set(GameState::CorruptedGrove);
            }
            "QuestTurnIn_grove" => {
                info!("Story Event: Quest turned in — purify_grove");
                quest_journal.turn_in("purify_grove");
                inventory.add_currency("gold", 75);
                flags.0.insert("QuestTurnedIn_grove".to_string(), true);
                if let Some(ref mut flash) = flash_events {
                    flash.write(ScreenFlashEvent::gold());
                }
            }
            "QuestAccept_crypt" => {
                info!("Story Event: Quest accepted — cleanse_the_crypt");
                quest_journal.accept("cleanse_the_crypt");
                quest_journal.add_objective("cleanse_the_crypt", "defeat_lich", 1);
                flags.0.insert("QuestAccepted_crypt".to_string(), true);
            }
            "EnterHauntedCrypt" => {
                info!("Story Event: Entering haunted crypt");
                next_state.set(GameState::HauntedCrypt);
            }
            "QuestTurnIn_crypt" => {
                info!("Story Event: Quest turned in — cleanse_the_crypt");
                quest_journal.turn_in("cleanse_the_crypt");
                inventory.add_currency("gold", 150);
                inventory.add_item("lichs_staff", 1, 1);
                flags.0.insert("QuestTurnedIn_crypt".to_string(), true);
                if let Some(ref mut flash) = flash_events {
                    flash.write(ScreenFlashEvent::gold());
                }
            }
            "DemoComplete" => {
                info!("Story Event: Demo complete! Entering victory screen.");
                flags.0.insert("DemoComplete".to_string(), true);
                next_state.set(GameState::Victory);
                if let Some(ref mut flash) = flash_events {
                    flash.write(ScreenFlashEvent::gold());
                }
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

    #[test]
    fn test_enter_cellar_sets_state() {
        let mut app = setup_story_test_app();

        app.world_mut()
            .resource_mut::<Messages<StoryEvent>>()
            .write(StoryEvent {
                id: "EnterCellar".into(),
                payload: String::new(),
            });

        app.update();

        let bp = app.world().resource::<BattlePending>();
        assert!(bp.0, "BattlePending should be set for cellar entry");

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            matches!(*next, NextState::Pending(GameState::Cellar)),
            "NextState should be Pending(Cellar)"
        );
    }

    #[test]
    fn test_demo_complete_sets_flag() {
        let mut app = setup_story_test_app();

        app.world_mut()
            .resource_mut::<Messages<StoryEvent>>()
            .write(StoryEvent {
                id: "DemoComplete".into(),
                payload: String::new(),
            });

        app.update();

        let flags = app.world().resource::<StoryFlags>();
        assert!(
            flags.0.get("DemoComplete").copied().unwrap_or(false),
            "DemoComplete flag should be set"
        );
    }
}
