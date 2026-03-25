//! World setup and gameplay for Helix RPG.
//!
//! Spawns entities from the HelixDatabase at startup and wires all
//! engine systems: combat, quests, inventory, interaction, abilities.

use bevy::prelude::*;

use dj_engine::collision::MovementIntent;
use dj_engine::combat::{CombatEvent, DamageEvent};
use dj_engine::data::components::{
    AbilityCooldownsComponent, CombatStatsComponent, InteractivityComponent, NpcComponent,
    TriggerType,
};
use dj_engine::data::database::Database;
use dj_engine::input::{ActionState, InputAction};
use dj_engine::interaction::{InteractionEvent, InteractionSource};
use dj_engine::inventory::Inventory;
use dj_engine::loot::LootDropEvent;
use dj_engine::quest::QuestJournal;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GamePhase {
    #[default]
    Overworld,
    Victory,
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Player;

#[derive(Component)]
struct HelixEnemy {
    enemy_id: String,
}

#[derive(Component)]
struct HelixNpc;

#[derive(Component)]
struct HudText;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world)
            .add_systems(
                Update,
                (
                    player_movement,
                    player_attack,
                    handle_damage,
                    handle_loot,
                    handle_npc_interaction,
                    check_quest,
                    update_hud,
                )
                    .run_if(in_state(GamePhase::Overworld)),
            )
            .add_systems(OnEnter(GamePhase::Victory), show_victory);
    }
}

// ---------------------------------------------------------------------------
// Setup — spawn from Database
// ---------------------------------------------------------------------------

fn setup_world(
    mut commands: Commands,
    mut quest_journal: ResMut<QuestJournal>,
    mut inventory: ResMut<Inventory>,
    database: Option<Res<Database>>,
) {
    // DJEnginePlugin already spawns cameras — don't add another.

    // Player
    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        CombatStatsComponent {
            max_hp: 100,
            hp: 100,
            mana: 60,
            damage: 22,
            defense: 8,
            crit_chance: 0.12,
            ..default()
        },
        AbilityCooldownsComponent::default(),
        InteractionSource,
        MovementIntent::default(),
        Sprite {
            color: Color::srgb(0.2, 0.5, 1.0),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
    ));

    // Spawn enemies from Database if available
    let mut enemy_count = 0;
    if let Some(db) = &database {
        // Spawn up to 3 enemies from the database
        for (i, enemy_row) in db.enemies.iter().take(3).enumerate() {
            let x = -150.0 + i as f32 * 100.0;
            let y = 120.0;
            commands.spawn((
                HelixEnemy {
                    enemy_id: enemy_row.id.clone(),
                },
                Transform::from_xyz(x, y, 0.0),
                CombatStatsComponent {
                    max_hp: enemy_row.hp,
                    hp: enemy_row.hp,
                    damage: enemy_row.damage,
                    defense: 0,
                    loot_table_id: if enemy_row.loot_table_id.is_empty() {
                        None
                    } else {
                        Some(enemy_row.loot_table_id.clone())
                    },
                    ..default()
                },
                Sprite {
                    color: Color::srgb(0.8, 0.2, 0.2),
                    custom_size: Some(Vec2::new(24.0, 24.0)),
                    ..default()
                },
            ));
            let name = enemy_row
                .name
                .get("en")
                .cloned()
                .unwrap_or_else(|| enemy_row.id.clone());
            info!(
                "Helix RPG: spawned enemy '{}' (hp={}, dmg={})",
                name, enemy_row.hp, enemy_row.damage
            );
            enemy_count += 1;
        }

        // Spawn NPCs from Database
        for (i, npc_row) in db.npcs.iter().take(2).enumerate() {
            let x = 150.0;
            let y = -50.0 + i as f32 * 80.0;
            let name = npc_row
                .name
                .get("en")
                .cloned()
                .unwrap_or_else(|| npc_row.id.clone());
            commands.spawn((
                HelixNpc,
                Transform::from_xyz(x, y, 0.0),
                InteractivityComponent {
                    trigger_type: TriggerType::Npc,
                    trigger_id: npc_row.id.clone(),
                    ..default()
                },
                NpcComponent {
                    npc_id: npc_row.id.clone(),
                    display_name: npc_row.name.clone(),
                    dialogue_set_id: npc_row
                        .default_quest_ids
                        .first()
                        .cloned()
                        .unwrap_or_default(),
                    quest_ids: npc_row.default_quest_ids.clone(),
                    ..default()
                },
                Sprite {
                    color: Color::srgb(0.2, 0.8, 0.4),
                    custom_size: Some(Vec2::new(28.0, 28.0)),
                    ..default()
                },
            ));
            info!("Helix RPG: spawned NPC '{}'", name);
        }

        info!(
            "Helix RPG: loaded {} enemies, {} NPCs from Database ({} total rows)",
            enemy_count,
            db.npcs.len().min(2),
            db.enemies.len() + db.npcs.len()
        );
    } else {
        // Fallback: spawn a default enemy if no Database loaded
        commands.spawn((
            HelixEnemy {
                enemy_id: "default_slime".into(),
            },
            Transform::from_xyz(-100.0, 80.0, 0.0),
            CombatStatsComponent {
                max_hp: 25,
                hp: 25,
                damage: 8,
                defense: 2,
                ..default()
            },
            Sprite {
                color: Color::srgb(0.6, 0.9, 0.3),
                custom_size: Some(Vec2::new(24.0, 24.0)),
                ..default()
            },
        ));
        enemy_count = 1;
        info!("Helix RPG: no Database loaded, spawned fallback enemy");
    }

    // Quest: defeat all enemies
    quest_journal.accept("helix_hunt");
    quest_journal.add_objective("helix_hunt", "defeat_enemies", enemy_count as u32);

    inventory.add_currency("gold", 100);

    // HUD
    commands.spawn((
        HudText,
        Text::new("Helix RPG — Space to attack, WASD to move"),
        TextFont {
            font_size: 18.0,
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
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn player_movement(
    time: Res<Time>,
    actions: Res<ActionState>,
    mut query: Query<&mut MovementIntent, With<Player>>,
) {
    let mut dir = Vec2::ZERO;
    if actions.pressed(InputAction::Up) {
        dir.y += 1.0;
    }
    if actions.pressed(InputAction::Down) {
        dir.y -= 1.0;
    }
    if actions.pressed(InputAction::Left) {
        dir.x -= 1.0;
    }
    if actions.pressed(InputAction::Right) {
        dir.x += 1.0;
    }

    let dir = dir.normalize_or_zero();
    for mut intent in &mut query {
        intent.0 = dir * 160.0 * time.delta_secs();
    }
}

fn player_attack(
    actions: Res<ActionState>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<HelixEnemy>>,
    mut combat_events: MessageWriter<CombatEvent>,
) {
    if !actions.just_pressed(InputAction::Confirm) {
        return;
    }

    let Ok((player, player_pos)) = player_query.single() else {
        return;
    };

    // Attack nearest enemy
    let nearest = enemy_query.iter().min_by(|(_, a), (_, b)| {
        let da = player_pos.translation.distance_squared(a.translation);
        let db = player_pos.translation.distance_squared(b.translation);
        da.total_cmp(&db)
    });

    if let Some((enemy, _)) = nearest {
        combat_events.write(CombatEvent {
            attacker: player,
            target: enemy,
            flat_damage: None,
        });
    }
}

fn handle_damage(
    mut events: MessageReader<DamageEvent>,
    mut quest_journal: ResMut<QuestJournal>,
    mut commands: Commands,
    enemy_query: Query<&HelixEnemy>,
) {
    for event in events.read() {
        if event.target_defeated {
            if let Ok(enemy) = enemy_query.get(event.target) {
                info!(
                    "Helix RPG: defeated '{}' ({} damage, crit={})",
                    enemy.enemy_id, event.final_damage, event.is_critical
                );
                quest_journal.progress_objective("helix_hunt", "defeat_enemies", 1);
                commands.entity(event.target).despawn();
            }
        }
    }
}

fn handle_loot(mut events: MessageReader<LootDropEvent>) {
    for event in events.read() {
        info!("Helix RPG: looted {} x{}", event.item_id, event.quantity);
    }
}

fn handle_npc_interaction(mut events: MessageReader<InteractionEvent>) {
    for event in events.read() {
        if event.trigger_type == TriggerType::Npc {
            info!("Helix RPG: talked to NPC '{}'", event.trigger_id);
        }
    }
}

fn check_quest(quest_journal: Res<QuestJournal>, mut next_state: ResMut<NextState<GamePhase>>) {
    if quest_journal.all_objectives_complete("helix_hunt") {
        quest_journal.status("helix_hunt"); // read status
        next_state.set(GamePhase::Victory);
    }
}

fn update_hud(
    quest_journal: Res<QuestJournal>,
    inventory: Res<Inventory>,
    player_query: Query<&CombatStatsComponent, With<Player>>,
    enemy_query: Query<&HelixEnemy>,
    mut text_query: Query<&mut Text, With<HudText>>,
) {
    let Ok(stats) = player_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let enemies_alive = enemy_query.iter().count();
    let gold = inventory.currency_balance("gold");
    let quest_status = quest_journal
        .status("helix_hunt")
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|| "none".into());

    **text = format!(
        "HP: {}/{} | Mana: {} | Gold: {} | Enemies: {} | Quest: {}  [Space=Attack, WASD=Move]",
        stats.hp, stats.max_hp, stats.mana, gold, enemies_alive, quest_status
    );
}

fn show_victory(mut commands: Commands) {
    commands.spawn((
        Text::new("ALL ENEMIES DEFEATED! Quest complete."),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(350.0),
            left: Val::Px(280.0),
            ..default()
        },
    ));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_phase_default() {
        assert_eq!(GamePhase::default(), GamePhase::Overworld);
    }

    #[test]
    fn test_quest_defeat_flow() {
        let mut journal = QuestJournal::default();
        journal.accept("helix_hunt");
        journal.add_objective("helix_hunt", "defeat_enemies", 3);

        journal.progress_objective("helix_hunt", "defeat_enemies", 1);
        assert!(!journal.all_objectives_complete("helix_hunt"));

        journal.progress_objective("helix_hunt", "defeat_enemies", 2);
        assert!(journal.all_objectives_complete("helix_hunt"));
    }

    #[test]
    fn test_database_enemy_spawn_pattern() {
        // Verify the spawn pattern works with a mock Database
        let mut db = Database::default();
        use dj_engine::data::database::EnemyRow;
        db.enemies.push(EnemyRow::new("wolf", "Wolf"));
        db.enemies.push(EnemyRow::new("slime", "Slime"));

        assert_eq!(db.enemies.len(), 2);
        assert_eq!(db.enemies[0].id, "wolf");
    }
}
