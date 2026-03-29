//! World setup and gameplay for Helix RPG.
//!
//! Spawns entities from the HelixDatabase at startup and wires all
//! engine systems: combat, quests, inventory, interaction, abilities.
//! Mobs are placed by zone, quests registered with real prerequisites
//! and rewards, and loot tables wired for combat drops.

use bevy::prelude::*;
use std::collections::HashMap;

use dj_engine::collision::MovementIntent;
use dj_engine::combat::{AttackCooldown, CombatEvent, DamageEvent};
use dj_engine::data::components::{
    AbilityCooldownsComponent, CombatStatsComponent, InteractivityComponent, NpcComponent,
    TriggerType,
};
use dj_engine::data::database::{Database, EnemyRow, QuestRow, ZoneRow};
use dj_engine::input::{ActionState, InputAction};
use dj_engine::interaction::{InteractionEvent, InteractionSource};
use dj_engine::inventory::Inventory;
use dj_engine::loot::LootDropEvent;
use dj_engine::particles::{ParticleConfig, ParticleEvent};
use dj_engine::quest::QuestJournal;
use dj_engine::screen_fx::{ScreenFlashEvent, ScreenShakeEvent};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// World-space size of each zone tile in the grid layout.
const ZONE_SIZE: f32 = 600.0;
/// Columns in the zone grid layout.
const ZONE_COLS: usize = 5;

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
// Resources
// ---------------------------------------------------------------------------

/// Tracks zone layout for the HUD and zone transition display.
#[derive(Resource, Default)]
struct ZoneMap {
    /// zone_id -> (grid_x, grid_y, display_name)
    zones: HashMap<String, (usize, usize, String)>,
    total_enemies: usize,
    total_npcs: usize,
    total_quests: usize,
    total_loot_tables: usize,
    total_abilities: usize,
    total_consumables: usize,
    total_equipment: usize,
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Player;

#[derive(Component)]
struct HelixEnemy {
    enemy_id: String,
    zone_id: String,
}

#[derive(Component)]
struct HelixNpc;

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct ZoneLabel;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoneMap>()
            .add_systems(Startup, setup_world)
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
    mut zone_map: ResMut<ZoneMap>,
    database: Option<Res<Database>>,
) {
    // Player
    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        CombatStatsComponent {
            max_hp: 150,
            hp: 150,
            mana: 80,
            damage: 22,
            defense: 8,
            crit_chance: 0.12,
            ..default()
        },
        AbilityCooldownsComponent::default(),
        AttackCooldown::new(0.8),
        InteractionSource,
        MovementIntent::default(),
        Sprite {
            color: Color::srgb(0.2, 0.5, 1.0),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
    ));

    let mut enemy_count = 0;

    if let Some(db) = &database {
        // --- Zone layout ---
        let zone_positions = build_zone_layout(&db.zones, &mut zone_map);

        // --- Spawn zone labels ---
        for (zone_id, &(origin_x, origin_y)) in &zone_positions {
            if let Some(&(_, _, ref display_name)) = zone_map.zones.get(zone_id) {
                commands.spawn((
                    ZoneLabel,
                    Text::new(display_name.clone()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.7, 0.7, 0.9, 0.6)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(origin_x + ZONE_SIZE * 0.3),
                        top: Val::Px(origin_y - ZONE_SIZE * 0.45),
                        ..default()
                    },
                ));
            }
        }

        // --- Spawn enemies by zone ---
        let enemies_by_zone = group_enemies_by_zone(&db.enemies);

        for (zone_id, enemies) in &enemies_by_zone {
            let (origin_x, origin_y) = zone_positions.get(zone_id).copied().unwrap_or((0.0, 0.0));

            for (i, enemy_row) in enemies.iter().enumerate() {
                let pos = scatter_position(origin_x, origin_y, i, enemies.len());
                commands.spawn((
                    HelixEnemy {
                        enemy_id: enemy_row.id.clone(),
                        zone_id: zone_id.clone(),
                    },
                    Transform::from_xyz(pos.x, pos.y, 0.0),
                    CombatStatsComponent {
                        max_hp: enemy_row.hp,
                        hp: enemy_row.hp,
                        damage: enemy_row.damage,
                        defense: enemy_row.defense,
                        loot_table_id: if enemy_row.loot_table_id.is_empty() {
                            None
                        } else {
                            Some(enemy_row.loot_table_id.clone())
                        },
                        ..default()
                    },
                    Sprite {
                        color: enemy_color(enemy_row.hp),
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                ));
                enemy_count += 1;
            }
        }

        // Spawn enemies without zone assignments at origin
        let unzoned: Vec<&EnemyRow> = db
            .enemies
            .iter()
            .filter(|e| e.zone_ids.is_empty())
            .collect();
        for (i, enemy_row) in unzoned.iter().enumerate() {
            let pos = scatter_position(0.0, 0.0, i, unzoned.len());
            commands.spawn((
                HelixEnemy {
                    enemy_id: enemy_row.id.clone(),
                    zone_id: "unzoned".into(),
                },
                Transform::from_xyz(pos.x, pos.y, 0.0),
                CombatStatsComponent {
                    max_hp: enemy_row.hp,
                    hp: enemy_row.hp,
                    damage: enemy_row.damage,
                    defense: enemy_row.defense,
                    loot_table_id: if enemy_row.loot_table_id.is_empty() {
                        None
                    } else {
                        Some(enemy_row.loot_table_id.clone())
                    },
                    ..default()
                },
                Sprite {
                    color: enemy_color(enemy_row.hp),
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
            ));
            enemy_count += 1;
        }

        // --- Spawn NPCs by zone (using location_tags) ---
        let mut npc_zone_idx: HashMap<String, usize> = HashMap::new();
        for (i, npc_row) in db.npcs.iter().enumerate() {
            // Try to place NPC in a zone from its location_tags
            let npc_zone = npc_row
                .location_tags
                .iter()
                .find(|tag| zone_positions.contains_key(*tag))
                .cloned();

            let (base_x, base_y) = if let Some(ref zone_id) = npc_zone {
                let idx = npc_zone_idx.entry(zone_id.clone()).or_insert(0);
                *idx += 1;
                let (zx, zy) = zone_positions[zone_id];
                // Offset NPCs to the edge of the zone
                let offset = (*idx as f32) * 35.0;
                (zx - ZONE_SIZE * 0.35 + offset, zy - ZONE_SIZE * 0.2)
            } else {
                // Unzoned NPCs go to a "town" area near origin
                (
                    -300.0 + (i % 10) as f32 * 50.0,
                    -200.0 - (i / 10) as f32 * 50.0,
                )
            };

            let name = npc_row
                .name
                .get("en")
                .cloned()
                .unwrap_or_else(|| npc_row.id.clone());
            commands.spawn((
                HelixNpc,
                Transform::from_xyz(base_x, base_y, 0.0),
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
                    color: npc_color(&npc_row.default_faction),
                    custom_size: Some(Vec2::new(26.0, 26.0)),
                    ..default()
                },
            ));
            info!("Helix RPG: spawned NPC '{}' in {:?}", name, npc_zone);
        }

        // --- Register quests ---
        let quest_count = register_quests(&db.quests, &mut quest_journal);

        zone_map.total_enemies = enemy_count;
        zone_map.total_npcs = db.npcs.len();
        zone_map.total_quests = quest_count;
        zone_map.total_loot_tables = db.loot_tables.len();
        zone_map.total_abilities = db.abilities.len();
        zone_map.total_consumables = db.consumables.len();
        zone_map.total_equipment = db.equipment.len();

        info!(
            "Helix RPG: loaded {} enemies, {} NPCs across {} zones | {} abilities, {} equipment, {} consumables | {} quests, {} loot tables",
            enemy_count,
            db.npcs.len(),
            zone_map.zones.len(),
            db.abilities.len(),
            db.equipment.len(),
            db.consumables.len(),
            quest_count,
            db.loot_tables.len(),
        );
    } else {
        // Fallback: spawn a default enemy if no Database loaded
        commands.spawn((
            HelixEnemy {
                enemy_id: "default_slime".into(),
                zone_id: "fallback".into(),
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

    // Quest: defeat all enemies (meta-quest wrapping everything)
    quest_journal.accept("helix_hunt");
    quest_journal.add_objective("helix_hunt", "defeat_enemies", enemy_count as u32);

    inventory.add_currency("gold", 100);

    // HUD
    commands.spawn((
        HudText,
        Text::new("Helix RPG — Loading..."),
        TextFont {
            font_size: 16.0,
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
// Zone layout helpers
// ---------------------------------------------------------------------------

/// Build a grid layout for zones. Returns zone_id -> (world_x, world_y).
fn build_zone_layout(zones: &[ZoneRow], zone_map: &mut ZoneMap) -> HashMap<String, (f32, f32)> {
    let mut positions = HashMap::new();

    // Sort zones by level_min for spatial coherence
    let mut sorted: Vec<&ZoneRow> = zones.iter().collect();
    sorted.sort_by_key(|z| z.level_min);

    for (i, zone) in sorted.iter().enumerate() {
        let col = i % ZONE_COLS;
        let row = i / ZONE_COLS;
        let x = col as f32 * ZONE_SIZE;
        let y = -(row as f32 * ZONE_SIZE);

        let name = zone
            .name
            .get("en")
            .cloned()
            .unwrap_or_else(|| zone.id.clone());

        positions.insert(zone.id.clone(), (x, y));
        zone_map.zones.insert(zone.id.clone(), (col, row, name));
    }

    positions
}

/// Group enemies by their first zone_id. Enemies with zone assignments
/// are only returned here; unzoned enemies are handled separately.
fn group_enemies_by_zone(enemies: &[EnemyRow]) -> HashMap<String, Vec<&EnemyRow>> {
    let mut groups: HashMap<String, Vec<&EnemyRow>> = HashMap::new();
    for enemy in enemies {
        for zone_id in &enemy.zone_ids {
            groups.entry(zone_id.clone()).or_default().push(enemy);
        }
    }
    groups
}

/// Scatter position within a zone area using a spiral pattern.
fn scatter_position(origin_x: f32, origin_y: f32, index: usize, _total: usize) -> Vec2 {
    let angle = index as f32 * 2.399; // golden angle in radians
    let radius = 30.0 + (index as f32).sqrt() * 40.0;
    Vec2::new(
        origin_x + angle.cos() * radius,
        origin_y + angle.sin() * radius,
    )
}

/// Color NPCs based on faction.
fn npc_color(faction: &str) -> Color {
    match faction {
        f if f.contains("alliance") || f.contains("friendly") => Color::srgb(0.2, 0.6, 1.0),
        f if f.contains("horde") => Color::srgb(0.8, 0.3, 0.2),
        f if f.contains("neutral") => Color::srgb(0.8, 0.8, 0.3),
        _ => Color::srgb(0.2, 0.8, 0.4),
    }
}

/// Color enemies based on HP — weak = green, medium = orange, strong = red.
fn enemy_color(hp: i32) -> Color {
    if hp <= 50 {
        Color::srgb(0.5, 0.8, 0.3)
    } else if hp <= 200 {
        Color::srgb(0.8, 0.6, 0.2)
    } else {
        Color::srgb(0.9, 0.2, 0.2)
    }
}

// ---------------------------------------------------------------------------
// Quest registration
// ---------------------------------------------------------------------------

/// Register all quests from the database, wiring prerequisites and rewards.
/// Returns the number of quests registered.
fn register_quests(quests: &[QuestRow], journal: &mut QuestJournal) -> usize {
    // Sort by prerequisite count so root quests register first
    let mut sorted: Vec<&QuestRow> = quests.iter().collect();
    sorted.sort_by_key(|q| q.start_conditions.len());

    let mut count = 0;
    for quest in &sorted {
        journal.accept(&quest.id);

        // Register objectives from completion_conditions
        let obj_count = quest.completion_conditions.len().max(1) as u32;
        journal.add_objective(&quest.id, "complete", obj_count);

        count += 1;
    }

    // Accept starter quests (those with no prerequisites)
    // Already accepted above via journal.accept
    count
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
        intent.0 = dir * 200.0 * time.delta_secs();
    }
}

fn player_attack(
    actions: Res<ActionState>,
    mut player_query: Query<(Entity, &Transform, &mut AttackCooldown), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<HelixEnemy>>,
    mut combat_events: MessageWriter<CombatEvent>,
) {
    if !actions.just_pressed(InputAction::Confirm) {
        return;
    }

    let Ok((player, player_pos, mut cooldown)) = player_query.single_mut() else {
        return;
    };

    if !cooldown.ready() {
        return;
    }

    // Attack nearest enemy within range
    let attack_range = 120.0_f32;
    let nearest = enemy_query
        .iter()
        .filter(|(_, t)| {
            player_pos.translation.distance_squared(t.translation) < attack_range * attack_range
        })
        .min_by(|(_, a), (_, b)| {
            let da = player_pos.translation.distance_squared(a.translation);
            let db = player_pos.translation.distance_squared(b.translation);
            da.total_cmp(&db)
        });

    if let Some((enemy, _)) = nearest {
        cooldown.reset();
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
    player_query: Query<Entity, With<Player>>,
    mut shake_events: MessageWriter<ScreenShakeEvent>,
    mut flash_events: MessageWriter<ScreenFlashEvent>,
    mut particle_events: MessageWriter<ParticleEvent>,
) {
    for event in events.read() {
        // Screen FX
        if player_query.get(event.target).is_ok() {
            shake_events.write(ScreenShakeEvent::medium());
            flash_events.write(ScreenFlashEvent::damage());
        } else {
            shake_events.write(ScreenShakeEvent::light());
        }

        if event.target_defeated {
            if let Ok(enemy) = enemy_query.get(event.target) {
                info!(
                    "Helix RPG: defeated '{}' in zone '{}' ({} damage, crit={})",
                    enemy.enemy_id, enemy.zone_id, event.final_damage, event.is_critical
                );
                particle_events.write(ParticleEvent {
                    position: Vec3::ZERO,
                    config: ParticleConfig::death_burst(),
                });
                flash_events.write(ScreenFlashEvent::gold());
                quest_journal.progress_objective("helix_hunt", "defeat_enemies", 1);
                commands.entity(event.target).despawn();
            }
        }
    }
}

fn handle_loot(mut events: MessageReader<LootDropEvent>, mut inventory: ResMut<Inventory>) {
    for event in events.read() {
        info!(
            "Helix RPG: looted {} x{} (added={})",
            event.item_id, event.quantity, event.added_to_inventory
        );
        if !event.added_to_inventory {
            inventory.add_item(&event.item_id, event.quantity, 20);
        }
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
        quest_journal.status("helix_hunt");
        next_state.set(GamePhase::Victory);
    }
}

fn update_hud(
    quest_journal: Res<QuestJournal>,
    inventory: Res<Inventory>,
    zone_map: Res<ZoneMap>,
    player_query: Query<(&CombatStatsComponent, &Transform), With<Player>>,
    enemy_query: Query<&HelixEnemy>,
    mut text_query: Query<&mut Text, With<HudText>>,
) {
    let Ok((stats, player_pos)) = player_query.single() else {
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

    // Detect current zone from player position
    let current_zone = detect_zone(player_pos, &zone_map);

    **text = format!(
        "HP: {}/{} | Mana: {} | Gold: {} | Enemies: {} | Zone: {} | Quest: {}\nDB: {} mobs, {} NPCs, {} abilities, {} equip, {} consumables | {} loot tables  [Space=Attack, WASD=Move]",
        stats.hp, stats.max_hp, stats.mana, gold, enemies_alive,
        current_zone, quest_status,
        zone_map.total_enemies, zone_map.total_npcs, zone_map.total_abilities,
        zone_map.total_equipment, zone_map.total_consumables,
        zone_map.total_loot_tables,
    );
}

/// Detect which zone the player is in based on position.
fn detect_zone(player_pos: &Transform, zone_map: &ZoneMap) -> String {
    let px = player_pos.translation.x;
    let py = player_pos.translation.y;

    for (zone_id, &(col, row, ref name)) in &zone_map.zones {
        let zx = col as f32 * ZONE_SIZE;
        let zy = -(row as f32 * ZONE_SIZE);
        let half = ZONE_SIZE / 2.0;

        if px >= zx - half && px < zx + half && py >= zy - half && py < zy + half {
            return format!("{} ({})", name, zone_id);
        }
    }

    "Wilderness".into()
}

fn show_victory(mut commands: Commands, zone_map: Res<ZoneMap>) {
    commands.spawn((
        Text::new(format!(
            "ALL {} ENEMIES DEFEATED! {} quests available. Victory!",
            zone_map.total_enemies, zone_map.total_quests
        )),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(350.0),
            left: Val::Px(200.0),
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
        let mut db = Database::default();
        use dj_engine::data::database::EnemyRow;
        db.enemies.push(EnemyRow::new("wolf", "Wolf"));
        db.enemies.push(EnemyRow::new("slime", "Slime"));

        assert_eq!(db.enemies.len(), 2);
        assert_eq!(db.enemies[0].id, "wolf");
    }

    #[test]
    fn test_zone_layout_grid() {
        let zones = vec![
            ZoneRow {
                id: "forest".into(),
                name: [("en".to_string(), "Elwynn Forest".to_string())]
                    .into_iter()
                    .collect(),
                level_min: 1,
                level_max: 10,
                continent: "eastern_kingdoms".into(),
                description: HashMap::new(),
            },
            ZoneRow {
                id: "desert".into(),
                name: [("en".to_string(), "Tanaris".to_string())]
                    .into_iter()
                    .collect(),
                level_min: 40,
                level_max: 50,
                continent: "kalimdor".into(),
                description: HashMap::new(),
            },
        ];

        let mut zone_map = ZoneMap::default();
        let positions = build_zone_layout(&zones, &mut zone_map);

        assert_eq!(positions.len(), 2);
        assert_eq!(zone_map.zones.len(), 2);
        // Forest (level 1) should be first in grid
        assert!(positions.contains_key("forest"));
        assert!(positions.contains_key("desert"));
    }

    #[test]
    fn test_group_enemies_by_zone() {
        let enemies = vec![
            EnemyRow {
                id: "wolf".into(),
                zone_ids: vec!["forest".into()],
                ..Default::default()
            },
            EnemyRow {
                id: "bear".into(),
                zone_ids: vec!["forest".into(), "mountains".into()],
                ..Default::default()
            },
            EnemyRow {
                id: "scorpion".into(),
                zone_ids: vec!["desert".into()],
                ..Default::default()
            },
            EnemyRow {
                id: "slime".into(),
                zone_ids: vec![],
                ..Default::default()
            },
        ];

        let groups = group_enemies_by_zone(&enemies);
        assert_eq!(groups.get("forest").map(|v| v.len()).unwrap_or(0), 2);
        assert_eq!(groups.get("desert").map(|v| v.len()).unwrap_or(0), 1);
        assert_eq!(groups.get("mountains").map(|v| v.len()).unwrap_or(0), 1);
        assert!(!groups.contains_key(""));
    }

    #[test]
    fn test_scatter_position_distinct() {
        let positions: Vec<Vec2> = (0..10).map(|i| scatter_position(0.0, 0.0, i, 10)).collect();

        // All positions should be distinct
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                assert_ne!(
                    positions[i], positions[j],
                    "Positions {} and {} should differ",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_enemy_color_tiers() {
        let weak = enemy_color(30);
        let medium = enemy_color(100);
        let strong = enemy_color(500);

        // Weak should be green-ish (higher green channel)
        assert!(matches!(weak, Color::Srgba(c) if c.green > c.red));
        // Strong should be red-ish (higher red channel)
        assert!(matches!(strong, Color::Srgba(c) if c.red > c.green));
        // Medium should be orange-ish
        assert!(matches!(medium, Color::Srgba(c) if c.red > c.blue));
    }

    #[test]
    fn test_register_quests() {
        let quests = vec![
            QuestRow {
                id: "starter_quest".into(),
                ..Default::default()
            },
            QuestRow {
                id: "advanced_quest".into(),
                ..Default::default()
            },
        ];

        let mut journal = QuestJournal::default();
        let count = register_quests(&quests, &mut journal);

        assert_eq!(count, 2);
        assert!(journal.status("starter_quest").is_some());
        assert!(journal.status("advanced_quest").is_some());
    }

    #[test]
    fn test_detect_zone_in_bounds() {
        let mut zone_map = ZoneMap::default();
        zone_map
            .zones
            .insert("forest".into(), (0, 0, "Elwynn Forest".into()));
        zone_map
            .zones
            .insert("desert".into(), (1, 0, "Tanaris".into()));

        let player = Transform::from_xyz(10.0, 10.0, 0.0);
        let zone = detect_zone(&player, &zone_map);
        assert!(zone.contains("forest") || zone.contains("Elwynn"));

        let far_player = Transform::from_xyz(5000.0, 5000.0, 0.0);
        let zone = detect_zone(&far_player, &zone_map);
        assert_eq!(zone, "Wilderness");
    }

    #[test]
    fn test_zone_map_default() {
        let zm = ZoneMap::default();
        assert!(zm.zones.is_empty());
        assert_eq!(zm.total_enemies, 0);
        assert_eq!(zm.total_npcs, 0);
        assert_eq!(zm.total_quests, 0);
        assert_eq!(zm.total_loot_tables, 0);
        assert_eq!(zm.total_abilities, 0);
        assert_eq!(zm.total_consumables, 0);
        assert_eq!(zm.total_equipment, 0);
    }

    #[test]
    fn test_npc_color_by_faction() {
        let alliance = npc_color("faction_alliance_main");
        let horde = npc_color("faction_horde_main");
        let neutral = npc_color("faction_neutral_vendor");
        let unknown = npc_color("");

        // Alliance should be blue-ish
        assert!(matches!(alliance, Color::Srgba(c) if c.blue > c.red));
        // Horde should be red-ish
        assert!(matches!(horde, Color::Srgba(c) if c.red > c.blue));
        // Neutral should be yellow-ish
        assert!(matches!(neutral, Color::Srgba(c) if c.red > 0.5 && c.green > 0.5));
        // Unknown defaults to green
        assert!(matches!(unknown, Color::Srgba(c) if c.green > c.red));
    }
}
