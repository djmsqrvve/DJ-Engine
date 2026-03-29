use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tower targeting mode (TD-specific).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum TargetingMode {
    /// Target the enemy that entered first
    #[default]
    First,
    /// Target the enemy that entered last
    Last,
    /// Target the closest enemy
    Closest,
    /// Target the enemy with highest HP
    Strongest,
}

/// Localized string (text in multiple languages).
pub type LocalizedString = HashMap<String, String>;

/// NPC component data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct NpcComponent {
    /// NPC ID (links to database NpcRow)
    pub npc_id: String,
    /// Display name per language
    #[serde(default)]
    pub display_name: LocalizedString,
    /// Dialogue set ID
    #[serde(default)]
    pub dialogue_set_id: String,
    /// Quest IDs this NPC is associated with
    #[serde(default)]
    pub quest_ids: Vec<String>,
    /// Inventory preset ID
    #[serde(default)]
    pub inventory_preset_id: Option<String>,
    /// Faction/alignment
    #[serde(default)]
    pub faction: String,
}

/// Enemy component data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct EnemyComponent {
    /// Enemy ID (links to database EnemyRow)
    pub enemy_id: String,
    /// AI behavior profile ID
    #[serde(default)]
    pub behavior_profile_id: String,
    /// Aggro detection range
    #[serde(default)]
    pub aggro_range: f32,
    /// Patrol path ID
    #[serde(default)]
    pub patrol_path_id: Option<String>,
}

/// Combat stats component data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct CombatStatsComponent {
    /// Maximum hit points
    pub max_hp: i32,
    /// Current hit points
    pub hp: i32,
    /// Current mana/resource
    #[serde(default)]
    pub mana: i32,
    /// Attack damage
    #[serde(default)]
    pub damage: i32,
    /// Defense/armor value
    #[serde(default)]
    pub defense: i32,
    /// Attacks per second
    #[serde(default = "default_attack_speed")]
    pub attack_speed: f32,
    /// Movement speed (units per second)
    #[serde(default = "default_move_speed")]
    pub move_speed: f32,
    /// Critical hit chance (0.0 - 1.0)
    #[serde(default)]
    pub crit_chance: f32,
    /// Loot table ID for drops
    #[serde(default)]
    pub loot_table_id: Option<String>,
}

fn default_attack_speed() -> f32 {
    1.0
}

fn default_move_speed() -> f32 {
    100.0
}

impl Default for CombatStatsComponent {
    fn default() -> Self {
        Self {
            max_hp: 100,
            hp: 100,
            mana: 0,
            damage: 10,
            defense: 0,
            attack_speed: 1.0,
            move_speed: 100.0,
            crit_chance: 0.0,
            loot_table_id: None,
        }
    }
}

/// Tower component data (TD-specific).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct TowerComponent {
    /// Tower ID (links to database TowerRow)
    pub tower_id: String,
    /// Attack damage
    #[serde(default)]
    pub damage: i32,
    /// Attack range in pixels
    #[serde(default = "default_tower_range")]
    pub range: f32,
    /// Attack cooldown in seconds
    #[serde(default = "default_tower_cooldown")]
    pub cooldown: f32,
    /// Build cost (resources)
    #[serde(default)]
    pub build_cost: i32,
    /// Build time in seconds
    #[serde(default)]
    pub build_time: f32,
    /// Upgrade path ID
    #[serde(default)]
    pub upgrade_path_id: Option<String>,
    /// Targeting behavior
    #[serde(default)]
    pub targeting_mode: TargetingMode,
    /// Projectile asset ID
    #[serde(default)]
    pub projectile_id: String,
    /// Effect/VFX ID
    #[serde(default)]
    pub effect_id: Option<String>,
}

fn default_tower_range() -> f32 {
    200.0
}

fn default_tower_cooldown() -> f32 {
    1.0
}

impl Default for TowerComponent {
    fn default() -> Self {
        Self {
            tower_id: String::new(),
            damage: 25,
            range: 200.0,
            cooldown: 1.0,
            build_cost: 100,
            build_time: 0.0,
            upgrade_path_id: None,
            targeting_mode: TargetingMode::First,
            projectile_id: String::new(),
            effect_id: None,
        }
    }
}

/// Wave definition for spawners.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct SpawnerWave {
    /// Enemy template ID to spawn
    pub enemy_template_id: String,
    /// Number of enemies in this wave segment
    pub count: u32,
    /// Interval between spawns in this segment
    #[serde(default = "default_spawn_interval")]
    pub interval: f32,
}

fn default_spawn_interval() -> f32 {
    1.0
}

/// Spawner component data (TD and JRPG).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct SpawnerComponent {
    /// Total number of waves
    pub wave_count: u32,
    /// Interval between wave starts
    #[serde(default = "default_spawn_interval")]
    pub spawn_interval: f32,
    /// Delay before first wave
    #[serde(default)]
    pub start_delay: f32,
    /// Whether waves loop
    #[serde(default)]
    pub loop_waves: bool,
    /// Wave definitions
    #[serde(default)]
    pub waves: Vec<SpawnerWave>,
    /// Path ID for spawned units to follow
    #[serde(default)]
    pub path_id: Option<String>,
}

impl Default for SpawnerComponent {
    fn default() -> Self {
        Self {
            wave_count: 1,
            spawn_interval: 1.0,
            start_delay: 0.0,
            loop_waves: false,
            waves: Vec::new(),
            path_id: None,
        }
    }
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<TargetingMode>()
        .register_type::<NpcComponent>()
        .register_type::<EnemyComponent>()
        .register_type::<CombatStatsComponent>()
        .register_type::<TowerComponent>()
        .register_type::<SpawnerWave>()
        .register_type::<SpawnerComponent>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_targeting_mode_default() {
        assert_eq!(TargetingMode::default(), TargetingMode::First);
    }

    #[test]
    fn test_npc_component_default() {
        let npc = NpcComponent::default();
        assert!(npc.npc_id.is_empty());
        assert!(npc.display_name.is_empty());
        assert!(npc.quest_ids.is_empty());
    }

    #[test]
    fn test_combat_stats_default() {
        let stats = CombatStatsComponent::default();
        assert_eq!(stats.max_hp, 100);
        assert_eq!(stats.hp, 100);
        assert_eq!(stats.damage, 10);
        assert_eq!(stats.defense, 0);
    }

    #[test]
    fn test_combat_stats_loot_table_default_none() {
        let stats = CombatStatsComponent::default();
        assert!(stats.loot_table_id.is_none());
    }

    #[test]
    fn test_enemy_component_default() {
        let enemy = EnemyComponent::default();
        assert!(enemy.enemy_id.is_empty());
    }

    #[test]
    fn test_spawner_component_default() {
        let spawner = SpawnerComponent::default();
        assert_eq!(spawner.wave_count, 1);
        assert_eq!(spawner.spawn_interval, 1.0);
        assert!(!spawner.loop_waves);
        assert!(spawner.waves.is_empty());
    }
}
