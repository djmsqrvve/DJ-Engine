//! Runtime debug console for DJ Engine.
//!
//! Press F1 to toggle. Provides admin commands for testing:
//! - Set/clear story flags
//! - Force game state transitions
//! - Advance/complete quests
//! - Grant items, currency, titles
//! - Spawn combat events
//!
//! Only active in dev builds (feature-gated via `dev` feature).

use bevy::prelude::*;

use crate::character::PlayerTitle;
use crate::inventory::Inventory;
use crate::quest::QuestJournal;
use crate::story_graph::StoryFlags;

/// Resource tracking debug console state.
#[derive(Resource, Default)]
pub struct DebugConsole {
    pub open: bool,
    pub selected_action: usize,
}

/// Available debug actions.
#[derive(Debug, Clone)]
pub enum DebugAction {
    SetFlag(String, bool),
    ForceQuestComplete(String),
    GrantItem(String, u32),
    GrantCurrency(String, u64),
    GrantTitle(String),
    AdvanceQuest(String, String, u32),
}

/// Component for the debug UI overlay.
#[derive(Component)]
struct DebugOverlay;

/// Component for debug text content.
#[derive(Component)]
struct DebugText;

pub struct DebugConsolePlugin;

impl Plugin for DebugConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugConsole>()
            .add_systems(Startup, setup_debug_overlay)
            .add_systems(
                Update,
                (toggle_debug_console, handle_debug_keys, update_debug_text),
            );
    }
}

fn setup_debug_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            DebugOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("[F1] Debug Console"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                DebugText,
            ));
        });
}

fn toggle_debug_console(
    keys: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<DebugConsole>,
    mut overlay_query: Query<&mut Node, With<DebugOverlay>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        console.open = !console.open;
        if let Ok(mut node) = overlay_query.single_mut() {
            node.display = if console.open {
                Display::Flex
            } else {
                Display::None
            };
        }
        info!(
            "Debug console: {}",
            if console.open { "OPEN" } else { "CLOSED" }
        );
    }
}

fn handle_debug_keys(
    keys: Res<ButtonInput<KeyCode>>,
    console: Res<DebugConsole>,
    mut flags: ResMut<StoryFlags>,
    mut quest_journal: ResMut<QuestJournal>,
    mut inventory: ResMut<Inventory>,
    mut player_title: ResMut<PlayerTitle>,
) {
    if !console.open {
        return;
    }

    // F2: Set MetHamster flag
    if keys.just_pressed(KeyCode::F2) {
        flags.0.insert("MetHamster".to_string(), true);
        info!("DEBUG: Set flag MetHamster = true");
    }

    // F3: Set DefeatedGlitch flag
    if keys.just_pressed(KeyCode::F3) {
        flags.0.insert("DefeatedGlitch".to_string(), true);
        info!("DEBUG: Set flag DefeatedGlitch = true");
    }

    // F4: Grant 100 gold
    if keys.just_pressed(KeyCode::F4) {
        inventory.add_currency("gold", 100);
        info!("DEBUG: Granted 100 gold");
    }

    // F5: Grant health potions
    if keys.just_pressed(KeyCode::F5) {
        inventory.add_item("health_potion", 5, 10);
        info!("DEBUG: Granted 5 health potions");
    }

    // F6: Complete all active quests
    if keys.just_pressed(KeyCode::F6) {
        let active: Vec<String> = quest_journal
            .quests
            .values()
            .filter(|q| {
                matches!(
                    q.status,
                    crate::quest::QuestStatus::Accepted | crate::quest::QuestStatus::InProgress
                )
            })
            .map(|q| q.quest_id.clone())
            .collect();
        for id in &active {
            quest_journal.complete(id);
        }
        info!("DEBUG: Completed {} active quests", active.len());
    }

    // F7: Grant champion title
    if keys.just_pressed(KeyCode::F7) {
        player_title.earn("champion");
        player_title.equip("champion");
        info!("DEBUG: Granted and equipped 'champion' title");
    }

    // F8: Clear all flags
    if keys.just_pressed(KeyCode::F8) {
        let count = flags.0.len();
        flags.0.clear();
        info!("DEBUG: Cleared {} story flags", count);
    }

    // F9: Print current state
    if keys.just_pressed(KeyCode::F9) {
        info!("DEBUG STATE:");
        info!("  Flags: {:?}", flags.0);
        info!(
            "  Quests: {} total, {} active",
            quest_journal.quests.len(),
            quest_journal.active_count()
        );
        info!(
            "  Inventory: {} slots used, {} gold",
            inventory.used_slots(),
            inventory.currency_balance("gold")
        );
        info!("  Title: {:?}", player_title.active_title_id);
    }
}

fn update_debug_text(
    console: Res<DebugConsole>,
    flags: Res<StoryFlags>,
    quest_journal: Res<QuestJournal>,
    inventory: Res<Inventory>,
    mut text_query: Query<&mut Text, With<DebugText>>,
) {
    if !console.open {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let flag_list: Vec<&String> = flags.0.keys().collect();
    let quest_count = quest_journal.quests.len();
    let active = quest_journal.active_count();
    let gold = inventory.currency_balance("gold");
    let slots = inventory.used_slots();

    *text = Text::new(format!(
        "[F1] Debug Console\n\
         ──────────────────\n\
         F2: Set MetHamster\n\
         F3: Set DefeatedGlitch\n\
         F4: +100 Gold\n\
         F5: +5 Potions\n\
         F6: Complete Quests\n\
         F7: Grant Title\n\
         F8: Clear Flags\n\
         F9: Print State\n\
         ──────────────────\n\
         Flags: {:?}\n\
         Quests: {}/{} active\n\
         Gold: {} | Slots: {}",
        flag_list, active, quest_count, gold, slots
    ));
}
