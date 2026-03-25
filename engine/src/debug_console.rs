//! Runtime debug console + objective navigator for DJ Engine.
//!
//! **Objective Navigator** (always visible at bottom):
//!   `[ < ]  Step 2: MetHamster  [ > ]`
//!   Press `[` to rewind, `]` to advance through game checkpoints.
//!
//! **Debug Panel** (F1 to toggle):
//!   F2-F9 hotkeys for flags, items, quests, state dump.

use bevy::prelude::*;

use crate::character::PlayerTitle;
use crate::inventory::Inventory;
use crate::quest::QuestJournal;
use crate::story_graph::StoryFlags;

// ---------------------------------------------------------------------------
// Checkpoint definitions
// ---------------------------------------------------------------------------

/// A named checkpoint in the game's progression.
#[derive(Debug, Clone)]
pub struct Checkpoint {
    name: &'static str,
    flags_to_set: &'static [&'static str],
    flags_to_clear: &'static [&'static str],
    quest_action: Option<QuestAction>,
}

#[derive(Debug, Clone)]
pub enum QuestAction {
    Accept(&'static str),
    Complete(&'static str),
}

fn default_checkpoints() -> Vec<Checkpoint> {
    vec![
        Checkpoint {
            name: "Start (no flags)",
            flags_to_set: &[],
            flags_to_clear: &["MetHamster", "DefeatedGlitch"],
            quest_action: None,
        },
        Checkpoint {
            name: "Met Hamster",
            flags_to_set: &["MetHamster"],
            flags_to_clear: &["DefeatedGlitch"],
            quest_action: None,
        },
        Checkpoint {
            name: "Defeated Glitch",
            flags_to_set: &["MetHamster", "DefeatedGlitch"],
            flags_to_clear: &[],
            quest_action: None,
        },
        Checkpoint {
            name: "Quest Accepted",
            flags_to_set: &["MetHamster"],
            flags_to_clear: &[],
            quest_action: Some(QuestAction::Accept("slay_slimes")),
        },
        Checkpoint {
            name: "Quest Complete",
            flags_to_set: &["MetHamster", "DefeatedGlitch"],
            flags_to_clear: &[],
            quest_action: Some(QuestAction::Complete("slay_slimes")),
        },
    ]
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct DebugConsole {
    pub open: bool,
    pub checkpoint_index: usize,
    pub checkpoints: Vec<Checkpoint>,
}

impl Default for DebugConsole {
    fn default() -> Self {
        Self {
            open: false,
            checkpoint_index: 0,
            checkpoints: default_checkpoints(),
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct DebugOverlay;

#[derive(Component)]
struct DebugText;

#[derive(Component)]
struct ObjectiveNavBar;

#[derive(Component)]
struct ObjectiveNavText;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct DebugConsolePlugin;

impl Plugin for DebugConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugConsole>()
            .add_systems(Startup, (setup_debug_overlay, setup_objective_nav))
            .add_systems(
                Update,
                (
                    toggle_debug_console,
                    handle_debug_keys,
                    update_debug_text,
                    handle_objective_nav,
                    update_objective_nav_text,
                ),
            );
    }
}

// ---------------------------------------------------------------------------
// Objective Navigator (always visible)
// ---------------------------------------------------------------------------

fn setup_objective_nav(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            ObjectiveNavBar,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("[ < ]  Step 0: Start  [ > ]   (press [ or ] to navigate)"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
                ObjectiveNavText,
            ));
        });
}

fn handle_objective_nav(
    keys: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<DebugConsole>,
    mut flags: ResMut<StoryFlags>,
    mut quest_journal: ResMut<QuestJournal>,
) {
    let count = console.checkpoints.len();
    if count == 0 {
        return;
    }

    let mut changed = false;

    // [ key = rewind
    if keys.just_pressed(KeyCode::BracketLeft) {
        if console.checkpoint_index > 0 {
            console.checkpoint_index -= 1;
        } else {
            console.checkpoint_index = count - 1;
        }
        changed = true;
    }

    // ] key = advance
    if keys.just_pressed(KeyCode::BracketRight) {
        console.checkpoint_index = (console.checkpoint_index + 1) % count;
        changed = true;
    }

    if changed {
        let checkpoint = console.checkpoints[console.checkpoint_index].clone();

        // Apply flags
        for flag in checkpoint.flags_to_clear {
            flags.0.remove(*flag);
        }
        for flag in checkpoint.flags_to_set {
            flags.0.insert(flag.to_string(), true);
        }

        // Apply quest action
        if let Some(action) = &checkpoint.quest_action {
            match action {
                QuestAction::Accept(id) => {
                    quest_journal.accept(id.to_string());
                }
                QuestAction::Complete(id) => {
                    quest_journal.accept(id.to_string());
                    quest_journal.complete(id);
                }
            }
        }

        info!(
            "CHECKPOINT [{}]: {} | flags: {:?}",
            console.checkpoint_index,
            checkpoint.name,
            flags.0.keys().collect::<Vec<_>>()
        );
    }
}

fn update_objective_nav_text(
    console: Res<DebugConsole>,
    mut text_query: Query<&mut Text, With<ObjectiveNavText>>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let count = console.checkpoints.len();
    let name = if count > 0 {
        console.checkpoints[console.checkpoint_index].name
    } else {
        "no checkpoints"
    };

    *text = Text::new(format!(
        "[ < ]  Step {}/{}: {}  [ > ]   (press [ or ])",
        console.checkpoint_index + 1,
        count,
        name,
    ));
}

// ---------------------------------------------------------------------------
// Debug Panel (F1 toggle)
// ---------------------------------------------------------------------------

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

    if keys.just_pressed(KeyCode::F2) {
        flags.0.insert("MetHamster".to_string(), true);
        info!("DEBUG: Set flag MetHamster = true");
    }

    if keys.just_pressed(KeyCode::F3) {
        flags.0.insert("DefeatedGlitch".to_string(), true);
        info!("DEBUG: Set flag DefeatedGlitch = true");
    }

    if keys.just_pressed(KeyCode::F4) {
        inventory.add_currency("gold", 100);
        info!("DEBUG: Granted 100 gold");
    }

    if keys.just_pressed(KeyCode::F5) {
        inventory.add_item("health_potion", 5, 10);
        info!("DEBUG: Granted 5 health potions");
    }

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

    if keys.just_pressed(KeyCode::F7) {
        player_title.earn("champion");
        player_title.equip("champion");
        info!("DEBUG: Granted 'champion' title");
    }

    if keys.just_pressed(KeyCode::F8) {
        let count = flags.0.len();
        flags.0.clear();
        info!("DEBUG: Cleared {} story flags", count);
    }

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
