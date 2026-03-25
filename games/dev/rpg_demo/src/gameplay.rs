//! Core gameplay demonstrating all DJ Engine runtime systems.
//!
//! This single file shows how to wire combat, quests, inventory,
//! NPC interaction, abilities, loot, and status effects into a
//! playable game loop.

use bevy::prelude::*;

use dj_engine::collision::MovementIntent;
use dj_engine::combat::{CombatEvent, DamageEvent};
use dj_engine::data::components::{
    AbilityCooldownsComponent, CombatStatsComponent, InteractivityComponent, NpcComponent,
    TriggerType,
};
use dj_engine::data::database::{Database, LootEntry, LootTableRow};
use dj_engine::input::{ActionState, InputAction};
use dj_engine::interaction::{InteractionEvent, InteractionSource};
use dj_engine::inventory::Inventory;
use dj_engine::loot::LootDropEvent;
use dj_engine::quest::{QuestJournal, QuestStatus};

// ---------------------------------------------------------------------------
// Game state
// ---------------------------------------------------------------------------

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GamePhase {
    #[default]
    Playing,
    QuestComplete,
}

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy {
    id: String,
}

#[derive(Component)]
struct QuestGiver;

#[derive(Component)]
struct HudText;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world)
            .add_systems(
                Update,
                (
                    player_attack_input,
                    handle_npc_interaction,
                    handle_damage_feedback,
                    handle_loot_feedback,
                    check_quest_completion,
                    update_hud,
                )
                    .run_if(in_state(GamePhase::Playing)),
            )
            .add_systems(OnEnter(GamePhase::QuestComplete), show_victory);
    }
}

// ---------------------------------------------------------------------------
// Setup — spawn the world
// ---------------------------------------------------------------------------

fn setup_world(
    mut commands: Commands,
    mut quest_journal: ResMut<QuestJournal>,
    mut inventory: ResMut<Inventory>,
) {
    // Note: DJEnginePlugin's RenderingPlugin already spawns Camera2d (MainCamera + DisplayCamera).
    // Do NOT spawn another Camera2d here — it causes order ambiguity warnings.

    // Player entity — demonstrates CombatStats, Cooldowns, InteractionSource, Movement
    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        CombatStatsComponent {
            max_hp: 100,
            hp: 100,
            mana: 50,
            damage: 12,
            defense: 5,
            crit_chance: 0.1,
            ..default()
        },
        AbilityCooldownsComponent::default(),
        InteractionSource,
        MovementIntent::default(),
        Sprite {
            color: Color::srgb(0.2, 0.6, 1.0),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
    ));

    // NPC quest giver — demonstrates InteractivityComponent + NpcComponent
    commands.spawn((
        QuestGiver,
        Transform::from_xyz(100.0, 0.0, 0.0),
        InteractivityComponent {
            trigger_type: TriggerType::Npc,
            trigger_id: "elder".into(),
            ..default()
        },
        NpcComponent {
            npc_id: "village_elder".into(),
            display_name: [("en".into(), "Village Elder".into())].into(),
            dialogue_set_id: "elder_greeting".into(),
            quest_ids: vec!["slay_slimes".into()],
            ..default()
        },
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.2),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
    ));

    // Enemy — demonstrates CombatStatsComponent with loot_table_id
    commands.spawn((
        Enemy {
            id: "green_slime".into(),
        },
        Transform::from_xyz(-100.0, 50.0, 0.0),
        CombatStatsComponent {
            max_hp: 60,
            hp: 60,
            mana: 0,
            damage: 8,
            defense: 2,
            loot_table_id: Some("slime_loot".into()),
            ..default()
        },
        Sprite {
            color: Color::srgb(0.9, 0.2, 0.2),
            custom_size: Some(Vec2::new(24.0, 24.0)),
            ..default()
        },
    ));

    // Set up the loot table in the database
    let mut db = Database::default();
    let mut loot = LootTableRow::new("slime_loot");
    loot.entries.push(LootEntry {
        item_id: "slime_gel".into(),
        chance: 1.0,
        min_quantity: 1,
        max_quantity: 3,
    });
    loot.entries.push(LootEntry {
        item_id: "health_potion".into(),
        chance: 0.5,
        min_quantity: 1,
        max_quantity: 1,
    });
    db.loot_tables.push(loot);
    commands.insert_resource(db);

    // Pre-register the quest
    quest_journal.accept("slay_slimes");
    quest_journal.add_objective("slay_slimes", "kill_slime", 1);

    // Give the player some starting gold
    inventory.add_currency("gold", 50);

    // HUD text
    commands.spawn((
        HudText,
        Text::new("RPG Demo — Press Space to attack, E to interact"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    info!("RPG Demo: world set up — player, NPC, enemy, quest, loot table");
}

// ---------------------------------------------------------------------------
// Systems — demonstrate engine features
// ---------------------------------------------------------------------------

/// Player attacks nearest enemy when Space is pressed.
/// Demonstrates: CombatEvent dispatch.
fn player_attack_input(
    actions: Res<ActionState>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut combat_events: MessageWriter<CombatEvent>,
) {
    if !actions.just_pressed(InputAction::Confirm) {
        return;
    }

    let Ok(player) = player_query.single() else {
        return;
    };

    // Attack the first enemy found
    if let Some(enemy) = enemy_query.iter().next() {
        combat_events.write(CombatEvent {
            attacker: player,
            target: enemy,
            flat_damage: None,
        });
        info!("RPG Demo: player attacks!");
    }
}

/// React to NPC interaction events.
/// Demonstrates: InteractionEvent handling, QuestJournal.
fn handle_npc_interaction(
    mut events: MessageReader<InteractionEvent>,
    quest_journal: Res<QuestJournal>,
) {
    for event in events.read() {
        if event.trigger_type == TriggerType::Npc {
            info!(
                "RPG Demo: interacted with NPC (trigger_id='{}')",
                event.trigger_id
            );

            // Check quest status
            if let Some(status) = quest_journal.status("slay_slimes") {
                info!("RPG Demo: quest 'slay_slimes' status = {:?}", status);
            }
        }
    }
}

/// React to damage events — log combat results, progress quest on kill.
/// Demonstrates: DamageEvent handling, QuestJournal progress.
fn handle_damage_feedback(
    mut events: MessageReader<DamageEvent>,
    mut quest_journal: ResMut<QuestJournal>,
    mut commands: Commands,
    enemy_query: Query<&Enemy>,
) {
    for event in events.read() {
        info!(
            "RPG Demo: {} damage dealt (crit={}, hp_after={})",
            event.final_damage, event.is_critical, event.target_hp_after
        );

        if event.target_defeated {
            info!("RPG Demo: enemy defeated!");

            // Check if it's an enemy with an ID for quest tracking
            if let Ok(enemy) = enemy_query.get(event.target) {
                if enemy.id == "green_slime" {
                    let complete = quest_journal.progress_objective("slay_slimes", "kill_slime", 1);
                    if complete {
                        quest_journal.complete("slay_slimes");
                        info!("RPG Demo: quest 'slay_slimes' COMPLETED!");
                    }
                }
            }

            // Despawn the defeated entity
            commands.entity(event.target).despawn();
        }
    }
}

/// React to loot drop events.
/// Demonstrates: LootDropEvent → Inventory integration.
fn handle_loot_feedback(mut events: MessageReader<LootDropEvent>) {
    for event in events.read() {
        if event.added_to_inventory {
            info!(
                "RPG Demo: looted {} x{} (added to inventory)",
                event.item_id, event.quantity
            );
        } else {
            info!(
                "RPG Demo: looted {} x{} (inventory full!)",
                event.item_id, event.quantity
            );
        }
    }
}

/// Check if the quest is complete and transition game state.
/// Demonstrates: QuestStatus queries, Bevy state transitions.
fn check_quest_completion(
    quest_journal: Res<QuestJournal>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if quest_journal.status("slay_slimes") == Some(QuestStatus::Completed) {
        next_state.set(GamePhase::QuestComplete);
    }
}

/// Update the HUD text with current game state.
/// Demonstrates: reading QuestJournal + Inventory from Bevy resources.
fn update_hud(
    quest_journal: Res<QuestJournal>,
    inventory: Res<Inventory>,
    player_query: Query<&CombatStatsComponent, With<Player>>,
    mut text_query: Query<&mut Text, With<HudText>>,
) {
    let Ok(stats) = player_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let quest_status = quest_journal
        .status("slay_slimes")
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|| "none".into());

    let gold = inventory.currency_balance("gold");
    let potions = inventory.count_item("health_potion");
    let slime_gel = inventory.count_item("slime_gel");

    **text = format!(
        "HP: {}/{} | Mana: {} | Gold: {} | Potions: {} | Slime Gel: {} | Quest: {}  [Space=Attack]",
        stats.hp, stats.max_hp, stats.mana, gold, potions, slime_gel, quest_status
    );
}

/// Show victory screen when quest is complete.
fn show_victory(mut commands: Commands) {
    commands.spawn((
        Text::new("QUEST COMPLETE! You slew the slime and collected your loot."),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(300.0),
            left: Val::Px(200.0),
            ..default()
        },
    ));
    info!("RPG Demo: VICTORY!");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_phase_default_is_playing() {
        assert_eq!(GamePhase::default(), GamePhase::Playing);
    }

    #[test]
    fn test_quest_setup() {
        let mut journal = QuestJournal::default();
        journal.accept("slay_slimes");
        journal.add_objective("slay_slimes", "kill_slime", 1);
        assert_eq!(journal.status("slay_slimes"), Some(QuestStatus::Accepted));
        assert!(!journal.all_objectives_complete("slay_slimes"));
    }

    #[test]
    fn test_quest_completion_flow() {
        let mut journal = QuestJournal::default();
        journal.accept("slay_slimes");
        journal.add_objective("slay_slimes", "kill_slime", 1);

        journal.progress_objective("slay_slimes", "kill_slime", 1);
        assert!(journal.all_objectives_complete("slay_slimes"));

        journal.complete("slay_slimes");
        assert_eq!(journal.status("slay_slimes"), Some(QuestStatus::Completed));
    }

    #[test]
    fn test_inventory_starting_gold() {
        let mut inv = Inventory::new(20);
        inv.add_currency("gold", 50);
        assert_eq!(inv.currency_balance("gold"), 50);
    }

    #[test]
    fn test_loot_adds_to_inventory() {
        let mut inv = Inventory::new(20);
        let leftover = inv.add_item("slime_gel", 2, 99);
        assert_eq!(leftover, 0);
        assert_eq!(inv.count_item("slime_gel"), 2);
    }
}
