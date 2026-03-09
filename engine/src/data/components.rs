//! Component data structures for scene entities.
//!
//! These are serializable data structures that describe entity components.
//! They map to Bevy ECS components at runtime via the spawner system.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 3D vector (used for positions, rotations, scales).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct Vec3Data {
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub z: f32,
}

impl Vec3Data {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn xy(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }
}

/// RGBA color with float components.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ColorData {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "default_alpha")]
    pub a: f32,
}

fn default_alpha() -> f32 {
    1.0
}

impl Default for ColorData {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl ColorData {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }
}

/// Physics body type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum BodyType {
    /// Does not move, affected by nothing
    #[default]
    Static,
    /// Fully simulated physics body
    Dynamic,
    /// Controlled programmatically, affects other bodies
    Kinematic,
}

/// Collision shape type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum CollisionShape {
    #[default]
    Box,
    Circle,
    Polygon,
}

/// Trigger type for interactive objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    #[default]
    None,
    Door,
    Chest,
    Npc,
    Custom,
}

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

/// Animation configuration for sprites.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct AnimationData {
    /// Animation clip asset ID
    pub clip_id: Option<String>,
    /// Playback speed multiplier
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Whether to loop the animation
    #[serde(default = "default_true")]
    pub loop_anim: bool,
}

fn default_speed() -> f32 {
    1.0
}
fn default_true() -> bool {
    true
}

/// Transform component data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct TransformComponent {
    /// World position
    pub position: Vec3Data,
    /// Rotation (degrees)
    #[serde(default)]
    pub rotation: Vec3Data,
    /// Scale factor
    #[serde(default = "default_scale")]
    pub scale: Vec3Data,
    /// Lock uniform scaling
    #[serde(default)]
    pub lock_uniform_scale: bool,
}

fn default_scale() -> Vec3Data {
    Vec3Data::new(1.0, 1.0, 1.0)
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3Data::default(),
            rotation: Vec3Data::default(),
            scale: default_scale(),
            lock_uniform_scale: false,
        }
    }
}

/// Sprite/visual appearance component data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct SpriteComponent {
    /// Sprite asset ID
    pub sprite_id: String,
    /// Sorting layer name
    #[serde(default)]
    pub sorting_layer: String,
    /// Order within sorting layer
    #[serde(default)]
    pub sorting_order: i32,
    /// Color tint
    #[serde(default)]
    pub tint: ColorData,
    /// Flip horizontally
    #[serde(default)]
    pub flip_x: bool,
    /// Flip vertically
    #[serde(default)]
    pub flip_y: bool,
    /// Animation settings
    #[serde(default)]
    pub animation: AnimationData,
}

/// Collision/physics component data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct CollisionComponent {
    /// Whether collision is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Physics body type
    #[serde(default)]
    pub body_type: BodyType,
    /// Collision shape
    #[serde(default)]
    pub shape: CollisionShape,
    /// Box dimensions (if shape is Box)
    #[serde(default)]
    pub box_size: Option<Vec3Data>,
    /// Circle radius (if shape is Circle)
    #[serde(default)]
    pub circle_radius: Option<f32>,
    /// Polygon points (if shape is Polygon)
    #[serde(default)]
    pub polygon_points: Vec<Vec3Data>,
    /// Shape offset from entity center
    #[serde(default)]
    pub offset: Vec3Data,
    /// Collision layer name
    #[serde(default)]
    pub layer: String,
    /// Collision mask (layers this collides with)
    #[serde(default)]
    pub mask: Vec<String>,
    /// Whether this is a trigger (non-solid)
    #[serde(default)]
    pub is_trigger: bool,
}

impl Default for CollisionComponent {
    fn default() -> Self {
        Self {
            enabled: true,
            body_type: BodyType::Static,
            shape: CollisionShape::Box,
            box_size: Some(Vec3Data::new(32.0, 32.0, 0.0)),
            circle_radius: None,
            polygon_points: Vec::new(),
            offset: Vec3Data::default(),
            layer: "default".to_string(),
            mask: vec!["default".to_string()],
            is_trigger: false,
        }
    }
}

/// Event hooks for interactive objects.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct InteractivityEvents {
    /// Event/script to run on interaction (E key, click, etc.)
    pub on_interact: Option<String>,
    /// Event/script to run when player enters trigger
    pub on_enter: Option<String>,
    /// Event/script to run when player exits trigger
    pub on_exit: Option<String>,
    /// Event/script to run on entity death
    pub on_death: Option<String>,
}

/// Interactivity component data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct InteractivityComponent {
    /// Type of trigger
    #[serde(default)]
    pub trigger_type: TriggerType,
    /// Unique trigger identifier
    #[serde(default)]
    pub trigger_id: String,
    /// Custom parameters for the trigger
    #[serde(default)]
    #[reflect(ignore)]
    pub parameters: HashMap<String, serde_json::Value>,
    /// Lua script ID to execute
    #[serde(default)]
    pub lua_script_id: Option<String>,
    /// Event hooks
    #[serde(default)]
    pub events: InteractivityEvents,
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

/// Audio source component data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct AudioSourceComponent {
    /// Audio clip asset ID
    pub clip_id: String,
    /// Volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Whether to loop
    #[serde(default)]
    pub loop_audio: bool,
    /// Whether to use spatial (3D) audio
    #[serde(default)]
    pub spatial: bool,
}

fn default_volume() -> f32 {
    1.0
}

impl Default for AudioSourceComponent {
    fn default() -> Self {
        Self {
            clip_id: String::new(),
            volume: 1.0,
            loop_audio: false,
            spatial: false,
        }
    }
}

/// Camera bounds for anchoring.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct CameraBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

/// Camera anchor component data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct CameraAnchorComponent {
    /// Camera movement bounds
    #[serde(default)]
    pub bounds: CameraBounds,
    /// Entity ID to follow
    #[serde(default)]
    pub follow_entity_id: Option<String>,
}

/// Container for all possible entity components.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct EntityComponents {
    /// Transform (always present)
    pub transform: TransformComponent,
    /// Sprite/visual appearance
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite: Option<SpriteComponent>,
    /// Collision/physics
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collision: Option<CollisionComponent>,
    /// Interactivity (triggers, events)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactivity: Option<InteractivityComponent>,
    /// NPC data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npc: Option<NpcComponent>,
    /// Enemy data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enemy: Option<EnemyComponent>,
    /// Combat stats
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat_stats: Option<CombatStatsComponent>,
    /// Tower data (TD)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tower: Option<TowerComponent>,
    /// Spawner data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawner: Option<SpawnerComponent>,
    /// Audio source
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_source: Option<AudioSourceComponent>,
    /// Camera anchor
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera_anchor: Option<CameraAnchorComponent>,
    /// Custom/extension properties
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[reflect(ignore)]
    pub custom: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_serialization() {
        let transform = TransformComponent {
            position: Vec3Data::xy(100.0, 200.0),
            rotation: Vec3Data::default(),
            scale: Vec3Data::new(1.0, 1.0, 1.0),
            lock_uniform_scale: false,
        };
        let json = serde_json::to_string(&transform).unwrap();
        let parsed: TransformComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(transform, parsed);
    }

    #[test]
    fn test_entity_components_optional_fields() {
        let components = EntityComponents {
            transform: TransformComponent::default(),
            sprite: Some(SpriteComponent {
                sprite_id: "hero.png".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let json = serde_json::to_string_pretty(&components).unwrap();
        assert!(json.contains("hero.png"));
        assert!(!json.contains("collision")); // Optional, not set
    }
}
